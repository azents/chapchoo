use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey, SystemOwnerVerifier};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::control::{
    self as wire, peer_relay_client::PeerRelayClient,
    provider_connection_client::ProviderConnectionClient,
};
use chapchoo_rpc::{
    ComponentAuthenticationServices, ComponentConnectionServices, PeerDelivery, PeerServices,
    memory_transport,
};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, ComputerRequest, ComputersService,
    CreationOutcome, PasswordWork, ProviderCapabilities, ProviderSource, SessionPolicy,
};
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
use tonic::{Request, transport::Server};
type Error = Box<dyn std::error::Error + Send + Sync>;
fn scope(context: &wire::ComputerContext) -> ComputerRequest<'_> {
    ComputerRequest {
        computer_id: &context.computer_id,
        request_id: &context.request_id,
    }
}
async fn next(
    stream: &mut tonic::Streaming<wire::ControlFrame>,
) -> Result<wire::ControlFrame, Error> {
    Ok(
        tokio::time::timeout(Duration::from_secs(5), stream.message())
            .await??
            .ok_or("Missing frame")?,
    )
}
#[test]
fn memory_and_tcp_peer_handlers_revalidate_domain_and_exact_reconciler_authority()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("peer.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([34; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let system = BearerSecret::generate()?;
    let setup = BearerSecret::generate()?;
    let verifier =
        SystemOwnerVerifier::new("system".into(), vec!["provider".into()], system.digest())?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let app = Arc::new(ComputersService::new(store.clone(), Some(verifier)));
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
                EmailAddress::parse("peer@example.invalid")?,
                Password::parse("Aa1!peer-user".into())?,
                AcceptanceTime::System,
                None,
            )
            .await?;
        app.configure_system_provider(
            &system,
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
            .rotate_system_provider_key(&system, "provider", "key", AcceptanceTime::System)
            .await?;
        let tcp = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", tcp.local_addr()?);
        let control = app.register_control(&endpoint).await?;
        let connections = ComponentConnectionServices::new(
            app.clone(),
            control.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let peer = PeerServices::new(app.clone(), connections.clone());
        let auth = ComponentAuthenticationServices::new(app.clone(), control.id.clone());
        let (component_channel, component_incoming) = memory_transport();
        let (peer_channel, peer_incoming) = memory_transport();
        let (stop_components, stop_components_rx) = oneshot::channel();
        let (stop_memory, stop_memory_rx) = oneshot::channel();
        let (stop_tcp, stop_tcp_rx) = oneshot::channel();
        let component_server = tokio::spawn(
            Server::builder()
                .add_service(connections.provider_server())
                .add_service(auth.provider_server())
                .serve_with_incoming_shutdown(component_incoming, async {
                    let _ = stop_components_rx.await;
                }),
        );
        let memory_server = tokio::spawn(
            Server::builder()
                .add_service(peer.server())
                .serve_with_incoming_shutdown(peer_incoming, async {
                    let _ = stop_memory_rx.await;
                }),
        );
        let tcp_server = tokio::spawn(
            Server::builder()
                .add_service(peer.server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(tcp), async {
                    let _ = stop_tcp_rx.await;
                }),
        );
        let mut component = ProviderConnectionClient::new(component_channel.clone());
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
        let mut request = Request::new(ReceiverStream::new(receiver));
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", key.token.expose()).parse()?,
        );
        let mut stream = component.open(request).await?.into_inner();
        let Some(wire::control_frame::Payload::Connected(connected)) =
            next(&mut stream).await?.payload
        else {
            return Err("Missing Connected".into());
        };
        let CreationOutcome::Present(agent) = app
            .create_cloud_agent(
                &user.token.digest(),
                "create",
                "provider",
                "Fixture",
                AcceptanceTime::System,
            )
            .await?
        else {
            return Err("Missing Agent".into());
        };
        let mut context = wire::ComputerContext {
            computer_id: agent.computer.id,
            request_id: "create".into(),
            execution_id: String::new(),
        };
        let source = ProviderSource {
            provider_id: "provider",
            control_id: &control.id,
            connection_id: &connected.connection_id,
            credential: &key.token.digest(),
        };
        let execution = app
            .reserve_execution(&source, scope(&context), "fixture")
            .await?;
        context.execution_id = execution.execution_id;
        let resources = execution
            .resources
            .iter()
            .map(|r| wire::ManagedResource {
                kind: match r.resource_kind.as_str() {
                    "POD" => wire::ResourceKind::Pod,
                    "PVC" => wire::ResourceKind::Pvc,
                    _ => wire::ResourceKind::BootstrapSecret,
                } as i32,
                namespace: r.namespace.clone(),
                name: r.name.clone(),
                uid: r.resource_uid.clone(),
                execution_id: r.execution_id.clone(),
                absence_confirmed: r.absence_confirmed_at.is_some(),
            })
            .collect();
        let owner = wire::ExpectedOwner {
            instance_id: control.id.clone(),
            connection_id: connected.connection_id.clone(),
        };
        let command = wire::ProviderCommand {
            identity: Some(wire::CallIdentity {
                request_id: context.request_id.clone(),
                call_id: "call".into(),
                connection_id: connected.connection_id.clone(),
            }),
            computer: Some(context.clone()),
            remaining_ms: 10_000,
            operation: Some(wire::provider_command::Operation::Observe(
                wire::ObserveResources { resources },
            )),
        };
        let invocation = wire::ProviderInvocation {
            provider_id: "provider".into(),
            owner: Some(owner.clone()),
            authority: Some(wire::PeerAuthority {
                context: Some(wire::peer_authority::Context::User(wire::UserAuthority {
                    session_credential_digest: user.token.digest().as_bytes().to_vec(),
                })),
            }),
            command: Some(command),
        };
        let mut check = wire::provider_authentication_client::ProviderAuthenticationClient::new(
            component_channel.clone(),
        );
        let check_value = wire::ProviderCommandCheck {
            source: Some(wire::ProviderConnectionCheck {
                provider_id: "provider".into(),
                instance_id: control.id.clone(),
                connection_id: connected.connection_id.clone(),
            }),
            command: invocation.command.clone(),
        };
        let authenticated = |value| -> Result<Request<wire::ProviderCommandCheck>, Error> {
            let mut request = Request::new(value);
            request.metadata_mut().insert(
                "authorization",
                format!("Bearer {}", key.token.expose()).parse()?,
            );
            Ok(request)
        };
        check
            .check_command(authenticated(check_value.clone())?)
            .await?;
        let mut old_source = check_value.clone();
        old_source.source.as_mut().ok_or("source")?.connection_id = "old-connection".into();
        assert!(
            check
                .check_command(authenticated(old_source)?)
                .await
                .is_err()
        );
        let mut stale_request = check_value;
        let stale_command = stale_request.command.as_mut().ok_or("command")?;
        stale_command.computer.as_mut().ok_or("context")?.request_id = "old".into();
        stale_command
            .identity
            .as_mut()
            .ok_or("identity")?
            .request_id = "old".into();
        assert!(
            check
                .check_command(authenticated(stale_request)?)
                .await
                .is_err()
        );
        let mut clients = [
            PeerRelayClient::new(peer_channel.clone()),
            PeerRelayClient::connect(endpoint).await?,
        ];
        // The component listener has no impersonation-capable peer methods.
        assert_eq!(
            PeerRelayClient::new(component_channel)
                .invoke_provider(invocation.clone())
                .await
                .err()
                .ok_or("Peer listener leak")?
                .code(),
            tonic::Code::Unimplemented
        );
        for client in &mut clients {
            let mut missing = invocation.clone();
            missing.authority = None;
            assert_eq!(
                client
                    .invoke_provider(missing)
                    .await
                    .err()
                    .ok_or("Missing authority was accepted")?
                    .code(),
                tonic::Code::PermissionDenied
            );
            let mut stale = invocation.clone();
            stale
                .command
                .as_mut()
                .ok_or("command")?
                .computer
                .as_mut()
                .ok_or("context")?
                .request_id = "old".into();
            stale
                .command
                .as_mut()
                .ok_or("command")?
                .identity
                .as_mut()
                .ok_or("identity")?
                .request_id = "old".into();
            assert!(client.invoke_provider(stale).await.is_err());
            let mut foreign = invocation.clone();
            let Some(wire::provider_command::Operation::Observe(observe)) = foreign
                .command
                .as_mut()
                .ok_or("command")?
                .operation
                .as_mut()
            else {
                return Err("observe".into());
            };
            observe.resources[0].uid = Some("foreign-uid".into());
            assert_eq!(
                client
                    .invoke_provider(foreign)
                    .await
                    .err()
                    .ok_or("Foreign UID accepted")?
                    .code(),
                tonic::Code::FailedPrecondition
            );
            let mut wrong = invocation.clone();
            wrong.provider_id = "other-provider".into();
            assert_eq!(
                client
                    .invoke_provider(wrong)
                    .await
                    .err()
                    .ok_or("Wrong target accepted")?
                    .code(),
                tonic::Code::PermissionDenied
            );
            let mut reconcile = invocation.clone();
            reconcile.authority = Some(wire::PeerAuthority {
                context: Some(wire::peer_authority::Context::Reconciliation(
                    wire::ReconciliationAuthority {
                        computer: Some(context.clone()),
                        owner: Some(wire::ExpectedOwner {
                            instance_id: control.id.clone(),
                            connection_id: "old-connection".into(),
                        }),
                    },
                )),
            });
            assert_eq!(
                client
                    .invoke_provider(reconcile.clone())
                    .await
                    .err()
                    .ok_or("Stale reconciler accepted")?
                    .code(),
                tonic::Code::PermissionDenied
            );
            assert_eq!(connections.pending_calls(), 0);
            reconcile.authority = Some(wire::PeerAuthority {
                context: Some(wire::peer_authority::Context::Reconciliation(
                    wire::ReconciliationAuthority {
                        computer: Some(context.clone()),
                        owner: Some(owner.clone()),
                    },
                )),
            });
            let mut call_client = client.clone();
            let call = tokio::spawn(async move { call_client.invoke_provider(reconcile).await });
            let Some(wire::control_frame::Payload::Provider(command)) =
                next(&mut stream).await?.payload
            else {
                return Err("Missing provider command".into());
            };
            sender
                .send(wire::ProviderFrame {
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
                })
                .await?;
            assert_eq!(
                call.await??.into_inner().outcome,
                wire::ForwardingOutcome::Completed as i32
            );
        }
        // Local delivery uses the same peer handler, without opening a peer socket.
        let delivery = PeerDelivery::new(app.clone());
        delivery.register_local(control.id.clone(), peer_channel)?;
        let call = tokio::spawn(async move {
            delivery
                .provider(invocation, Instant::now() + Duration::from_secs(5))
                .await
        });
        let Some(wire::control_frame::Payload::Provider(command)) =
            next(&mut stream).await?.payload
        else {
            return Err("Missing local delivery".into());
        };
        sender
            .send(wire::ProviderFrame {
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
            })
            .await?;
        assert_eq!(
            call.await??.outcome,
            wire::ForwardingOutcome::Completed as i32
        );
        let pvc = execution
            .resources
            .iter()
            .find(|r| r.resource_kind == "PVC")
            .ok_or("PVC reservation")?;
        let report = wire::ProviderReport {
            source: Some(wire::ProviderConnectionCheck {
                provider_id: "provider".into(),
                instance_id: control.id.clone(),
                connection_id: connected.connection_id.clone(),
            }),
            computer: Some(context.clone()),
            resources: vec![wire::ResourceEvidence {
                resource: Some(wire::ManagedResource {
                    kind: wire::ResourceKind::Pvc as i32,
                    namespace: pvc.namespace.clone(),
                    name: pvc.name.clone(),
                    uid: Some("acknowledged-uid".into()),
                    execution_id: None,
                    absence_confirmed: false,
                }),
                evidence: Some(wire::resource_evidence::Evidence::Present(
                    wire::OwnedPresent {
                        uid: "acknowledged-uid".into(),
                    },
                )),
            }],
            execution: wire::ExecutionFact::Unspecified as i32,
            cleanup_complete: false,
        };
        let report_request = |value| -> Result<Request<wire::ProviderReport>, Error> {
            let mut request = Request::new(value);
            request.metadata_mut().insert(
                "authorization",
                format!("Bearer {}", key.token.expose()).parse()?,
            );
            Ok(request)
        };
        check.report(report_request(report.clone())?).await?;
        // The unary ACK follows durable evidence, not stream enqueue.
        assert!(
            app.managed_resources(&source, scope(&context))
                .await?
                .iter()
                .any(|r| r.resource_kind == "PVC"
                    && r.resource_uid.as_deref() == Some("acknowledged-uid"))
        );
        let mut stale = report.clone();
        stale.source.as_mut().ok_or("Source")?.connection_id = "old-connection".into();
        assert!(check.report(report_request(stale)?).await.is_err());
        let mut stale = report;
        stale.computer.as_mut().ok_or("Context")?.request_id = "old-request".into();
        assert!(check.report(report_request(stale)?).await.is_err());
        drop(stream);
        drop(sender);
        drop(component);
        drop(clients);
        app.drain_control(&control.id).await?;
        connections.drain().await?;
        // A memory acceptor also finishes normally when its last client drops.
        let _ = stop_components.send(());
        let _ = stop_memory.send(());
        let _ = stop_tcp.send(());
        component_server.await??;
        memory_server.await??;
        tcp_server.await??;
        Ok(())
    })
}
