#![cfg(target_os = "linux")]

use base64::{Engine, engine::general_purpose::URL_SAFE};
use nix::{
    errno::Errno,
    sys::signal::{Signal, kill, killpg},
    unistd::Pid,
};
use std::{
    error::Error,
    io::Write,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdout, Command},
};

type TestError = Box<dyn Error + Send + Sync>;
#[path = "../test_support.rs"]
mod test_support;

struct Bundle {
    directory: tempfile::TempDir,
    binary: PathBuf,
}

impl Bundle {
    fn new(mode: &str) -> Result<Self, TestError> {
        let directory = tempfile::tempdir()?;
        let root = directory.path();
        let binary = root.join("chapchoo");
        std::fs::copy(env!("CARGO_BIN_EXE_chapchoo"), &binary)?;
        let helper = root.join("fixture-helper");
        std::fs::copy(std::env::current_exe()?, &helper)?;
        let node = root.join("runtime/bin/node");
        std::fs::create_dir_all(node.parent().ok_or("Missing fixture Node parent")?)?;
        // This synthetic lower-layer helper is not a real Node/package test.
        let escaped = helper.display().to_string().replace('\'', "'\\''");
        std::fs::write(
            &node,
            format!("#!/bin/sh\nexec '{escaped}' --exact fixture_web_process --nocapture\n"),
        )?;
        std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o755))?;
        for app in [
            "web/general/typescript/apps/web",
            "web/admin/typescript/apps/admin-web",
        ] {
            let app = root.join(app);
            std::fs::create_dir_all(&app)?;
            std::fs::write(app.join("server.mjs"), mode)?;
        }
        Ok(Self { directory, binary })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .env_clear()
            .env("TMPDIR", self.directory.path())
            .env("RUST_LOG", "chapchoo=info")
            .env("NO_COLOR", "1")
            .env("CHAPCHOO_GENERAL_BACKEND_URL", "http://127.0.0.1:50051")
            .env("CHAPCHOO_ADMIN_BACKEND_URL", "http://127.0.0.1:50051")
            // Web-only must not parse these or forward them to its child.
            .env("CHAPCHOO_SECRET_KEY", "parent-key-canary")
            .env("CHAPCHOO_POSTGRES_URL", "parent-database-canary")
            .env("CHAPCHOO_SETUP_PROOF", "parent-proof-canary")
            .env("AWS_SECRET_ACCESS_KEY", "parent-aws-canary")
            .env("NODE_OPTIONS", "--require parent-preload-canary")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn child_group(&self, port: u16) -> Result<Pid, TestError> {
        let value = std::fs::read_to_string(self.directory.path().join(format!("web-{port}.pid")))?;
        Ok(Pid::from_raw(value.parse()?))
    }
}

struct OwnedRoot(Child);

impl OwnedRoot {
    async fn run(
        child: Child,
        exercise: impl AsyncFnOnce(&mut Child) -> Result<(), TestError>,
    ) -> Result<(), TestError> {
        let mut root = Self(child);
        let primary = exercise(&mut root.0).await;
        let cleanup = async {
            let observed = root.0.try_wait();
            if matches!(observed, Ok(Some(_))) {
                return Ok(());
            }
            // Keep product-coordinated group cleanup on normal Result failure.
            // A deadline is not evidence that the root or descendants stopped.
            let signal = (|| -> Result<(), TestError> {
                let id = i32::try_from(root.0.id().ok_or("Missing owned root PID")?)?;
                kill(Pid::from_raw(id), Signal::SIGTERM)?;
                Ok(())
            })();
            let preparation =
                test_support::completed(observed.map(|_| ()).map_err(Into::into), signal);
            let waited = match tokio::time::timeout(Duration::from_secs(30), root.0.wait()).await {
                Ok(result) => result.map(|_| ()).map_err(Into::into),
                Err(deadline) => {
                    let completion = root.0.wait().await.map(|_| ()).map_err(Into::into);
                    test_support::completed(Err(deadline.into()), completion)
                }
            };
            test_support::completed(preparation, waited)
        }
        .await;
        test_support::completed(primary, cleanup)
    }

