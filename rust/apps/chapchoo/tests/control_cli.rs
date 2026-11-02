#![cfg(unix)]
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::{
    accounts::authentication_client::AuthenticationClient,
    control::{
        self as control, peer_relay_client::PeerRelayClient,
        provider_authentication_client::ProviderAuthenticationClient,
    },
};
use std::{net::TcpListener, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
type Error = Box<dyn std::error::Error + Send + Sync>;
#[tokio::test]
async fn control_group_has_components_and_separate_network_peer_listener() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("control.sqlite");
    let command = || {
        let mut c = Command::new(env!("CARGO_BIN_EXE_chapchoo"));
        c.env_clear()
            .env("CHAPCHOO_SQLITE_PATH", &path)
            .env("CHAPCHOO_SECRET_KEY", URL_SAFE.encode([1; 32]))
            .env("RUST_LOG", "chapchoo=info")
            .env("NO_COLOR", "1");
        c
    };
    assert!(command().arg("prepare").output().await?.status.success());
    let public = TcpListener::bind("127.0.0.1:0")?;
    let peer = TcpListener::bind("127.0.0.1:0")?;
    let public_addr = public.local_addr()?;
    let peer_addr = peer.local_addr()?;
    drop((public, peer));
    let owner = BearerSecret::generate()?;
    let mut child = command()
        .env("CHAPCHOO_SYSTEM_OWNER_ID", "fixture-owner")
        .env("CHAPCHOO_SYSTEM_OWNER_PROVIDER_IDS", "provider")
        .env("CHAPCHOO_SYSTEM_OWNER_TOKEN", owner.expose())
        .args([
            "serve",
            "--groups",
            "control",
            "--listen",
            &public_addr.to_string(),
            "--peer-listen",
            &peer_addr.to_string(),
            "--peer-advertise",
            &format!("http://{peer_addr}"),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut lines = BufReader::new(child.stdout.take().ok_or("Missing output")?).lines();
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(line) = lines.next_line().await? {
            if line.contains("Selected backend service groups are ready") {
                return Ok::<(), Error>(());
            }
        }
        Err("Control startup failed".into())
    })
    .await??;
    let public_url = format!("http://{public_addr}");
    let peer_url = format!("http://{peer_addr}");
    assert_eq!(
        AuthenticationClient::connect(public_url.clone())
            .await?
            .methods(())
            .await
            .err()
            .ok_or("Unexpected Client group")?
            .code(),
        tonic::Code::Unimplemented
    );
    assert_eq!(
        PeerRelayClient::connect(public_url.clone())
            .await?
            .invoke_provider(control::ProviderInvocation::default())
            .await
            .err()
            .ok_or("Public peer leak")?
            .code(),
        tonic::Code::Unimplemented
    );
    assert_eq!(
        PeerRelayClient::connect(peer_url.clone())
            .await?
            .invoke_provider(control::ProviderInvocation::default())
            .await
            .err()
            .ok_or("Missing peer validation")?
            .code(),
        tonic::Code::DeadlineExceeded
    );
    assert_eq!(
        ProviderAuthenticationClient::connect(peer_url)
            .await?
            .check_connection(control::ProviderConnectionCheck::default())
            .await
            .err()
            .ok_or("Peer component leak")?
            .code(),
        tonic::Code::Unimplemented
    );
    let mut provider = ProviderAuthenticationClient::connect(public_url).await?;
    let configuration = control::ProviderConfiguration {
        provider_id: "provider".into(),
        name: "Fixture".into(),
        capabilities: Some(control::ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        }),
    };
    let authenticated = |value| -> Result<tonic::Request<control::ProviderConfiguration>, Error> {
        let mut r = tonic::Request::new(value);
        r.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", owner.expose()).parse()?,
        );
        Ok(r)
    };
    provider
        .configure_system_provider(authenticated(configuration.clone())?)
        .await?;
    let mut foreign = configuration;
    foreign.provider_id = "unscoped-provider".into();
    assert_eq!(
        provider
            .configure_system_provider(authenticated(foreign)?)
            .await
            .err()
            .ok_or("Unscoped Provider accepted")?
            .code(),
        tonic::Code::PermissionDenied
    );
    drop(provider);
    let id = i32::try_from(child.id().ok_or("Missing PID")?)?;
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(id),
        nix::sys::signal::Signal::SIGTERM,
    )?;
    assert!(
        tokio::time::timeout(Duration::from_secs(10), child.wait())
            .await??
            .success()
    );
    assert!(TcpListener::bind(public_addr).is_ok());
    assert!(TcpListener::bind(peer_addr).is_ok());
    Ok(())
}
