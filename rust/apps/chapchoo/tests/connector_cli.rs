#![cfg(unix)]
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey, SystemOwnerVerifier};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::control as wire;
use chapchoo_rpc::{ComponentAuthenticationServices, ComponentConnectionServices};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, ComputerRequest, ComputersService,
    CreationOutcome, PasswordWork, ProviderCapabilities, ProviderSource, SessionPolicy,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::{net::TcpListener, process::Command, time::Instant};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
type Error = Box<dyn std::error::Error + Send + Sync>;
#[tokio::test]
async fn connector_process_exchanges_connects_and_observes_independently_of_provider_key()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("connector.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([38; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let setup = BearerSecret::generate()?;
    let owner = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("system".into(), vec!["provider".into()], owner.digest())?;
    let store = Store::connect_sqlite(path, cipher).await?;
    let accounts = AccountsService::new(
        store.clone(),
        PasswordWork::new(1, PasswordCost::new(10)?)?,
        None,
        AccountsConfig {
            general_web_url: "http://localhost:3000".into(),
            setup_proof: Some(setup.digest()),
            session_policy: SessionPolicy::default(),
        },
    )?;
    let user = accounts
        .initialize(
            &setup,
            EmailAddress::parse("connector@example.invalid")?,
            Password::parse("Aa1!fixture-connector".into())?,
            AcceptanceTime::System,
            None,
        )
        .await?;
    let app = Arc::new(ComputersService::new(store, Some(verifier)));
    app.configure_system_provider(
        &owner,
        "provider",
        "Fixture",
        ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        },
        AcceptanceTime::System,
    )
    .await?;
    let key = app
        .rotate_system_provider_key(&owner, "provider", "key", AcceptanceTime::System)
        .await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let control = app.register_control(&endpoint).await?;
    let prepared = app
        .prepare_provider_connection("provider", &key.token.digest(), &control.id)
        .await?;
    app.admit_component_connection(&prepared).await?;
    let CreationOutcome::Present(agent) = app
        .create_cloud_agent(
            &user.token.digest(),
            &accounts
                .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
                .await?
                .remove(0)
                .id,
            "create",
            "provider",
            "Fixture",
            AcceptanceTime::System,
        )
        .await?
    else {
        return Err("Missing Agent".into());
    };
    let source = ProviderSource {
        provider_id: "provider",
        control_id: &control.id,
        connection_id: &prepared.owner().connection_id,
        credential: &key.token.digest(),
    };
    let scope = ComputerRequest {
        computer_id: &agent.computer.id,
        request_id: "create",
    };
    let execution = app.reserve_execution(&source, scope, "fixture").await?;
    let session = app
        .issue_connector_session_from_source(
            &source,
            &execution.execution_id,
            "session",
            AcceptanceTime::System,
        )
        .await?;
    let authentication = ComponentAuthenticationServices::new(app.clone(), control.id.clone());
    let connections = ComponentConnectionServices::new(
        app.clone(),
        control.id.clone(),
        Arc::new(AtomicBool::new(true)),
    );
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(
        Server::builder()
            .add_service(authentication.connector_server())
            .add_service(connections.connector_server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = stopped.await;
            }),
    );
    // No DB/root/SystemOwner/Provider credentials are present in the child.
    let mut child = Command::new(env!("CARGO_BIN_EXE_chapchoo"))
        .env_clear()
        .env("CHAPCHOO_CONTROL_ENDPOINT", &endpoint)
        .env("CHAPCHOO_CONNECTOR_SESSION_ID", &session.id)
        .env(
            "CHAPCHOO_CONNECTOR_INITIAL_KEY",
            session.initial_key.expose(),
        )
        .arg("computer-connector")
        .kill_on_drop(true)
        .spawn()?;
    let route = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(route) = app.resolve_connector_route(&agent.computer.id).await? {
                return Ok::<_, Error>(route);
            }
            if child.try_wait()?.is_some() {
                return Err("Connector process exited before current route".into());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await??;
    app.revoke_system_provider_key(&owner, "provider", "revoke", AcceptanceTime::System)
        .await?;
    let result = connections
        .invoke_connector(
            &agent.computer.id,
            &wire::ExpectedOwner {
                instance_id: route.component.control_id,
                connection_id: route.component.connection_id.clone(),
            },
            wire::ConnectorCommand {
                identity: Some(wire::CallIdentity {
                    request_id: "create".into(),
                    call_id: "observe".into(),
                    connection_id: route.component.connection_id,
                }),
                computer: Some(wire::ComputerContext {
                    computer_id: agent.computer.id.clone(),
                    request_id: "create".into(),
                    execution_id: execution.execution_id.clone(),
                }),
                remaining_ms: 10_000,
                observe: Some(wire::ObserveConnector {}),
            },
            Instant::now() + Duration::from_secs(5),
        )
        .await?;
    assert_eq!(result.outcome, wire::ForwardingOutcome::Completed as i32);
    let result = result.result.ok_or("Missing Connector result")?;
    assert!(result.ready);
    assert_eq!(result.execution_id, execution.execution_id);
    let id = i32::try_from(child.id().ok_or("Missing Connector PID")?)?;
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(id),
        nix::sys::signal::Signal::SIGTERM,
    )?;
    assert!(
        tokio::time::timeout(Duration::from_secs(10), child.wait())
            .await??
            .success()
    );
    app.drain_control(&control.id).await?;
    connections.drain().await?;
    let _ = stop.send(());
    server.await??;
    Ok(())
}
