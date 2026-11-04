mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey,
};
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
    PasswordWork, ProviderSource, SessionPolicy,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::{net::TcpListener, sync::mpsc};
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
fn exchange_request(
    request: &str,
    proof: &str,
) -> Result<wire::ProviderAuthenticationRequest, TestError> {
    static START: std::sync::OnceLock<std::time::SystemTime> = std::sync::OnceLock::new();
    let now = START
        .get_or_init(std::time::SystemTime::now)
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    Ok(wire::ProviderAuthenticationRequest {
        configuration: Some(wire::ProviderConfiguration {
            registration_key: "provider".into(),
            name: "Fixture".into(),
            capabilities: Some(wire::ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            }),
            kind: wire::ProviderKind::Kubernetes as i32,
            docker_daemon_id: None,
        }),
        request_id: format!("{now}:{request}"),
        evidence: Some(
            wire::provider_authentication_request::Evidence::BootstrapCredential(proof.into()),
        ),
    })
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
    let owner = cipher.issue_provider_bootstrap("provider")?;
    let setup = BearerSecret::generate()?;
    tokio::runtime::Runtime::new()?.block_on(support::scope(async |fixture| {
        let store = Store::connect_sqlite(path, cipher.clone()).await?;
        let app = Arc::new(ComputersService::new(store.clone()));
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let control = app.register_control(&endpoint).await?;
        let admission = Arc::new(AtomicBool::new(true));
        let connections =
            ComponentConnectionServices::new(app.clone(), control.id.clone(), admission.clone());
        let services = ComponentAuthenticationServices::new(
            app.clone(),
            control.id.clone(),
            connections.clone(),
        )
        .with_admission(admission.clone())
        .with_provider_authenticator(Arc::new(chapchoo_rpc::ProviderAuthenticator::new(
            chapchoo_rpc::ProviderBootstrapConfig {
                secret_key_provider_ids: vec!["provider".into()],
                ..Default::default()
            },
            cipher.clone(),
            None,
        )?));
        let stopped = fixture.shutdown();
        fixture.server(
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
            .exchange(exchange_request("issue", owner.expose())?)
            .await?
            .into_inner();
        let repeated = provider
            .exchange(exchange_request("issue", owner.expose())?)
            .await?
            .into_inner();
        assert_eq!(first.provider_id, repeated.provider_id);
        let provider_id = first.provider_id;
        let first = first.key.ok_or("Missing Provider key")?;
        let repeated = repeated.key.ok_or("Missing recovered Provider key")?;
        assert!(first.token == repeated.token);
        assert_eq!(first.expires_at, repeated.expires_at);
        assert_eq!(
            provider
                .exchange(exchange_request("self-rotate", &first.token)?)
                .await
                .err()
                .ok_or("Expected owner-role rejection")?
                .code(),
            Code::PermissionDenied
        );
        let mut invalid = exchange_request("wrong-id", owner.expose())?;
        if let Some(config) = invalid.configuration.as_mut() {
            config.registration_key = "foreign-provider".into();
        }
        assert_eq!(
            provider
                .exchange(invalid)
                .await
                .err()
                .ok_or("Expected scope rejection")?
                .code(),
            Code::PermissionDenied
        );
        let mut missing = exchange_request("no-proof", owner.expose())?;
        missing.evidence = None;
        assert_eq!(
            provider
                .exchange(missing)
                .await
                .err()
                .ok_or("Expected public Trust rejection")?
                .code(),
            Code::PermissionDenied
        );
        let expiry = first.expires_at.as_ref().ok_or("Missing expiry")?;
        let issued = first.issued_at.as_ref().ok_or("Missing issuance")?;
        let refresh = first.proactive_after.as_ref().ok_or("Missing renewal")?;
        assert_eq!(expiry.seconds - issued.seconds, 900);
        assert_eq!(refresh.seconds - issued.seconds, 300);
        let token = BearerSecret::parse(first.token.clone())?;
        // A controlled admitted-route fixture supplies stream ownership. This
        // test verifies credential RPC, not the pending component stream adapter.
        let prepared = app
            .prepare_provider_connection(&provider_id, &token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&prepared).await?;
        provider
            .check_connection(authenticated(
                wire::ProviderConnectionCheck {
                    provider_id: provider_id.clone(),
                    instance_id: control.id.clone(),
                    connection_id: prepared.owner().connection_id.clone(),
                },
                &first.token,
            )?)
            .await?;
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
                EmailAddress::parse("owner@example.invalid")?,
                Password::parse("Aa1!owner".into())?,
                AcceptanceTime::System,
                None,
            )
            .await?;
        // Account key-management authority remains scoped to USER-owned Providers.
        assert_eq!(
            provider
                .rotate_key(authenticated(
                    wire::ProviderKeyRequest {
                        provider_id: provider_id.clone(),
                        request_id: format!(
                            "{}:account-cannot-manage-system",
                            AcceptanceTime::System.now()?
                        ),
                    },
                    user.token.expose()
                )?)
                .await
                .err()
                .ok_or("Account adopted system Provider")?
                .code(),
            Code::PermissionDenied,
        );
        assert_eq!(
            provider
                .revoke_key(authenticated(
                    wire::ProviderKeyRequest {
                        provider_id: provider_id.clone(),
                        request_id: format!(
                            "{}:component-cannot-revoke",
                            AcceptanceTime::System.now()?
                        ),
                    },
                    &first.token
                )?)
                .await
                .err()
                .ok_or("Operational token gained owner rights")?
                .code(),
            Code::Unauthenticated,
        );
        let created = app
            .create_cloud_agent(
                &user.token.digest(),
                &chapchoo_service::CreateCloudAgent {
                    team_id: accounts
                        .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
                        .await?
                        .remove(0)
                        .id,
                    request_id: "create".into(),
                    provider_id: provider_id.clone(),
                    name: "Fixture".into(),
                    profile: chapchoo_service::CreateAgentProfile {
                        handle: "auth-agent".into(),
                        image_id: app
                            .agent_profile_images(
                                &user.token.digest(),
                                None,
                                1,
                                AcceptanceTime::System,
                            )
                            .await?[0]
                            .id
                            .clone(),
                        palette_key: "sage".into(),
                    },
                },
                AcceptanceTime::System,
            )
            .await?;
        let CreationOutcome::Present(created) = created else {
            return Err("Expected created Agent".into());
        };
        let source = ProviderSource {
            provider_id: &provider_id,
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
                    provider_id: provider_id.clone(),
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
        let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
        store
            .revoke_provider_key(
                chapchoo_persistence::ProviderOwner::System(&proof),
                "provider",
                &format!("{}:revoke", AcceptanceTime::System.now()?),
                AcceptanceTime::System.now()?,
            )
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
        let pending = fixture.call(async move {
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
                .await?
                .result
                .ok_or("Missing Connector reply")?
                .ready
        );
        let replacement = provider
            .exchange(exchange_request("replacement", owner.expose())?)
            .await?
            .into_inner();
        assert_eq!(replacement.provider_id, provider_id);
        let replacement = replacement.key.ok_or("Missing replacement key")?;
        let replacement_token = BearerSecret::parse(replacement.token.clone())?;
        let replacement_connection = app
            .prepare_provider_connection(&provider_id, &replacement_token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&replacement_connection)
            .await?;
        provider
            .revoke_connector_session(authenticated(
                wire::RevokeConnectorSessionRequest {
                    provider_id: provider_id.clone(),
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
                .exchange(exchange_request("drained", owner.expose())?)
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
        Ok::<(), TestError>(())
    }))
}

#[test]
fn standalone_trust_is_private_bound_and_prohibitable() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("internal-trust.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([19; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(support::scope(async |fixture| {
        let store = Store::connect_sqlite(path, cipher.clone()).await?;
        let app = Arc::new(ComputersService::new(store));
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let control = app.register_control(&endpoint).await?;
        let connections = ComponentConnectionServices::new(
            app.clone(),
            control.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let auth = Arc::new(chapchoo_rpc::ProviderAuthenticator::new(
            chapchoo_rpc::ProviderBootstrapConfig {
                trust_provider_ids: vec!["provider".into()],
                ..Default::default()
            },
            cipher.clone(),
            None,
        )?);
        let services = ComponentAuthenticationServices::new(
            app.clone(),
            control.id.clone(),
            connections.clone(),
        )
        .with_provider_authenticator(auth);
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(services.provider_server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                }),
        );
        let (channel, incoming) = chapchoo_rpc::memory_transport();
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(services.internal_provider_server("provider".into()))
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = stopped.await;
                }),
        );
        let mut public = ProviderAuthenticationClient::connect(endpoint).await?;
        let mut internal = ProviderAuthenticationClient::new(channel);
        let request = |id: &str, request_id: &str| {
            let mut request = exchange_request(request_id, "unused")?;
            request.evidence = None;
            if let Some(config) = request.configuration.as_mut() {
                config.registration_key = id.to_owned();
            }
            Ok::<_, TestError>(request)
        };
        assert_eq!(
            public
                .exchange(request("provider", "public")?)
                .await
                .err()
                .ok_or("Public Trust admitted")?
                .code(),
            Code::PermissionDenied
        );
        let first = internal
            .exchange(request("provider", "internal")?)
            .await?
            .into_inner();
        let recovered = internal
            .exchange(request("provider", "internal")?)
            .await?
            .into_inner();
        assert_eq!(first.provider_id, recovered.provider_id);
        let first = first.key.ok_or("Missing first key")?;
        let recovered = recovered.key.ok_or("Missing recovered key")?;
        assert_eq!(first.token, recovered.token);
        assert_eq!(first.expires_at, recovered.expires_at);
        assert_eq!(
            internal
                .exchange(request("foreign", "wrong-id")?)
                .await
                .err()
                .ok_or("Internal channel changed identity")?
                .code(),
            Code::PermissionDenied
        );
        let deny =
            ComponentAuthenticationServices::new(
                app.clone(),
                control.id.clone(),
                connections.clone(),
            )
            .with_provider_authenticator(Arc::new(
                chapchoo_rpc::ProviderAuthenticator::new(Default::default(), cipher, None)?,
            ));
        let (channel, incoming) = chapchoo_rpc::memory_transport();
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(deny.internal_provider_server("provider".into()))
                .serve_with_incoming_shutdown(incoming, async {
                    let _ = stopped.await;
                }),
        );
        let mut prohibited = ProviderAuthenticationClient::new(channel);
        assert_eq!(
            prohibited
                .exchange(request("provider", "prohibited")?)
                .await
                .err()
                .ok_or("Prohibited Trust admitted")?
                .code(),
            Code::PermissionDenied
        );
        drop(public);
        drop(internal);
        drop(prohibited);
        app.drain_control(&control.id).await?;
        connections.drain().await?;
        app.withdraw_drained_control_connections(&control.id)
            .await?;
        Ok::<(), TestError>(())
    }))
}
