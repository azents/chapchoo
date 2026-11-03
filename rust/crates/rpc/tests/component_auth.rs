use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey, SystemOwnerVerifier};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::control::{
    self as wire, connector_authentication_client::ConnectorAuthenticationClient,
    connector_connection_client::ConnectorConnectionClient,
    provider_authentication_client::ProviderAuthenticationClient,
};
use chapchoo_rpc::{ComponentAuthenticationServices, ComponentConnectionServices};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, ComputersService, CreationOutcome,
    PasswordWork, ProviderCapabilities, ProviderSource, SessionPolicy,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Code, Request, transport::Server};
type TestError = Box<dyn std::error::Error + Send + Sync>;
fn authenticated<T>(body: T, token: &str) -> Result<Request<T>, TestError> {
    let mut request = Request::new(body);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse()?);
    Ok(request)
}
async fn open_connector(
    client: &mut ConnectorConnectionClient<tonic::transport::Channel>,
    session_id: &str,
    key: &str,
) -> Result<
    (
        mpsc::Sender<wire::ConnectorFrame>,
        tonic::Streaming<wire::ControlFrame>,
        wire::Connected,
    ),
    TestError,
> {
    let (sender, receiver) = mpsc::channel(8);
    sender
        .send(wire::ConnectorFrame {
            payload: Some(wire::connector_frame::Payload::Hello(
                wire::ConnectorHello {
                    session_id: session_id.into(),
                },
            )),
        })
        .await?;
    let mut stream = client
        .open(authenticated(ReceiverStream::new(receiver), key)?)
        .await?
        .into_inner();
    let frame = tokio::time::timeout(Duration::from_secs(5), stream.message())
        .await??
        .ok_or("Missing Connector acceptance")?;
    let Some(wire::control_frame::Payload::Connected(connected)) = frame.payload else {
        return Err("Expected Connector acceptance".into());
    };
    Ok((sender, stream, connected))
}
fn owner_request(request: &str) -> wire::ProviderKeyRequest {
    wire::ProviderKeyRequest {
        provider_id: "provider".into(),
        request_id: request.into(),
        authority: wire::OwnerAuthority::System as i32,
    }
}

