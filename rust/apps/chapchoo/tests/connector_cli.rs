#![cfg(unix)]
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey,
};
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
#[path = "connector_cli/conversation_operations.rs"]
mod conversation_operations;
#[path = "../test_support.rs"]
mod test_support;
#[tokio::test]
async fn connector_process_exchanges_connects_and_observes_independently_of_provider_key()
-> Result<(), Error> {
    let dir = tempfile::tempdir()?;
    test_support::scope(async |fixture| {
        let path = dir.path().join("connector.sqlite");
        let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([38; 32]),
        )?)?);
        prepare_sqlite(&path, &cipher)?;
        let setup = BearerSecret::generate()?;
        let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
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
        let app = Arc::new(ComputersService::new(store.clone()));
        let capabilities = ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        };
        let substrate = chapchoo_domain::ProviderSubstrate::Kubernetes;
        let key = app
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                capabilities,
                &substrate,
                &format!(
                    "{}:key",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_millis()
                ),
                AcceptanceTime::System,
            )
            .await?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let control = app.register_control(&endpoint).await?;
        let prepared = app
            .prepare_provider_connection(&key.provider_id, &key.key.token.digest(), &control.id)
            .await?;
        app.admit_component_connection(&prepared).await?;
        let images = app
            .agent_profile_images(&user.token.digest(), None, 1, AcceptanceTime::System)
            .await?;
        let CreationOutcome::Present(agent) = app
            .create_cloud_agent(
                &user.token.digest(),
                &chapchoo_service::CreateCloudAgent {
                    team_id: accounts
                        .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
                        .await?
                        .remove(0)
                        .id,
                    request_id: "create".into(),
                    authentication_id: test_support::authentication(
                        &store,
                        &user.token.digest(),
                        AcceptanceTime::System.now()?,
                    )
                    .await?,
                    provider_id: key.provider_id.clone(),
                    name: "Fixture".into(),
                    profile: chapchoo_service::CreateAgentProfile {
                        handle: "connector-agent".into(),
                        image_id: images[0].id.clone(),
                        palette_key: "sage".into(),
                    },
                },
                AcceptanceTime::System,
            )
            .await?
        else {
            return Err("Missing Agent".into());
        };
        let source = ProviderSource {
            provider_id: &key.provider_id,
            control_id: &control.id,
            connection_id: &prepared.owner().connection_id,
            credential: &key.key.token.digest(),
        };
        let scope = ComputerRequest {
            computer_id: &agent.computer.id,
            request_id: "create",
        };
        let execution = app
            .reserve_execution(
                &source,
                scope,
                &chapchoo_domain::NativeScope::kubernetes("fixture".into())?,
            )
            .await?;
        let session = app
            .issue_connector_session_from_source(
                &source,
                &execution.execution_id,
                "session",
                AcceptanceTime::System,
            )
            .await?;
        let accepted=store.accept_conversation_input(&user.token.digest(),&chapchoo_persistence::AcceptConversationInput {
            agent_id:agent.id.clone(),conversation_id:None,request_id:"initial-text".into(),target:chapchoo_domain::ConversationTarget::NotStarted,
            text:chapchoo_domain::ConversationText::parse("Synthetic input; no model execution".into())?,
        },AcceptanceTime::System.now()?).await?;
        let saved=chapchoo_harness::NativeStorage::open(dir.path())?;
        let captured=chapchoo_harness::CaptureLog::create(&saved)?;
        captured.bind_native(&chapchoo_domain::NativeConversationId::parse("synthetic-native".into())?)?;
        let raw=format!(r#"{{"method":"session/update","params":{{"sessionId":"synthetic-native","update":{{"sessionUpdate":"agent_message_chunk","messageId":"original-message","content":{{"type":"text","text":"{}"}}}}}}}}"#,"한글🦀".repeat(9000));
        let position=captured.append(&raw,None)?;
        let request_position=captured.append(r#"{"method":"elicitation/create","params":{"mode":"form","sessionId":"synthetic-native","message":"Original model-free fixture question","requestedSchema":{"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}}}"#,None)?;
        let state_position=captured.append(r#"{"method":"session/update","params":{"sessionId":"synthetic-native","update":{"sessionUpdate":"state_update","state":"running"}}}"#,None)?;
        let snapshot=captured.snapshot()?;
        drop(captured);
        let capture_server=chapchoo_rpc::ConversationCaptureServices::new(
            Arc::new(chapchoo_service::ConversationService::new(store.clone())),Arc::new(AtomicBool::new(true)),
        );
        // Own only a digest of this synthetic child's presented test key. Keep
        // ComponentRoute's private dispatch material behind its normal boundary.
        let captured_credential = Arc::new(std::sync::Mutex::new(None));
        let presented = captured_credential.clone();
        let capture_server = tonic::service::interceptor::InterceptedService::new(
            capture_server.server(),
            move |request: tonic::Request<()>| {
                let value = request.metadata().get("authorization")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.strip_prefix("Bearer "))
                    .ok_or_else(|| tonic::Status::unauthenticated("Fixture credential is absent"))?;
                let digest = BearerSecret::parse(value.to_owned())
                    .map_err(|_| tonic::Status::unauthenticated("Fixture credential is invalid"))?.digest();
                *presented.lock().map_err(|_| tonic::Status::unavailable("Fixture credential owner is unavailable"))? = Some(digest);
                Ok(request)
            },
        );
        let connections = ComponentConnectionServices::new(
            app.clone(),
            control.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let authentication = ComponentAuthenticationServices::new(
            app.clone(),
            control.id.clone(),
            connections.clone(),
        );
        let cleanup_app = app.clone();
        let cleanup_control = control.id.clone();
        let cleanup_connections = connections.clone();
        fixture.cleanup(async move {
            let primary = cleanup_app
                .drain_control(&cleanup_control)
                .await
                .map_err(Into::into);
            let cleanup = cleanup_connections.drain().await.map_err(Into::into);
            test_support::completed(primary, cleanup)
        });
        let stopped = fixture.shutdown();
        fixture.server(
            Server::builder()
                .add_service(authentication.connector_server())
                .add_service(connections.connector_server())
                .add_service(capture_server)
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                }),
        );
        // No DB/root/SystemOwner/Provider credentials are present in the child.
        let child = fixture.child(
            Command::new(env!("CARGO_BIN_EXE_chapchoo"))
                .env_clear()
                .env("CHAPCHOO_CONTROL_ENDPOINT", &endpoint)
                .env("CHAPCHOO_CONNECTOR_SESSION_ID", &session.id)
                .env("CHAPCHOO_COMPUTER_SAVED_ROOT", dir.path())
                .env(
                    "CHAPCHOO_CONNECTOR_INITIAL_KEY",
                    session.initial_key.expose(),
                )
                .arg("computer-connector")
                .kill_on_drop(true),
        )?;
        let route = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if let Some(route) = app.resolve_connector_route(&agent.computer.id).await? {
                    return Ok::<_, Error>(route);
                }
                if child.lock().await.try_wait()?.is_some() {
                    return Err("Connector process exited before current route".into());
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await??;
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                let current=store.current_conversation(&user.token.digest(),&agent.id,AcceptanceTime::System.now()?).await?.ok_or("Missing conversation")?;
                if matches!(current.source,chapchoo_domain::ConversationSource::Ready {..}) {
                    if current.source.snapshot()!=Some(&snapshot) || current.source.position()!=Some(&state_position) {return Err("Connector published a different retained source".into());}
                    return Ok::<(),Error>(());
                }
                if child.lock().await.try_wait()?.is_some() {return Err("Connector exited during capture synchronization".into());}
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await??;
        let mut retained=Vec::new();
        while retained.len()<raw.len() {
            let chunk=store.conversation_event_chunk(&user.token.digest(),&chapchoo_persistence::EventChunkRead {
                agent_id:&agent.id,conversation_id:&accepted.conversation_id,snapshot:&snapshot,occurrence:position.occurrence(),
                kind:chapchoo_persistence::EventPayloadKind::Raw,offset:u32::try_from(retained.len())?,length:32768,
            },AcceptanceTime::System.now()?).await?;
            assert!(!chunk.bytes.is_empty());retained.extend(chunk.bytes);
        }
        assert_eq!(retained,raw.as_bytes());
        let (peer_channel,peer_incoming)=chapchoo_rpc::memory_transport();
        let peer=chapchoo_rpc::PeerServices::new(app.clone(),connections.clone());
        let peer_stopped=fixture.shutdown();
        fixture.server(Server::builder().add_service(peer.server()).serve_with_incoming_shutdown(peer_incoming,async {let _=peer_stopped.await;}));
        let delivery=chapchoo_rpc::PeerDelivery::new(app.clone());
        delivery.register_local(control.id.clone(),peer_channel)?;
        let processor=Arc::new(chapchoo_rpc::ConversationControlProcessor::new(app.clone(),
            Arc::new(chapchoo_service::ConversationService::new(store.clone())),delivery,Arc::new(AtomicBool::new(true))));
        let input_owner=chapchoo_service::ConversationInputService::new(store.clone(),processor.clone(),4)?;
        let setup_input=input_owner.accept(user.token.digest(),chapchoo_persistence::AcceptConversationInput {
            agent_id:agent.id.clone(),conversation_id:Some(accepted.conversation_id.clone()),request_id:"setup-existing".into(),
            target:chapchoo_domain::ConversationTarget::Snapshot(snapshot.clone()),text:chapchoo_domain::ConversationText::parse("Synthetic no-model setup".into())?,
        },AcceptanceTime::System).await?;
        let setup_draft=chapchoo_persistence::ConversationInputPreparation {
            agent_id:&agent.id,conversation_id:&accepted.conversation_id,operation_id:&setup_input.id,
        };
        // The unprivileged CLI fixture is intentionally not a restricted PID1
        // runtime. It reports this known pre-launch denial separately from ACK.
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {
                let facts=store.conversation_input_preparation(&user.token.digest(),&setup_draft,AcceptanceTime::System.now()?).await?;
                if let Some(chapchoo_persistence::ConversationPreparationFacts {state:chapchoo_persistence::ConversationPreparationState::Known(outcome),..})=facts {
                    if outcome!=chapchoo_persistence::ConversationPreparationOutcome::Failed(chapchoo_persistence::ConversationPreparationFailure::NotAdmitted) {return Err("Fixture unexpectedly admitted native execution".into());}
                    return Ok::<(),Error>(());
                }
                if child.lock().await.try_wait()?.is_some() {return Err("Connector exited before setup report".into());}
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await??;
        input_owner.finish().await?;
        let connector_digest = captured_credential.lock().map_err(|_| "Fixture credential owner is unavailable")?
            .clone().ok_or("Missing presented fixture credential")?;
        conversation_operations::exercise(fixture, conversation_operations::Context {
            store: &store, actor: &user.token.digest(), agent: &agent, processor,
            source: chapchoo_persistence::ConnectorSource {
                session_id: &session.id, control_id: &route.component.control_id,
                connection_id: &route.component.connection_id,
                credential: &connector_digest,
            },
            conversation: &accepted.conversation_id, snapshot: &snapshot,
            request_position: &request_position, state_position: &state_position,
        }).await?;
        store
            .revoke_provider_key(
                chapchoo_persistence::ProviderOwner::Bootstrap {
                    proof: &proof,
                    name: "Fixture",
                    capabilities,
                    substrate: &substrate,
                },
                "provider",
                &format!("{}:revoke", AcceptanceTime::System.now()?),
                AcceptanceTime::System.now()?,
            )
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
                    operation: Some(wire::connector_command::Operation::Observe(wire::ObserveConnector {})),
                },
                Instant::now() + Duration::from_secs(5),
            )
            .await?;
        assert_eq!(result.outcome, wire::ForwardingOutcome::Completed as i32);
        let result = result.result.ok_or("Missing Connector result")?;
        assert!(result.ready);
        assert_eq!(result.execution_id, execution.execution_id);
        let id = i32::try_from(child.lock().await.id().ok_or("Missing Connector PID")?)?;
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(id),
            nix::sys::signal::Signal::SIGTERM,
        )?;
        assert!(
            tokio::time::timeout(Duration::from_secs(10), child.lock().await.wait())
                .await??
                .success()
        );
        Ok(())
    })
    .await
}
