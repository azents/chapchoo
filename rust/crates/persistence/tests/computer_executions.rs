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
fn seed(
    cipher: &CredentialCipher,
    owner: &BearerSecret,
    provider: &BearerSecret,
    postgres: bool,
) -> Result<String, TestError> {
    let binary = |b: &[u8]| {
        let hex = b.iter().map(|b| format!("{b:02x}")).collect::<String>();
        if postgres {
            format!("decode('{hex}','hex')")
        } else {
            format!("X'{hex}'")
        }
    };
    let expiry = now()? + 259_200_000;
    let session = cipher.protect_latest("session", owner)?;
    let key = cipher.protect_latest("computer-provider:provider", provider)?;
    Ok(format!(
        "BEGIN; INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('user','fixture@example.invalid','fixture@example.invalid','synthetic',0,'en','UTC','en-US'); INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal','PERSONAL','Personal','personal','user',0); INSERT INTO memberships VALUES('personal','user','MEMBER'); INSERT INTO personal_contexts VALUES('user','personal'); INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('session','user',{},{},{},0,0,{expiry}); INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at) VALUES('provider','SYSTEM','owner','Fixture','KUBERNETES',TRUE,TRUE,TRUE,TRUE,0); INSERT INTO provider_keys(provider_id,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('provider',{},0,{expiry},'key-request',{},{}); INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://127.0.0.1:1',0,{expiry}); INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES('provider','control','connection',{}, {expiry}); COMMIT;",
        binary(owner.digest().as_bytes()),
        binary(session.nonce()),
        binary(session.ciphertext()),
        binary(provider.digest().as_bytes()),
        binary(key.nonce()),
        binary(key.ciphertext()),
        binary(provider.digest().as_bytes())
    ))
}
fn creation(id: &str) -> CreateCloudAgent {
    CreateCloudAgent {
        agent_id: format!("agent-{id}"),
        team_id: "personal".into(),
        computer_id: id.into(),
        request_id: format!("create-{id}"),
        provider_id: "provider".into(),
        name: "Fixture".into(),
    }
}
fn draft(id: &str) -> ExecutionDraft {
    ExecutionDraft {
        id: id.into(),
        native: chapchoo_persistence::ExecutionResources::Kubernetes {
            namespace: "fixture".into(),
            pod_name: format!("pod-{id}"),
            bootstrap_secret_name: format!("secret-{id}"),
            saved_pvc_name: format!("pvc-{id}"),
        },
    }
}
fn scope<'a>(computer_id: &'a str, request_id: &'a str) -> ComputerRequest<'a> {
    ComputerRequest {
        computer_id,
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

async fn exercise(
    store: Store,
    connection: &mut impl SimpleConnection,
    owner: BearerSecret,
    provider: BearerSecret,
) -> Result<(), TestError> {
    let owner_digest = owner.digest();
    let provider_digest = provider.digest();
    let source = ProviderSource {
        provider_id: "provider",
        control_id: "control",
        connection_id: "connection",
        credential: &provider_digest,
    };
    for id in ["one", "two", "collision", "pending", "empty", "dispatch"] {
        store
            .create_cloud_agent(&owner_digest, &creation(id), now()?)
            .await?;
    }
    let dispatch = scope("dispatch", "create-dispatch");
    store
        .reserve_execution(&source, dispatch, &draft("dispatch-once"))
        .await?;
    assert!(
        store
            .claim_execution_dispatch(&source, dispatch, "dispatch-once")
            .await?
    );
    let admitted = store
        .delivery_snapshot(Some(&owner_digest), dispatch, Some("dispatch-once"))
        .await?;
    assert!(admitted.computer.prepare_admitted_at.is_some());
    assert!(admitted.computer.initial_start_dispatch_at.is_none());
    store
        .release_undelivered_prepare(&source, dispatch, "dispatch-once")
        .await?;
    assert!(
        store
            .claim_execution_dispatch(&source, dispatch, "dispatch-once")
            .await?
    );
    store
        .mark_start_dispatched(&source, dispatch, "dispatch-once")
        .await?;
    store
        .release_undelivered_prepare(&source, dispatch, "dispatch-once")
        .await?;
    assert!(
        store
            .delivery_snapshot(Some(&owner_digest), dispatch, Some("dispatch-once"))
            .await?
            .computer
            .prepare_admitted_at
            .is_some(),
        "A dequeued call never becomes replayable"
    );
    assert!(
        !store
            .claim_execution_dispatch(&source, dispatch, "dispatch-once")
            .await?
    );
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET connection_id='dispatch-successor' WHERE provider_id='provider'",
    )?;
    let successor = ProviderSource {
        connection_id: "dispatch-successor",
        ..source
    };
    assert!(
        !store
            .claim_execution_dispatch(&successor, dispatch, "dispatch-once")
            .await?
    );
    assert!(matches!(
        store
            .claim_execution_dispatch(&source, dispatch, "dispatch-once")
            .await,
        Err(StoreError::Unauthenticated)
    ));
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET connection_id='connection' WHERE provider_id='provider'",
    )?;
    let dispatch_agent = store.agent(&owner_digest, "agent-dispatch", now()?).await?;
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
        .report_preparation_blocked(&source, dispatch, "dispatch-once")
        .await?;
    mutate_fixture(
        connection,
        "UPDATE computer_providers SET can_create=FALSE WHERE id='provider'",
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
        "UPDATE computer_providers SET can_create=TRUE WHERE id='provider'",
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
    let retry_scope = scope("dispatch", "retry-dispatch");
    let retry_snapshot = store
        .delivery_snapshot(Some(&owner_digest), retry_scope, Some("dispatch-once"))
        .await?;
    assert_eq!(
        retry_snapshot.computer.current_execution_id.as_deref(),
        Some("dispatch-once")
    );
    assert!(retry_snapshot.computer.initial_start_dispatch_at.is_none());
    assert!(
        store
            .claim_execution_dispatch(&source, retry_scope, "dispatch-once")
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
    let one = scope("one", "create-one");
    let reserved = store
        .reserve_execution(&source, one, &draft("first"))
        .await?;
    assert_eq!(reserved.execution_id, "first");
    assert_eq!(reserved.resources.len(), 3);
    let delivery = store
        .delivery_snapshot(Some(&owner_digest), one, Some("first"))
        .await?;
    assert_eq!(delivery.computer.provider_id, "provider");
    assert_eq!(delivery.resources.len(), 3);
    assert!(delivery.execution.is_some_and(|e| e.retired_at.is_none()));
    assert!(matches!(
        store
            .delivery_snapshot(
                Some(&BearerSecret::generate()?.digest()),
                one,
                Some("first")
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .delivery_snapshot(
                Some(&owner_digest),
                scope("one", "stale-request"),
                Some("first")
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .delivery_snapshot(Some(&owner_digest), one, Some("foreign-execution"))
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
    assert_eq!(replay.execution_id, "first");
    store
        .report_lifecycle_issue(&source, one, Some("first"), LifecyclePhase::Uncertain)
        .await?;
    assert_eq!(
        store
            .reserve_execution(&source, one, &draft("still-ignored"))
            .await?
            .execution_id,
        "first"
    );
    assert!(matches!(
        store.retire_execution(&source, one, "first").await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .reserve_execution(
                &source,
                scope("collision", "create-collision"),
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
            .reserve_execution(&source, scope("collision", "create-collision"), &collision)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .managed_resources(&source, scope("collision", "create-collision"))
            .await?
            .is_empty()
    );

    let wrong_digest = BearerSecret::generate()?.digest();
    for invalid in [
        ProviderSource {
            connection_id: "old-connection",
            ..source
        },
        ProviderSource {
            control_id: "old-control",
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
        "UPDATE control_instances SET expires_at=1",
        "UPDATE provider_keys SET expires_at=1",
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
            .managed_resources(&source, scope("one", "stale-request"))
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.report_execution_running(&source, one, "first").await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(
        store
            .agent(&owner_digest, "agent-one", now()?)
            .await?
            .computer
            .progress
            .initial_start_dispatch_ms
            .is_none()
    );
    store.mark_start_dispatched(&source, one, "first").await?;
    mutate_fixture(
        connection,
        "UPDATE computers SET initial_start_dispatch_at=123 WHERE id='one'",
    )?;
    store.mark_start_dispatched(&source, one, "first").await?;
    assert_eq!(
        store
            .agent(&owner_digest, "agent-one", now()?)
            .await?
            .computer
            .progress
            .initial_start_dispatch_ms,
        Some(123)
    );
    present_all(&store, &source, one, "first").await?;
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
    store
        .report_execution_running(&source, one, "first")
        .await?;
    assert_eq!(
        store
            .agent(&owner_digest, "agent-one", now()?)
            .await?
            .computer
            .power,
        ObservedPower::Running
    );
    let bootstrap = store
        .issue_connector_session(
            "provider",
            &provider_digest,
            "first",
            &chapchoo_persistence::ConnectorSessionDraft {
                id: "session-first".into(),
                request_id: "session-first-request".into(),
                initial_key: BearerSecret::generate()?,
            },
            now()?,
        )
        .await?;
    store
        .request_lifecycle(
            &owner_digest,
            "agent-one",
            &LifecycleRequest {
                request_id: "restart-one".into(),
                expected_request_id: "create-one".into(),
                action: LifecycleAction::Restart,
            },
            now()?,
        )
        .await?;
    let restarting = scope("one", "restart-one");
    store
        .report_lifecycle_issue(
            &source,
            restarting,
            Some("first"),
            LifecyclePhase::Uncertain,
        )
        .await?;
    assert!(matches!(
        store
            .reserve_execution(&source, restarting, &draft("premature"))
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .report_execution_running(&source, restarting, "first")
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .mark_start_dispatched(&source, restarting, "first")
            .await,
        Err(StoreError::InvalidTransition)
    ));
    report(&store, &source, restarting, &pod, ResourceEvidence::Unknown).await?;
    assert!(matches!(
        store.retire_execution(&source, restarting, "first").await,
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
    store.retire_execution(&source, restarting, "first").await?;
    store.retire_execution(&source, restarting, "first").await?;
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
    assert_eq!(replacement.execution_id, "second");
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
        store.retire_execution(&source, restarting, "first").await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .report_execution_running(&source, restarting, "first")
            .await,
        Err(StoreError::InvalidTransition)
    ));
    present_all(&store, &source, restarting, "second").await?;
    store
        .report_execution_running(&source, restarting, "second")
        .await?;

    // A reconnect invalidates old report authority and requires fresh PVC evidence.
    let pod2 = store
        .managed_resources(&source, restarting)
        .await?
        .into_iter()
        .find(|r| {
            r.kind() == ManagedResourceKind::Pod && r.execution_id.as_deref() == Some("second")
        })
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
    store
        .retire_execution(&source, restarting, "second")
        .await?;
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET connection_id='new-connection'",
    )?;
    let current_source = ProviderSource {
        connection_id: "new-connection",
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
    assert_eq!(restored.execution_id, "restored");
    present_all(&store, &current_source, restarting, "restored").await?;
    store
        .report_execution_running(&current_source, restarting, "restored")
        .await?;

    // Competing reservation drafts recover one exact candidate on either DB.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut tasks = Vec::new();
    for id in ["race-a", "race-b"] {
        let store = store.clone();
        let digest = provider_digest.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .reserve_execution(
                    &ProviderSource {
                        provider_id: "provider",
                        control_id: "control",
                        connection_id: "new-connection",
                        credential: &digest,
                    },
                    scope("two", "create-two"),
                    &draft(id),
                )
                .await
        }));
    }
    let (a, b) = tasks::join_pair(tasks.remove(0), tasks.remove(0), None).await?;
    let (a, b) = support::combine(a.map_err(Into::into), b.map_err(Into::into))?;
    assert_eq!(a.execution_id, b.execution_id);

    // Once allocated storage is missing, neither name reuse nor empty replacement is allowed.
    let two = scope("two", "create-two");
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
            "agent-one",
            &LifecycleRequest {
                request_id: "delete-one".into(),
                expected_request_id: "restart-one".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .force_remove_agent(&owner_digest, "agent-one", "delete-one", now()?)
        .await?;
    let deleting = scope("one", "delete-one");
    store
        .report_lifecycle_issue(
            &current_source,
            deleting,
            Some("restored"),
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
        .retire_execution(&current_source, deleting, "restored")
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
        .find(|r| r.id == "one")
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
            .find(|r| r.id == "one")
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
            "agent-pending",
            &LifecycleRequest {
                request_id: "delete-pending".into(),
                expected_request_id: "create-pending".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .force_remove_agent(&owner_digest, "agent-pending", "delete-pending", now()?)
        .await?;
    assert!(
        store
            .managed_resources(&current_source, scope("pending", "delete-pending"))
            .await?
            .iter()
            .all(|r| r.identity().is_none()
                && r.absence_confirmed_at.is_some()
                && r.source_connection_id.is_none())
    );
    let uncertain_candidate = scope("collision", "create-collision");
    store
        .reserve_execution(
            &current_source,
            uncertain_candidate,
            &draft("possibly-forwarded"),
        )
        .await?;
    assert!(
        store
            .claim_execution_dispatch(&current_source, uncertain_candidate, "possibly-forwarded")
            .await?
    );
    store
        .request_lifecycle(
            &owner_digest,
            "agent-collision",
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
            .managed_resources(&current_source, scope("collision", "delete-uncertain"))
            .await?
            .iter()
            .all(|r| r.absence_confirmed_at.is_none()),
        "Possible forwarding cannot become non-allocation proof"
    );
    mutate_fixture(
        connection,
        &format!(
            "UPDATE computers SET created_at=0,agent_removed_at=0 WHERE id='pending'; UPDATE computers SET cleanup_completed_at={} WHERE id='one'",
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
            "UPDATE computers SET cleanup_completed_at={} WHERE id='one'",
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
            .force_remove_agent(&owner_digest, "agent-one", "delete-one", now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .managed_computers(&current_source, None, 100)
            .await?
            .iter()
            .any(|r| r.id == "pending" && r.cleanup_completed_at.is_none())
    );
    assert!(matches!(
        store
            .create_cloud_agent(&owner_digest, &creation("one"), now()?)
            .await?,
        CreationOutcome::Removed { .. }
    ));

    // Normal cleanup removes the still-visible Agent only after confirmation.
    store
        .request_lifecycle(
            &owner_digest,
            "agent-empty",
            &LifecycleRequest {
                request_id: "delete-empty".into(),
                expected_request_id: "create-empty".into(),
                action: LifecycleAction::Delete,
            },
            now()?,
        )
        .await?;
    store
        .confirm_computer_cleanup(&current_source, scope("empty", "delete-empty"))
        .await?;
    assert!(matches!(
        store.agent(&owner_digest, "agent-empty", now()?).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .force_remove_agent(&owner_digest, "agent-empty", "delete-empty", now()?)
            .await?,
        chapchoo_persistence::AgentRemoval::AlreadyRemoved
    );
    assert!(matches!(
        store
            .force_remove_agent(&owner_digest, "agent-empty", "unaccepted-delete", now()?)
            .await,
        Err(StoreError::Conflict)
    ));
    exercise_stop_and_initial(&store, connection, &current_source, &owner_digest).await?;
    exercise_review_regressions(&store, connection, &current_source, &owner_digest).await?;
    exercise_provider_directory(&store, connection, &provider_digest).await?;
    Ok(())
}

async fn exercise_provider_directory(
    store: &Store,
    connection: &mut impl SimpleConnection,
    credential: &chapchoo_crypto::SecretDigest,
) -> Result<(), TestError> {
    use chapchoo_persistence::{ConnectionOwner, DirectoryPolicy};
    let policy = DirectoryPolicy::default();
    for control in ["control-a", "control-b"] {
        let registered = store
            .register_control(control, "http://127.0.0.1:1", policy)
            .await?;
        assert!(!registered.draining);
        store.renew_control(control, policy).await?;
    }
    exercise_connector_directory(store, connection, credential).await?;
    assert!(matches!(
        store
            .register_control("control-a", "http://127.0.0.1:2", policy)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .withdraw_drained_control_connections("control-a")
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let expected = store
        .provider_connection_owner("provider", credential)
        .await?
        .ok_or("Missing original route")?;
    let candidates = [
        ConnectionOwner {
            control_id: "control-a".into(),
            connection_id: "candidate-a".into(),
        },
        ConnectionOwner {
            control_id: "control-b".into(),
            connection_id: "candidate-b".into(),
        },
    ];
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut tasks = Vec::new();
    for candidate in candidates {
        let store = store.clone();
        let key = credential.clone();
        let expected = expected.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .admit_provider_connection("provider", &key, Some(&expected), &candidate, policy)
                .await
        }));
    }
    let (a, b) = tasks::join_pair(tasks.remove(0), tasks.remove(0), None).await?;
    let winner = match (a, b) {
        (Ok(route), Err(StoreError::Conflict)) | (Err(StoreError::Conflict), Ok(route)) => route,
        _ => return Err("Exactly one concurrent stream admission must succeed".into()),
    };
    assert!(
        !store
            .close_provider_connection("provider", credential, &expected)
            .await?
    );
    assert!(matches!(
        store
            .renew_provider_connection("provider", credential, &expected, policy)
            .await,
        Err(StoreError::Conflict)
    ));
    let winner_owner = winner.owner();
    let resolved = store
        .resolve_provider_route("provider")
        .await?
        .ok_or("Missing winning route")?;
    assert_eq!(resolved.component.owner(), winner_owner);
    store
        .renew_provider_connection("provider", credential, &winner_owner, policy)
        .await?;
    let next = ConnectionOwner {
        control_id: if winner_owner.control_id == "control-a" {
            "control-b"
        } else {
            "control-a"
        }
        .into(),
        connection_id: "replacement".into(),
    };
    store
        .admit_provider_connection("provider", credential, Some(&winner_owner), &next, policy)
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
            .resolve_provider_route("provider")
            .await?
            .ok_or("Other Control route was cleared")?
            .component
            .owner(),
        next
    );
    assert!(
        !store
            .close_provider_connection("provider", credential, &winner_owner)
            .await?
    );
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET expires_at=1 WHERE provider_id='provider'",
    )?;
    assert!(store.resolve_provider_route("provider").await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection("provider", credential, &next, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    // An expired tuple is still compared exactly, and only a fresh connection wins.
    let expired = store
        .provider_connection_owner("provider", credential)
        .await?
        .ok_or("Missing expired tuple")?;
    let fresh = ConnectionOwner {
        control_id: next.control_id.clone(),
        connection_id: "fresh-after-expiry".into(),
    };
    store
        .admit_provider_connection("provider", credential, Some(&expired), &fresh, policy)
        .await?;
    store.drain_control(&fresh.control_id).await?;
    assert!(matches!(
        store.renew_control(&fresh.control_id, policy).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(store.resolve_provider_route("provider").await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection("provider", credential, &fresh, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .withdraw_drained_control_connections(&fresh.control_id)
            .await?,
        1
    );
    assert!(
        store
            .provider_connection_owner("provider", credential)
            .await?
            .is_none()
    );
    store
        .register_control("control-expired", "http://127.0.0.1:3", policy)
        .await?;
    mutate_fixture(
        connection,
        "UPDATE control_instances SET renewed_at=0,expires_at=1 WHERE id='control-expired'",
    )?;
    assert!(matches!(
        store.renew_control("control-expired", policy).await,
        Err(StoreError::InvalidTransition)
    ));
    let invalid = ConnectionOwner {
        control_id: "control-expired".into(),
        connection_id: "expired-control-connection".into(),
    };
    assert!(matches!(
        store
            .admit_provider_connection("provider", credential, None, &invalid, policy)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .register_control("control-new", "http://127.0.0.1:4", policy)
        .await?;
    let new_owner = ConnectionOwner {
        control_id: "control-new".into(),
        connection_id: "new-active".into(),
    };
    store
        .admit_provider_connection("provider", credential, None, &new_owner, policy)
        .await?;
    let wrong = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .renew_provider_connection("provider", &wrong, &new_owner, policy)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=1 WHERE provider_id='provider'",
    )?;
    assert!(store.resolve_provider_route("provider").await?.is_none());
    assert!(matches!(
        store
            .renew_provider_connection("provider", credential, &new_owner, policy)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    // Owned drain still withdraws rows when the component credential expired/revoked.
    store.drain_control("control-new").await?;
    assert_eq!(
        store
            .withdraw_drained_control_connections("control-new")
            .await?,
        1
    );
    Ok(())
}

async fn exercise_connector_directory(
    store: &Store,
    connection: &mut impl SimpleConnection,
    provider: &chapchoo_crypto::SecretDigest,
) -> Result<(), TestError> {
    use chapchoo_persistence::{
        ComponentKeyDraft, ComponentKeyPolicy, ConnectionOwner, ConnectorSessionDraft,
        DirectoryPolicy,
    };
    let route = store
        .resolve_provider_route("provider")
        .await?
        .ok_or("Missing Provider route")?;
    let source = ProviderSource {
        provider_id: "provider",
        control_id: &route.component.control_id,
        connection_id: &route.component.connection_id,
        credential: provider,
    };
    let draft = ConnectorSessionDraft {
        id: "directory-session".into(),
        request_id: "directory-session-create".into(),
        initial_key: BearerSecret::generate()?,
    };
    let stale = ProviderSource {
        connection_id: "stale-connection",
        ..source
    };
    assert!(matches!(
        store
            .issue_connector_session_from_source(&stale, "initial-resumed", &draft, now()?)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let session = store
        .issue_connector_session_from_source(&source, "initial-resumed", &draft, now()?)
        .await?;
    assert!(matches!(
        store
            .revoke_connector_session_from_source(&stale, &session.id, "stale-revoke", now()?)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .connector_connection_owner(&session.id, &session.initial_key.digest())
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
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut tasks = Vec::new();
    for control in ["control-a", "control-b"] {
        let store = store.clone();
        let key = key.token.digest();
        let session_id = session.id.clone();
        let barrier = barrier.clone();
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            store
                .admit_connector_connection(
                    &session_id,
                    &key,
                    None,
                    &ConnectionOwner {
                        control_id: control.into(),
                        connection_id: format!("connector-{control}"),
                    },
                    DirectoryPolicy::default(),
                )
                .await
        }));
    }
    let (a, b) = tasks::join_pair(tasks.remove(0), tasks.remove(0), None).await?;
    let winner = match (a, b) {
        (Ok(route), Err(StoreError::Conflict)) | (Err(StoreError::Conflict), Ok(route)) => route,
        _ => return Err("Exactly one empty-tuple Connector admission must succeed".into()),
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
    assert_eq!(identity.computer_id, "unallocated");
    assert_eq!(
        store
            .resolve_connector_route("unallocated")
            .await?
            .ok_or("Missing verified Connector")?
            .component
            .expires_at,
        winner.expires_at
    );
    store
        .renew_connector_connection(
            &session.id,
            &key.token.digest(),
            &first,
            DirectoryPolicy::default(),
        )
        .await?;
    let replacement = ConnectionOwner {
        control_id: first.control_id.clone(),
        connection_id: "connector-replaced".into(),
    };
    store
        .admit_connector_connection(
            &session.id,
            &key.token.digest(),
            Some(&first),
            &replacement,
            DirectoryPolicy::default(),
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
            .renew_connector_connection(
                &session.id,
                &key.token.digest(),
                &first,
                DirectoryPolicy::default()
            )
            .await,
        Err(StoreError::Conflict)
    ));
    // Provider key/route authority is not a prerequisite for Connector heartbeats.
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=1 WHERE provider_id='provider'",
    )?;
    store
        .renew_connector_connection(
            &session.id,
            &key.token.digest(),
            &replacement,
            DirectoryPolicy::default(),
        )
        .await?;
    assert_eq!(
        store
            .resolve_connector_route("unallocated")
            .await?
            .ok_or("Missing independent Connector route")?
            .component
            .owner(),
        replacement
    );
    mutate_fixture(
        connection,
        "UPDATE provider_keys SET revoked_at=NULL WHERE provider_id='provider'",
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
        .renew_connector_connection(
            &session.id,
            &key.token.digest(),
            &replacement,
            DirectoryPolicy::default(),
        )
        .await?;
    mutate_fixture(
        connection,
        "UPDATE connector_sessions SET previous_valid_until=1 WHERE id='directory-session'",
    )?;
    assert!(
        store
            .resolve_connector_route("unallocated")
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &key.token.digest(),
                &replacement,
                DirectoryPolicy::default()
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let expected = store
        .connector_connection_owner(&session.id, &refreshed.token.digest())
        .await?
        .ok_or("Missing previous-key route")?;
    let current = ConnectionOwner {
        control_id: expected.control_id.clone(),
        connection_id: "connector-refreshed".into(),
    };
    store
        .admit_connector_connection(
            &session.id,
            &refreshed.token.digest(),
            Some(&expected),
            &current,
            DirectoryPolicy::default(),
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
            .resolve_connector_route("unallocated")
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &refreshed.token.digest(),
                &current,
                DirectoryPolicy::default()
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let latest = ConnectionOwner {
        control_id: current.control_id.clone(),
        connection_id: "connector-latest".into(),
    };
    store
        .admit_connector_connection(
            &session.id,
            &exchanged.token.digest(),
            None,
            &latest,
            DirectoryPolicy::default(),
        )
        .await?;
    store
        .revoke_connector_session(
            "provider",
            provider,
            &session.id,
            "directory-revoke",
            now()?,
        )
        .await?;
    assert!(
        store
            .resolve_connector_route("unallocated")
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .renew_connector_connection(
                &session.id,
                &exchanged.token.digest(),
                &latest,
                DirectoryPolicy::default()
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    Ok(())
}

async fn exercise_stop_and_initial(
    store: &Store,
    connection: &mut impl SimpleConnection,
    source: &ProviderSource<'_>,
    owner: &chapchoo_crypto::SecretDigest,
) -> Result<(), TestError> {
    for id in ["stopping", "unallocated"] {
        store
            .create_cloud_agent(owner, &creation(id), now()?)
            .await?;
    }
    let stopping = scope("stopping", "create-stopping");
    store
        .reserve_execution(source, stopping, &draft("stop-original"))
        .await?;
    present_all(store, source, stopping, "stop-original").await?;
    store
        .report_execution_running(source, stopping, "stop-original")
        .await?;
    store
        .request_lifecycle(
            owner,
            "agent-stopping",
            &LifecycleRequest {
                request_id: "stop".into(),
                expected_request_id: "create-stopping".into(),
                action: LifecycleAction::Stop,
            },
            now()?,
        )
        .await?;
    let stop = scope("stopping", "stop");
    assert!(matches!(
        store.retire_execution(source, stop, "stop-original").await,
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
    store
        .retire_execution(source, stop, "stop-original")
        .await?;
    let stopped = store.agent(owner, "agent-stopping", now()?).await?;
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
            .agent(owner, "agent-stopping", now()?)
            .await?
            .computer
            .progress
            .phase,
        LifecyclePhase::Completed
    );
    mutate_fixture(
        connection,
        "UPDATE provider_connections SET connection_id='stop-reconnected' WHERE provider_id='provider'",
    )?;
    let reconnected = ProviderSource {
        connection_id: "stop-reconnected",
        ..*source
    };
    assert_eq!(
        store
            .agent(owner, "agent-stopping", now()?)
            .await?
            .computer
            .power,
        ObservedPower::Unknown
    );
    assert!(matches!(
        store
            .retire_execution(&reconnected, stop, "stop-original")
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                owner,
                "agent-stopping",
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
        .retire_execution(&reconnected, stop, "stop-original")
        .await?;
    let refreshed = store.agent(owner, "agent-stopping", now()?).await?;
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
            "agent-stopping",
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
            scope("stopping", "start"),
            &draft("stop-successor"),
        )
        .await?;
    mutate_fixture(
        connection,
        &format!(
            "UPDATE provider_connections SET connection_id='{}' WHERE provider_id='provider'",
            source.connection_id
        ),
    )?;

    // Quiescent initial allocation without any observed UID may continue, while
    // allocated/missing storage above remains irreplaceable.
    let initial = scope("unallocated", "create-unallocated");
    store
        .reserve_execution(source, initial, &draft("never-created"))
        .await?;
    for resource in store.managed_resources(source, initial).await? {
        report(
            store,
            source,
            initial,
            &resource,
            ResourceEvidence::DefinitivelyAbsent { expected_uid: None },
        )
        .await?;
    }
    store
        .retire_execution(source, initial, "never-created")
        .await?;
    let resumed = store
        .reserve_execution(source, initial, &draft("initial-resumed"))
        .await?;
    assert!(
        resumed
            .resources
            .iter()
            .any(|r| r.kind() == ManagedResourceKind::Pvc
                && r.native().name() == "pvc-never-created"
                && r.identity().is_none())
    );
    present_all(store, source, initial, "initial-resumed").await?;
    store
        .report_execution_running(source, initial, "initial-resumed")
        .await?;
    Ok(())
}
async fn exercise_review_regressions(
    store: &Store,
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
        let agent_id = format!("agent-{id}");
        let execution = format!("execution-{id}");
        let request = scope(id, &create_id);
        mutate_fixture(
            connection,
            "UPDATE computer_providers SET can_create=TRUE,can_start=FALSE",
        )?;
        store
            .create_cloud_agent(owner, &creation(id), now()?)
            .await?;
        store
            .reserve_execution(source, request, &draft(&execution))
            .await?;
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
            &format!("UPDATE computers SET initial_start_dispatch_at=123 WHERE id='{id}'"),
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
                .agent(owner, &agent_id, now()?)
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
        store
            .reserve_execution(source, request, &draft(&successor))
            .await?;
        store
            .mark_start_dispatched(source, request, &successor)
            .await?;
        let progress = store
            .agent(owner, &agent_id, now()?)
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
                .agent(owner, &agent_id, now()?)
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
    connection.batch_execute(&seed(&cipher, &owner, &provider, false)?)?;
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
        connection.batch_execute(&seed(&cipher, &owner, &provider, true)?)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url, cipher).await?;
            exercise(store, &mut connection, owner, provider).await
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