#[test]
fn component_credential_rpc_preserves_owner_bootstrap_and_operational_roles()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("component-rpc.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([9; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let owner = BearerSecret::generate()?;
    let setup = BearerSecret::generate()?;
    let verifier = SystemOwnerVerifier::new(
        "system-owner".into(),
        vec!["provider".into()],
        owner.digest(),
    )?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let app = Arc::new(ComputersService::new(store.clone(), Some(verifier)));
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let control = app.register_control(&endpoint).await?;
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
            &chapchoo_domain::ProviderSubstrate::Kubernetes,
            AcceptanceTime::System,
        )
        .await?;
        let admission = Arc::new(AtomicBool::new(true));
        let connections =
            ComponentConnectionServices::new(app.clone(), control.id.clone(), admission.clone());
        let services = ComponentAuthenticationServices::new(
            app.clone(),
            control.id.clone(),
            connections.clone(),
        )
        .with_admission(admission.clone());
        let (stop, stopped) = oneshot::channel();
        let server = tokio::spawn(
            Server::builder()
                .add_service(services.provider_server())
                .add_service(services.connector_server())
                .add_service(connections.connector_server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                }),
        );
        let mut provider = ProviderAuthenticationClient::connect(endpoint.clone()).await?;
        let mut connector = ConnectorAuthenticationClient::connect(endpoint.clone()).await?;
        let mut connection_client = ConnectorConnectionClient::connect(endpoint).await?;
        let first = provider
            .rotate_key(authenticated(owner_request("issue"), owner.expose())?)
            .await?
            .into_inner();
        let repeated = provider
            .rotate_key(authenticated(owner_request("issue"), owner.expose())?)
            .await?
            .into_inner();
        assert!(first.token == repeated.token);
        assert_eq!(first.expires_at, repeated.expires_at);
        assert_eq!(
            provider
                .rotate_key(authenticated(owner_request("self-rotate"), &first.token)?)
                .await
                .err()
                .ok_or("Expected owner-role rejection")?
                .code(),
            Code::PermissionDenied
        );
        let mut invalid = owner_request("invalid-role");
        invalid.authority = i32::MAX;
        assert_eq!(
            provider
                .rotate_key(authenticated(invalid, owner.expose())?)
                .await
                .err()
                .ok_or("Expected invalid authority rejection")?
                .code(),
            Code::InvalidArgument
        );
        let token = BearerSecret::parse(first.token.clone())?;
        // A controlled admitted-route fixture supplies stream ownership. This
        // test verifies credential RPC, not the pending component stream adapter.
        let prepared = app
            .prepare_provider_connection("provider", &token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&prepared).await?;
        provider
            .check_connection(authenticated(
                wire::ProviderConnectionCheck {
                    provider_id: "provider".into(),
                    instance_id: control.id.clone(),
                    connection_id: prepared.owner().connection_id.clone(),
                },
                &first.token,
            )?)
            .await?;
        let accounts = AccountsService::new(
            store,
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
                EmailAddress::parse("owner@example.invalid")?,
                Password::parse("Aa1!owner".into())?,
                AcceptanceTime::System,
                None,
            )
            .await?;
        let created = app
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
            .await?;
        let CreationOutcome::Present(created) = created else {
            return Err("Expected created Agent".into());
        };
        let source = ProviderSource {
            provider_id: "provider",
            control_id: &control.id,
            connection_id: &prepared.owner().connection_id,
            credential: &token.digest(),
        };
        let execution = app
            .reserve_execution(
                &source,
                chapchoo_service::ComputerRequest {
                    computer_id: &created.computer.id,
                    request_id: "create",
                },
                &chapchoo_domain::NativeScope::kubernetes("fixture".into())?,
            )
            .await?;
        let bootstrap = provider
            .issue_connector_session(authenticated(
                wire::ConnectorSessionRequest {
                    provider_id: "provider".into(),
                    execution_id: execution.execution_id,
                    request_id: "session".into(),
                    connection_id: prepared.owner().connection_id.clone(),
                },
                &first.token,
            )?)
            .await?
            .into_inner();
        let exchanged = connector
            .exchange(authenticated(
                wire::ConnectorKeyRequest {
                    session_id: bootstrap.session_id.clone(),
                    request_id: "exchange".into(),
                },
                &bootstrap.initial_key,
            )?)
            .await?
            .into_inner();
        assert_eq!(
            connector
                .refresh(authenticated(
                    wire::ConnectorKeyRequest {
                        session_id: bootstrap.session_id.clone(),
                        request_id: "wrong-role".into()
                    },
                    &bootstrap.initial_key
                )?)
                .await
                .err()
                .ok_or("Expected bootstrap-role rejection")?
                .code(),
            Code::Unauthenticated
        );
        let failed = open_connector(
            &mut connection_client,
            &bootstrap.session_id,
            &bootstrap.initial_key,
        )
        .await
        .err()
        .ok_or("Bootstrap opened an operational stream")?;
        assert_eq!(
            failed
                .downcast_ref::<tonic::Status>()
                .map(|value| value.code()),
            Some(Code::Unauthenticated)
        );
        let (frames, mut stream, connected) = open_connector(
            &mut connection_client,
            &bootstrap.session_id,
            &exchanged.token,
        )
        .await?;
        connector
            .check_connection(authenticated(
                wire::ConnectorConnectionCheck {
                    session_id: bootstrap.session_id.clone(),
                    instance_id: connected.instance_id.clone(),
                    connection_id: connected.connection_id.clone(),
                },
                &exchanged.token,
            )?)
            .await?;
        provider
            .revoke_key(authenticated(owner_request("revoke"), owner.expose())?)
            .await?;
        // Operational Connector refresh is independent of Provider authority.
        let refreshed = connector
            .refresh(authenticated(
                wire::ConnectorKeyRequest {
                    session_id: bootstrap.session_id.clone(),
                    request_id: "refresh".into(),
                },
                &exchanged.token,
            )?)
            .await?
            .into_inner();
        assert!(refreshed.token != exchanged.token);
        frames
            .send(wire::ConnectorFrame {
                payload: Some(wire::connector_frame::Payload::Heartbeat(wire::Heartbeat {
                    connection_id: connected.connection_id.clone(),
                })),
            })
            .await?;
        let heartbeat = tokio::time::timeout(Duration::from_secs(5), stream.message())
            .await??
            .ok_or("Connector disconnected with Provider revocation")?;
        assert!(matches!(
            heartbeat.payload,
            Some(wire::control_frame::Payload::Heartbeat(_))
        ));
        let calls = connections.clone();
        let computer_id = created.computer.id.clone();
        let execution_id = bootstrap.execution_id.clone();
        let target = connected.clone();
        let pending = tokio::spawn(async move {
            calls
                .invoke_connector(
                    &computer_id,
                    &wire::ExpectedOwner {
                        instance_id: target.instance_id,
                        connection_id: target.connection_id.clone(),
                    },
                    wire::ConnectorCommand {
                        identity: Some(wire::CallIdentity {
                            request_id: "create".into(),
                            call_id: "connector-call".into(),
                            connection_id: target.connection_id,
                        }),
                        computer: Some(wire::ComputerContext {
                            computer_id: computer_id.clone(),
                            request_id: "create".into(),
                            execution_id,
                        }),
                        remaining_ms: 5_000,
                        observe: Some(wire::ObserveConnector {}),
                    },
                    tokio::time::Instant::now() + Duration::from_secs(5),
                )
                .await
        });
        let command = tokio::time::timeout(Duration::from_secs(5), stream.message())
            .await??
            .ok_or("Missing Connector command")?;
        let Some(wire::control_frame::Payload::Connector(command)) = command.payload else {
            return Err("Expected Connector command".into());
        };
        frames
            .send(wire::ConnectorFrame {
                payload: Some(wire::connector_frame::Payload::Result(
                    wire::ConnectorResult {
                        identity: command.identity,
                        execution_id: bootstrap.execution_id.clone(),
                        ready: true,
                    },
                )),
            })
            .await?;
        assert!(
            pending
                .await??
                .result
                .ok_or("Missing Connector reply")?
                .ready
        );
        let replacement = provider
            .rotate_key(authenticated(owner_request("replacement"), owner.expose())?)
            .await?
            .into_inner();
        let replacement_token = BearerSecret::parse(replacement.token.clone())?;
        let replacement_connection = app
            .prepare_provider_connection("provider", &replacement_token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&replacement_connection)
            .await?;
        provider
            .revoke_connector_session(authenticated(
                wire::RevokeConnectorSessionRequest {
                    provider_id: "provider".into(),
                    session_id: bootstrap.session_id.clone(),
                    request_id: "revoke-session".into(),
                    connection_id: replacement_connection.owner().connection_id.clone(),
                },
                &replacement.token,
            )?)
            .await?;
        assert_eq!(
            connector
                .refresh(authenticated(
                    wire::ConnectorKeyRequest {
                        session_id: bootstrap.session_id,
                        request_id: "revoked-refresh".into()
                    },
                    &refreshed.token
                )?)
                .await
                .err()
                .ok_or("Expected revoked session")?
                .code(),
            Code::Unauthenticated
        );
        let _ = frames
            .send(wire::ConnectorFrame {
                payload: Some(wire::connector_frame::Payload::Heartbeat(wire::Heartbeat {
                    connection_id: connected.connection_id,
                })),
            })
            .await;
        let closed = tokio::time::timeout(Duration::from_secs(5), stream.message()).await?;
        assert!(closed.is_err() || matches!(closed, Ok(None)));
        admission.store(false, Ordering::Release);
        assert_eq!(
            provider
                .rotate_key(authenticated(owner_request("drained"), owner.expose())?)
                .await
                .err()
                .ok_or("Expected admission rejection")?
                .code(),
            Code::Unavailable
        );
        app.drain_control(&control.id).await?;
        connections.drain().await?;
        app.withdraw_drained_control_connections(&control.id)
            .await?;
        let _ = stop.send(());
        server.await??;
        Ok::<(), TestError>(())
    })
}
