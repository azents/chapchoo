//! One process-lifetime OpenSSH authentication/target, with independent mux
//! channels for SDK streams. No new authentication after that master is lost.
use crate::Error;
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
    sync::Mutex,
};

pub(super) struct Session {
    master: Mutex<Child>,
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
    _child: Child,
    input: ChildStdin,
    output: ChildStdout,
}
impl std::fmt::Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DockerSshStream([REDACTED])")
    }
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
            .map_err(|_| Error::Transport)?;
        let path = socket_path(directory.path());
        let master = Command::new("ssh")
            .args(master_arguments(&path, host, user, port)?)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| Error::Transport)?;
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
        loop {
            if master.try_wait().map_err(|_| Error::Transport)?.is_some() {
                return Err(Error::Transport);
            }
            // `-O check` never opens an alternate SSH connection. Socket
            // existence is only readiness to attempt this authoritative check.
            if path.exists() {
                let checked = Command::new("ssh")
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
                    .status();
                if tokio::time::timeout_at(deadline, checked)
                    .await
                    .map_err(|_| Error::Transport)?
                    .map_err(|_| Error::Transport)?
                    .success()
                {
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
        Ok(Arc::new(Self {
            master: Mutex::new(master),
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
            .try_wait()
            .map_err(|_| Error::Transport)?
            .is_some()
        {
            return Err(Error::Transport);
        }
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
            .map_err(|_| Error::Transport)?;
        let input = child.stdin.take().ok_or(Error::Transport)?;
        let output = child.stdout.take().ok_or(Error::Transport)?;
        Ok(Stream {
            _session: self.clone(),
            _child: child,
            input,
            output,
        })
    }
    pub(super) async fn shutdown(&self) -> Result<(), Error> {
        let mut master = self.master.lock().await;
        if master.try_wait().map_err(|_| Error::Transport)?.is_none() {
            master.kill().await.map_err(|_| Error::Transport)?;
        }
        Ok(())
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
