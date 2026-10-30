//! Invocation-owned bundled Node processes; never discover targets globally.
use std::{fmt, num::NonZeroI32, process::ExitStatus, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    process::{Child, Command},
    sync::oneshot,
    task::JoinHandle,
};

use crate::{
    AppError,
    config::{ConfigError, WebApp, WebConfig},
};

#[cfg(unix)]
use nix::{
    errno::Errno,
    sys::{
        signal::{Signal, killpg},
        wait::{WaitPidFlag, WaitStatus, waitpid},
    },
    unistd::Pid,
};

#[cfg(target_os = "linux")]
use nix::sys::wait::{Id, waitid};

#[derive(thiserror::Error)]
#[error("An owned web health probe did not succeed before the startup deadline")]
struct ProbeStartupFailure {
    #[source]
    source: std::io::Error,
}

impl fmt::Debug for ProbeStartupFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProbeStartupFailure([REDACTED])")
    }
}

/// Linux adoption is local to this invocation. Reaping still selects only the
/// exact process groups created here, never unrelated children or global PIDs.
pub fn prepare_ownership() -> Result<(), AppError> {
    #[cfg(target_os = "linux")]
    {
        nix::sys::prctl::set_child_subreaper(true)?;
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(ConfigError::WebPlatform.into())
    }
}

pub struct WebChild {
    pub app: WebApp,
    child: Child,
    group: NonZeroI32,
    ready: oneshot::Receiver<()>,
    output: Option<JoinHandle<Result<(), std::io::Error>>>,
    address: std::net::SocketAddr,
    health_path: String,
    probe_failure: Option<std::io::Error>,
    cleaned: bool,
}

impl fmt::Debug for WebChild {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedWebChild")
            .field("app", &self.app)
            .finish_non_exhaustive()
    }
}

impl WebChild {
    pub fn spawn(config: WebConfig) -> Result<Self, AppError> {
        #[cfg(not(unix))]
        {
            let _ = config;
            Err(ConfigError::WebPlatform.into())
        }
        #[cfg(unix)]
        {
            let mut command = Command::new(&config.node);
            command
                .arg(&config.script)
                .current_dir(&config.directory)
                .env_clear()
                .envs(config.environment())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                // Node's existing safe error-reporting boundary stays visible.
                .stderr(std::process::Stdio::inherit())
                .process_group(0)
                .kill_on_drop(true);
            let mut child = command.spawn()?;
            let group = child.id().ok_or(ConfigError::ComponentExit)?;
            let group = NonZeroI32::new(i32::try_from(group)?).ok_or(ConfigError::ComponentExit)?;
            let mut stdout = child.stdout.take().ok_or(ConfigError::ComponentExit)?;
            let (ready_sender, ready) = oneshot::channel();
            let output = tokio::spawn(async move {
                let mut sender = Some(ready_sender);
                let mut tail = Vec::new();
                let mut buffer = [0; 1024];
                loop {
                    let count = stdout.read(&mut buffer).await?;
                    if count == 0 {
                        return Ok(());
                    }
                    if sender.is_some() {
                        tail.extend_from_slice(&buffer[..count]);
                        if tail
                            .windows(b"Ready in".len())
                            .any(|part| part == b"Ready in")
                        {
                            if let Some(sender) = sender.take() {
                                let _ = sender.send(());
                            }
                            tail.clear();
                        } else if tail.len() > 32 {
                            tail.drain(..tail.len() - 32);
                        }
                    }
                    // Drain bounded chunks, not raw commands/configuration or
                    // request output. The ready banner alone is not health.
                }
            });
            let mut address = config.listen;
            if address.ip().is_unspecified() {
                address.set_ip(match address.ip() {
                    std::net::IpAddr::V4(_) => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                    std::net::IpAddr::V6(_) => std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
                });
            }
            Ok(Self {
                app: config.app,
                child,
                group,
                ready,
                output: Some(output),
                address,
                health_path: config.health_path(),
                probe_failure: None,
                cleaned: false,
            })
        }
    }