    async fn output(
        command: &mut Command,
        deadline: Duration,
    ) -> Result<std::process::Output, TestError> {
        let mut captured = None;
        Self::run(command.spawn()?, async |child| {
            let mut stdout = child.stdout.take().ok_or("Missing stdout")?;
            let mut stderr = child.stderr.take().ok_or("Missing stderr")?;
            let mut out = Vec::new();
            let mut err = Vec::new();
            let status = tokio::time::timeout(deadline, async {
                let (status, output, errors) = tokio::join!(
                    child.wait(),
                    stdout.read_to_end(&mut out),
                    stderr.read_to_end(&mut err)
                );
                let reads = test_support::completed(
                    output.map(|_| ()).map_err(Into::into),
                    errors.map(|_| ()).map_err(Into::into),
                );
                test_support::completed(status.map_err(Into::into), reads)
            })
            .await??;
            captured = Some(std::process::Output {
                status,
                stdout: out,
                stderr: err,
            });
            Ok(())
        })
        .await?;
        captured.ok_or_else(|| "Missing captured root output".into())
    }
}

impl Drop for OwnedRoot {
    fn drop(&mut self) {
        // Failure fallback requests the product's own coordinated cleanup; do
        // not SIGKILL its parent and strand the product's owned web groups.
        if let Some(id) = self.0.id()
            && let Ok(id) = i32::try_from(id)
        {
            let _ = kill(Pid::from_raw(id), Signal::SIGTERM);
        }
    }
}

fn port() -> Result<u16, TestError> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

async fn ready(child: &mut Child) -> Result<BufReader<ChildStdout>, TestError> {
    let mut reader = BufReader::new(child.stdout.take().ok_or("Missing owned root output")?);
    tokio::time::timeout(Duration::from_secs(15), async {
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).await? == 0 {
                return Err("Owned root exited before selected-component readiness".into());
            }
            if line.contains("Selected components are ready") {
                return Ok::<_, TestError>(());
            }
        }
    })
    .await??;
    Ok(reader)
}

async fn stop(child: &mut Child, deadline: u64) -> Result<std::process::ExitStatus, TestError> {
    let id = i32::try_from(child.id().ok_or("Missing owned root PID")?)?;
    kill(Pid::from_raw(id), Signal::SIGTERM)?;
    Ok(tokio::time::timeout(Duration::from_secs(deadline), child.wait()).await??)
}

