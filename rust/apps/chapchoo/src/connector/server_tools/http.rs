use super::*;
use http_body_util::{BodyExt, Full, Limited, combinators::BoxBody};
use hyper::{
    Request, Response, StatusCode,
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::sync::OwnedSemaphorePermit;
use tower::Service;

type Body = BoxBody<Bytes, Infallible>;
type Sdk = StreamableHttpService<handler::Handler, sessions::Sessions>;
struct Reservation {
    _request: OwnedSemaphorePermit,
    _bytes: OwnedSemaphorePermit,
}
#[derive(Clone)]
pub(super) struct Endpoint {
    pub(super) state: Arc<State>,
    pub(super) sessions: Arc<sessions::Sessions>,
    sdk: Sdk,
}
impl Endpoint {
    pub(super) fn new(address: SocketAddr, state: Arc<State>) -> Self {
        let mut config = StreamableHttpServerConfig::default();
        config.legacy_session_mode = true;
        config.allowed_hosts = vec![address.to_string()];
        config.allowed_origins = vec![format!("http://{address}")];
        config.max_request_body_bytes = 64 * 1024;
        config.sse_retry = None;
        config.sse_keep_alive = None;
        config.cancellation_token = state.stop.clone();
        let sessions = Arc::new(sessions::Sessions::new());
        let factory = state.clone();
        let sdk = StreamableHttpService::new(
            move || Ok(handler::Handler::new(factory.clone())),
            sessions.clone(),
            config,
        );
        Self {
            state,
            sessions,
            sdk,
        }
    }
}

struct Connection {
    // Initially owns a bounded parsing slot. After capability validation the
    // socket is charged to that endpoint, not to another integration or send.
    lane: Option<Arc<Semaphore>>,
    _permit: OwnedSemaphorePermit,
}
impl Connection {
    fn bind(&mut self, lane: &Arc<Semaphore>) -> bool {
        if self
            .lane
            .as_ref()
            .is_some_and(|owned| Arc::ptr_eq(owned, lane))
        {
            return true;
        }
        let Ok(permit) = lane.clone().try_acquire_owned() else {
            return false;
        };
        self._permit = permit;
        self.lane = Some(lane.clone());
        true
    }
}

pub(super) async fn serve(
    listener: tokio::net::TcpListener,
    stop: CancellationToken,
    builtin: Endpoint,
    optional: Arc<OnceLock<BTreeMap<String, Endpoint>>>,
) -> Result<(), AppError> {
    let parsing = Arc::new(Semaphore::new(128));
    let mut tasks = JoinSet::new();
    let mut result = Ok(());
    loop {
        tokio::select! {
            biased;
            _ = stop.cancelled() => break,
            completed = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(Err(error)) = completed {
                    result = crate::completion::complete(result, Err(error.into()));
                    stop.cancel();
                }
            }
            accepted = listener.accept() => {
                let socket = match accepted {
                    Ok((socket, _)) => socket,
                    Err(error) => {
                        result = crate::completion::complete(result, Err(error.into()));
                        stop.cancel();
                        break;
                    }
                };
                let Ok(permit) = parsing.clone().try_acquire_owned() else { continue; };
                let budget = Arc::new(Mutex::new(Connection { lane: None, _permit: permit }));
                let builtin = builtin.clone();
                let optional = optional.clone();
                let stopped = stop.clone();
                tasks.spawn(async move {
                    let service = service_fn(move |request| {
                        handle(request, builtin.clone(), optional.clone(), budget.clone())
                    });
                    let mut builder = http1::Builder::new();
                    builder.timer(TokioTimer::new()).header_read_timeout(Duration::from_secs(10))
                        .max_buf_size(16 * 1024).max_headers(128);
                    let connection = builder.serve_connection(TokioIo::new(socket), service);
                    tokio::pin!(connection);
                    let ended = tokio::select! {
                        ended = &mut connection => ended,
                        _ = stopped.cancelled() => {
                            connection.as_mut().graceful_shutdown();
                            match tokio::time::timeout(Duration::from_secs(10), connection).await {
                                Ok(result) => result,
                                Err(_) => return,
                            }
                        }
                    };
                    // Client parse/read/disconnect errors do not prove anything
                    // about a separately owned tool effect and contain no safe
                    // operator diagnostics to echo.
                    if ended.is_err() { tracing::debug!("Local MCP HTTP connection ended"); }
                });
            }
        }
    }
    drop(listener);
    while let Some(completed) = tasks.join_next().await {
        if let Err(error) = completed {
            result = crate::completion::complete(result, Err(error.into()));
        }
    }
    result
}
fn response(status: StatusCode, message: &'static str) -> Response<Body> {
    let mut response = Response::new(Full::new(Bytes::from_static(message.as_bytes())).boxed());
    *response.status_mut() = status;
    response
}
async fn handle(
    mut request: Request<Incoming>,
    builtin: Endpoint,
    optional: Arc<OnceLock<BTreeMap<String, Endpoint>>>,
    connection: Arc<Mutex<Connection>>,
) -> Result<Response<Body>, Infallible> {
    let endpoint = match request.uri().path() {
        "/mcp" => Some(&builtin),
        path => optional.get().and_then(|routes| routes.get(path)),
    };
    let Some(endpoint) = endpoint.filter(|_| request.uri().query().is_none()) else {
        return Ok(response(StatusCode::NOT_FOUND, "MCP endpoint not found"));
    };
    let state = &endpoint.state;
    if state.stop.is_cancelled() {
        return Ok(response(
            StatusCode::SERVICE_UNAVAILABLE,
            "Native MCP owner retired",
        ));
    }
    let header_bytes = request
        .headers()
        .iter()
        .map(|(name, value)| name.as_str().len() + value.as_bytes().len() + 4)
        .sum::<usize>();
    if header_bytes > 16 * 1024 {
        return Ok(response(
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            "MCP headers too large",
        ));
    }
    let authorized = {
        let mut values = request.headers().get_all("authorization").iter();
        values
            .next()
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .and_then(|value| BearerSecret::parse(value.to_owned()).ok())
            .is_some_and(|value| values.next().is_none() && state.key.digest().matches(&value))
    };
    if !authorized {
        return Ok(response(
            StatusCode::UNAUTHORIZED,
            "MCP capability is required",
        ));
    }
    // The SDK's request context/logging never needs our local bearer.
    request.headers_mut().remove("authorization");
    if request.headers().contains_key("last-event-id") {
        return Ok(response(
            StatusCode::BAD_REQUEST,
            "MCP event replay is unavailable",
        ));
    }
    if request
        .headers()
        .get_all("mcp-protocol-version")
        .iter()
        .any(|value| value.as_bytes() != b"2025-06-18")
    {
        return Ok(response(StatusCode::BAD_REQUEST, "Unsupported MCP profile"));
    }
    let reservation = match (
        lock(&connection).bind(&state.connections),
        state.requests.clone().try_acquire_owned(),
        state.bytes.clone().try_acquire_many_owned(80 * 1024),
    ) {
        (true, Ok(request), Ok(bytes)) => Arc::new(Reservation {
            _request: request,
            _bytes: bytes,
        }),
        _ => {
            return Ok(response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Local MCP capacity exhausted",
            ));
        }
    };
    request.extensions_mut().insert(reservation.clone());
    let (parts, body) = request.into_parts();
    let bytes = match tokio::time::timeout(
        Duration::from_secs(10),
        Limited::new(body, 64 * 1024).collect(),
    )
    .await
    {
        Err(_) => {
            return Ok(response(
                StatusCode::REQUEST_TIMEOUT,
                "MCP body read expired",
            ));
        }
        Ok(Err(_)) => {
            return Ok(response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "MCP body is unavailable or too large",
            ));
        }
        Ok(Ok(bytes)) => bytes.to_bytes(),
    };
    let request = Request::from_parts(parts, Full::new(bytes));
    let response = match endpoint.sdk.clone().call(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    Ok(response.map(|body| {
        body.map_frame(move |frame| {
            let _ = &reservation;
            frame
        })
        .boxed()
    }))
}