    pub async fn ready(&mut self) -> Result<(), AppError> {
        tokio::select! {
            ready = &mut self.ready => ready.map_err(|_| ConfigError::ComponentExit)?,
            result = self.child.wait() => {
                result?;
                return Err(ConfigError::ComponentExit.into());
            }
        }
        loop {
            let healthy = tokio::select! {
                result = probe(self.address, &self.health_path) => {
                    match result {
                        Ok(healthy) => {
                            self.probe_failure = None;
                            healthy
                        }
                        Err(error) => {
                            self.probe_failure = Some(error);
                            false
                        }
                    }
                },
                result = self.child.wait() => {
                    result?;
                    return Err(ConfigError::ComponentExit.into());
                }
            };
            if healthy {
                if self.child.try_wait()?.is_some() {
                    return Err(ConfigError::ComponentExit.into());
                }
                return Ok(());
            }
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {},
                result = self.child.wait() => {
                    result?;
                    return Err(ConfigError::ComponentExit.into());
                }
            }
        }
    }

    pub async fn wait(&mut self) -> Result<ExitStatus, AppError> {
        Ok(self.child.wait().await?)
    }

    pub fn ensure_running(&mut self) -> Result<(), AppError> {
        if self.child.try_wait()?.is_some() {
            return Err(ConfigError::ComponentExit.into());
        }
        Ok(())
    }

    pub fn take_probe_failure(&mut self) -> Option<AppError> {
        self.probe_failure.take().map(|source| {
            tracing::error!(app = ?self.app, "Owned web health did not succeed before startup deadline");
            Box::new(ProbeStartupFailure { source }) as AppError
        })
    }

    pub fn terminate(&self) -> Result<(), AppError> {
        self.signal(false)
    }

    pub fn force(&self) -> Result<(), AppError> {
        self.signal(true)
    }

    fn signal(&self, force: bool) -> Result<(), AppError> {
        if self.cleaned {
            return Ok(());
        }
        #[cfg(unix)]
        {
            #[cfg(target_os = "linux")]
            if !owned_group_member(self.group)? {
                // A completed group number may have been reused. No current
                // owned child means no authority to signal that group.
                return Ok(());
            }
            let signal = if force {
                Signal::SIGKILL
            } else {
                Signal::SIGTERM
            };
            match killpg(Pid::from_raw(self.group.get()), signal) {
                Ok(()) | Err(Errno::ESRCH) => Ok(()),
                Err(error) => Err(error.into()),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = force;
            Err(ConfigError::WebPlatform.into())
        }
    }

    pub async fn reap(&mut self) -> Result<(), AppError> {
        // Let Tokio reap the direct child before waitpid can select any adopted
        // descendants in its group; do not race a global SIGCHLD consumer.
        self.child.wait().await?;
        #[cfg(unix)]
        loop {
            loop {
                match waitpid(Pid::from_raw(-self.group.get()), Some(WaitPidFlag::WNOHANG)) {
                    Ok(WaitStatus::StillAlive) | Err(Errno::ECHILD) => break,
                    Ok(_) | Err(Errno::EINTR) => {}
                    Err(error) => return Err(error.into()),
                }
            }
            #[cfg(target_os = "linux")]
            if !owned_group_member(self.group)? {
                break;
            }
            match killpg(Pid::from_raw(self.group.get()), None) {
                Err(Errno::ESRCH) => break,
                Ok(()) => tokio::time::sleep(Duration::from_millis(25)).await,
                Err(error) => return Err(error.into()),
            }
        }
        self.cleaned = true;
        if let Some(output) = self.output.as_mut() {
            output.await??;
        }
        self.output.take();
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn owned_group_member(group: NonZeroI32) -> Result<bool, AppError> {
    // WNOWAIT leaves the owned member unreaped until the caller signals or
    // drains it, preventing PID reuse between this check and killpg.
    loop {
        match waitid(
            Id::PGid(Pid::from_raw(group.get())),
            WaitPidFlag::WEXITED | WaitPidFlag::WNOHANG | WaitPidFlag::WNOWAIT,
        ) {
            Ok(_) => return Ok(true),
            Err(Errno::ECHILD) => return Ok(false),
            Err(Errno::EINTR) => {}
            Err(error) => return Err(error.into()),
        }
    }
}

impl Drop for WebChild {
    fn drop(&mut self) {
        if !self.cleaned && self.force().is_err() {
            tracing::error!(app = ?self.app, "Could not signal a remaining owned web group");
        }
        if let Some(output) = self.output.take() {
            output.abort();
        }
    }
}

async fn probe(address: std::net::SocketAddr, path: &str) -> Result<bool, std::io::Error> {
    let attempt = async {
        let mut stream = TcpStream::connect(address).await?;
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .await?;
        let mut buffer = [0; 128];
        let mut length = 0;
        while length < buffer.len() {
            let read = stream.read(&mut buffer[length..]).await?;
            if read == 0 {
                return Ok(false);
            }
            length += read;
            if buffer[..length].contains(&b'\n') {
                return Ok(buffer[..length].starts_with(b"HTTP/1.1 200 ")
                    || buffer[..length].starts_with(b"HTTP/1.0 200 "));
            }
        }
        Ok(false)
    };
    tokio::time::timeout(Duration::from_secs(2), attempt)
        .await
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Owned web health deadline exceeded",
            )
        })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn probe_failure_keeps_its_source_out_of_default_debug() -> Result<(), AppError> {
        let failure = ProbeStartupFailure {
            source: std::io::Error::other("native-source-canary"),
        };
        assert!(!format!("{failure:?}").contains("native-source-canary"));
        assert!(!failure.to_string().contains("native-source-canary"));
        let source = failure.source().ok_or("Missing native health source")?;
        assert!(source.to_string().contains("native-source-canary"));
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn group_checks_do_not_claim_authority_over_the_callers_group() -> Result<(), AppError> {
        let caller = NonZeroI32::new(nix::unistd::getpgrp().as_raw())
            .ok_or("Missing test caller process group")?;
        // No signal is sent. The current process/group is not its own child.
        assert!(!owned_group_member(caller)?);
        Ok(())
    }
}
