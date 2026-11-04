mod support;
#[path = "support/tasks.rs"]
mod tasks;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_domain::{
    KubernetesUid, LifecycleAction, LifecyclePhase, NativeIdentity, ObservedPower, ResourcePresence,
};
use chapchoo_persistence::{
    ComputerRequest, CreateCloudAgent, CreationOutcome, ExecutionDraft, LifecycleRequest,
    ManagedResource, ManagedResourceKind, ProviderSource, ResourceEvidence as NativeEvidence,
    ResourceReport, Store, StoreError, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type TestError = Box<dyn Error + Send + Sync>;
// Keep this Kubernetes-only fixture's native assertions readable. Production
// ingress and storage consume only the shared typed evidence contract.
#[derive(Clone, Copy)]
enum ResourceEvidence<'a> {
    OwnedPresent { uid: &'a str },
    DefinitivelyAbsent { expected_uid: Option<&'a str> },
    Unknown,
}
fn now() -> Result<i64, TestError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
#[derive(Clone)]
struct FixtureIds {
    team: String,
    provider: String,
    image: String,
    agents: std::collections::BTreeMap<String, chapchoo_persistence::AgentSummary>,
}
impl FixtureIds {
    fn computer(&self, name: &str) -> &str {
        &self.agents[name].computer.id
    }
    fn agent(&self, name: &str) -> &str {
        &self.agents[name].id
    }
}
fn creation(ids: &FixtureIds, id: &str) -> CreateCloudAgent {
    CreateCloudAgent {
        team_id: ids.team.clone(),
        request_id: format!("create-{id}"),
        provider_id: ids.provider.clone(),
        name: "Fixture".into(),
        profile: chapchoo_persistence::CreateAgentProfile {
            handle: id.into(),
            image_id: ids.image.clone(),
            palette_key: "sage".into(),
        },
    }
}
async fn create_fixture(
    store: &Store,
    ids: &mut FixtureIds,
    owner: &chapchoo_crypto::SecretDigest,
    name: &str,
) -> Result<(), TestError> {
    let CreationOutcome::Present(agent) = store
        .create_cloud_agent(owner, &creation(ids, name), now()?)
        .await?
    else {
        return Err("Missing fixture Agent".into());
    };
    ids.agents.insert(name.into(), *agent);
    Ok(())
}
fn draft(id: &str) -> ExecutionDraft {
    ExecutionDraft {
        native: chapchoo_persistence::ExecutionResources::Kubernetes {
            namespace: "fixture".into(),
            pod_name: format!("pod-{id}"),
            bootstrap_secret_name: format!("secret-{id}"),
            saved_pvc_name: format!("pvc-{id}"),
        },
    }
}
fn scope<'a>(ids: &'a FixtureIds, name: &str, request_id: &'a str) -> ComputerRequest<'a> {
    ComputerRequest {
        computer_id: ids.computer(name),
        request_id,
    }
}
fn kind(resource: &ManagedResource) -> Result<ManagedResourceKind, TestError> {
    Ok(resource.kind())
}
async fn report(
    store: &Store,
    source: &ProviderSource<'_>,
    scope: ComputerRequest<'_>,
    resource: &ManagedResource,
    evidence: ResourceEvidence<'_>,
) -> Result<(), StoreError> {
    let identity = match evidence {
        ResourceEvidence::OwnedPresent { uid } => Some(uid),
        ResourceEvidence::DefinitivelyAbsent { expected_uid } => expected_uid,
        ResourceEvidence::Unknown => None,
    }
    .map(|uid| {
        KubernetesUid::parse(uid.to_owned())
            .map(NativeIdentity::KubernetesObject)
            .map_err(|_| StoreError::InvalidInput)
    })
    .transpose()?;
    let native_evidence = match evidence {
        ResourceEvidence::OwnedPresent { .. } => NativeEvidence::OwnedPresent {
            identity: identity.as_ref().ok_or(StoreError::InvalidInput)?,
        },
        ResourceEvidence::DefinitivelyAbsent { .. } => NativeEvidence::DefinitivelyAbsent {
            expected_identity: identity.as_ref(),
        },
        ResourceEvidence::Unknown => NativeEvidence::Unknown,
    };
    store
        .report_resource(
            source,
            scope,
            ResourceReport {
                native: resource.native(),
                execution_id: resource.execution_id.as_deref(),
                evidence: native_evidence,
            },
        )
        .await
}
async fn present_all(
    store: &Store,
    source: &ProviderSource<'_>,
    scope: ComputerRequest<'_>,
    execution: &str,
) -> Result<(), TestError> {
    for resource in store.managed_resources(source, scope).await? {
        if resource
            .execution_id
            .as_deref()
            .is_some_and(|id| id != execution)
        {
            continue;
        }
        let uid = resource
            .kubernetes_uid()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("uid-{}", resource.native().name()));
        report(
            store,
            source,
            scope,
            &resource,
            ResourceEvidence::OwnedPresent { uid: &uid },
        )
        .await?;
    }
    Ok(())
}

fn mutate_fixture(connection: &mut impl SimpleConnection, sql: &str) -> Result<(), TestError> {
    // The synchronous PostgreSQL test adapter owns a runtime. Fixture-only
    // mutations leave the async context explicitly instead of nesting runtimes.
    tokio::task::block_in_place(|| connection.batch_execute(sql))?;
    Ok(())
}

async fn replace_provider(
    store: &Store,
    source: &ProviderSource<'_>,
) -> Result<chapchoo_persistence::ConnectionOwner, TestError> {
    let prepared = store
        .prepare_provider_connection(source.provider_id, source.credential, source.control_id)
        .await?;
    store
        .admit_provider_connection(
            source.provider_id,
            source.credential,
            prepared.expected.as_ref(),
            &prepared.owner,
            chapchoo_persistence::DirectoryPolicy::default(),
        )
        .await?;
    Ok(prepared.owner)
}

async fn exercise(
    store: Store,
    connection: &mut impl SimpleConnection,
    owner: BearerSecret,
    provider: BearerSecret,
) -> Result<(), TestError> {
    let owner_digest = owner.digest();
    let provider_digest = provider.digest();
    let setup = BearerSecret::generate()?;
    let initialized = store
        .initialize_admin(
            &setup.digest(),
            &setup,
            &chapchoo_persistence::NewAccount {
                email: chapchoo_domain::EmailAddress::parse("fixture@example.invalid")?,
                password: chapchoo_crypto::PasswordHasher::new(chapchoo_crypto::PasswordCost::new(
                    10,
                )?)
                .hash(&chapchoo_domain::Password::parse("Aa1!fixture".into())?)?,
                session: chapchoo_persistence::NewSession {
                    token: BearerSecret::parse(owner.expose().into())?,
                },
                display: None,
            },
            now()?,
            chapchoo_persistence::SessionPolicy::default(),
        )
        .await?;
    assert_eq!(
        initialized.user.id,
        store.current_identity(&owner_digest, now()?).await?.id
    );
    let proof = chapchoo_crypto::BootstrapProviderPermit::new("provider".into())?.authenticated();
    store
        .configure_system_provider(
            &proof,
            "Fixture",
            chapchoo_persistence::ProviderCapabilities {
                create: true,
                start: true,
                stop: true,
                restart: true,
            },
            &chapchoo_domain::ProviderSubstrate::Kubernetes,
            now()?,
        )
        .await?;
    let key = store
        .rotate_provider_key(
            chapchoo_persistence::ProviderOwner::System(&proof),
            "provider",
            &chapchoo_persistence::ComponentKeyDraft {
                request_id: format!("{}:key", now()?),
                token: BearerSecret::parse(provider.expose().into())?,
            },
            chapchoo_persistence::ComponentKeyPolicy::default(),
            now()?,
        )
        .await?;
    let provider_id = key.provider_id;
    let control = store
        .register_control(
            "http://127.0.0.1:1",
            chapchoo_persistence::DirectoryPolicy::default(),
        )
        .await?;
    let prepared = store
        .prepare_provider_connection(&provider_id, &provider_digest, &control.id)
        .await?;
    store
        .admit_provider_connection(
            &provider_id,
            &provider_digest,
            prepared.expected.as_ref(),
            &prepared.owner,
            chapchoo_persistence::DirectoryPolicy::default(),
        )
        .await?;
    let mut ids = FixtureIds {
        team: store.teams(&owner_digest, now()?).await?.remove(0).id,
        provider: provider_id.clone(),
        image: store
            .agent_profile_images(&owner_digest, None, 1, now()?)
            .await?
            .remove(0)
            .id,
        agents: Default::default(),
    };
    let source = ProviderSource {
        provider_id: &provider_id,
        control_id: &control.id,
        connection_id: &prepared.owner.connection_id,
        credential: &provider_digest,
    };
    for id in ["one", "two", "collision", "pending", "empty", "dispatch"] {
        create_fixture(&store, &mut ids, &owner_digest, id).await?;
    }
    let dispatch = scope(&ids, "dispatch", "create-dispatch");
    let dispatch_execution = store
        .reserve_execution(&source, dispatch, &draft("dispatch-once"))
        .await?;
    let dispatch_id = dispatch_execution.execution_id.as_str();
    assert!(
        store
            .claim_execution_dispatch(&source, dispatch, dispatch_id)
            .await?
    );
    let admitted = store
        .delivery_snapshot(Some(&owner_digest), dispatch, Some(dispatch_id))
        .await?;
    assert!(admitted.computer.prepare_admitted_at.is_some());
    assert!(admitted.computer.initial_start_dispatch_at.is_none());
    store
        .release_undelivered_prepare(&source, dispatch, dispatch_id)
        .await?;
    assert!(
        store
            .claim_execution_dispatch(&source, dispatch, dispatch_id)
            .await?
    );
    store
        .mark_start_dispatched(&source, dispatch, dispatch_id)
        .await?;
    store
        .release_undelivered_prepare(&source, dispatch, dispatch_id)
        .await?;
    assert!(
        store
            .delivery_snapshot(Some(&owner_digest), dispatch, Some(dispatch_id))
            .await?
            .computer
            .prepare_admitted_at
            .is_some(),
        "A dequeued call never becomes replayable"
    );
    assert!(
        !store
            .claim_execution_dispatch(&source, dispatch, dispatch_id)
            .await?
    );
    let successor_owner = replace_provider(&store, &source).await?;
    let successor = ProviderSource {
        connection_id: &successor_owner.connection_id,
        ..source
    };
    assert!(
        !store
            .claim_execution_dispatch(&successor, dispatch, dispatch_id)
            .await?
    );
    assert!(matches!(
        store
            .claim_execution_dispatch(&source, dispatch, dispatch_id)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let source = successor;
    let dispatch_agent = store
        .agent(&owner_digest, ids.agent("dispatch"), now()?)
        .await?;
    assert!(
        store
            .retry_preparation(
                &owner_digest,
                &dispatch_agent.id,
                "retry-dispatch",
                "create-dispatch",
                now()?
            )
            .await
            .is_err()
    );
    store
        .report_preparation_blocked(&source, dispatch, dispatch_id)
        .await?;
    mutate_fixture(
        connection,
        "UPDATE computer_providers SET can_create=FALSE WHERE registration_key='provider'",
    )?;
    assert!(
        !store
            .agent(&owner_digest, &dispatch_agent.id, now()?)
            .await?
            .computer
            .preparation_retry_available
    );
    assert!(
        store
            .retry_preparation(
                &owner_digest,
                &dispatch_agent.id,
                "incapable-retry",
                "create-dispatch",
                now()?
            )
            .await
            .is_err()
    );
    mutate_fixture(
        connection,
        "UPDATE computer_providers SET can_create=TRUE WHERE registration_key='provider'",
    )?;
    assert!(
        store
            .agent(&owner_digest, &dispatch_agent.id, now()?)
            .await?
            .computer
            .preparation_retry_available
    );
    let retried = store
        .retry_preparation(
            &owner_digest,
            &dispatch_agent.id,
            "retry-dispatch",
            "create-dispatch",
            now()?,
        )
        .await?;
    assert_eq!(retried.computer.current_request_id, "retry-dispatch");
    assert_eq!(
        store
            .retry_preparation(
                &owner_digest,
                &dispatch_agent.id,
                "retry-dispatch",
                "create-dispatch",
                now()?
            )
            .await?
            .id,
        retried.id
    );
    let retry_scope = scope(&ids, "dispatch", "retry-dispatch");
    let retry_snapshot = store
        .delivery_snapshot(Some(&owner_digest), retry_scope, Some(dispatch_id))
        .await?;
    assert_eq!(
        retry_snapshot.computer.current_execution_id.as_deref(),
        Some(dispatch_id)
    );
    assert!(retry_snapshot.computer.initial_start_dispatch_at.is_none());
    assert!(
        store
            .claim_execution_dispatch(&source, retry_scope, dispatch_id)
            .await?
    );
    assert!(
        store
            .retry_preparation(
                &owner_digest,
                &dispatch_agent.id,
                "stale-retry",
                "create-dispatch",
                now()?
            )
            .await
            .is_err()
    );
    let one = scope(&ids, "one", "create-one");
    let reserved = store
        .reserve_execution(&source, one, &draft("first"))
        .await?;
    let first = reserved.execution_id.as_str();
    assert_ne!(first, dispatch_id);
    assert_eq!(reserved.resources.len(), 3);
    let delivery = store
        .delivery_snapshot(Some(&owner_digest), one, Some(first))
        .await?;
    assert_eq!(delivery.computer.provider_id, provider_id);
    assert_eq!(delivery.resources.len(), 3);
    assert!(delivery.execution.is_some_and(|e| e.retired_at.is_none()));
    assert!(matches!(
        store
            .delivery_snapshot(Some(&BearerSecret::generate()?.digest()), one, Some(first))
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .delivery_snapshot(
                Some(&owner_digest),
                scope(&ids, "one", "stale-request"),
                Some(first)
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .delivery_snapshot(Some(&owner_digest), one, Some(dispatch_id))
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        reserved
            .resources
            .iter()
            .all(|r| r.presence == ResourcePresence::Unknown && r.identity().is_none())
    );
    let replay = store
        .reserve_execution(&source, one, &draft("ignored"))
        .await?;
    assert_eq!(replay.execution_id, first);
    store
        .report_lifecycle_issue(&source, one, Some(first), LifecyclePhase::Uncertain)
        .await?;
    assert_eq!(
        store
            .reserve_execution(&source, one, &draft("still-ignored"))
            .await?
            .execution_id,
        first
    );
    assert!(matches!(
        store.retire_execution(&source, one, first).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .reserve_execution(
                &source,
                scope(&ids, "collision", "create-collision"),
                &draft("first")
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let mut collision = draft("unique-execution");
    if let chapchoo_persistence::ExecutionResources::Kubernetes { pod_name, .. } =
        &mut collision.native
    {
        *pod_name = "pod-first".into();
    }
    assert!(matches!(
        store
            .reserve_execution(
                &source,
                scope(&ids, "collision", "create-collision"),
                &collision
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .managed_resources(&source, scope(&ids, "collision", "create-collision"))
            .await?
            .is_empty()
    );

    let wrong_digest = BearerSecret::generate()?.digest();
    let unused = store
        .prepare_provider_connection(source.provider_id, source.credential, source.control_id)
        .await?;
    let unrelated_control = store
        .register_control(
            "http://127.0.0.1:5",
            chapchoo_persistence::DirectoryPolicy::default(),
        )
        .await?;
    for invalid in [
        ProviderSource {
            connection_id: &unused.owner.connection_id,
            ..source
        },
        ProviderSource {
            control_id: &unrelated_control.id,
            ..source
        },
        ProviderSource {
            credential: &wrong_digest,
            ..source
        },
    ] {
        assert!(matches!(
            store.managed_resources(&invalid, one).await,
            Err(StoreError::Unauthenticated)
        ));
    }
    for statement in [
        "UPDATE control_instances SET draining=TRUE",
        "UPDATE provider_connections SET expires_at=1",
        "UPDATE control_instances SET renewed_at=0,expires_at=1",
        "UPDATE provider_keys SET issued_at=0,expires_at=1",
        "UPDATE provider_keys SET revoked_at=1",
    ] {
        mutate_fixture(connection, statement)?;
        assert!(matches!(
            store.managed_resources(&source, one).await,
            Err(StoreError::Unauthenticated)
        ));
        mutate_fixture(
            connection,
            &format!(
                "UPDATE control_instances SET draining=FALSE,expires_at={}; UPDATE provider_connections SET expires_at={}; UPDATE provider_keys SET revoked_at=NULL,expires_at={}",
                now()? + 259_200_000,
                now()? + 259_200_000,
                now()? + 259_200_000
            ),
        )?;
    }
    assert!(matches!(
        store
            .managed_resources(&source, scope(&ids, "one", "stale-request"))
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.report_execution_running(&source, one, first).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(
        store
            .agent(&owner_digest, ids.agent("one"), now()?)
            .await?
            .computer
            .progress
            .initial_start_dispatch_ms
            .is_none()
    );
    store.mark_start_dispatched(&source, one, first).await?;
    mutate_fixture(
        connection,
        "UPDATE computers SET initial_start_dispatch_at=123 WHERE current_request_id='create-one'",
    )?;
    store.mark_start_dispatched(&source, one, first).await?;
    assert_eq!(
        store
            .agent(&owner_digest, ids.agent("one"), now()?)
            .await?
            .computer
            .progress
            .initial_start_dispatch_ms,
        Some(123)
    );
    present_all(&store, &source, one, first).await?;
    let pod = store
        .managed_resources(&source, one)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pod)
        .ok_or("Missing Pod")?;
    assert!(matches!(
        report(
            &store,
            &source,
            one,
            &pod,
            ResourceEvidence::OwnedPresent { uid: "foreign-uid" }
        )
        .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        report(
            &store,
            &source,
            one,
            &pod,
            ResourceEvidence::DefinitivelyAbsent { expected_uid: None }
        )
        .await,
        Err(StoreError::Conflict)
    ));
    store.report_execution_running(&source, one, first).await?;
    assert_eq!(
        store
            .agent(&owner_digest, ids.agent("one"), now()?)
            .await?
            .computer
            .power,
        ObservedPower::Running
    );
    let bootstrap = store
        .issue_connector_session(
            &provider_id,
            &provider_digest,
            first,
            &chapchoo_persistence::ConnectorSessionDraft {
                request_id: "session-first-request".into(),
                initial_key: BearerSecret::generate()?,
            },
            now()?,
        )
        .await?;
    store
        .request_lifecycle(
            &owner_digest,
            ids.agent("one"),
            &LifecycleRequest {
                request_id: "restart-one".into(),
                expected_request_id: "create-one".into(),
                action: LifecycleAction::Restart,
            },
            now()?,
        )
        .await?;
    let restarting = scope(&ids, "one", "restart-one");
    store
        .report_lifecycle_issue(&source, restarting, Some(first), LifecyclePhase::Uncertain)
        .await?;
    assert!(matches!(
        store
            .reserve_execution(&source, restarting, &draft("premature"))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .report_execution_running(&source, restarting, first)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .mark_start_dispatched(&source, restarting, first)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    report(&store, &source, restarting, &pod, ResourceEvidence::Unknown).await?;
    assert!(matches!(
        store.retire_execution(&source, restarting, first).await,
        Err(StoreError::InvalidTransition)
    ));
    report(
        &store,
        &source,
        restarting,
        &pod,
        ResourceEvidence::DefinitivelyAbsent {
            expected_uid: pod.kubernetes_uid(),
        },
    )
    .await?;
    store.retire_execution(&source, restarting, first).await?;
    store.retire_execution(&source, restarting, first).await?;
    assert!(matches!(
        store
            .exchange_connector_key(
                &bootstrap.id,
                &bootstrap.initial_key.digest(),
                &chapchoo_persistence::ComponentKeyDraft {
                    request_id: "late-exchange".into(),
                    token: BearerSecret::generate()?
                },
                chapchoo_persistence::ComponentKeyPolicy::default(),
                now()?
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let replacement = store
        .reserve_execution(&source, restarting, &draft("second"))
        .await?;
    let second = replacement.execution_id.as_str();
    assert_ne!(second, first);
    assert_eq!(
        replacement
            .resources
            .iter()
            .filter(|r| r.kind() == ManagedResourceKind::Pvc)
            .count(),
        1
    );
    assert!(
        replacement
            .resources
            .iter()
            .any(|r| r.kind() == ManagedResourceKind::Pvc
                && r.native().name() == "pvc-first"
                && r.kubernetes_uid() == Some("uid-pvc-first"))
    );
    assert!(matches!(
        store.retire_execution(&source, restarting, first).await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .report_execution_running(&source, restarting, first)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    present_all(&store, &source, restarting, second).await?;
    store
        .report_execution_running(&source, restarting, second)
        .await?;

    // A reconnect invalidates old report authority and requires fresh PVC evidence.
    let pod2 = store
        .managed_resources(&source, restarting)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pod && r.execution_id.as_deref() == Some(second))
        .ok_or("Missing replacement Pod")?;
    report(
        &store,
        &source,
        restarting,
        &pod2,
        ResourceEvidence::DefinitivelyAbsent {
            expected_uid: pod2.kubernetes_uid(),
        },
    )
    .await?;
    store.retire_execution(&source, restarting, second).await?;
    let current_owner = replace_provider(&store, &source).await?;
    let current_source = ProviderSource {
        connection_id: &current_owner.connection_id,
        ..source
    };
    assert!(matches!(
        store
            .reserve_execution(&source, restarting, &draft("stale-restore"))
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .reserve_execution(&current_source, restarting, &draft("unverified-restore"))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let pvc = store
        .managed_resources(&current_source, restarting)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pvc)
        .ok_or("Missing saved PVC")?;
    report(
        &store,
        &current_source,
        restarting,
        &pvc,
        ResourceEvidence::OwnedPresent {
            uid: pvc.kubernetes_uid().ok_or("Missing saved UID")?,
        },
    )
    .await?;
    mutate_fixture(connection, "UPDATE computer_providers SET can_start=FALSE")?;
    assert!(matches!(
        store
            .reserve_execution(&current_source, restarting, &draft("unsupported-restore"))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    mutate_fixture(connection, "UPDATE computer_providers SET can_start=TRUE")?;
    let mut wrong_namespace = draft("wrong-namespace");
    if let chapchoo_persistence::ExecutionResources::Kubernetes { namespace, .. } =
        &mut wrong_namespace.native
    {
        *namespace = "other".into();
    }
    assert!(matches!(
        store
            .reserve_execution(&current_source, restarting, &wrong_namespace)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let restored = store
        .reserve_execution(&current_source, restarting, &draft("restored"))
        .await?;
    let restored_id = restored.execution_id.as_str();
    assert_ne!(restored_id, second);
    present_all(&store, &current_source, restarting, restored_id).await?;
    store
        .report_execution_running(&current_source, restarting, restored_id)
        .await?;

    // Competing reservation drafts recover one exact candidate on either DB.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut tasks = Vec::new();
    for id in ["race-a", "race-b"] {
        let store = store.clone();
        let digest = provider_digest.clone();
        let barrier = barrier.clone();
        let ids = ids.clone();
        let race_owner = current_owner.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .reserve_execution(
                    &ProviderSource {
                        provider_id: &ids.provider,
                        control_id: &race_owner.control_id,
                        connection_id: &race_owner.connection_id,
                        credential: &digest,
                    },
                    scope(&ids, "two", "create-two"),
                    &draft(id),
                )
                .await
        }));
    }
    let (a, b) = tasks::join_pair(tasks.remove(0), tasks.remove(0), None).await?;
    let (a, b) = support::combine(a.map_err(Into::into), b.map_err(Into::into))?;
    assert_eq!(a.execution_id, b.execution_id);

    // Once allocated storage is missing, neither name reuse nor empty replacement is allowed.
    let two = scope(&ids, "two", "create-two");
    present_all(&store, &current_source, two, &a.execution_id).await?;
    for resource in store.managed_resources(&current_source, two).await? {
        if kind(&resource)? == ManagedResourceKind::BootstrapSecret {
            continue;
        }
        report(
            &store,
            &current_source,
            two,
            &resource,
            ResourceEvidence::DefinitivelyAbsent {
                expected_uid: resource.kubernetes_uid(),
            },
        )
        .await?;
    }
    store
        .retire_execution(&current_source, two, &a.execution_id)
        .await?;
    assert!(matches!(
        store
            .reserve_execution(&current_source, two, &draft("empty-replacement"))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let lost = store
        .managed_resources(&current_source, two)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pvc)
        .ok_or("Missing original PVC")?;
    report(
        &store,
        &current_source,
        two,
        &lost,
        ResourceEvidence::Unknown,
    )
    .await?;
    assert!(matches!(
        report(
            &store,
            &current_source,
            two,
            &lost,
            ResourceEvidence::OwnedPresent {
                uid: lost.kubernetes_uid().ok_or("Missing old UID")?
            }
        )
        .await,
        Err(StoreError::Conflict)
    ));

    store
        .request_lifecycle(
            &owner_digest,
            ids.agent("one"),
            &LifecycleRequest {
                request_id: "delete-one".into(),
                expected_request_id: "restart-one".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .force_remove_agent(&owner_digest, ids.agent("one"), "delete-one", now()?)
        .await?;
    let deleting = scope(&ids, "one", "delete-one");
    store
        .report_lifecycle_issue(
            &current_source,
            deleting,
            Some(restored_id),
            chapchoo_domain::LifecyclePhase::Uncertain,
        )
        .await?;
    assert!(matches!(
        store.managed_resources(&current_source, restarting).await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .confirm_computer_cleanup(&current_source, deleting)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    for resource in store.managed_resources(&current_source, deleting).await? {
        report(
            &store,
            &current_source,
            deleting,
            &resource,
            ResourceEvidence::DefinitivelyAbsent {
                expected_uid: resource.kubernetes_uid(),
            },
        )
        .await?;
    }
    store
        .retire_execution(&current_source, deleting, restored_id)
        .await?;
    store
        .report_lifecycle_issue(
            &current_source,
            deleting,
            None,
            chapchoo_domain::LifecyclePhase::Blocked,
        )
        .await?;
    let cleanup_state = store.delivery_snapshot(None, deleting, None).await?;
    assert_eq!(
        cleanup_state.computer.current_phase, "UNCERTAIN",
        "Retirement and issue reports cannot reopen an uncertain deletion"
    );
    store
        .confirm_computer_cleanup(&current_source, deleting)
        .await?;
    let completion = store
        .managed_computers(&current_source, None, 100)
        .await?
        .into_iter()
        .find(|r| r.id == ids.computer("one"))
        .ok_or("Missing retained Computer")?
        .cleanup_completed_at;
    store
        .confirm_computer_cleanup(&current_source, deleting)
        .await?;
    assert_eq!(
        completion,
        store
            .managed_computers(&current_source, None, 100)
            .await?
            .into_iter()
            .find(|r| r.id == ids.computer("one"))
            .ok_or("Missing retained Computer")?
            .cleanup_completed_at
    );
    assert_eq!(
        store
            .collect_completed_computers(&current_source, 100)
            .await?,
        0
    );
    let completed_resources = store.managed_resources(&current_source, deleting).await?;
    assert!(matches!(
        report(
            &store,
            &current_source,
            deleting,
            &completed_resources[0],
            ResourceEvidence::Unknown
        )
        .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .managed_resources(&current_source, deleting)
            .await?
            .iter()
            .all(|r| r.presence == ResourcePresence::Absent)
    );
    assert!(matches!(
        report(
            &store,
            &current_source,
            deleting,
            &completed_resources[0],
            ResourceEvidence::OwnedPresent { uid: "resurrected" }
        )
        .await,
        Err(StoreError::Conflict)
    ));
    store
        .request_lifecycle(
            &owner_digest,
            ids.agent("pending"),
            &LifecycleRequest {
                request_id: "delete-pending".into(),
                expected_request_id: "create-pending".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .force_remove_agent(
            &owner_digest,
            ids.agent("pending"),
            "delete-pending",
            now()?,
        )
        .await?;
    assert!(
        store
            .managed_resources(&current_source, scope(&ids, "pending", "delete-pending"))
            .await?
            .iter()
            .all(|r| r.identity().is_none()
                && r.absence_confirmed_at.is_some()
                && r.source_connection_id.is_none())
    );
    let uncertain_candidate = scope(&ids, "collision", "create-collision");
    let possibly_forwarded = store
        .reserve_execution(
            &current_source,
            uncertain_candidate,
            &draft("possibly-forwarded"),
        )
        .await?;
    assert!(
        store
            .claim_execution_dispatch(
                &current_source,
                uncertain_candidate,
                &possibly_forwarded.execution_id
            )
            .await?
    );
    store
        .request_lifecycle(
            &owner_digest,
            ids.agent("collision"),
            &LifecycleRequest {
                request_id: "delete-uncertain".into(),
                expected_request_id: "create-collision".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    assert!(
        store
            .managed_resources(
                &current_source,
                scope(&ids, "collision", "delete-uncertain")
            )
            .await?
            .iter()
            .all(|r| r.absence_confirmed_at.is_none()),
        "Possible forwarding cannot become non-allocation proof"
    );
    mutate_fixture(
        connection,
        &format!(
            "UPDATE computers SET created_at=0,agent_removed_at=0 WHERE current_request_id='delete-pending'; UPDATE computers SET cleanup_completed_at={} WHERE current_request_id='delete-one'",
            now()? - 86_400_000 + 30_000
        ),
    )?;
    assert_eq!(
        store
            .collect_completed_computers(&current_source, 100)
            .await?,
        0
    );
    mutate_fixture(
        connection,
        &format!(
            "UPDATE computers SET cleanup_completed_at={} WHERE current_request_id='delete-one'",
            now()? - 86_400_000 - 30_000
        ),
    )?;
    assert_eq!(
        store
            .collect_completed_computers(&current_source, 100)
            .await?,
        1
    );
    assert!(matches!(
        store
            .force_remove_agent(&owner_digest, ids.agent("one"), "delete-one", now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .managed_computers(&current_source, None, 100)
            .await?
            .iter()
            .any(|r| r.id == ids.computer("pending") && r.cleanup_completed_at.is_none())
    );
    assert!(matches!(
        store
            .create_cloud_agent(&owner_digest, &creation(&ids, "one"), now()?)
            .await?,
        CreationOutcome::Removed { .. }
    ));

    // Normal cleanup removes the still-visible Agent only after confirmation.
    store
        .request_lifecycle(
            &owner_digest,
            ids.agent("empty"),
            &LifecycleRequest {
                request_id: "delete-empty".into(),
                expected_request_id: "create-empty".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .confirm_computer_cleanup(&current_source, scope(&ids, "empty", "delete-empty"))
        .await?;
    assert!(matches!(
        store.agent(&owner_digest, ids.agent("empty"), now()?).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .force_remove_agent(&owner_digest, ids.agent("empty"), "delete-empty", now()?)
            .await?,
        chapchoo_persistence::AgentRemoval::AlreadyRemoved
    );
    assert!(matches!(
        store
            .force_remove_agent(
                &owner_digest,
                ids.agent("empty"),
                "unaccepted-delete",
                now()?
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let stop_owner =
        exercise_stop_and_initial(&store, &mut ids, &current_source, &owner_digest).await?;
    let current_source = ProviderSource {
        connection_id: &stop_owner.connection_id,
        ..current_source
    };
    exercise_review_regressions(&store, &mut ids, connection, &current_source, &owner_digest)
        .await?;
    exercise_provider_directory(&store, &ids, connection, &provider_digest).await?;
    Ok(())
}

async fn exercise_provider_directory(
    store: &Store,
    ids: &FixtureIds,
    connection: &mut impl SimpleConnection,
    credential: &chapchoo_crypto::SecretDigest,
) -> Result<(), TestError> {
    use chapchoo_persistence::DirectoryPolicy;
    let policy = DirectoryPolicy::default();
    let a = store.register_control("http://127.0.0.1:6", policy).await?;
    let b = store.register_control("http://127.0.0.1:6", policy).await?;
    assert_ne!(
        a.id, b.id,
        "Repository issues a new process identity even for the same endpoint"
    );
    for control in [&a, &b] {
        assert!(!control.draining);
        store.renew_control(&control.id, policy).await?;
    }
    exercise_connector_directory(store, ids, connection, credential, [&a.id, &b.id]).await?;
    assert!(matches!(
        store.withdraw_drained_control_connections(&a.id).await,
        Err(StoreError::InvalidTransition)
    ));
    let one = store
        .prepare_provider_connection(&ids.provider, credential, &a.id)
        .await?;
    let two = store
        .prepare_provider_connection(&ids.provider, credential, &b.id)
        .await?;
    assert_eq!(one.expected, two.expected);
    let expected = one.expected.clone().ok_or("Original route")?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut pending = Vec::new();
    for candidate in [one, two] {
        let store = store.clone();
        let provider = ids.provider.clone();
        let key = credential.clone();
        let barrier = barrier.clone();
        pending.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .admit_provider_connection(
                    &provider,
                    &key,
                    candidate.expected.as_ref(),
                    &candidate.owner,
                    policy,
                )
                .await
        }));
    }
    let (one, two) = tasks::join_pair(pending.remove(0), pending.remove(0), None).await?;
    let winner = match (one, two) {
        (Ok(value), Err(StoreError::Conflict)) | (Err(StoreError::Conflict), Ok(value)) => value,
        _ => return Err("Exactly one concurrent admission must win".into()),
    };
    assert!(
        !store
            .close_provider_connection(&ids.provider, credential, &expected)
            .await?
    );
    assert!(matches!(
        store
            .renew_provider_connection(&ids.provider, credential, &expected, policy)
            .await,
        Err(StoreError::Conflict)
    ));
    let winner_owner = winner.owner();
    assert_eq!(
        store
            .resolve_provider_route(&ids.provider)
            .await?
            .ok_or("Winning route")?
            .component
            .owner(),
        winner_owner
    );
    store
        .renew_provider_connection(&ids.provider, credential, &winner_owner, policy)
        .await?;
    let other_control = if winner_owner.control_id == a.id {
        &b.id
    } else {
        &a.id
    };
    let next = store
        .prepare_provider_connection(&ids.provider, credential, other_control)
        .await?;
    assert_eq!(next.expected.as_ref(), Some(&winner_owner));
    store
        .admit_provider_connection(
            &ids.provider,
            credential,
            next.expected.as_ref(),
            &next.owner,
            policy,
        )
        .await?;
    store.drain_control(&winner_owner.control_id).await?;
    assert_eq!(
        store
            .withdraw_drained_control_connections(&winner_owner.control_id)
            .await?,
        0
    );
    assert_eq!(
        store
            .resolve_provider_route(&ids.provider)
            .await?
            .ok_or("Other Control route was cleared")?
            .component
            .owner(),
        next.owner
    );
    assert!(
        !store
            .close_provider_connection(&ids.provider, credential, &winner_owner)
            .await?
    );
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET expires_at=1 WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='provider')",
    )?;
    assert!(store.resolve_provider_route(&ids.provider).await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection(&ids.provider, credential, &next.owner, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let fresh = store
        .prepare_provider_connection(&ids.provider, credential, &next.owner.control_id)
        .await?;
    assert_eq!(
        fresh.expected.as_ref(),
        Some(&next.owner),
        "Expired tuple remains an exact comparison fence"
    );
    assert_ne!(fresh.owner.connection_id, next.owner.connection_id);
    store
        .admit_provider_connection(
            &ids.provider,
            credential,
            fresh.expected.as_ref(),
            &fresh.owner,
            policy,
        )
        .await?;
    store.drain_control(&fresh.owner.control_id).await?;
    assert!(matches!(
        store.renew_control(&fresh.owner.control_id, policy).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(store.resolve_provider_route(&ids.provider).await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection(&ids.provider, credential, &fresh.owner, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .withdraw_drained_control_connections(&fresh.owner.control_id)
            .await?,
        1
    );
    let expiring = store.register_control("http://127.0.0.1:3", policy).await?;
    let invalid = store
        .prepare_provider_connection(&ids.provider, credential, &expiring.id)
        .await?;
    assert!(invalid.expected.is_none());
    mutate_fixture(
        connection,
        "UPDATE control_instances SET renewed_at=0,expires_at=1 WHERE endpoint='http://127.0.0.1:3'",
    )?;
    assert!(matches!(
        store.renew_control(&expiring.id, policy).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .admit_provider_connection(&ids.provider, credential, None, &invalid.owner, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let active = store.register_control("http://127.0.0.1:4", policy).await?;
    let new = store
        .prepare_provider_connection(&ids.provider, credential, &active.id)
        .await?;
    assert!(new.expected.is_none());
    store
        .admit_provider_connection(&ids.provider, credential, None, &new.owner, policy)
        .await?;
    let wrong = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .renew_provider_connection(&ids.provider, &wrong, &new.owner, policy)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=1 WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='provider')",
    )?;
    assert!(store.resolve_provider_route(&ids.provider).await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection(&ids.provider, credential, &new.owner, policy)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    store.drain_control(&active.id).await?;
    assert_eq!(
        store
            .withdraw_drained_control_connections(&active.id)
            .await?,
        1,
        "Owned shutdown does not require still-valid component credentials"
    );
    Ok(())
}

async fn exercise_connector_directory(
    store: &Store,
    ids: &FixtureIds,
    connection: &mut impl SimpleConnection,
    provider: &chapchoo_crypto::SecretDigest,
    controls: [&str; 2],
) -> Result<(), TestError> {
    use chapchoo_persistence::{
        ComponentKeyDraft, ComponentKeyPolicy, ConnectorSessionDraft, DirectoryPolicy,
    };
    let policy = DirectoryPolicy::default();
    let route = store
        .resolve_provider_route(&ids.provider)
        .await?
        .ok_or("Provider route")?;
    let source = ProviderSource {
        provider_id: &ids.provider,
        control_id: &route.component.control_id,
        connection_id: &route.component.connection_id,
        credential: provider,
    };
    let execution = store
        .provider_delivery_snapshot(
            &source,
            scope(ids, "unallocated", "create-unallocated"),
            None,
        )
        .await?
        .computer
        .current_execution_id
        .ok_or("Unallocated execution")?;
    let draft = ConnectorSessionDraft {
        request_id: "directory-session-create".into(),
        initial_key: BearerSecret::generate()?,
    };
    let unused = store
        .prepare_provider_connection(&ids.provider, provider, controls[0])
        .await?;
    let stale = ProviderSource {
        connection_id: &unused.owner.connection_id,
        ..source
    };
    assert!(matches!(
        store
            .issue_connector_session_from_source(&stale, &execution, &draft, now()?)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let session = store
        .issue_connector_session_from_source(&source, &execution, &draft, now()?)
        .await?;
    assert!(matches!(
        store
            .revoke_connector_session_from_source(&stale, &session.id, "stale-revoke", now()?)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .prepare_connector_connection(&session.id, &session.initial_key.digest(), controls[0])
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let key = store
        .exchange_connector_key(
            &session.id,
            &session.initial_key.digest(),
            &ComponentKeyDraft {
                request_id: "directory-exchange".into(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now()?,
        )
        .await?;
    let candidates = [
        store
            .prepare_connector_connection(&session.id, &key.token.digest(), controls[0])
            .await?,
        store
            .prepare_connector_connection(&session.id, &key.token.digest(), controls[1])
            .await?,
    ];
    assert!(candidates.iter().all(|value| value.expected.is_none()));
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut pending = Vec::new();
    for candidate in candidates {
        let store = store.clone();
        let digest = key.token.digest();
        let session_id = session.id.clone();
        let barrier = barrier.clone();
        pending.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .admit_connector_connection(
                    &session_id,
                    &digest,
                    candidate.expected.as_ref(),
                    &candidate.owner,
                    policy,
                )
                .await
        }));
    }
    let (one, two) = tasks::join_pair(pending.remove(0), pending.remove(0), None).await?;
    let winner = match (one, two) {
        (Ok(value), Err(StoreError::Conflict)) | (Err(StoreError::Conflict), Ok(value)) => value,
        _ => return Err("Exactly one Connector admission must win".into()),
    };
    let first = winner.owner();
    let identity = store
        .verify_connector_connection(&chapchoo_persistence::ConnectorSource {
            session_id: &session.id,
            control_id: &first.control_id,
            connection_id: &first.connection_id,
            credential: &key.token.digest(),
        })
        .await?;
    assert_eq!(identity.computer_id, ids.computer("unallocated"));
    assert_eq!(
        store
            .resolve_connector_route(ids.computer("unallocated"))
            .await?
            .ok_or("Connector route")?
            .component
            .expires_at,
        winner.expires_at
    );
    store
        .renew_connector_connection(&session.id, &key.token.digest(), &first, policy)
        .await?;
    let replacement = store
        .prepare_connector_connection(&session.id, &key.token.digest(), &first.control_id)
        .await?;
    assert_eq!(replacement.expected.as_ref(), Some(&first));
    store
        .admit_connector_connection(
            &session.id,
            &key.token.digest(),
            replacement.expected.as_ref(),
            &replacement.owner,
            policy,
        )
        .await?;
    assert!(
        !store
            .close_connector_connection(&session.id, &key.token.digest(), &first)
            .await?
    );
    assert!(matches!(
        store
            .verify_connector_connection(&chapchoo_persistence::ConnectorSource {
                session_id: &session.id,
                control_id: &first.control_id,
                connection_id: &first.connection_id,
                credential: &key.token.digest()
            })
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .renew_connector_connection(&session.id, &key.token.digest(), &first, policy)
            .await,
        Err(StoreError::Conflict)
    ));
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=1 WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='provider')",
    )?;
    store
        .renew_connector_connection(&session.id, &key.token.digest(), &replacement.owner, policy)
        .await?;
    assert_eq!(
        store
            .resolve_connector_route(ids.computer("unallocated"))
            .await?
            .ok_or("Independent Connector route")?
            .component
            .owner(),
        replacement.owner
    );
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=NULL WHERE provider_id=(SELECT id FROM computer_providers WHERE registration_key='provider')",
    )?;
    let refreshed = store
        .refresh_connector_key(
            &session.id,
            &key.token.digest(),
            &ComponentKeyDraft {
                request_id: "directory-refresh".into(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now()?,
        )
        .await?;
    store
        .renew_connector_connection(&session.id, &key.token.digest(), &replacement.owner, policy)
        .await?;
    mutate_fixture(
        connection,
        "UPDATE connector_sessions SET previous_valid_until=1 WHERE initial_request_id='directory-session-create'",
    )?;
    assert!(
        store
            .resolve_connector_route(ids.computer("unallocated"))
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &key.token.digest(),
                &replacement.owner,
                policy
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let current = store
        .prepare_connector_connection(
            &session.id,
            &refreshed.token.digest(),
            &replacement.owner.control_id,
        )
        .await?;
    assert_eq!(current.expected.as_ref(), Some(&replacement.owner));
    store
        .admit_connector_connection(
            &session.id,
            &refreshed.token.digest(),
            current.expected.as_ref(),
            &current.owner,
            policy,
        )
        .await?;
    let exchanged = store
        .exchange_connector_key(
            &session.id,
            &session.initial_key.digest(),
            &ComponentKeyDraft {
                request_id: "directory-reexchange".into(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now()?,
        )
        .await?;
    assert!(
        store
            .resolve_connector_route(ids.computer("unallocated"))
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &refreshed.token.digest(),
                &current.owner,
                policy
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let latest = store
        .prepare_connector_connection(
            &session.id,
            &exchanged.token.digest(),
            &current.owner.control_id,
        )
        .await?;
    assert!(latest.expected.is_none());
    store
        .admit_connector_connection(
            &session.id,
            &exchanged.token.digest(),
            latest.expected.as_ref(),
            &latest.owner,
            policy,
        )
        .await?;
    store
        .revoke_connector_session(
            &ids.provider,
            provider,
            &session.id,
            "directory-revoke",
            now()?,
        )
        .await?;
    assert!(
        store
            .resolve_connector_route(ids.computer("unallocated"))
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &exchanged.token.digest(),
                &latest.owner,
                policy
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    Ok(())
}

async fn exercise_stop_and_initial(
    store: &Store,
    ids: &mut FixtureIds,
    source: &ProviderSource<'_>,
    owner: &chapchoo_crypto::SecretDigest,
) -> Result<chapchoo_persistence::ConnectionOwner, TestError> {
    for id in ["stopping", "unallocated"] {
        create_fixture(store, ids, owner, id).await?;
    }
    let stopping = scope(ids, "stopping", "create-stopping");
    let original_execution = store
        .reserve_execution(source, stopping, &draft("stop-original"))
        .await?;
    let stop_original = original_execution.execution_id.as_str();
    present_all(store, source, stopping, stop_original).await?;
    store
        .report_execution_running(source, stopping, stop_original)
        .await?;
    store
        .request_lifecycle(
            owner,
            ids.agent("stopping"),
            &LifecycleRequest {
                request_id: "stop".into(),
                expected_request_id: "create-stopping".into(),
                action: LifecycleAction::Stop,
            },
            now()?,
        )
        .await?;
    let stop = scope(ids, "stopping", "stop");
    assert!(matches!(
        store.retire_execution(source, stop, stop_original).await,
        Err(StoreError::InvalidTransition)
    ));
    let pod = store
        .managed_resources(source, stop)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pod)
        .ok_or("Missing stop Pod")?;
    report(
        store,
        source,
        stop,
        &pod,
        ResourceEvidence::DefinitivelyAbsent {
            expected_uid: pod.kubernetes_uid(),
        },
    )
    .await?;
    store.retire_execution(source, stop, stop_original).await?;
    let stopped = store.agent(owner, ids.agent("stopping"), now()?).await?;
    assert_eq!(stopped.computer.power, ObservedPower::Stopped);
    assert_eq!(stopped.computer.progress.phase, LifecyclePhase::Completed);
    for phase in [LifecyclePhase::Blocked, LifecyclePhase::Uncertain] {
        assert!(matches!(
            store
                .report_lifecycle_issue(source, stop, None, phase)
                .await,
            Err(StoreError::Conflict)
        ));
    }
    assert_eq!(
        store
            .agent(owner, ids.agent("stopping"), now()?)
            .await?
            .computer
            .progress
            .phase,
        LifecyclePhase::Completed
    );
    let reconnected_owner = replace_provider(store, source).await?;
    let reconnected = ProviderSource {
        connection_id: &reconnected_owner.connection_id,
        ..*source
    };
    assert_eq!(
        store
            .agent(owner, ids.agent("stopping"), now()?)
            .await?
            .computer
            .power,
        ObservedPower::Unknown
    );
    assert!(matches!(
        store
            .retire_execution(&reconnected, stop, stop_original)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                owner,
                ids.agent("stopping"),
                &LifecycleRequest {
                    request_id: "premature-start".into(),
                    expected_request_id: "stop".into(),
                    action: LifecycleAction::Start
                },
                now()?
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    report(
        store,
        &reconnected,
        stop,
        &pod,
        ResourceEvidence::DefinitivelyAbsent {
            expected_uid: pod.kubernetes_uid(),
        },
    )
    .await?;
    store
        .retire_execution(&reconnected, stop, stop_original)
        .await?;
    let refreshed = store.agent(owner, ids.agent("stopping"), now()?).await?;
    assert_eq!(refreshed.computer.power, ObservedPower::Stopped);
    assert_eq!(refreshed.computer.progress.phase, LifecyclePhase::Completed);
    let pvc = store
        .managed_resources(&reconnected, stop)
        .await?
        .into_iter()
        .find(|r| r.kind() == ManagedResourceKind::Pvc)
        .ok_or("Missing stopped PVC")?;
    report(
        store,
        &reconnected,
        stop,
        &pvc,
        ResourceEvidence::OwnedPresent {
            uid: pvc.kubernetes_uid().ok_or("Missing stopped PVC UID")?,
        },
    )
    .await?;
    store
        .request_lifecycle(
            owner,
            ids.agent("stopping"),
            &LifecycleRequest {
                request_id: "start".into(),
                expected_request_id: "stop".into(),
                action: LifecycleAction::Start,
            },
            now()?,
        )
        .await?;
    store
        .reserve_execution(
            &reconnected,
            scope(ids, "stopping", "start"),
            &draft("stop-successor"),
        )
        .await?;
    // Quiescent initial allocation without any observed UID may continue, while
    // allocated/missing storage above remains irreplaceable.
    let initial = scope(ids, "unallocated", "create-unallocated");
    let never_created = store
        .reserve_execution(&reconnected, initial, &draft("never-created"))
        .await?;
    for resource in store.managed_resources(&reconnected, initial).await? {
        report(
            store,
            &reconnected,
            initial,
            &resource,
            ResourceEvidence::DefinitivelyAbsent { expected_uid: None },
        )
        .await?;
    }
    store
        .retire_execution(&reconnected, initial, &never_created.execution_id)
        .await?;
    let resumed = store
        .reserve_execution(&reconnected, initial, &draft("initial-resumed"))
        .await?;
    assert!(
        resumed
            .resources
            .iter()
            .any(|r| r.kind() == ManagedResourceKind::Pvc
                && r.native().name() == "pvc-never-created"
                && r.identity().is_none())
    );
    present_all(store, &reconnected, initial, &resumed.execution_id).await?;
    store
        .report_execution_running(&reconnected, initial, &resumed.execution_id)
        .await?;
    Ok(reconnected_owner)
}
async fn exercise_review_regressions(
    store: &Store,
    ids: &mut FixtureIds,
    connection: &mut impl SimpleConnection,
    source: &ProviderSource<'_>,
    owner: &chapchoo_crypto::SecretDigest,
) -> Result<(), TestError> {
    for issue in [false, true] {
        let id = if issue {
            "review-uncertain"
        } else {
            "review-direct"
        };
        let create_id = format!("create-{id}");
        let execution_name = format!("execution-{id}");
        mutate_fixture(
            connection,
            "UPDATE computer_providers SET can_create=TRUE,can_start=FALSE",
        )?;
        create_fixture(store, ids, owner, id).await?;
        let agent_id = ids.agent(id);
        let request = scope(ids, id, &create_id);
        let execution = store
            .reserve_execution(source, request, &draft(&execution_name))
            .await?
            .execution_id;
        // The initial candidate keeps create authority after PVC allocation.
        let pvc = store
            .managed_resources(source, request)
            .await?
            .into_iter()
            .find(|r| r.kind() == ManagedResourceKind::Pvc)
            .ok_or("Missing initial PVC")?;
        report(
            store,
            source,
            request,
            &pvc,
            ResourceEvidence::OwnedPresent {
                uid: &format!("uid-{}", pvc.native().name()),
            },
        )
        .await?;
        store
            .mark_start_dispatched(source, request, &execution)
            .await?;
        present_all(store, source, request, &execution).await?;
        store
            .report_execution_running(source, request, &execution)
            .await?;
        mutate_fixture(
            connection,
            &format!(
                "UPDATE computers SET initial_start_dispatch_at=123 WHERE current_request_id='{create_id}'"
            ),
        )?;
        if issue {
            store
                .report_lifecycle_issue(
                    source,
                    request,
                    Some(&execution),
                    LifecyclePhase::Uncertain,
                )
                .await?;
        }
        let pod = store
            .managed_resources(source, request)
            .await?
            .into_iter()
            .find(|r| r.kind() == ManagedResourceKind::Pod)
            .ok_or("Missing running Pod")?;
        report(
            store,
            source,
            request,
            &pod,
            ResourceEvidence::DefinitivelyAbsent {
                expected_uid: pod.kubernetes_uid(),
            },
        )
        .await?;
        store.retire_execution(source, request, &execution).await?;
        assert!(
            store
                .agent(owner, agent_id, now()?)
                .await?
                .computer
                .progress
                .initial_start_dispatch_ms
                .is_none()
        );
        let successor = format!("successor-{id}");
        // Actual prior running, not PVC UID or mutable issue phase, distinguishes recovery.
        assert!(matches!(
            store
                .reserve_execution(source, request, &draft(&successor))
                .await,
            Err(StoreError::InvalidTransition)
        ));
        mutate_fixture(
            connection,
            "UPDATE computer_providers SET can_create=FALSE,can_start=TRUE",
        )?;
        let successor = store
            .reserve_execution(source, request, &draft(&successor))
            .await?
            .execution_id;
        store
            .mark_start_dispatched(source, request, &successor)
            .await?;
        let progress = store
            .agent(owner, agent_id, now()?)
            .await?
            .computer
            .progress;
        assert!(
            progress
                .initial_start_dispatch_ms
                .is_some_and(|time| time > 123)
        );
        assert!(!progress.start_delayed(now()?));
        store
            .mark_start_dispatched(source, request, &successor)
            .await?;
        assert_eq!(
            store
                .agent(owner, agent_id, now()?)
                .await?
                .computer
                .progress
                .initial_start_dispatch_ms,
            progress.initial_start_dispatch_ms
        );
    }
    mutate_fixture(
        connection,
        "UPDATE computer_providers SET can_create=TRUE,can_start=TRUE",
    )?;
    Ok(())
}

fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([4; 32]),
    )?)?))
}
#[test]
fn sqlite_current_source_execution_and_cleanup() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("execution.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    let owner = BearerSecret::generate()?;
    let provider = BearerSecret::generate()?;

    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        exercise(store, &mut connection, owner, provider).await
    })
}
#[test]
fn postgres_current_source_execution_and_cleanup() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("execution_test_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        let owner = BearerSecret::generate()?;
        let provider = BearerSecret::generate()?;

        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            exercise(store, &mut connection, owner, provider).await
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
