use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey, SystemOwnerVerifier};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::control::{
    self as wire, peer_relay_client::PeerRelayClient,
    provider_connection_client::ProviderConnectionClient,
};
use chapchoo_rpc::ComponentConnectionServices;
use chapchoo_service::{AcceptanceTime, ComputersService, ProviderCapabilities};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
    time::Instant,
};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, Streaming, transport::Server};
type TestError = Box<dyn std::error::Error + Send + Sync>;
fn request<T>(value: T, token: &str) -> Result<Request<T>, TestError> {
    let mut request = Request::new(value);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    Ok(request)
}
async fn frame(
    stream: &mut Streaming<wire::ControlFrame>,
) -> Result<wire::ControlFrame, TestError> {
    Ok(
        tokio::time::timeout(Duration::from_secs(5), stream.message())
            .await??
            .ok_or("Unexpected component EOF")?,
    )
}
fn command(id: &str, connection: &str) -> wire::ProviderCommand {
    wire::ProviderCommand {
        identity: Some(wire::CallIdentity {
            request_id: "request".into(),
            call_id: id.into(),
            connection_id: connection.into(),
        }),
        computer: Some(wire::ComputerContext {
            computer_id: "computer".into(),
            request_id: "request".into(),
            execution_id: "execution".into(),
        }),
        remaining_ms: 10_000,
        operation: Some(wire::provider_command::Operation::Observe(
            wire::ObserveResources { resources: vec![] },
        )),
    }
}
fn reply(command: wire::ProviderCommand) -> wire::ProviderFrame {
    wire::ProviderFrame {
        payload: Some(wire::provider_frame::Payload::Result(
            wire::ProviderResult {
                identity: command.identity,
                computer: command.computer,
                outcome: wire::CommandOutcome::Completed as i32,
                resources: vec![],
                execution: wire::ExecutionFact::Unspecified as i32,
                execution_terminal: false,
            },
        )),
    }
}
async fn open(
    client: &mut ProviderConnectionClient<tonic::transport::Channel>,
    key: &str,
) -> Result<
    (
        mpsc::Sender<wire::ProviderFrame>,
        Streaming<wire::ControlFrame>,
        wire::Connected,
    ),
    TestError,
> {
    let (sender, receiver) = mpsc::channel(8);
    sender
        .send(wire::ProviderFrame {
            payload: Some(wire::provider_frame::Payload::Hello(wire::ProviderHello {
                provider_id: "provider".into(),
                namespace: "fixture".into(),
                connector_endpoint: "http://127.0.0.1:1".into(),
            })),
        })
        .await?;
    let mut stream = client
        .open(request(ReceiverStream::new(receiver), key)?)
        .await?
        .into_inner();
    let Some(wire::control_frame::Payload::Connected(connected)) =
        frame(&mut stream).await?.payload
    else {
        return Err("Expected Connected".into());
    };
    Ok((sender, stream, connected))
}

