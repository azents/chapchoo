//! Opt-in real local OpenSSH boundary. No Engine, root or system daemon changes.
use super::*;
use std::os::fd::OwnedFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
type TestError = Box<dyn std::error::Error + Send + Sync>;
fn executable(name: &str) -> Result<PathBuf, TestError> {
    std::env::split_paths(&std::env::var_os("PATH").ok_or("Missing fixture PATH")?)
        .map(|path| path.join(name))
        .find(|path| path.is_file())
        .ok_or_else(|| format!("OpenSSH fixture requires {name} on PATH").into())
}
async fn generate(path: &Path) -> Result<(), TestError> {
    if !Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(path)
        .status()
        .await?
        .success()
    {
        return Err("Fixture key generation failed".into());
    }
    Ok(())
}
#[tokio::test]
async fn authenticated_mux_target_survives_config_loss_but_never_reauthenticates_after_socket_loss()
-> Result<(), TestError> {
    let sshd = executable("sshd")?;
    let directory = tempfile::Builder::new().prefix("ssh-fixture-").tempdir()?;
    let host_key = directory.path().join("host");
    let identity = directory.path().join("identity");
    generate(&host_key).await?;
    generate(&identity).await?;
    let commands = directory.path().join("commands");
    let script = directory.path().join("echo.sh");
    std::fs::write(
        &script,
        format!(
            "printf '%s\\n' \"$SSH_ORIGINAL_COMMAND\" >> {}\nexec /bin/cat\n",
            quoted(commands.to_str().ok_or("Fixture path")?)
        ),
    )?;
    let server_config = directory.path().join("server");
    std::fs::write(
        &server_config,
        format!(
            "HostKey {}\nAuthorizedKeysFile {}.pub\nStrictModes no\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nForceCommand /bin/sh {}\nPidFile {}\nLogLevel ERROR\n",
            host_key.display(),
            identity.display(),
            script.display(),
            directory.path().join("pid").display()
        ),
    )?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let error_log = std::fs::File::create(directory.path().join("server.log"))?;
    let (listening, ready) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await?;
        let stream = stream.into_std()?;
        stream.set_nonblocking(false)?;
        let mut daemon = Command::new(sshd)
            .args(["-i", "-e", "-f"])
            .arg(server_config)
            .stdin(Stdio::from(OwnedFd::from(stream.try_clone()?)))
            .stdout(Stdio::from(OwnedFd::from(stream)))
            .stderr(Stdio::from(error_log))
            .kill_on_drop(true)
            .spawn()?;
        let _ = listening.send(());
        // Keep the listener available to prove no fallback authentication.
        let status = daemon.wait().await?;
        if listener.into_std()?.accept().is_ok() {
            return Err::<(), TestError>("A lost mux channel opened a new SSH connection".into());
        }
        if !status.success() {
            return Err("Fixture SSH daemon failed; inspect private fixture log".into());
        }
        Ok(())
    });
    let client_config = directory.path().join("client");
    let account = Command::new("id").arg("-un").output().await?;
    if !account.status.success() {
        return Err("Fixture account lookup failed".into());
    }
    let user = String::from_utf8(account.stdout)?.trim().to_owned();
    std::fs::write(
        &client_config,
        format!(
            "Host fixture-target\n HostName 127.0.0.1\n Port {port}\n User {user}\n IdentityFile {}\n IdentitiesOnly yes\n UserKnownHostsFile {}\n StrictHostKeyChecking no\n BatchMode yes\n",
            identity.display(),
            directory.path().join("known_hosts").display()
        ),
    )?;
    let session_directory = tempfile::Builder::new().prefix("chapchoo-ssh-").tempdir()?;
    let master = Command::new("ssh")
        .arg("-F")
        .arg(&client_config)
        .args(master_arguments(
            &socket_path(session_directory.path()),
            "fixture-target",
            None,
            None,
        )?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;
    let session = Session::ready("fixture-target", None, None, session_directory, master).await?;
    ready.await?;
    std::fs::write(&client_config, "InvalidAfterStartup definitely\n")?;
    std::fs::remove_file(&identity)?;
    let socket = "/run/socket with 'quote';not-a-command.sock";
    for message in [b"first channel".as_slice(), b"second channel".as_slice()] {
        let mut stream = session.connect(Some(socket)).await?;
        stream.write_all(message).await?;
        stream.flush().await?;
        let mut echoed = vec![0; message.len()];
        tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut echoed)).await??;
        assert_eq!(echoed, message);
        stream.shutdown().await?;
    }
    let observed = std::fs::read_to_string(commands)?;
    let expected = "'docker' '--host' 'unix:///run/socket with '\\''quote'\\'';not-a-command.sock' 'system' 'dial-stdio'";
    assert_eq!(
        observed.lines().collect::<Vec<_>>(),
        vec![expected, expected]
    );
    std::fs::remove_file(socket_path(session.directory.path()))?;
    let mut disconnected = session.connect(None).await?;
    let mut byte = [0];
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(5), disconnected.read(&mut byte)).await??,
        0
    );
    session.shutdown().await?;
    assert_eq!(
        session.permits.available_permits(),
        super::super::CONNECTIONS
    );
    assert!(session.channels.lock().await.owned.tasks.is_empty());
    assert!(session.connect(None).await.is_err());
    tokio::time::timeout(Duration::from_secs(5), server).await???;
    Ok(())
}
