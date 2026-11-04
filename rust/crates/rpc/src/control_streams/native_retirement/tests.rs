//! Real Store/Control queue regressions with a deterministic synthetic Provider.
//! Native SDK stages and real daemon acceptance belong to the adapter layer.
use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey,
};
use chapchoo_domain::{
    DockerDaemonId, DockerVolumeIdentity, EmailAddress, NativeIdentity, NativeScope, Password,
};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AccountsConfig, AccountsService, LifecycleRequest, PasswordWork, ProviderCapabilities,
    ProviderSubstrate, SessionPolicy,
};
use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::AtomicUsize,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
#[path = "../../../tests/support/mod.rs"]
mod support;

#[derive(Debug)]
struct FailedCompletion {
    primary: Error,
    cleanup: Error,
}
impl std::fmt::Display for FailedCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Native fixture exercise and service completion both failed")
    }
}
impl std::error::Error for FailedCompletion {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let _ = &self.cleanup;
        Some(self.primary.as_ref())
    }
}

struct Fixture {
    _directory: tempfile::TempDir,
    services: ComponentConnectionServices,
    entry: Arc<Connection>,
    receivers: OutboundReceivers,
    context: wire::ComputerContext,
    credential: SecretDigest,
    token: BearerSecret,
}

impl Fixture {
    async fn run(
        mut self,
        exercise: impl AsyncFnOnce(&mut Self, &mut support::Fixture) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let primary = support::scope(async |owner| exercise(&mut self, owner).await).await;
        let cleanup = self.services.drain().await.map_err(Into::into);
        completed(primary, cleanup)
    }
}

fn completed(primary: Result<(), Error>, cleanup: Result<(), Error>) -> Result<(), Error> {
    match (primary, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(primary), Err(cleanup)) => Err(Box::new(FailedCompletion { primary, cleanup })),
    }
}

async fn fixture() -> Result<Fixture, Error> {
    fixture_with_action(Some(chapchoo_domain::LifecycleAction::Delete)).await
}

