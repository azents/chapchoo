//! Strict HTTP request and response boundary for the SDK worker.

use super::MAX_CATALOGUE_BYTES;
use futures_util::{StreamExt, stream::BoxStream};
use parking_lot::Mutex;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use rmcp::{
    RoleClient,
    model::{
        ClientJsonRpcMessage, ClientRequest, ErrorData, RequestId, ResultType,
        ServerJsonRpcMessage, ServerResult,
    },
    transport::{
        Transport,
        streamable_http_client::{
            HttpStatusError, StreamableHttpClient, StreamableHttpError, StreamableHttpPostResponse,
        },
    },
};
use sse_stream::{Error as SseError, Sse, SseStream};
use std::{
    collections::HashMap,
    future::ready,
    io,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};
use tokio::sync::OwnedSemaphorePermit;

type HttpError = StreamableHttpError<reqwest::Error>;
type SseBody = BoxStream<'static, Result<Sse, SseError>>;

/// Witness of the typed reply before rmcp has filtered its catalogue.
/// Discovery sends one request at a time and stops on the first failed page.
#[derive(Clone, Default)]
pub(super) struct ListWitness(Arc<Mutex<Option<(RequestId, usize)>>>);

impl ListWitness {
    fn remember(&self, message: &ServerJsonRpcMessage, request_id: &RequestId) {
        if let ServerJsonRpcMessage::Response(response) = message
            && &response.id == request_id
            && let ServerResult::ListToolsResult(page) = &response.result
        {
            *self.0.lock() = Some((response.id.clone(), page.tools.len()));
        }
    }

    fn matches_page(&self, request: &RequestId, count: usize) -> bool {
        self.0
            .lock()
            .take()
            .is_some_and(|(id, raw_count)| id == *request && count == raw_count)
    }
}

#[derive(Clone)]
pub(super) struct ProfileHttp {
    client: reqwest::Client,
    // The SDK owns clones of this adapter beyond the caller's deadline. An
    // SSE body also keeps a lease until it is dropped by the SDK.
    slot: Arc<OwnedSemaphorePermit>,
    lists: ListWitness,
    dispatch: Option<Arc<SingleDispatch>>,
}

#[derive(Default)]
pub(super) struct SingleDispatch(AtomicU8);

impl SingleDispatch {
    const READY: u8 = 0;
    const POSSIBLE: u8 = 1;
    const CLOSED: u8 = 2;

