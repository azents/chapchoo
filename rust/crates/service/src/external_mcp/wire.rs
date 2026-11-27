//! Strict HTTP request and response boundary for the SDK worker.

use super::MAX_CATALOGUE_BYTES;
use futures_util::{StreamExt, stream::BoxStream};
use parking_lot::Mutex;
use reqwest::{
    StatusCode,
    header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};
use rmcp::{
    RoleClient,
    model::{
        ClientJsonRpcMessage, ClientNotification, ClientRequest, ErrorCode, ErrorData,
        ProtocolVersion, RequestId, ResultType, ServerJsonRpcMessage, ServerResult,
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
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};
use tokio::sync::OwnedSemaphorePermit;

type HttpError = StreamableHttpError<reqwest::Error>;
type SseBody = BoxStream<'static, Result<Sse, SseError>>;

const SESSION_HEADER: &str = "mcp-session-id";
const VERSION_HEADER: &str = "mcp-protocol-version";
// The SDK logs its session identifier; only this adapter handles the real one.
const LOCAL_SESSION: &str = "external-mcp-session";
const MAX_SESSION_BYTES: usize = 1024;

#[derive(Default)]
pub(super) struct Probe(AtomicBool);

impl Probe {
    pub(super) fn fallback_allowed(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    fn inspect(&self, status: StatusCode, id: &RequestId, message: Option<&ServerJsonRpcMessage>) {
        let fallback = match message {
            Some(ServerJsonRpcMessage::Error(error)) => {
                let correlated = error.id.as_ref() == Some(id);
                !modern_rejection(error.error.code)
                    && (status == StatusCode::BAD_REQUEST && (correlated || error.id.is_none())
                        || status == StatusCode::OK
                            && correlated
                            && error.error.code == ErrorCode::METHOD_NOT_FOUND)
            }
            None => status == StatusCode::BAD_REQUEST,
            Some(_) => false,
        };
        if fallback {
            self.0.store(true, Ordering::Release);
        }
    }
}

fn modern_rejection(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::UNSUPPORTED_PROTOCOL_VERSION
            | ErrorCode::MISSING_REQUIRED_CLIENT_CAPABILITY
            | ErrorCode::HEADER_MISMATCH
    )
}

#[derive(Default)]
pub(super) struct Earlier {
    version: Option<HeaderValue>,
    session: Option<HeaderValue>,
    pub(super) unsupported_version: bool,
}

#[derive(Clone)]
pub(super) enum Negotiation {
    Modern(Arc<Probe>),
    Earlier(Arc<Mutex<Earlier>>),
}

#[derive(PartialEq, Eq)]
enum Method {
    Discover,
    Initialize,
    Initialized,
    List,
    Call,
}

fn outside_profile() -> HttpError {
    HttpError::UnexpectedServerResponse("outside the MCP request profile".into())
}

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
    negotiation: Negotiation,
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
        slot: Arc<OwnedSemaphorePermit>,
        lists: ListWitness,
        dispatch: Option<Arc<SingleDispatch>>,
        negotiation: Negotiation,
    ) -> Self {
        Self {
            client,
            slot,
            lists,
            dispatch,
            negotiation,
        }
    }

    fn method(
        &self,
        message: &ClientJsonRpcMessage,
    ) -> Result<(Method, Option<RequestId>), HttpError> {
        let earlier = matches!(self.negotiation, Negotiation::Earlier(_));
        match message {
            ClientJsonRpcMessage::Request(request) => {
                let method = match &request.request {
                    ClientRequest::DiscoverRequest(_) if !earlier => Method::Discover,
                    ClientRequest::InitializeRequest(_) if earlier => Method::Initialize,
                    ClientRequest::ListToolsRequest(_) => Method::List,
                    ClientRequest::CallToolRequest(_) if self.dispatch.is_some() => Method::Call,
                    _ => return Err(outside_profile()),
                };
                Ok((method, Some(request.id.clone())))
            }
            ClientJsonRpcMessage::Notification(notification)
                if earlier
                    && matches!(
                        &notification.notification,
                        ClientNotification::InitializedNotification(_)
                    ) =>
            {
                Ok((Method::Initialized, None))
            }
            _ => Err(outside_profile()),
        }
    }

    fn earlier_headers(&self, session: Option<&str>) -> Result<HeaderMap, HttpError> {
        let Negotiation::Earlier(state) = &self.negotiation else {
            return Err(outside_profile());
        };
        let state = state.lock();
        let version = state.version.as_ref().ok_or_else(outside_profile)?;
        if session != state.session.as_ref().map(|_| LOCAL_SESSION) {
            return Err(outside_profile());
        }
        let mut headers = HeaderMap::new();
        headers.insert(VERSION_HEADER, version.clone());
        if let Some(value) = &state.session {
            headers.insert(SESSION_HEADER, value.clone());
        }
        Ok(headers)
    }

    fn initialize(
        &self,
        message: &ServerJsonRpcMessage,
        id: &RequestId,
        session: Option<HeaderValue>,
    ) -> Result<Option<String>, HttpError> {
        let Negotiation::Earlier(state) = &self.negotiation else {
            return Err(outside_profile());
        };
        if matches!(message, ServerJsonRpcMessage::Error(error) if error.id.as_ref() == Some(id)) {
            return Ok(None);
        }
        let ServerJsonRpcMessage::Response(response) = message else {
            return Err(outside_profile());
        };
        let ServerResult::InitializeResult(result) = &response.result else {
            return Err(outside_profile());
        };
        if &response.id != id {
            return Err(outside_profile());
        }
        let mut state = state.lock();
        if ![
            ProtocolVersion::V_2025_11_25,
            ProtocolVersion::V_2025_06_18,
            ProtocolVersion::V_2025_03_26,
        ]
        .contains(&result.protocol_version)
        {
            state.unsupported_version = true;
            return Err(outside_profile());
        }
        if state.version.is_some() {
            return Err(outside_profile());
        }
        state.version = Some(
            HeaderValue::from_str(result.protocol_version.as_str())
                .map_err(|_| outside_profile())?,
        );
        state.session = session;
        Ok(state.session.as_ref().map(|_| LOCAL_SESSION.to_owned()))
    }

    fn typed_reply(
        &self,
        method: Method,
        id: &RequestId,
        status: StatusCode,
        message: ServerJsonRpcMessage,
        session: Option<HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, HttpError> {
        let marker = match method {
            Method::Discover => {
                if let Negotiation::Modern(probe) = &self.negotiation {
                    probe.inspect(status, id, Some(&message));
                }
                if matches!(
                    &message,
                    ServerJsonRpcMessage::Error(error) if modern_rejection(error.error.code)
                        && error.id.as_ref() != Some(id)
                ) {
                    return Err(HttpError::UnexpectedServerResponse(
                        "uncorrelated MCP negotiation response".into(),
                    ));
                }
                None
            }
            Method::Initialize => self.initialize(&message, id, session)?,
            Method::List => {
                self.lists.remember(&message, id);
                None
            }
            _ => None,
        };
        Ok(StreamableHttpPostResponse::Json(message, marker))
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
        let (method, id) = self.method(&message)?;
        let modern = matches!(self.negotiation, Negotiation::Modern(_));
        if auth_header.is_some() || session_id.is_some() && (modern || method == Method::Initialize)
        {
            return Err(outside_profile());
        }
        let mut headers = if !modern && method != Method::Initialize {
            self.earlier_headers(session_id.as_deref())?
        } else {
            HeaderMap::new()
        };
        for (name, value) in custom_headers {
            match name.as_str() {
                VERSION_HEADER if !modern => {}
                "mcp-method" | VERSION_HEADER if modern => {
                    headers.insert(name, value);
                }
                name_text
                    if modern
                        && method == Method::Call
                        && (name_text == "mcp-name" || name_text.starts_with("mcp-param-")) =>
                {
                    headers.insert(name, value);
                }
                _ => return Err(HttpError::ReservedHeaderConflict(name.to_string())),
            }
        }
        let outgoing = self
            .client
            .post(uri.as_ref())
            .headers(headers)
            .header(ACCEPT, "application/json, text/event-stream")
            .json(&message)
            .build()
            .map_err(HttpError::Client)?;
        if method == Method::Call
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
        if method == Method::Initialized && status == StatusCode::ACCEPTED {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        if status.is_success() && status != StatusCode::OK {
            return Err(failure());
        }
        if status.is_redirection() || matches!(status.as_u16(), 401 | 403 | 429) {
            return Err(failure());
        }
        let id = id.as_ref().ok_or_else(failure)?;
        let session = if method == Method::Initialize && status.is_success() {
            session_header(response.headers())?
        } else {
            None
        };
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
            if method != Method::Call {
                let (message, _) = reply.expect_initialized::<reqwest::Error>().await?;
                return self.typed_reply(method, id, status, message, session);
            }
            return Ok(reply);
        }
        if media != Some(Media::Json) {
            if method == Method::Discover
                && status == StatusCode::BAD_REQUEST
                && let Negotiation::Modern(probe) = &self.negotiation
            {
                let body = read_body(&mut response).await?;
                let parsed = serde_json::from_slice(&body).ok();
                probe.inspect(status, id, parsed.as_ref());
            }
            return Err(if status.is_success() {
                HttpError::UnexpectedContentType(None)
            } else {
                failure()
            });
        }
        let body = read_body(&mut response).await?;
        let parsed = if status.is_success() {
            serde_json::from_slice(&body)?
        } else {
            match serde_json::from_slice(&body) {
                Ok(error @ ServerJsonRpcMessage::Error(_)) => error,
                other => {
                    if method == Method::Discover
                        && let Negotiation::Modern(probe) = &self.negotiation
                    {
                        probe.inspect(status, id, other.as_ref().ok());
                    }
                    return Err(failure());
                }
            }
        };
        self.typed_reply(method, id, status, parsed, session)
    }

    async fn delete_session(
        &self,
        uri: Arc<str>,
        session: Arc<str>,
        auth: Option<String>,
        _: HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), HttpError> {
        if auth.is_some() {
            return Err(outside_profile());
        }
        let response = self
            .client
            .delete(uri.as_ref())
            .headers(self.earlier_headers(Some(session.as_ref()))?)
            .send()
            .await
            .map_err(HttpError::Client)?;
        match response.status() {
            status if status.is_success() => Ok(()),
            StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND => {
                Err(HttpError::ServerDoesNotSupportDeleteSession)
            }
            status => Err(HttpError::HttpStatus(HttpStatusError::new(
                status.as_u16(),
                None,
            ))),
        }
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

async fn read_body(response: &mut reqwest::Response) -> Result<Vec<u8>, HttpError> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(HttpError::Client)? {
        if body.len().saturating_add(chunk.len()) > MAX_CATALOGUE_BYTES {
            return Err(HttpError::ResponseBodyTooLarge {
                max_size: MAX_CATALOGUE_BYTES,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn session_header(headers: &HeaderMap) -> Result<Option<HeaderValue>, HttpError> {
    let mut entries = headers.get_all(SESSION_HEADER).iter();
    let Some(value) = entries.next() else {
        return Ok(None);
    };
    let bytes = value.as_bytes();
    if entries.next().is_some()
        || bytes.is_empty()
        || bytes.len() > MAX_SESSION_BYTES
        || !bytes.iter().all(u8::is_ascii_graphic)
    {
        return Err(outside_profile());
    }
    let mut value = value.clone();
    value.set_sensitive(true);
    Ok(Some(value))
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
