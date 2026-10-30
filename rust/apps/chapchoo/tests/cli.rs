#![cfg(unix)]

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::accounts::{
    InitializeRequest, IssueGrantRequest, RedeemSignupRequest, admin_client::AdminClient,
    auth_client::AuthClient, general_client::GeneralClient,
};
use std::{error::Error, net::TcpListener, path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use tonic_health::pb::{
    HealthCheckRequest, health_check_response::ServingStatus, health_client::HealthClient,
};

type TestError = Box<dyn Error + Send + Sync>;

fn command(path: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_chapchoo"));
    command
        .env_clear()
        .env("CHAPCHOO_SQLITE_PATH", path)
        .env("CHAPCHOO_SECRET_KEY", URL_SAFE.encode([1; 32]))
        .env("RUST_LOG", "chapchoo=info")
        .env("NO_COLOR", "1");
    command
}

async fn ready(child: &mut Child) -> Result<(), TestError> {
    let output = child.stdout.take().ok_or("Missing child output")?;
    let mut lines = BufReader::new(output).lines();
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(line) = lines.next_line().await? {
            if line.contains("Selected backend roles are ready") {
                return Ok::<_, TestError>(());
            }
        }
        Err("Backend exited before readiness".into())
    })
    .await?
}

async fn terminate(child: &mut Child) -> Result<(), TestError> {
    let id = child.id().ok_or("Missing owned process")?;
    let status = Command::new("kill")
        .args(["-TERM", &id.to_string()])
        .status()
        .await?;
    assert!(status.success());
    assert!(
        tokio::time::timeout(Duration::from_secs(10), child.wait())
            .await??
            .success()
    );
    Ok(())
}

#[tokio::test]
async fn secret_key_name_is_required_without_previous_name_fallback() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("rename.sqlite");
    let failed = command(&path)
        .env_remove("CHAPCHOO_SECRET_KEY")
        .env("CHAPCHOO_INSTALLATION_KEY", URL_SAFE.encode([1; 32]))
        .arg("prepare")
        .output()
        .await?;
    assert!(!failed.status.success());
    assert!(!path.exists());
    let prepared = command(&path).arg("prepare").output().await?;
    assert!(prepared.status.success());
    Ok(())
}

#[tokio::test]
async fn prepare_readiness_common_auth_recovery_and_owned_shutdown() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("cli.sqlite");
    // Ordinary startup does not create a missing schema.
    let failed = command(&path)
        .args(["serve", "--role", "general"])
        .output()
        .await?;
    assert!(!failed.status.success());
    let prepared = command(&path).arg("prepare").output().await?;
    assert!(prepared.status.success());
    let wrong_key = command(&path)
        .env("CHAPCHOO_SECRET_KEY", URL_SAFE.encode([2; 32]))
        .args(["serve", "--role", "admin"])
        .output()
        .await?;
    assert!(!wrong_key.status.success());
    let unavailable = command(&path)
        .args(["grant-admin", "--email", "absent@example.invalid"])
        .output()
        .await?;
    assert!(!unavailable.status.success());
    let general = TcpListener::bind("127.0.0.1:0")?;
    let admin = TcpListener::bind("127.0.0.1:0")?;
    let general_addr = general.local_addr()?;
    let admin_addr = admin.local_addr()?;
    drop((general, admin));
    let proof = BearerSecret::generate()?;
    let mut child = command(&path)
        .env("CHAPCHOO_SETUP_PROOF", proof.expose())
        .args([
            "serve",
            "--general-listen",
            &general_addr.to_string(),
            "--admin-listen",
            &admin_addr.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    ready(&mut child).await?;
    let mut health = HealthClient::new(
        tonic::transport::Endpoint::from_shared(format!("http://{general_addr}"))?
            .connect()
            .await?,
    );
    assert_eq!(
        health
            .check(HealthCheckRequest {
                service: String::new()
            })
            .await?
            .into_inner()
            .status,
        ServingStatus::Serving as i32
    );
    assert_eq!(
        health
            .check(HealthCheckRequest {
                service: "chapchoo.liveness".to_owned()
            })
            .await?
            .into_inner()
            .status,
        ServingStatus::Serving as i32
    );
    let offline = path.with_extension("offline");
    std::fs::rename(&path, &offline)?;
    let unavailable = health
        .check(HealthCheckRequest {
            service: String::new(),
        })
        .await?;
    assert_eq!(
        unavailable.into_inner().status,
        ServingStatus::NotServing as i32
    );
    assert_eq!(
        health
            .check(HealthCheckRequest {
                service: "chapchoo.liveness".to_owned()
            })
            .await?
            .into_inner()
            .status,
        ServingStatus::Serving as i32
    );
    std::fs::rename(&offline, &path)?;
    let mut admin = AdminClient::connect(format!("http://{admin_addr}")).await?;
    let session = admin
        .initialize(InitializeRequest {
            installation_proof: proof.expose().to_owned(),
            email: "operator@example.invalid".to_owned(),
            password: "Aa1!operator".to_owned(),
        })
        .await?
        .into_inner();
    let mut auth = AuthClient::connect(format!("http://{general_addr}")).await?;
    let mut request = tonic::Request::new(());
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", session.token).parse()?,
    );
    assert!(auth.identity(request).await?.into_inner().is_admin);
    let mut issue = tonic::Request::new(IssueGrantRequest {
        email: "recover@example.invalid".to_owned(),
    });
    issue.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", session.token).parse()?,
    );
    let proof = admin.issue_signup_grant(issue).await?.into_inner().proof;
    let mut general = GeneralClient::connect(format!("http://{general_addr}")).await?;
    let account = general
        .redeem_signup(RedeemSignupRequest {
            proof,
            email: "recover@example.invalid".to_owned(),
            password: "Aa1!recovery".to_owned(),
        })
        .await?
        .into_inner();
    assert!(account.user.as_ref().is_some_and(|user| !user.is_admin));
    let recover = command(&path)
        .args(["grant-admin", "--email", "RECOVER@example.invalid"])
        .output()
        .await?;
    assert!(recover.status.success());
    let mut request = tonic::Request::new(());
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", account.token).parse()?,
    );
    assert!(auth.identity(request).await?.into_inner().is_admin);
    terminate(&mut child).await?;
    assert!(TcpListener::bind(general_addr).is_ok());
    assert!(TcpListener::bind(admin_addr).is_ok());
    Ok(())
}

#[tokio::test]
async fn failed_combined_bind_preserves_foreign_listener_and_releases_owned_socket()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("failure.sqlite");
    assert!(
        command(&path)
            .arg("prepare")
            .output()
            .await?
            .status
            .success()
    );
    let general = TcpListener::bind("127.0.0.1:0")?;
    let general_addr = general.local_addr()?;
    drop(general);
    let foreign = TcpListener::bind("127.0.0.1:0")?;
    let foreign_addr = foreign.local_addr()?;
    let failed = tokio::time::timeout(
        Duration::from_secs(15),
        command(&path)
            .args([
                "serve",
                "--general-listen",
                &general_addr.to_string(),
                "--admin-listen",
                &foreign_addr.to_string(),
            ])
            .output(),
    )
    .await??;
    assert!(!failed.status.success());
    assert!(TcpListener::bind(general_addr).is_ok());
    assert!(std::net::TcpStream::connect(foreign_addr).is_ok());
    Ok(())
}
