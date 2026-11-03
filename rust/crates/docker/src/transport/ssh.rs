//! One process-lifetime OpenSSH authentication/target, with independent mux
//! channels for SDK streams. No new authentication after that master is lost.
use crate::{
    Error,
    completion::{OwnedTasks, complete},
};
use std::{
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::{Mutex, OwnedSemaphorePermit, Semaphore, oneshot},
};

struct Channels {
    admitting: bool,
    owned: OwnedTasks,
}

pub(super) struct Session {
    master: Mutex<Option<Child>>,
    channels: Mutex<Channels>,
    permits: Arc<Semaphore>,
    directory: tempfile::TempDir,
    host: String,
    user: Option<String>,
    port: Option<u16>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DockerSshSession([REDACTED])")
    }
}
pub(super) struct Stream {
    _session: Arc<Session>,
    stop: Option<oneshot::Sender<()>>,
    input: ChildStdin,
    output: ChildStdout,
}
impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DockerSshStream([REDACTED])")
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        // The Session's worker owns fallible termination and reaping.
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

async fn terminate(child: &mut Child) -> Result<(), Error> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(()),
        state => {
            let primary = state.map(|_| ()).map_err(Error::TransportIo);
            let signal = child.start_kill().map_err(Error::TransportIo);
            let reaped = child.wait().await.map(|_| ()).map_err(Error::TransportIo);
            complete(complete(primary, signal), reaped)
        }
    }
}

fn own_channel(
    child: Child,
    mut stop: oneshot::Receiver<()>,
    permit: OwnedSemaphorePermit,
    owned: &mut OwnedTasks,
) {
    owned.tasks.spawn(async move {
        let _permit = permit;
        let mut child = child;
        tokio::select! {
            exit = child.wait() => match exit {
                Ok(_) => Ok(()),
                Err(error) => {
                    let cleanup = terminate(&mut child).await;
                    complete(Err(Error::TransportIo(error)), cleanup)
                }
            },
            _ = &mut stop => terminate(&mut child).await,
        }
    });
}