fn group_gone(group: Pid) -> Result<bool, TestError> {
    match killpg(group, None) {
        Err(Errno::ESRCH) => Ok(true),
        Ok(()) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[tokio::test]
async fn ordinary_exercise_error_waits_for_root_and_owned_group_cleanup() -> Result<(), TestError> {
    let bundle = Bundle::new("serve")?;
    let web_port = port()?;
    let mut group = None;
    let failure = OwnedRoot::run(
        bundle
            .command()
            .args([
                "serve",
                "--groups",
                "--web",
                "general",
                "--general-web-listen",
                &format!("127.0.0.1:{web_port}"),
            ])
            .spawn()?,
        async |root| {
            let _output = ready(root).await?;
            group = Some(bundle.child_group(web_port)?);
            Err("Primary web exercise failure".into())
        },
    )
    .await
    .err()
    .ok_or("Expected exercise failure")?;
    assert_eq!(failure.to_string(), "Primary web exercise failure");
    assert!(group_gone(group.ok_or("Missing owned web group")?)?);
    let _released = TcpListener::bind(("127.0.0.1", web_port))?;
    Ok(())
}

#[tokio::test]
async fn web_only_needs_no_database_key_or_host_node_and_stops_its_group() -> Result<(), TestError>
{
    let bundle = Bundle::new("serve")?;
    let web_port = port()?;
    OwnedRoot::run(
        bundle
            .command()
            .args([
                "serve",
                "--groups",
                "--web",
                "general",
                "--general-web-listen",
                &format!("127.0.0.1:{web_port}"),
            ])
            .spawn()?,
        async |root| {
            let _output = ready(root).await?;
            let group = bundle.child_group(web_port)?;
            let status = stop(root, 10).await?;
            assert!(status.success());
            assert!(group_gone(group)?);
            Ok(())
        },
    )
    .await
}

#[tokio::test]
async fn forced_cleanup_reaps_adopted_descendants_and_reports_failure() -> Result<(), TestError> {
    let bundle = Bundle::new("leak-descendant")?;
    let web_port = port()?;
    OwnedRoot::run(
        bundle
            .command()
            .args([
                "serve",
                "--groups",
                "--web",
                "general",
                "--general-web-listen",
                &format!("127.0.0.1:{web_port}"),
            ])
            .spawn()?,
        async |root| {
            let _output = ready(root).await?;
            let group = bundle.child_group(web_port)?;
            let status = stop(root, 30).await?;
            assert!(!status.success());
            assert!(group_gone(group)?);
            Ok(())
        },
    )
    .await
}

#[tokio::test]
async fn occupied_foreign_web_listener_cannot_satisfy_owned_startup() -> Result<(), TestError> {
    let bundle = Bundle::new("serve")?;
    test_support::scope(async |fixture| {
    let foreign = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = foreign.local_addr()?;
    let mut shutdown = fixture.shutdown();
    fixture.server(async move {
        let mut requests = tokio::task::JoinSet::new();
        let primary = async {
        loop {
            let (mut stream, _) = tokio::select! {
                _ = &mut shutdown => break,
                accepted = foreign.accept() => accepted?,
            };
            requests.spawn(async move {
                let mut buffer = [0; 1024];
                // A caller checking only listener reachability may close without
                // requesting HTTP. No reply is owed on that deliberate probe.
                if stream.read(&mut buffer).await? > 0 {
                    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").await?;
                }
                Ok::<(), TestError>(())
            });
        }
        Ok::<(), TestError>(())
        }.await;
        let cleanup = test_support::requests(&mut requests).await;
        test_support::completed(primary, cleanup)
    });
        let output = OwnedRoot::output(
            bundle.command()
                .args([
                    "serve",
                    "--groups",
                    "--web",
                    "general",
                    "--general-web-listen",
                    &address.to_string(),
                ]),
            Duration::from_secs(15),
        )
        .await?;
        let stream = tokio::net::TcpStream::connect(address).await?;
        drop(stream);
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("Selected components are ready"));
    Ok(())
    }).await
}

#[tokio::test]
async fn first_ready_web_exit_is_supervised_while_other_web_starts() -> Result<(), TestError> {
    for first in ["general", "admin"] {
        let bundle = Bundle::new("wait-for-termination")?;
        let first_app = if first == "general" {
            "web/general/typescript/apps/web/server.mjs"
        } else {
            "web/admin/typescript/apps/admin-web/server.mjs"
        };
        std::fs::write(bundle.directory.path().join(first_app), "serve")?;
        let general_port = port()?;
        let mut admin_port = port()?;
        while admin_port == general_port {
            admin_port = port()?;
        }
        OwnedRoot::run(
            bundle
                .command()
                .env("RUST_LOG", "chapchoo=debug")
                .args([
                    "serve",
                    "--groups",
                    "--web",
                    "both",
                    "--general-web-listen",
                    &format!("127.0.0.1:{general_port}"),
                    "--admin-web-listen",
                    &format!("127.0.0.1:{admin_port}"),
                ])
                .spawn()?,
            async |root| {
                let mut reader = BufReader::new(root.stdout.take().ok_or("Missing root output")?);
                // Product-side readiness is an explicit barrier, not a timing sleep.
                tokio::time::timeout(Duration::from_secs(10), async {
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).await? == 0 {
                            return Err("Root exited before first-child readiness".into());
                        }
                        assert!(!line.contains("Selected components are ready"));
                        if line.contains("Selected web process is ready") {
                            assert!(line.contains(if first == "general" {
                                "General"
                            } else {
                                "Admin"
                            }));
                            return Ok::<_, TestError>(());
                        }
                    }
                })
                .await??;
                let waiting_port = if first == "general" {
                    admin_port
                } else {
                    general_port
                };
                let barrier = bundle
                    .directory
                    .path()
                    .join(format!("waiting-{waiting_port}"));
                tokio::time::timeout(Duration::from_secs(10), async {
                    while !barrier.exists() {
                        tokio::time::sleep(Duration::from_millis(25)).await;
                    }
                })
                .await?;
                let group = bundle.child_group(if first == "general" {
                    general_port
                } else {
                    admin_port
                })?;
                kill(group, Signal::SIGTERM)?;
                let mut remaining = String::new();
                // Observe failure promptly, separately from the approved20s+5s drain.
                // Waiting for full process exit in5s incorrectly shortened that budget.
                let observed = tokio::time::timeout(Duration::from_secs(10), async {
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).await? == 0 {
                            return Ok::<_, TestError>(false);
                        }
                        remaining.push_str(&line);
                        if line.contains("A selected web process exited unexpectedly") {
                            return Ok(true);
                        }
                    }
                })
                .await
                .unwrap_or(Ok(false))?;
                let status = if observed {
                    tokio::time::timeout(Duration::from_secs(30), root.wait()).await??
                } else {
                    stop(root, 30).await?
                };
                reader.read_to_string(&mut remaining).await?;
                assert!(
                    observed,
                    "First-ready exit waited for the other startup deadline ({first}): {remaining}"
                );
                assert!(!status.success());
                assert!(!remaining.contains("Selected components are ready"));
                assert!(group_gone(bundle.child_group(general_port)?)?);
                assert!(group_gone(bundle.child_group(admin_port)?)?);
                Ok(())
            },
        )
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn pending_health_readiness_survives_the_other_web_becoming_ready() -> Result<(), TestError> {
    let bundle = Bundle::new("wait-for-health-release")?;
    let general_port = port()?;
    let mut admin_port = port()?;
    while admin_port == general_port {
        admin_port = port()?;
    }
    OwnedRoot::run(
        bundle
            .command()
            .env("RUST_LOG", "chapchoo=debug")
            .args([
                "serve",
                "--groups",
                "--web",
                "both",
                "--general-web-listen",
                &format!("127.0.0.1:{general_port}"),
                "--admin-web-listen",
                &format!("127.0.0.1:{admin_port}"),
            ])
            .spawn()?,
        async |root| {
            let mut reader = BufReader::new(root.stdout.take().ok_or("Missing root output")?);
            // Both Ready oneshots have been consumed before either health completes.
            // Release General alone, forcing the first-completion branch to preserve
            // Admin's already-running health future rather than restart its receiver.
            for web_port in [general_port, admin_port] {
                let barrier = bundle
                    .directory
                    .path()
                    .join(format!("health-waiting-{web_port}"));
                tokio::time::timeout(Duration::from_secs(10), async {
                    while !barrier.exists() {
                        tokio::time::sleep(Duration::from_millis(25)).await;
                    }
                })
                .await?;
            }
            std::fs::write(
                bundle
                    .directory
                    .path()
                    .join(format!("health-release-{general_port}")),
                b"release",
            )?;
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).await? == 0 {
                        return Err("Root exited before the fast web became ready".into());
                    }
                    if line.contains("Selected web process is ready") {
                        assert!(line.contains("General"));
                        return Ok::<_, TestError>(());
                    }
                }
            })
            .await??;
            std::fs::write(
                bundle
                    .directory
                    .path()
                    .join(format!("health-release-{admin_port}")),
                b"release",
            )?;
            tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).await? == 0 {
                        return Err("Root discarded the pending web readiness future".into());
                    }
                    if line.contains("Selected components are ready") {
                        return Ok::<_, TestError>(());
                    }
                }
            })
            .await??;
            assert!(stop(root, 10).await?.success());
            assert!(group_gone(bundle.child_group(general_port)?)?);
            assert!(group_gone(bundle.child_group(admin_port)?)?);
            Ok(())
        },
    )
    .await
}

