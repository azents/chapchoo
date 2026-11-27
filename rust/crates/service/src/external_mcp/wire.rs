//! Strict HTTP response and request boundary for the SDK's discovery worker.

use super::MAX_CATALOGUE_BYTES;
use futures_util::{StreamExt, stream::BoxStream};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use rmcp::{
    RoleClient,
    model::{
        ClientJsonRpcMessage, ClientRequest, ErrorData, ResultType, ServerJsonRpcMessage,
        ServerResult,
    },
    transport::{
        Transport,
        streamable_http_client::{
            HttpStatusError, StreamableHttpClient, StreamableHttpError, StreamableHttpPostResponse,
        },
    },
};
use sse_stream::{Error as SseError, Sse, SseStream};
use std::{collections::HashMap, future::ready, io, sync::Arc};
use tokio::sync::OwnedSemaphorePermit;

type HttpError = StreamableHttpError<reqwest::Error>;
type SseBody = BoxStream<'static, Result<Sse, SseError>>;

#[derive(Clone)]
pub(super) struct DiscoveryHttp {
    client: reqwest::Client,
    // The SDK owns clones of this adapter beyond the caller's deadline. An
    // SSE body also keeps a lease until it is dropped by the SDK.
    slot: Arc<OwnedSemaphorePermit>,
}

impl DiscoveryHttp {
    pub(super) fn new(client: reqwest::Client, slot: OwnedSemaphorePermit) -> Self {
        Self {
            client,
            slot: Arc::new(slot),
        }
    }
}

impl StreamableHttpClient for DiscoveryHttp {
    type Error = reqwest::Error;

    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, HttpError> {
        if session_id.is_some()
            || auth_header.is_some()
            || !matches!(&message,
                ClientJsonRpcMessage::Request(request)
                    if matches!(&request.request, ClientRequest::DiscoverRequest(_) | ClientRequest::ListToolsRequest(_))
            )
        {
            return Err(HttpError::UnexpectedServerResponse(
                "outside the MCP discovery profile".into(),
            ));
        }
        let mut headers = HeaderMap::new();
        for (name, value) in custom_headers {
            if !matches!(name.as_str(), "mcp-method" | "mcp-protocol-version") {
                return Err(HttpError::ReservedHeaderConflict(name.to_string()));
            }
            headers.insert(name, value);
        }
        let mut response = self
            .client
            .post(uri.as_ref())
            .headers(headers)
            .header(ACCEPT, "application/json, text/event-stream")
            .json(&message)
            .send()
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
            return Ok(StreamableHttpPostResponse::Sse(
                SseStream::from_bytes_stream(chunks).boxed(),
                None,
            ));
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
pub(super) struct CompleteDiscovery<T>(pub(super) T);

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
            other => Some(other),
        }
    }

    fn close(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.close()
    }
}