fn quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
fn target(args: &mut Vec<String>, host: &str, user: Option<&str>, port: Option<u16>) {
    if let Some(port) = port {
        args.extend(["-p".into(), port.to_string()]);
    }
    if let Some(user) = user {
        args.extend(["-l".into(), user.into()]);
    }
    args.extend(["--".into(), host.into()]);
}
fn socket_path(directory: &Path) -> PathBuf {
    directory.join("control")
}
fn master_arguments(
    path: &Path,
    host: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<Vec<String>, Error> {
    let path = path.to_str().ok_or(Error::Configuration)?;
    let mut args = vec![
        "-M".into(),
        "-N".into(),
        "-S".into(),
        path.into(),
        "-o".into(),
        "ControlPersist=no".into(),
        "-o".into(),
        "ConnectTimeout=5".into(),
    ];
    target(&mut args, host, user, port);
    Ok(args)
}
fn client_arguments(
    path: &Path,
    host: &str,
    user: Option<&str>,
    port: Option<u16>,
    remote_socket: Option<&str>,
) -> Result<Vec<String>, Error> {
    let path = path.to_str().ok_or(Error::Configuration)?;
    // ProxyCommand is a fail-closed guard only for a missing mux master. It
    // cannot reconnect using reread authentication/config or a changed HostName.
    // An existing authenticated master's byte transport does not use this proxy.
    // The master has already consumed the selected account configuration.
    // Mux clients must not rerun Match/LocalCommand or reread changed host/auth
    // sources. The private socket selects the existing authenticated target.
    let mut args = vec![
        "-F".into(),
        "/dev/null".into(),
        "-T".into(),
        "-S".into(),
        path.into(),
        "-o".into(),
        "ControlMaster=no".into(),
        "-o".into(),
        "ProxyCommand=false".into(),
    ];
    target(&mut args, host, user, port);
    let mut remote = vec!["docker".to_owned()];
    if let Some(socket) = remote_socket {
        remote.extend(["--host".into(), format!("unix://{socket}")]);
    }
    remote.extend(["system".into(), "dial-stdio".into()]);
    args.push(
        remote
            .iter()
            .map(|word| quoted(word))
            .collect::<Vec<_>>()
            .join(" "),
    );
    Ok(args)
}
impl Session {
    pub(super) async fn start(
        host: &str,
        user: Option<&str>,
        port: Option<u16>,
    ) -> Result<Arc<Self>, Error> {
        let directory = tempfile::Builder::new()
            .prefix("chapchoo-ssh-")
            .tempdir()
            .map_err(Error::TransportIo)?;
        let path = socket_path(directory.path());
        let master = Command::new("ssh")
            .args(master_arguments(&path, host, user, port)?)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(Error::TransportIo)?;
        Self::ready(host, user, port, directory, master).await
    }
    async fn ready(
        host: &str,
        user: Option<&str>,
        port: Option<u16>,
        directory: tempfile::TempDir,
        mut master: Child,
    ) -> Result<Arc<Self>, Error> {
        let path = socket_path(directory.path());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let readiness = async {
            loop {
                if master.try_wait().map_err(Error::TransportIo)?.is_some() {
                    return Err(Error::Transport);
                }
                // `-O check` never opens an alternate SSH connection. Socket
                // existence is only readiness to attempt this authoritative check.
                if path.exists() {
                    let mut checked = Command::new("ssh")
                        .args(["-F", "/dev/null"])
                        .arg("-S")
                        .arg(&path)
                        .args(["-O", "check"])
                        .args({
                            let mut args = Vec::new();
                            target(&mut args, host, user, port);
                            args
                        })
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .kill_on_drop(true)
                        .spawn()
                        .map_err(Error::TransportIo)?;
                    let checked = match tokio::time::timeout_at(deadline, checked.wait()).await {
                        Ok(Ok(status)) => status,
                        result => {
                            let primary = match result {
                                Ok(Err(error)) => Error::TransportIo(error),
                                _ => Error::Transport,
                            };
                            return complete(Err(primary), terminate(&mut checked).await);
                        }
                    };
                    if checked.success() {
                        break;
                    }
                }
                if tokio::time::Instant::now() >= deadline {
                    return Err(Error::Transport);
                }
                tokio::time::sleep_until(
                    (tokio::time::Instant::now() + Duration::from_millis(25)).min(deadline),
                )
                .await;
            }
            Ok(())
        }
        .await;
        if let Err(primary) = readiness {
            return complete(Err(primary), terminate(&mut master).await);
        }
        Ok(Arc::new(Self {
            master: Mutex::new(Some(master)),
            channels: Mutex::new(Channels {
                admitting: true,
                owned: OwnedTasks::default(),
            }),
            permits: Arc::new(Semaphore::new(super::CONNECTIONS)),
            directory,
            host: host.into(),
            user: user.map(str::to_owned),
            port,
        }))
    }
    pub(super) async fn connect(self: &Arc<Self>, socket: Option<&str>) -> Result<Stream, Error> {
        if self
            .master
            .lock()
            .await
            .as_mut()
            .ok_or(Error::Transport)?
            .try_wait()
            .map_err(Error::TransportIo)?
            .is_some()
        {
            return Err(Error::Transport);
        }
        let mut channels = self.channels.lock().await;
        if !channels.admitting {
            return Err(Error::Transport);
        }
        channels.owned.reap();
        let permit = self
            .permits
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Transport)?;
        let mut child = Command::new("ssh")
            .args(client_arguments(
                &socket_path(self.directory.path()),
                &self.host,
                self.user.as_deref(),
                self.port,
                socket,
            )?)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(Error::TransportIo)?;
        let pipes = child.stdin.take().zip(child.stdout.take());
        let Some((input, output)) = pipes else {
            return complete(Err(Error::Transport), terminate(&mut child).await);
        };
        let (stop, stopped) = oneshot::channel();
        own_channel(child, stopped, permit, &mut channels.owned);
        Ok(Stream {
            _session: self.clone(),
            stop: Some(stop),
            input,
            output,
        })
    }
    pub(super) async fn shutdown(&self) -> Result<(), Error> {
        let mut channels = self.channels.lock().await;
        channels.admitting = false;
        channels.owned.join().await;
        let mut master = self.master.lock().await;
        if let Some(mut master) = master.take() {
            channels
                .owned
                .tasks
                .spawn(async move { terminate(&mut master).await });
        }
        channels.owned.join().await;
        channels.owned.finish()
    }
}
impl AsyncRead for Stream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        bytes: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.output).poll_read(context, bytes)
    }
}
impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.input).poll_write(context, bytes)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.input).poll_flush(context)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.input).poll_shutdown(context)
    }
}
#[cfg(all(test, feature = "ssh-fixture"))]
#[path = "ssh_fixture.rs"]
mod fixture;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn drain_releases_registration_lock_before_waiting_for_unregistered_ssh_stream()
    -> Result<(), Box<dyn std::error::Error>> {
        use super::super::{CONNECTIONS, Transport};
        use crate::connection::{Endpoint, Selection};
        let master = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let master_pid = master.id().ok_or("Missing synthetic master")?;
        let session = Arc::new(Session {
            master: Mutex::new(Some(master)),
            channels: Mutex::new(Channels {
                admitting: true,
                owned: OwnedTasks::default(),
            }),
            permits: Arc::new(Semaphore::new(CONNECTIONS)),
            directory: tempfile::tempdir()?,
            host: "unused".into(),
            user: None,
            port: None,
        });
        let mut transport = Transport::new(Selection {
            endpoint: Endpoint::Unix("/unused-registration-fixture".into()),
            tls: None,
            api_version: None,
        })
        .await?;
        Arc::get_mut(&mut transport.0)
            .ok_or("Transport already shared")?
            .ssh = Some(session.clone());
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let channel_pid = child.id().ok_or("Missing synthetic channel")?;
        let input = child.stdin.take().ok_or("Missing channel input")?;
        let output = child.stdout.take().ok_or("Missing channel output")?;
        let permit = session.permits.clone().try_acquire_owned()?;
        let (stop, stopped) = oneshot::channel();
        own_channel(
            child,
            stopped,
            permit,
            &mut session.channels.lock().await.owned,
        );
        let stream = Stream {
            _session: session.clone(),
            stop: Some(stop),
            input,
            output,
        };
        let mut draining = Box::pin(transport.drain());
        std::future::poll_fn(|context| {
            assert!(draining.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        // An already-open request must be able to recheck closed admission and
        // drop its not-yet-registered stream, independently of drain completion.
        let mut registration = Box::pin(async {
            let _guard = transport.0.connections.lock().await;
            assert!(
                !transport
                    .0
                    .admitting
                    .load(std::sync::atomic::Ordering::Acquire)
            );
            drop(stream);
        });
        std::future::poll_fn(|context| {
            assert!(registration.as_mut().poll(context).is_ready());
            std::task::Poll::Ready(())
        })
        .await;
        drop(registration);
        draining.await?;
        assert_eq!(session.permits.available_permits(), CONNECTIONS);
        for pid in [master_pid, channel_pid] {
            assert!(!Path::new(&format!("/proc/{pid}")).exists());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn closed_channel_is_reaped_before_capacity_is_released()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let pid = child.id().ok_or("Missing fixture process")?;
        let _input = child.stdin.take().ok_or("Missing fixture input")?;
        let _output = child.stdout.take().ok_or("Missing fixture output")?;
        let permits = Arc::new(Semaphore::new(1));
        let permit = permits.clone().try_acquire_owned()?;
        let mut owned = OwnedTasks::default();
        let (stop, stopped) = oneshot::channel();
        own_channel(child, stopped, permit, &mut owned);
        assert_eq!(permits.available_permits(), 0);
        drop(stop);
        owned.join().await;
        owned.finish()?;
        assert_eq!(permits.available_permits(), 1);
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn unsuccessful_master_readiness_returns_only_after_process_exit_is_reaped()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::Builder::new()
            .prefix("failed-ssh-fixture-")
            .tempdir()?;
        let master = Command::new("sh")
            .args(["-c", "exit 1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let pid = master.id().ok_or("Missing fixture master PID")?;
        assert!(
            Session::ready("unused-fixture", None, None, directory, master)
                .await
                .is_err()
        );
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        Ok(())
    }

    #[test]
    fn selected_authentication_policy_is_preserved_and_mux_clients_cannot_reauthenticate()
    -> Result<(), Error> {
        let master = master_arguments(
            Path::new("/private/control"),
            "selected-host",
            Some("operator"),
            Some(2222),
        )?;
        assert!(
            master
                .iter()
                .all(|value| !value.contains("StrictHostKeyChecking")
                    && !value.contains("UserKnownHostsFile")
                    && !value.contains("IdentityFile")
                    && !value.contains("BatchMode"))
        );
        assert!(!master.iter().any(|value| value.contains("ProxyCommand")));
        let args = client_arguments(
            Path::new("/private/control"),
            "selected-host",
            Some("operator"),
            Some(2222),
            Some("/run/socket with 'quote';not-a-command.sock"),
        )?;
        assert!(args.iter().any(|value| value == "ProxyCommand=false"));
        assert!(args.iter().any(|value| value == "ControlMaster=no"));
        assert_eq!(
            args.last().map(String::as_str),
            Some(
                "'docker' '--host' 'unix:///run/socket with '\\''quote'\\'';not-a-command.sock' 'system' 'dial-stdio'"
            )
        );
        Ok(())
    }
}