#[tokio::test]
async fn partial_web_failure_releases_only_its_co_selected_backend() -> Result<(), TestError> {
    let bundle = Bundle::new("exit")?;
    let data = bundle.directory.path().join("installation.sqlite");
    let key = URL_SAFE.encode([1; 32]);
    let mut prepare = Command::new(&bundle.binary);
    let prepared = prepare
        .env_clear()
        .env("CHAPCHOO_SQLITE_PATH", &data)
        .env("CHAPCHOO_SECRET_KEY", &key)
        .arg("prepare")
        .output()
        .await?;
    assert!(prepared.status.success());
    let backend_port = port()?;
    let web_port = port()?;
    let foreign = TcpListener::bind("127.0.0.1:0")?;
    let foreign_address = foreign.local_addr()?;
    let mut command = bundle.command();
    command
        .env_remove("CHAPCHOO_POSTGRES_URL")
        .env_remove("CHAPCHOO_SETUP_PROOF")
        .env("CHAPCHOO_SQLITE_PATH", &data)
        .env("CHAPCHOO_SECRET_KEY", key);
    let output = OwnedRoot::output(
        command.args([
            "serve",
            "--groups",
            "client",
            "--web",
            "general",
            "--listen",
            &format!("127.0.0.1:{backend_port}"),
            "--general-web-listen",
            &format!("127.0.0.1:{web_port}"),
        ]),
        Duration::from_secs(15),
    )
    .await?;
    assert!(!output.status.success());
    let released = TcpListener::bind(("127.0.0.1", backend_port))?;
    assert_eq!(released.local_addr()?.port(), backend_port);
    assert_eq!(foreign.local_addr()?, foreign_address);
    assert!(TcpListener::bind(foreign_address).is_err());
    assert!(group_gone(bundle.child_group(web_port)?)?);
    Ok(())
}