    fn begin(&self) -> bool {
        self.0
            .compare_exchange(
                Self::READY,
                Self::POSSIBLE,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    /// Close a still-unsent attempt, including when a worker is behind the
    /// waiting deadline. Never turn a possible dispatch back into unsent.
    pub(super) fn close(&self) -> bool {
        self.0.compare_exchange(
            Self::READY,
            Self::CLOSED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) == Err(Self::POSSIBLE)
    }
}

impl ProfileHttp {
    pub(super) fn new(
        client: reqwest::Client,
        slot: OwnedSemaphorePermit,
        lists: ListWitness,
        dispatch: Option<Arc<SingleDispatch>>,
    ) -> Self {
        Self {
            client,
            slot: Arc::new(slot),
            lists,
            dispatch,
        }
    }
}

impl StreamableHttpClient for ProfileHttp {
    type Error = reqwest::Error;

    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, HttpError> {
        let request = match &message {
            ClientJsonRpcMessage::Request(request)
                if session_id.is_none() && auth_header.is_none() =>
            {
                request
            }
            _ => {
                return Err(HttpError::UnexpectedServerResponse(
                    "outside the MCP request profile".into(),
                ));
            }
        };
        let is_call = matches!(&request.request, ClientRequest::CallToolRequest(_));
        if !matches!(
            &request.request,
            ClientRequest::DiscoverRequest(_) | ClientRequest::ListToolsRequest(_)
        ) && !(is_call && self.dispatch.is_some())
        {
            return Err(HttpError::UnexpectedServerResponse(
                "outside the MCP request profile".into(),
            ));
        }
        let list_id = match &message {
            ClientJsonRpcMessage::Request(request)
                if matches!(request.request, ClientRequest::ListToolsRequest(_)) =>
            {
                Some(request.id.clone())
            }
            _ => None,
        };
        let mut headers = HeaderMap::new();
        for (name, value) in custom_headers {
            if !matches!(name.as_str(), "mcp-method" | "mcp-protocol-version")
                && !(is_call
                    && (name.as_str() == "mcp-name" || name.as_str().starts_with("mcp-param-")))
            {
                return Err(HttpError::ReservedHeaderConflict(name.to_string()));
            }
            headers.insert(name, value);
        }
        let outgoing = self
            .client
            .post(uri.as_ref())
            .headers(headers)
            .header(ACCEPT, "application/json, text/event-stream")
            .json(&message)
            .build()
            .map_err(HttpError::Client)?;
        if is_call
            && !self
                .dispatch
                .as_ref()
                .is_some_and(|dispatch| dispatch.begin())
        {
            return Err(HttpError::UnexpectedServerResponse(
                "MCP tool dispatch already closed".into(),
            ));
        }
        let mut response = self
            .client
            .execute(outgoing)
            .await
            .map_err(HttpError::Client)?;
        let status = response.status();
        let failure = || HttpError::HttpStatus(HttpStatusError::new(status.as_u16(), None));
        if status.is_success() && status != reqwest::StatusCode::OK {
            return Err(HttpError::UnexpectedServerResponse(
                "MCP request did not receive a complete HTTP response".into(),
            ));
        }
        if status.is_redirection() || matches!(status.as_u16(), 401 | 403 | 429) {
            return Err(failure());
        }
        let media = media_type(response.headers());
        if media == Some(Media::Sse) && status.is_success() {
            let slot = self.slot.clone();
            let chunks = response
                .bytes_stream()
                .scan(Some(0usize), move |used, chunk| {
                    let _lease = &slot;
                    let Some(total) = used.as_mut() else {
                        return ready(None);
                    };
                    let chunk = chunk.map_err(io::Error::other).and_then(|bytes| {
                        *total = total.saturating_add(bytes.len());
                        if *total > MAX_CATALOGUE_BYTES {
                            Err(io::Error::other("MCP response body exceeded its limit"))
                        } else {
                            Ok(bytes)
                        }
                    });
                    if chunk.is_err() {
                        *used = None;
                    }
                    ready(Some(chunk))
                });
            let reply =
                StreamableHttpPostResponse::Sse(SseStream::from_bytes_stream(chunks).boxed(), None);
            if let Some(id) = list_id {
                // Lists have no follow-up methods. Use the SDK's typed SSE reply
                // decoder before admitting the page to its catalogue worker.
                let (message, _) = reply.expect_initialized::<reqwest::Error>().await?;
                self.lists.remember(&message, &id);
                return Ok(StreamableHttpPostResponse::Json(message, None));
            }
            return Ok(reply);
        }
        if media != Some(Media::Json) {
            return Err(if status.is_success() {
                HttpError::UnexpectedContentType(None)
            } else {
                failure()
            });
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(HttpError::Client)? {
            if body.len().saturating_add(chunk.len()) > MAX_CATALOGUE_BYTES {
                return Err(HttpError::ResponseBodyTooLarge {
                    max_size: MAX_CATALOGUE_BYTES,
                });
            }
            body.extend_from_slice(&chunk);
        }
        let parsed = if status.is_success() {
            serde_json::from_slice(&body)?
        } else {
            match serde_json::from_slice(&body) {
                Ok(error @ ServerJsonRpcMessage::Error(_)) => error,
                _ => return Err(failure()),
            }
        };
        if let Some(id) = list_id {
            self.lists.remember(&parsed, &id);
        }
        Ok(StreamableHttpPostResponse::Json(parsed, None))
    }

    async fn delete_session(
        &self,
        _: Arc<str>,
        _: Arc<str>,
        _: Option<String>,
        _: HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), HttpError> {
        Err(HttpError::ServerDoesNotSupportDeleteSession)
    }

    async fn get_stream(
        &self,
        _: Arc<str>,
        _: Option<Arc<str>>,
        _: Option<String>,
        _: Option<String>,
        _: HashMap<HeaderName, HeaderValue>,
    ) -> Result<SseBody, HttpError> {
        // The discovery profile uses POST responses only.
        Err(HttpError::ServerDoesNotSupportSse)
    }
}

#[derive(PartialEq)]
enum Media {
    Json,
    Sse,
}

fn media_type(headers: &HeaderMap) -> Option<Media> {
    let mut values = headers.get_all(CONTENT_TYPE).iter();
    let media: mime::Mime = values.next()?.to_str().ok()?.parse().ok()?;
    if values.next().is_some() || media.suffix().is_some() {
        return None;
    }
    match (media.type_().as_str(), media.subtype().as_str()) {
        ("application", "json") => Some(Media::Json),
        ("text", "event-stream") => Some(Media::Sse),
        _ => None,
    }
}

// rmcp negotiates the supported version list but does not validate or retain
// the discover result discriminator when it projects peer information.
pub(super) struct CompleteDiscovery<T>(pub(super) T, pub(super) ListWitness);

impl<T: Transport<RoleClient>> Transport<RoleClient> for CompleteDiscovery<T> {
    type Error = T::Error;

    fn send(
        &mut self,
        message: ClientJsonRpcMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        self.0.send(message)
    }

    async fn receive(&mut self) -> Option<ServerJsonRpcMessage> {
        let message = self.0.receive().await?;
        match message {
            ServerJsonRpcMessage::Response(response)
                if matches!(&response.result, ServerResult::DiscoverResult(result)
                    if result.result_type != ResultType::COMPLETE) =>
            {
                Some(ServerJsonRpcMessage::error(
                    ErrorData::invalid_request("incomplete MCP discovery result", None),
                    Some(response.id),
                ))
            }
            ServerJsonRpcMessage::Response(response)
                if matches!(&response.result, ServerResult::ListToolsResult(page)
                    if !self.1.matches_page(&response.id, page.tools.len())) =>
            {
                Some(ServerJsonRpcMessage::error(
                    ErrorData::invalid_request("MCP tool list was not preserved", None),
                    Some(response.id),
                ))
            }
            other => Some(other),
        }
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.close()
    }
}
