//! Actual child-process Docker entrypoint with isolated SDK HTTP and Control.
use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn external_docker_provider_reaches_scoped_control_without_db_root_or_ambient_context()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    test_support::scope(async |fixture| {
    let native = TcpListener::bind("127.0.0.1:0").await?;
    let host = format!("tcp://{}", native.local_addr()?);
    let mut native_stopped = fixture.shutdown();
    fixture.server(async move {
        let mut tasks = tokio::task::JoinSet::new();
        let primary = async {
        loop {
            tokio::select! {
                _ = &mut native_stopped => break,
                accepted = native.accept() => {
                    let (mut socket, _) = accepted?;
                    tasks.spawn(async move {
                        let mut request = Vec::new(); let mut byte = [0];
                        while !request.ends_with(b"\r\n\r\n") {
                            socket.read_exact(&mut byte).await?; request.extend_from_slice(&byte);
                            if request.len() > 16*1024 { return Err::<(),Error>("Unbounded native fixture request".into()); }
                        }
                        let request = std::str::from_utf8(&request)?;
                        if !request.starts_with("GET ") { return Err("Native mutation before registration".into()); }
                        let payload = if request.lines().next().is_some_and(|line| line.contains("/version ")) {
                            br#"{"ApiVersion":"1.53","MinAPIVersion":"1.24"}"#.as_slice()
                        } else if request.lines().next().is_some_and(|line| line.contains("/info ")) {
                            br#"{"ID":"external:docker-daemon","OSType":"linux","Architecture":"aarch64","MemoryLimit":true,"CpuCfsPeriod":true,"CpuCfsQuota":true,"SecurityOptions":["name=seccomp,profile=builtin"]}"#.as_slice()
                        } else { return Err("Unexpected native startup API".into()); };
                        socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n",payload.len()).as_bytes()).await?;
                        socket.write_all(payload).await?;
                        socket.shutdown().await?;
                        Ok(())
                    });
                }
            }
        }
        Ok::<(), Error>(())
        }.await;
        let cleanup = test_support::requests(&mut tasks).await;
        test_support::completed(primary, cleanup)
    });
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let owner = BearerSecret::generate()?;
    let token = owner.expose().to_owned();
    let credential_file=dir.path().join("provider-credential");
    std::fs::write(&credential_file,&token)?;
    let (configured, mut received) = mpsc::channel(2);
    let stopped = fixture.shutdown();
    fixture.server(
        Server::builder()
            .add_service(ProviderAuthenticationServer::new(ControlBoundary {
                configured,
                owner,
            }))
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = stopped.await;
            }),
    );
    let db_canary = dir.path().join("product-db-must-not-exist.sqlite");
    for poisoned in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_chapchoo"));
        command
            .env_clear()
            .env("CHAPCHOO_PROVIDER_AUTH_METHOD", "random-token")
            .env("CHAPCHOO_PROVIDER_AUTH_FILE", &credential_file)
            .env("DOCKER_HOST", "invalid-ambient-host")
            .env("DOCKER_CONTEXT", "unavailable-ambient-context")
            .env("DOCKER_TLS_VERIFY", "1")
            .arg("docker-provider")
            .arg("--endpoint")
            .arg(&endpoint)
            .arg("--provider-id")
            .arg("external-docker")
            .arg("--name")
            .arg("External Docker fixture")
            .arg("--image")
            .arg("fixture.invalid/computer:current")
            .arg("--docker-host")
            .arg(&host)
            .arg("--connector-endpoint")
            .arg("http://container-control.invalid:50051")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if poisoned {
            command
                .env("CHAPCHOO_SQLITE_PATH", &db_canary)
                .env("CHAPCHOO_POSTGRES_URL", "invalid-product-db")
                .env("CHAPCHOO_SECRET_KEY", "invalid-root");
        }
        let child = fixture.child(&mut command)?;
        let configured = tokio::time::timeout(Duration::from_secs(10), received.recv())
            .await?
            .ok_or("Docker Provider did not reach Control")?;
        assert_eq!(configured.provider_id, "external-docker");
        assert_eq!(configured.kind, wire::ProviderKind::Docker as i32);
        assert_eq!(
            configured.docker_daemon_id.as_deref(),
            Some("external:docker-daemon")
        );
        assert!(
            !tokio::time::timeout(Duration::from_secs(10), child.lock().await.wait())
                .await??
                .success()
        );
        assert!(!db_canary.exists());
    }
    Ok(())
    }).await
}