#[tokio::test]
async fn empty_selection_fails_without_reading_database_configuration() -> Result<(), TestError> {
    let output = Command::new(env!("CARGO_BIN_EXE_chapchoo"))
        .env_clear()
        .args(["serve", "--groups"])
        .output()
        .await?;
    assert!(!output.status.success());
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("NoComponents"));
    assert!(!log.contains("DatabaseSelection"));
    Ok(())
}

// Synthetic child fixtures are run only through the private bundle's exact
// subprocess command. Ordinary cargo execution of these tests is a no-op.
fn fixture_requested(name: &str) -> bool {
    let arguments: Vec<_> = std::env::args().collect();
    arguments.windows(2).any(|pair| pair == ["--exact", name])
        && std::env::var_os("CHAPCHOO_PUBLIC_URL").is_some()
        && Path::new("server.mjs").is_file()
}

#[tokio::test]
async fn fixture_web_process() -> Result<(), TestError> {
    if !fixture_requested("fixture_web_process") {
        return Ok(());
    }
    for name in [
        "CHAPCHOO_SECRET_KEY",
        "CHAPCHOO_POSTGRES_URL",
        "CHAPCHOO_SETUP_PROOF",
        "AWS_SECRET_ACCESS_KEY",
        "NODE_OPTIONS",
    ] {
        if std::env::var_os(name).is_some() {
            return Err("Backend or preload authority leaked to an owned web child".into());
        }
    }
    let web_port = std::env::var("PORT")?.parse::<u16>()?;
    let temporary =
        PathBuf::from(std::env::var_os("TMPDIR").ok_or("Missing fixture temporary path")?);
    std::fs::write(
        temporary.join(format!("web-{web_port}.pid")),
        std::process::id().to_string(),
    )?;
    let mode = std::fs::read_to_string("server.mjs")?;
    if mode == "exit" {
        writeln!(std::io::stdout(), "Ready in fixture")?;
        return Ok(());
    }
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", web_port)).await?;
    let mut termination =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    if mode == "wait-for-termination" {
        // Intentionally never report ready; the sibling's exit must still be
        // observed promptly, and coordinated cleanup must terminate this child.
        std::fs::write(temporary.join(format!("waiting-{web_port}")), b"waiting")?;
        termination.recv().await;
        return Ok(());
    }
    let mut descendant = if mode == "leak-descendant" {
        let mut child = Command::new(std::env::current_exe()?)
            .args(["--exact", "fixture_group_descendant", "--nocapture"])
            .stdout(Stdio::piped())
            .spawn()?;
        let mut output = BufReader::new(
            child
                .stdout
                .take()
                .ok_or("Missing descendant readiness output")?,
        )
        .lines();
        tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(line) = output.next_line().await? {
                if line.contains("Descendant ready") {
                    return Ok::<_, TestError>(());
                }
            }
            Err("Owned fixture descendant exited before readiness".into())
        })
        .await??;
        Some(child)
    } else {
        None
    };
    writeln!(std::io::stdout(), "Ready in fixture")?;
    loop {
        tokio::select! {
            _ = termination.recv() => {
                // Deliberately do not reap the TERM-resistant child: the
                // product must adopt and reap this same-group descendant.
                drop(descendant.take());
                return Ok(());
            }
            accepted = listener.accept() => {
                let (mut stream, _) = accepted?;
                let mut buffer = [0; 1024];
                if stream.read(&mut buffer).await? == 0 {
                    continue;
                }
                if mode == "wait-for-health-release" {
                    std::fs::write(temporary.join(format!("health-waiting-{web_port}")), b"waiting")?;
                    let release = temporary.join(format!("health-release-{web_port}"));
                    while !release.exists() {
                        tokio::select! {
                            _ = termination.recv() => return Ok(()),
                            _ = tokio::time::sleep(Duration::from_millis(25)) => {},
                        }
                    }
                }
                // A cancelled/expired startup probe need not stop the fixture.
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").await;
            }
        }
    }
}

#[tokio::test]
async fn fixture_group_descendant() -> Result<(), TestError> {
    if !fixture_requested("fixture_group_descendant") {
        return Ok(());
    }
    let mut termination =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    writeln!(std::io::stdout(), "Descendant ready")?;
    termination.recv().await;
    std::future::pending::<()>().await;
    Ok(())
}