#[tokio::test]
async fn failed_native_exercise_joins_later_task_before_service_completion() -> Result<(), Error> {
    use std::future::Future;
    let f = fixture().await?;
    let services = f.services.clone();
    let entry = f.entry.clone();
    let (release, released) = tokio::sync::oneshot::channel();
    let mut exercise = Box::pin(f.run(async |_f, owned| {
        let _waiter = owned.call(async move {
            released.await?;
            Err::<(), Error>("Independent native fixture task failure".into())
        });
        Err("Primary native fixture exercise failure".into())
    }));
    std::future::poll_fn(|context| {
        assert!(exercise.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert!(services.inner.admission.load(Ordering::Acquire));
    release.send(()).map_err(|_| "Owned task was dropped")?;
    let failure = exercise.await.err().ok_or("Expected fixture failure")?;
    let causes = format!("{failure:?}");
    assert!(causes.contains("Primary native fixture exercise failure"));
    assert!(causes.contains("Independent native fixture task failure"));
    assert!(!services.inner.admission.load(Ordering::Acquire));
    assert!(entry.calls.is_closed());
    Ok(())
}

#[test]
fn native_fixture_completion_retains_primary_and_service_failure() -> Result<(), Error> {
    let failure = completed(Err("Primary".into()), Err("Service cleanup".into()))
        .err()
        .ok_or("Expected combined completion")?;
    let retained = failure
        .downcast_ref::<FailedCompletion>()
        .ok_or("Missing retained causes")?;
    assert_eq!(retained.primary.to_string(), "Primary");
    assert_eq!(retained.cleanup.to_string(), "Service cleanup");
    Ok(())
}

async fn fixture_with_action(
    action: Option<chapchoo_domain::LifecycleAction>,
) -> Result<Fixture, Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("retirement.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([17; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let store = Store::connect_sqlite(path, cipher).await?;
    let app = Arc::new(ComputersService::new(store.clone()));
    let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
    let key = app
        .exchange_bootstrap_provider(
            &proof,
            "Docker",
            ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            },
            &ProviderSubstrate::Docker(DockerDaemonId::parse("daemon:original".into())?),
            "key",
            AcceptanceTime::System,
        )
        .await?;
    let control = app.register_control("http://127.0.0.1:1").await?;
    let services = ComponentConnectionServices::new(
        app.clone(),
        control.id.clone(),
        Arc::new(AtomicBool::new(true)),
    );
    let (mut entry, receivers) =
        crate::control_streams::tests::entry(&services.inner, key.token.digest(), None).await?;
    Arc::get_mut(&mut entry)
        .ok_or("Entry already shared")?
        .policy = Some(ProviderPolicy {
        native_scope: NativeScope::DockerDaemon(DockerDaemonId::parse("daemon:original".into())?),
        connector_endpoint: "http://127.0.0.1:1".into(),
    });
    services.inner.activate(entry.clone()).await?;
    let setup = BearerSecret::generate()?;
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
            EmailAddress::parse("native@example.invalid")?,
            Password::parse("Aa1!native-provider".into())?,
            AcceptanceTime::System,
            None,
        )
        .await?;
    let team = accounts
        .list_teams(&user.token.digest(), AcceptanceTime::System.now()?)
        .await?
        .remove(0);
    let chapchoo_service::CreationOutcome::Present(agent) = app
        .create_cloud_agent(
            &user.token.digest(),
            &team.id,
            "create",
            "provider",
            "Native",
            AcceptanceTime::System,
        )
        .await?
    else {
        return Err("Expected Agent".into());
    };
    let source = services.inner.source(&entry).ok_or("Missing source")?;
    let scope = ComputerRequest {
        computer_id: &agent.computer.id,
        request_id: "create",
    };
    let execution = app
        .reserve_execution(
            &source,
            scope,
            &NativeScope::DockerDaemon(DockerDaemonId::parse("daemon:original".into())?),
        )
        .await?;
    app.claim_execution_dispatch(&source, scope, &execution.execution_id)
        .await?;
    app.mark_start_dispatched(&source, scope, &execution.execution_id)
        .await?;
    for resource in &execution.resources {
        let identity = match resource.kind() {
            ManagedResourceKind::DockerContainer => NativeIdentity::DockerContainer(
                chapchoo_domain::DockerContainerId::parse("a".repeat(64))?,
            ),
            ManagedResourceKind::DockerVolume => {
                NativeIdentity::DockerVolume(DockerVolumeIdentity::local(
                    "local",
                    "local",
                    &BTreeMap::new(),
                    Some("2026-11-03T00:00:00Z".into()),
                    BTreeMap::new(),
                )?)
            }
            _ => return Err("Wrong native kind".into()),
        };
        app.report_resource(
            &source,
            scope,
            chapchoo_service::ResourceReport {
                native: resource.native(),
                execution_id: resource.execution_id.as_deref(),
                evidence: chapchoo_service::ResourceEvidence::OwnedPresent {
                    identity: &identity,
                },
            },
        )
        .await?;
    }
    app.report_execution_running(&source, scope, &execution.execution_id)
        .await?;
    if let Some(action) = action {
        app.request_lifecycle(
            &user.token.digest(),
            &agent.id,
            &LifecycleRequest {
                request_id: "delete".into(),
                expected_request_id: "create".into(),
                action,
            },
            AcceptanceTime::System,
        )
        .await?;
    }
    Ok(Fixture {
        _directory: directory,
        services,
        entry,
        receivers,
        context: wire::ComputerContext {
            computer_id: agent.computer.id,
            request_id: if action.is_some() { "delete" } else { "create" }.into(),
            execution_id: execution.execution_id,
        },
        credential: key.token.digest(),
        token: key.token,
    })
}

async fn snapshot(fixture: &Fixture) -> Result<DeliverySnapshot, Error> {
    let source = fixture
        .services
        .inner
        .source(&fixture.entry)
        .ok_or("Missing source")?;
    Ok(fixture
        .services
        .inner
        .service
        .provider_delivery_snapshot(
            &source,
            ComputerRequest {
                computer_id: &fixture.context.computer_id,
                request_id: &fixture.context.request_id,
            },
            None,
        )
        .await?)
}

async fn next(receivers: &mut OutboundReceivers) -> Result<wire::ProviderCommand, Error> {
    let queued = tokio::time::timeout(Duration::from_secs(5), receivers.commands.recv())
        .await?
        .ok_or("No command")?;
    let Some(wire::control_frame::Payload::Provider(command)) =
        queued.into_frame().and_then(|f| f.payload)
    else {
        return Err("Wrong frame".into());
    };
    Ok(command)
}

/// The synthetic Provider models already-qualified native observations, not SDK IO.
async fn respond(
    services: &ComponentConnectionServices,
    entry: &Connection,
    command: wire::ProviderCommand,
    absent: &mut HashSet<String>,
) -> Result<(), Error> {
    respond_with_terminal(services, entry, command, absent, false).await
}

async fn respond_with_terminal(
    services: &ComponentConnectionServices,
    entry: &Connection,
    command: wire::ProviderCommand,
    absent: &mut HashSet<String>,
    terminal: bool,
) -> Result<(), Error> {
    let source = services.inner.source(entry).ok_or("Missing source")?;
    let computer = command.computer.as_ref().ok_or("Missing context")?;
    let scope = ComputerRequest {
        computer_id: &computer.computer_id,
        request_id: &computer.request_id,
    };
    let current = services
        .inner
        .service
        .provider_delivery_snapshot(&source, scope, None)
        .await?;
    crate::peer::validate_provider(&command, &current)?;
    let (resources, retire, cleanup) = match command.operation.as_ref().ok_or("No operation")? {
        wire::provider_command::Operation::Observe(value) => {
            (value.resources.clone(), false, false)
        }
        wire::provider_command::Operation::Retire(value) => {
            services.verify_native_dequeue(&source, &command)?;
            let resource = value
                .docker_container
                .clone()
                .ok_or("No native container")?;
            absent.insert(resource.name.clone());
            (vec![resource], true, false)
        }
        wire::provider_command::Operation::Cleanup(value) => {
            services.verify_native_dequeue(&source, &command)?;
            services
                .inner
                .service
                .verify_native_cleanup_suffix(&source, scope)
                .await?;
            for resource in &value.resources {
                absent.insert(resource.name.clone());
            }
            (value.resources.clone(), false, value.confirm_complete)
        }
        _ => return Err("Unexpected preparation".into()),
    };
    for resource in resources {
        let decoded = crate::native_resources::decode(&resource)?;
        let identity = decoded
            .identity
            .as_ref()
            .ok_or("Unallocated fixture identity")?;
        let evidence = if absent.contains(&resource.name) {
            chapchoo_service::ResourceEvidence::DefinitivelyAbsent {
                expected_identity: Some(identity),
            }
        } else {
            chapchoo_service::ResourceEvidence::OwnedPresent { identity }
        };
        services
            .inner
            .service
            .report_resource(
                &source,
                scope,
                chapchoo_service::ResourceReport {
                    native: &decoded.native,
                    execution_id: decoded.execution_id.as_deref(),
                    evidence,
                },
            )
            .await?;
    }
    if retire {
        services
            .inner
            .service
            .retire_execution(&source, scope, &computer.execution_id)
            .await?;
    }
    if cleanup {
        services
            .inner
            .service
            .confirm_computer_cleanup(&source, scope)
            .await?;
    }
    services
        .inner
        .provider_frame(
            entry,
            wire::ProviderFrame {
                payload: Some(wire::provider_frame::Payload::Result(
                    wire::ProviderResult {
                        identity: command.identity,
                        computer: command.computer,
                        outcome: wire::CommandOutcome::Completed as i32,
                        resources: vec![],
                        execution: wire::ExecutionFact::Unspecified as i32,
                        execution_terminal: terminal,
                    },
                )),
            },
        )
        .await?;
    Ok(())
}

#[tokio::test]
async fn stop_restart_and_terminal_early_paths_all_use_the_same_native_gate() -> Result<(), Error> {
    for action in [
        Some(chapchoo_domain::LifecycleAction::Stop),
        Some(chapchoo_domain::LifecycleAction::Restart),
        None,
    ] {
        fixture_with_action(action)
            .await?
            .run(async |f, owned| {
                let services = f.services.clone();
                let context = f.context.clone();
                let owner = wire::ExpectedOwner {
                    instance_id: services.inner.instance_id.clone(),
                    connection_id: f.entry.prepared.owner().connection_id.clone(),
                };
                let convergence = owned.call(async move {
                    services
                        .converge_provider(
                            "provider",
                            &owner,
                            &context,
                            Instant::now() + RESPONSE_BUDGET,
                        )
                        .await
                });
                let first_observe = next(&mut f.receivers).await?;
                assert!(matches!(
                    first_observe.operation,
                    Some(wire::provider_command::Operation::Observe(_))
                ));
                respond_with_terminal(
                    &f.services,
                    &f.entry,
                    first_observe,
                    &mut HashSet::new(),
                    action.is_none(),
                )
                .await?;
                let retire = loop {
                    let command = next(&mut f.receivers).await?;
                    if matches!(
                        command.operation,
                        Some(wire::provider_command::Operation::Observe(_))
                    ) {
                        respond(&f.services, &f.entry, command, &mut HashSet::new()).await?;
                    } else {
                        break command;
                    }
                };
                let Some(wire::provider_command::Operation::Retire(value)) =
                    retire.operation.as_ref()
                else {
                    return Err("Expected a native retirement".into());
                };
                assert!(value.pod.is_none());
                assert_eq!(value.require_terminal, action.is_none());
                let source = f.services.inner.source(&f.entry).ok_or("No source")?;
                assert_eq!(snapshot(f).await?.computer.current_phase, "UNCERTAIN");
                assert!(f.services.verify_native_dequeue(&source, &retire).is_ok());
                respond(&f.services, &f.entry, retire.clone(), &mut HashSet::new()).await?;
                assert_eq!(
                    convergence.await?.outcome,
                    wire::ForwardingOutcome::Completed as i32
                );
                assert!(snapshot(f).await?.computer.current_execution_id.is_none());
                assert!(
                    f.services
                        .dispatch_entry(&f.entry, retire, Instant::now() + RESPONSE_BUDGET)
                        .await
                        .is_err()
                );
                assert!(f.receivers.commands.try_recv().is_err());
                Ok(())
            })
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn coordinated_delete_submits_one_retirement_then_only_the_unsent_volume_suffix()
-> Result<(), Error> {
    let f = fixture().await?;
    // Keep the remaining fixture (including its TempDir) outside the callback
    // while moving only its outbound receiver into the synthetic Provider.
    let completion = f.services.clone();
    let primary = support::scope(async |owned| {
        let services = f.services.clone();
        let entry = f.entry.clone();
        let mut receivers = f.receivers;
        let retired = Arc::new(AtomicUsize::new(0));
        let removed_volume = Arc::new(AtomicUsize::new(0));
        let retire_count = retired.clone();
        let volume_count = removed_volume.clone();
        let worker = owned.call(async move {
            let mut absent = HashSet::new();
            loop {
                let command = next(&mut receivers).await?;
                if matches!(
                    command.operation,
                    Some(wire::provider_command::Operation::Retire(_))
                ) {
                    retire_count.fetch_add(1, Ordering::SeqCst);
                }
                let final_cleanup = matches!(
                    command.operation,
                    Some(wire::provider_command::Operation::Cleanup(_))
                );
                if final_cleanup {
                    volume_count.fetch_add(1, Ordering::SeqCst);
                }
                respond(&services, &entry, command, &mut absent).await?;
                if final_cleanup {
                    return Ok::<_, Error>(());
                }
            }
        });
        let source = f.services.inner.source(&f.entry).ok_or("Missing source")?;
        let scope = ComputerRequest {
            computer_id: &f.context.computer_id,
            request_id: "delete",
        };
        let initial = f
            .services
            .inner
            .service
            .provider_delivery_snapshot(&source, scope, None)
            .await?;
        f.services
            .reconcile_native_retirement(
                &f.entry,
                &f.context,
                initial,
                false,
                Instant::now() + RESPONSE_BUDGET,
            )
            .await?;
        let after_retirement = f
            .services
            .inner
            .service
            .provider_delivery_snapshot(&source, scope, None)
            .await?;
        assert!(after_retirement.computer.current_execution_id.is_none());
        assert_eq!(after_retirement.computer.current_phase, "UNCERTAIN");
        f.services
            .reconcile_native_retirement(
                &f.entry,
                &f.context,
                after_retirement,
                false,
                Instant::now() + RESPONSE_BUDGET,
            )
            .await?;
        worker.await?;
        assert_eq!(retired.load(Ordering::SeqCst), 1);
        assert_eq!(removed_volume.load(Ordering::SeqCst), 1);
        Ok(())
    })
    .await;
    let cleanup = completion.drain().await.map_err(Into::into);
    completed(primary, cleanup)
}

#[tokio::test]
async fn reconnect_loses_mutation_correlation_and_allows_only_observation() -> Result<(), Error> {
    fixture()
        .await?
        .run(async |f, owned| {
            let initial = snapshot(f).await?;
            let services = f.services.clone();
            let old_entry = f.entry.clone();
            let context = f.context.clone();
            let pending = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &old_entry,
                        &context,
                        initial,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observe = next(&mut f.receivers).await?;
            respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
            let old_retire = next(&mut f.receivers).await?;
            owned.abort(&pending);
            assert!(pending.await.is_err());
            let old_entry = f.entry.clone();
            let (replacement, receiver) =
                crate::control_streams::tests::entry(&f.services.inner, f.credential.clone(), None)
                    .await?;
            f.services.inner.activate(replacement.clone()).await?;
            f.entry = replacement;
            f.receivers = receiver;
            assert!(old_entry.calls.is_closed());
            let old_source = f.services.inner.source(&old_entry).ok_or("No old source")?;
            assert!(
                f.services
                    .verify_native_dequeue(&old_source, &old_retire)
                    .is_err()
            );
            let new_source = f.services.inner.source(&f.entry).ok_or("No new source")?;
            let absent_container = match old_retire.operation.as_ref() {
                Some(wire::provider_command::Operation::Retire(value)) => value
                    .docker_container
                    .as_ref()
                    .ok_or("Missing container")?
                    .name
                    .clone(),
                _ => return Err("Expected retirement".into()),
            };
            let mut reconstructed = old_retire;
            reconstructed
                .identity
                .as_mut()
                .ok_or("No call")?
                .connection_id = new_source.connection_id.into();
            assert!(
                f.services
                    .verify_native_dequeue(&new_source, &reconstructed)
                    .is_err()
            );
            assert!(
                f.services
                    .dispatch_entry(&f.entry, reconstructed, Instant::now() + RESPONSE_BUDGET)
                    .await
                    .is_err()
            );
            let current = snapshot(f).await?;
            assert_eq!(current.computer.current_phase, "UNCERTAIN");
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let recovery = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        current,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observation = next(&mut f.receivers).await?;
            assert!(matches!(
                observation.operation,
                Some(wire::provider_command::Operation::Observe(_))
            ));
            respond(
                &f.services,
                &f.entry,
                observation,
                &mut HashSet::from([absent_container]),
            )
            .await?;
            assert_eq!(
                recovery.await?.outcome,
                wire::ForwardingOutcome::Uncertain as i32
            );
            assert!(f.receivers.commands.try_recv().is_err());
            let after = snapshot(f).await?;
            assert!(after.computer.current_execution_id.is_none());
            assert!(
                after
                    .resources
                    .iter()
                    .any(|r| r.kind() == ManagedResourceKind::DockerVolume
                        && r.presence == ResourcePresence::Present)
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn exact_not_started_can_release_but_late_old_result_cannot_release_a_new_call()
-> Result<(), Error> {
    fixture()
        .await?
        .run(async |f, owned| {
            let initial = snapshot(f).await?;
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let first = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        initial,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observe = next(&mut f.receivers).await?;
            respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
            let old = next(&mut f.receivers).await?;
            let rejected = wire::ProviderResult {
                identity: old.identity.clone(),
                computer: old.computer.clone(),
                outcome: wire::CommandOutcome::NotStarted as i32,
                resources: vec![],
                execution: wire::ExecutionFact::Unspecified as i32,
                execution_terminal: false,
            };
            f.services
                .inner
                .provider_frame(
                    &f.entry,
                    wire::ProviderFrame {
                        payload: Some(wire::provider_frame::Payload::Result(rejected.clone())),
                    },
                )
                .await?;
            assert_eq!(
                first.await?.result.ok_or("No result")?.outcome,
                wire::CommandOutcome::NotStarted as i32
            );
            let current = snapshot(f).await?;
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let releasing = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        current,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observe = next(&mut f.receivers).await?;
            respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
            assert_eq!(
                releasing.await?.outcome,
                wire::ForwardingOutcome::Unavailable as i32
            );
            assert_eq!(snapshot(f).await?.computer.current_phase, "DELETING");
            let current = snapshot(f).await?;
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let new = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        current,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observe = next(&mut f.receivers).await?;
            respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
            let new_command = next(&mut f.receivers).await?;
            assert_ne!(new_command.identity, old.identity);
            f.services
                .inner
                .provider_frame(
                    &f.entry,
                    wire::ProviderFrame {
                        payload: Some(wire::provider_frame::Payload::Result(rejected)),
                    },
                )
                .await?;
            assert_eq!(snapshot(f).await?.computer.current_phase, "UNCERTAIN");
            let source = f.services.inner.source(&f.entry).ok_or("No source")?;
            assert!(f.services.verify_native_dequeue(&source, &old).is_err());
            assert!(
                f.services
                    .verify_native_dequeue(&source, &new_command)
                    .is_ok()
            );
            owned.abort(&new);
            assert!(new.await.is_err());
            assert!(f.receivers.commands.try_recv().is_err());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn waiter_deadline_preserves_only_the_exact_live_task_until_native_settlement()
-> Result<(), Error> {
    use wire::provider_authentication_client::ProviderAuthenticationClient;

    for native_outcome in [
        wire::CommandOutcome::Completed,
        wire::CommandOutcome::Uncertain,
    ] {
        fixture()
            .await?
            .run(async |f, owned| {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
                let endpoint = format!("http://{}", listener.local_addr()?);
                let auth = crate::ComponentAuthenticationServices::new(
                    f.services.inner.service.clone(),
                    f.services.inner.instance_id.clone(),
                    f.services.clone(),
                );
                let stopped = owned.shutdown();
                owned.server(async move {
                    tonic::transport::Server::builder()
                        .add_service(auth.provider_server())
                        .serve_with_incoming_shutdown(
                            tokio_stream::wrappers::TcpListenerStream::new(listener),
                            async {
                                let _ = stopped.await;
                            },
                        )
                        .await
                });
                let mut client = ProviderAuthenticationClient::connect(endpoint).await?;
                let initial = snapshot(f).await?;
                let services = f.services.clone();
                let entry = f.entry.clone();
                let context = f.context.clone();
                let pending = owned.call(async move {
                    services
                        .reconcile_native_retirement(
                            &entry,
                            &context,
                            initial,
                            false,
                            Instant::now() + Duration::from_secs(2),
                        )
                        .await
                });
                let observe = next(&mut f.receivers).await?;
                respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
                let retire = next(&mut f.receivers).await?;
                assert!(matches!(
                    retire.operation,
                    Some(wire::provider_command::Operation::Retire(_))
                ));
                let check = || -> Result<tonic::Request<wire::ProviderCommandCheck>, Error> {
                    let mut request = tonic::Request::new(wire::ProviderCommandCheck {
                        source: Some(wire::ProviderConnectionCheck {
                            provider_id: "provider".into(),
                            instance_id: f.services.inner.instance_id.clone(),
                            connection_id: f.entry.prepared.owner().connection_id.clone(),
                        }),
                        command: Some(retire.clone()),
                    });
                    request.metadata_mut().insert(
                        "authorization",
                        format!("Bearer {}", f.token.expose()).parse()?,
                    );
                    Ok(request)
                };
                assert!(
                    client
                        .check_command(check()?)
                        .await?
                        .into_inner()
                        .prior_running_completion
                );
                // Await the actual invoke deadline, not cancellation or a scheduling
                // sleep. The synthetic native task remains unsettled behind this barrier.
                assert_eq!(
                    pending.await?.outcome,
                    wire::ForwardingOutcome::Uncertain as i32
                );
                assert!(
                    client
                        .check_command(check()?)
                        .await?
                        .into_inner()
                        .prior_running_completion
                );
                assert!(
                    f.services
                        .dispatch_entry(&f.entry, retire.clone(), Instant::now() + RESPONSE_BUDGET,)
                        .await
                        .is_err()
                );
                let current = snapshot(f).await?;
                let services = f.services.clone();
                let entry = f.entry.clone();
                let context = f.context.clone();
                let recovery = owned.call(async move {
                    services
                        .reconcile_native_retirement(
                            &entry,
                            &context,
                            current,
                            false,
                            Instant::now() + RESPONSE_BUDGET,
                        )
                        .await
                });
                let observe = next(&mut f.receivers).await?;
                assert!(matches!(
                    observe.operation,
                    Some(wire::provider_command::Operation::Observe(_))
                ));
                respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
                assert_eq!(
                    recovery.await?.outcome,
                    wire::ForwardingOutcome::Uncertain as i32
                );
                assert!(f.receivers.commands.try_recv().is_err());
                assert!(
                    client
                        .check_command(check()?)
                        .await?
                        .into_inner()
                        .prior_running_completion
                );
                if native_outcome == wire::CommandOutcome::Completed {
                    respond(&f.services, &f.entry, retire.clone(), &mut HashSet::new()).await?;
                } else {
                    f.services
                        .inner
                        .provider_frame(
                            &f.entry,
                            wire::ProviderFrame {
                                payload: Some(wire::provider_frame::Payload::Result(
                                    wire::ProviderResult {
                                        identity: retire.identity.clone(),
                                        computer: retire.computer.clone(),
                                        outcome: native_outcome as i32,
                                        resources: vec![],
                                        execution: wire::ExecutionFact::Unspecified as i32,
                                        execution_terminal: false,
                                    },
                                )),
                            },
                        )
                        .await?;
                }
                assert!(client.check_command(check()?).await.is_err());
                assert!(f.receivers.commands.try_recv().is_err());
                Ok(())
            })
            .await?;
    }
    Ok(())
}

#[tokio::test]
async fn lost_response_retains_exact_call_and_late_completion_does_not_replay_retire()
-> Result<(), Error> {
    fixture()
        .await?
        .run(async |f, owned| {
            let initial = snapshot(f).await?;
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let pending = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        initial,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observe = next(&mut f.receivers).await?;
            respond(&f.services, &f.entry, observe, &mut HashSet::new()).await?;
            let retire = next(&mut f.receivers).await?;
            assert!(matches!(
                retire.operation,
                Some(wire::provider_command::Operation::Retire(_))
            ));
            owned.abort(&pending);
            assert!(pending.await.is_err());
            let source = f.services.inner.source(&f.entry).ok_or("Missing source")?;
            assert!(f.services.verify_native_dequeue(&source, &retire).is_ok());
            let mut changed = retire.clone();
            changed.identity.as_mut().ok_or("Missing call")?.call_id = "foreign-call".into();
            assert!(f.services.verify_native_dequeue(&source, &changed).is_err());
            changed = retire.clone();
            if let Some(wire::provider_command::Operation::Retire(value)) =
                changed.operation.as_mut()
                && let Some(resource) = value.docker_container.as_mut()
                && let Some(wire::managed_resource::Native::DockerContainer(native)) =
                    resource.native.as_mut()
            {
                native.container_id = Some("b".repeat(64));
            }
            assert!(f.services.verify_native_dequeue(&source, &changed).is_err());
            assert!(
                f.services
                    .dispatch_entry(&f.entry, retire.clone(), Instant::now() + RESPONSE_BUDGET)
                    .await
                    .is_err()
            );
            let resumed_snapshot = snapshot(f).await?;
            let services = f.services.clone();
            let entry = f.entry.clone();
            let context = f.context.clone();
            let resumed = owned.call(async move {
                services
                    .reconcile_native_retirement(
                        &entry,
                        &context,
                        resumed_snapshot,
                        false,
                        Instant::now() + RESPONSE_BUDGET,
                    )
                    .await
            });
            let observation_only = next(&mut f.receivers).await?;
            assert!(matches!(
                observation_only.operation,
                Some(wire::provider_command::Operation::Observe(_))
            ));
            respond(&f.services, &f.entry, observation_only, &mut HashSet::new()).await?;
            assert_eq!(
                resumed.await?.outcome,
                wire::ForwardingOutcome::Uncertain as i32
            );
            respond(&f.services, &f.entry, retire, &mut HashSet::new()).await?;
            assert!(
                f.entry
                    .native
                    .lock()
                    .map_err(|_| "Registry poisoned")?
                    .values()
                    .all(|p| p.state == DispatchState::Completed)
            );
            assert!(f.receivers.commands.try_recv().is_err());
            Ok(())
        })
        .await
}
