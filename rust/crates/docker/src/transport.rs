//! Public SDK byte transport. Every request is submitted once on the selected
//! stream; no endpoint fallback, HTTP retry or handwritten Engine operation.
use crate::{
    Error,
    connection::{Endpoint, Selection},
};
use bollard::{BollardRequest, Docker};
use hyper::{Response, body::Incoming, client::conn::http1};
use hyper_util::rt::TokioIo;
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{Mutex, Semaphore},
    task::JoinSet,
};

mod ssh;
#[cfg(test)]
mod tests;
mod tls;

const CONNECT_BUDGET: Duration = Duration::from_secs(5);
const CONNECTIONS: usize = 64;

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

struct Inner {
    selected: Selection,
    tls: Option<Arc<rustls::ClientConfig>>,
    ssh: Option<Arc<ssh::Session>>,
    admitting: AtomicBool,
    permits: Arc<Semaphore>,
    connections: Mutex<JoinSet<()>>,
}
/// Keep this owner through SDK body consumption and explicit process drain.
#[derive(Clone)]
pub struct Transport(Arc<Inner>);
impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerTransport([REDACTED])")
    }
}

fn sdk_error() -> bollard::errors::Error {
    bollard::errors::Error::IOError {
        err: std::io::Error::other("Selected Docker transport failed"),
    }
}

impl Transport {
    pub async fn new(selected: Selection) -> Result<Self, Error> {
        let tls = selected.tls.as_ref().map(tls::configuration).transpose()?;
        let ssh = match &selected.endpoint {
            Endpoint::Ssh {
                host, user, port, ..
            } => Some(ssh::Session::start(host, user.as_deref(), *port).await?),
            _ => None,
        };
        Ok(Self(Arc::new(Inner {
            selected,
            tls,
            ssh,
            admitting: AtomicBool::new(true),
            permits: Arc::new(Semaphore::new(CONNECTIONS)),
            connections: Mutex::new(JoinSet::new()),
        })))
    }

    pub fn sdk(&self) -> Result<Docker, Error> {
        let owner = self.clone();
        let version = self
            .0
            .selected
            .api_version
            .map(|value| bollard::ClientVersion {
                major_version: value.major,
                minor_version: value.minor,
            });
        let address = match &self.0.selected.endpoint {
            Endpoint::Tcp { host, port } => {
                let host = if host.contains(':') {
                    format!("[{host}]")
                } else {
                    host.clone()
                };
                format!(
                    "{}://{host}:{port}",
                    if self.0.tls.is_some() {
                        "https"
                    } else {
                        "http"
                    }
                )
            }
            _ => "http://docker".into(),
        };
        Docker::connect_with_custom_transport(
            move |request: BollardRequest| {
                let owner = owner.clone();
                async move { owner.request(request).await }
            },
            Some(address),
            30,
            version.as_ref().unwrap_or(bollard::API_DEFAULT_VERSION),
        )
        .map_err(|_| Error::Configuration)
    }

    async fn stream(&self) -> Result<Box<dyn Io>, Error> {
        match &self.0.selected.endpoint {
            Endpoint::Unix(path) => Ok(Box::new(
                tokio::net::UnixStream::connect(path)
                    .await
                    .map_err(|_| Error::Transport)?,
            )),
            Endpoint::Tcp { host, port } => {
                let tcp = tokio::net::TcpStream::connect((host.as_str(), *port))
                    .await
                    .map_err(|_| Error::Transport)?;
                if let Some(config) = &self.0.tls {
                    let name = rustls::pki_types::ServerName::try_from(host.clone())
                        .map_err(|_| Error::Configuration)?;
                    let stream = tokio_rustls::TlsConnector::from(config.clone())
                        .connect(name, tcp)
                        .await
                        .map_err(|_| Error::Transport)?;
                    Ok(Box::new(stream))
                } else {
                    Ok(Box::new(tcp))
                }
            }
            Endpoint::Ssh { socket, .. } => Ok(Box::new(
                self.0
                    .ssh
                    .as_ref()
                    .ok_or(Error::Transport)?
                    .connect(socket.as_deref())
                    .await?,
            )),
        }
    }

    async fn request(
        &self,
        mut request: BollardRequest,
    ) -> Result<Response<Incoming>, bollard::errors::Error> {
        if !self.0.admitting.load(Ordering::Acquire) {
            return Err(sdk_error());
        }
        // Hyper's connection API does not supply Host or convert absolute URI
        // form. This is a direct selected connection, never an HTTP proxy.
        let authority = request.uri().authority().ok_or_else(sdk_error)?.clone();
        if !request.headers().contains_key(hyper::header::HOST) {
            request.headers_mut().insert(
                hyper::header::HOST,
                hyper::header::HeaderValue::from_str(authority.as_str())
                    .map_err(|_| sdk_error())?,
            );
        }
        let path = request
            .uri()
            .path_and_query()
            .ok_or_else(sdk_error)?
            .clone();
        *request.uri_mut() = hyper::Uri::builder()
            .path_and_query(path)
            .build()
            .map_err(|_| sdk_error())?;
        let permit = self
            .0
            .permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| sdk_error())?;
        let stream = tokio::time::timeout(CONNECT_BUDGET, self.stream())
            .await
            .map_err(|_| sdk_error())?
            .map_err(|_| sdk_error())?;
        let (mut sender, connection) = http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| sdk_error())?;
        {
            let mut connections = self.0.connections.lock().await;
            while connections.try_join_next().is_some() {}
            if !self.0.admitting.load(Ordering::Acquire) {
                return Err(sdk_error());
            }
            let owner = self.clone();
            connections.spawn(async move {
                // Header delivery is not body completion. The SDK stream and
                // connection driver keep the selected source alive independently
                // of a response waiter or a dropped temporary Docker clone.
                let _owner = owner;
                let _permit = permit;
                let _ = connection.await;
            });
        }
        sender.send_request(request).await.map_err(|_| sdk_error())
    }

    pub async fn drain(&self) -> Result<(), Error> {
        self.0.admitting.store(false, Ordering::Release);
        let mut connections = {
            let mut active = self.0.connections.lock().await;
            std::mem::take(&mut *active)
        };
        while let Some(result) = connections.join_next().await {
            result.map_err(|_| Error::Transport)?;
        }
        if let Some(session) = &self.0.ssh {
            session.shutdown().await?;
        }
        Ok(())
    }
}
