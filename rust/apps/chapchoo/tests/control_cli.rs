#![cfg(unix)]
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
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
#[path = "../test_support.rs"]
mod test_support;
#[tokio::test]
async fn control_group_has_components_and_separate_network_peer_listener() -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    test_support::scope(async |fixture| {
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
        let owner = CredentialCipher::new(RootKey::parse(URL_SAFE.encode([1; 32]))?)?
            .issue_provider_bootstrap("provider")?;
        let child = fixture.child(
            command()
                .env(
                    "CHAPCHOO_PROVIDER_BOOTSTRAP_CONFIG",
                    r#"{"secretKeyProviderIds":["provider"]}"#,
                )
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
                .kill_on_drop(true),
        )?;
        let mut lines =
            BufReader::new(child.lock().await.stdout.take().ok_or("Missing output")?).lines();
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
            registration_key: "provider".into(),
            name: "Fixture".into(),
            kind: control::ProviderKind::Kubernetes as i32,
            docker_daemon_id: None,
            capabilities: Some(control::ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            }),
        };
        let request_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();
        let authenticated = |value, request_id: &str| control::ProviderAuthenticationRequest {
            configuration: Some(value),
            request_id: format!("{request_time}:{request_id}"),
            evidence: Some(
                control::provider_authentication_request::Evidence::BootstrapCredential(
                    owner.expose().to_owned(),
                ),
            ),
        };
        provider
            .exchange(authenticated(configuration.clone(), "register"))
            .await?;
        let mut foreign = configuration;
        foreign.registration_key = "unscoped-provider".into();
        assert_eq!(
            provider
                .exchange(authenticated(foreign, "foreign"))
                .await
                .err()
                .ok_or("Unscoped Provider accepted")?
                .code(),
            tonic::Code::PermissionDenied
        );
        drop(provider);
        let id = i32::try_from(child.lock().await.id().ok_or("Missing PID")?)?;
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(id),
            nix::sys::signal::Signal::SIGTERM,
        )?;
        assert!(
            tokio::time::timeout(Duration::from_secs(10), child.lock().await.wait())
                .await??
                .success()
        );
        assert!(TcpListener::bind(public_addr).is_ok());
        assert!(TcpListener::bind(peer_addr).is_ok());
        Ok(())
    })
    .await
}