#[test]
fn two_control_replacement_cancellation_and_late_replies_preserve_current_stream()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("streams.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([10; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let owner = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("owner".into(), vec!["provider".into()], owner.digest())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let app = Arc::new(ComputersService::new(
            Store::connect_sqlite(path, cipher).await?,
            Some(verifier),
        ));
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
        let a = TcpListener::bind("127.0.0.1:0").await?;
        let b = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint_a = format!("http://{}", a.local_addr()?);
        let endpoint_b = format!("http://{}", b.local_addr()?);
        let control_a = app.register_control(&endpoint_a).await?;
        let control_b = app.register_control(&endpoint_b).await?;
        let services_a = ComponentConnectionServices::new(
            app.clone(),
            control_a.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let services_b = ComponentConnectionServices::new(
            app.clone(),
            control_b.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let (stop_a, stopped_a) = oneshot::channel();
        let (stop_b, stopped_b) = oneshot::channel();
        let server_a = tokio::spawn(
            Server::builder()
                .add_service(services_a.provider_server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(a), async {
                    let _ = stopped_a.await;
                }),
        );
        let server_b = tokio::spawn(
            Server::builder()
                .add_service(services_b.provider_server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(b), async {
                    let _ = stopped_b.await;
                }),
        );
        let mut client_a = ProviderConnectionClient::connect(endpoint_a.clone()).await?;
        let mut client_b = ProviderConnectionClient::connect(endpoint_b).await?;
        assert_eq!(
            PeerRelayClient::connect(endpoint_a)
                .await?
                .invoke_provider(wire::ProviderInvocation::default())
                .await
                .err()
                .ok_or("Peer service leaked onto component listener")?
                .code(),
            tonic::Code::Unimplemented
        );
        let (_sender_a, mut stream_a, connected_a) =
            open(&mut client_a, key.token.expose()).await?;
        let (sender_b, mut stream_b, connected_b) = open(&mut client_b, key.token.expose()).await?;
        assert_ne!(connected_a.connection_id, connected_b.connection_id);
        assert_ne!(connected_a.instance_id, connected_b.instance_id);
        let owner_a = wire::ExpectedOwner {
            instance_id: connected_a.instance_id,
            connection_id: connected_a.connection_id.clone(),
        };
        let owner_b = wire::ExpectedOwner {
            instance_id: connected_b.instance_id.clone(),
            connection_id: connected_b.connection_id.clone(),
        };
        // The old stream is fenced on its next authoritative activity. Explicit
        // heartbeat establishes that boundary without waiting for a periodic tick.
        let _ = _sender_a
            .send(wire::ProviderFrame {
                payload: Some(wire::provider_frame::Payload::Heartbeat(wire::Heartbeat {
                    connection_id: connected_a.connection_id.clone(),
                })),
            })
            .await;
        let old = tokio::time::timeout(Duration::from_secs(5), stream_a.message()).await?;
        assert!(old.is_err() || matches!(old, Ok(None)));
        app.drain_control(&control_a.id).await?;
        services_a.drain().await?;
        app.withdraw_drained_control_connections(&control_a.id)
            .await?;
        assert_eq!(
            app.resolve_provider_route("provider")
                .await?
                .ok_or("Old Control removed current owner")?
                .component
                .connection_id,
            connected_b.connection_id
        );
        let not_owner = services_a
            .invoke_provider(
                "provider",
                &owner_a,
                command("stale", &owner_a.connection_id),
                Instant::now() + Duration::from_secs(5),
            )
            .await?;
        // Draining is unavailable, not a transport-status-based retry promise.
        assert_eq!(
            not_owner.outcome,
            wire::ForwardingOutcome::Unavailable as i32
        );
        let wrong_owner = services_b
            .invoke_provider(
                "provider",
                &owner_a,
                command("wrong-owner", &owner_a.connection_id),
                Instant::now() + Duration::from_secs(5),
            )
            .await?;
        assert_eq!(
            wrong_owner.outcome,
            wire::ForwardingOutcome::NotOwner as i32
        );
        let service = services_b.clone();
        let target = owner_b.clone();
        let cancelled = tokio::spawn(async move {
            service
                .invoke_provider(
                    "provider",
                    &target,
                    command("cancelled", &target.connection_id),
                    Instant::now() + Duration::from_secs(5),
                )
                .await
        });
        let Some(wire::control_frame::Payload::Provider(cancelled_command)) =
            frame(&mut stream_b).await?.payload
        else {
            return Err("Expected first command".into());
        };
        cancelled.abort();
        assert!(cancelled.await.is_err());
        let Some(wire::control_frame::Payload::Cancel(cancel)) =
            frame(&mut stream_b).await?.payload
        else {
            return Err("Expected scoped cancellation".into());
        };
        assert_eq!(
            cancel
                .identity
                .ok_or("Missing cancellation identity")?
                .call_id,
            "cancelled"
        );
        let service = services_b.clone();
        let target = owner_b.clone();
        let pending = tokio::spawn(async move {
            service
                .invoke_provider(
                    "provider",
                    &target,
                    command("current", &target.connection_id),
                    Instant::now() + Duration::from_secs(5),
                )
                .await
        });
        let Some(wire::control_frame::Payload::Provider(current)) =
            frame(&mut stream_b).await?.payload
        else {
            return Err("Expected current command".into());
        };
        // Late cancelled and old-connection responses cannot finish this waiter
        // or close the replacement's shared stream.
        sender_b.send(reply(cancelled_command)).await?;
        let mut stale = current.clone();
        stale
            .identity
            .as_mut()
            .ok_or("Missing call identity")?
            .connection_id = connected_a.connection_id;
        sender_b.send(reply(stale)).await?;
        sender_b
            .send(wire::ProviderFrame {
                payload: Some(wire::provider_frame::Payload::Heartbeat(wire::Heartbeat {
                    connection_id: owner_b.connection_id.clone(),
                })),
            })
            .await?;
        assert!(matches!(
            frame(&mut stream_b).await?.payload,
            Some(wire::control_frame::Payload::Heartbeat(_))
        ));
        sender_b.send(reply(current)).await?;
        let result = pending.await??;
        assert_eq!(result.outcome, wire::ForwardingOutcome::Completed as i32);
        assert_eq!(
            result
                .result
                .ok_or("Missing correlated result")?
                .identity
                .ok_or("Missing result identity")?
                .call_id,
            "current"
        );
        assert_eq!(services_b.pending_calls(), 0);
        assert!(services_b.ensure_healthy().is_ok());
        app.drain_control(&control_b.id).await?;
        services_b.drain().await?;
        app.withdraw_drained_control_connections(&control_b.id)
            .await?;
        assert!(
            tokio::time::timeout(Duration::from_secs(5), stream_b.message())
                .await?
                .is_ok_and(|value| value.is_none())
        );
        let _ = stop_a.send(());
        let _ = stop_b.send(());
        tokio::time::timeout(Duration::from_secs(5), server_a).await???;
        tokio::time::timeout(Duration::from_secs(5), server_b).await???;
        Ok::<(), TestError>(())
    })
}
