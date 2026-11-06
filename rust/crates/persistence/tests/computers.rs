#[path = "support/authentication.rs"]
mod fixture_auth;
#[path = "support/legacy.rs"]
mod legacy;
mod support;
#[path = "support/tasks.rs"]
mod tasks;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, BootstrapProviderPermit, CredentialCipher, RootKey};
use chapchoo_domain::{DesiredState, LifecycleAction, ObservedPower};
use chapchoo_persistence::{
    AgentRemoval, ComponentKeyDraft, ComponentKeyPolicy, ConnectorSessionDraft, CreateCloudAgent,
    CreationOutcome, LifecycleRequest, ProviderCapabilities, ProviderOwner, Store, StoreError,
    prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{error::Error, sync::Arc};

const NOW: i64 = 1_000;
type TestError = Box<dyn Error + Send + Sync>;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn seed(
    cipher: &CredentialCipher,
    users: &[BearerSecret],
    provider: &BearerSecret,
    postgres: bool,
) -> Result<String, TestError> {
    let binary = |bytes: &[u8]| {
        if postgres {
            format!("decode('{}','hex')", hex(bytes))
        } else {
            format!("X'{}'", hex(bytes))
        }
    };
    let mut sql = String::from("BEGIN;");
    for (index, token) in users.iter().enumerate() {
        let session = format!("session-{index}");
        let protected = cipher.protect_latest(&session, token)?;
        sql.push_str(&format!("INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('user-{index}','{index}@example.invalid','{index}@example.invalid','synthetic',0,'en','UTC','en-US'); INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('{session}','user-{index}',{},{},{},0,0,10000000);",binary(token.digest().as_bytes()),binary(protected.nonce()),binary(protected.ciphertext())));
        sql.push_str(&format!("INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal-{index}','PERSONAL','Personal','personal-{index}','user-{index}',0); INSERT INTO memberships VALUES('personal-{index}','user-{index}','MEMBER'); INSERT INTO personal_contexts VALUES('user-{index}','personal-{index}');"));
    }
    sql.push_str("UPDATE users SET is_admin=TRUE WHERE id='user-0';");
    let protected = cipher.protect_latest("computer-provider:provider", provider)?;
    sql.push_str(&format!("INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at) VALUES('provider','SYSTEM','installation','Fixture','KUBERNETES',TRUE,TRUE,TRUE,TRUE,0); INSERT INTO provider_keys(provider_id,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('provider',{},0,10000000,'key-request',{},{}); INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://127.0.0.1:1',0,10000000); INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES('provider','control','connection',{},10000000);",binary(provider.digest().as_bytes()),binary(protected.nonce()),binary(protected.ciphertext()),binary(provider.digest().as_bytes())));
    for (id, power, source, environment) in [
        ("running", "RUNNING", "connection", "VERIFIED"),
        ("stale", "RUNNING", "old-connection", "VERIFIED"),
        ("stopped", "STOPPED", "connection", "VERIFIED"),
        ("missing", "STOPPED", "connection", "MISSING"),
    ] {
        sql.push_str(&format!("INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,observed_power,power_observed_at,power_source_connection_id,environment_source_connection_id,saved_environment,created_at) VALUES('{id}','user-0','CLOUD','provider','RUNNING','prior-{id}',{},'START','COMPLETED','{power}',0,'{source}','connection','{environment}',0); INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('agent-{id}','user-0','personal-0','{id}','{id}','CLOUD',0);",binary(&[0;32])));
        if power == "RUNNING" {
            sql.push_str(&format!("INSERT INTO computer_executions(id,computer_id,initial_allocation,running_confirmed_at,created_at) VALUES('execution-{id}','{id}',FALSE,0,0); UPDATE computers SET current_execution_id='execution-{id}',power_source_execution_id='execution-{id}' WHERE id='{id}';"));
        }
    }
    // Preserve an accepted pre-authentication payload as opaque history across
    // preparation. New required-auth mutation admission must not reinterpret it.
    sql.push_str(&format!("INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id) VALUES('user-0','personal-0','historical-auth-free',{},'agent-running');", binary(&[17;32])));
    let bootstrap = BearerSecret::generate()?;
    let operational = BearerSecret::generate()?;
    let initial =
        cipher.protect_latest("computer-connector-bootstrap:session-running", &bootstrap)?;
    let current = cipher.protect_latest(
        "computer-connector-operational:session-running",
        &operational,
    )?;
    let initial_payload = binary(&[0; 32]);
    sql.push_str(&format!("INSERT INTO connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,created_at) VALUES('session-running','running','execution-running','initial-session-key',{initial_payload},{},{},{},{},0,10000000,'session-key',{},{},0); INSERT INTO connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at) VALUES('running','session-running','control','connector-connection',{},10000000);",binary(bootstrap.digest().as_bytes()),binary(initial.nonce()),binary(initial.ciphertext()),binary(operational.digest().as_bytes()),binary(current.nonce()),binary(current.ciphertext()),binary(operational.digest().as_bytes())));
    sql.push_str("COMMIT;");
    Ok(sql)
}
struct FixtureIds {
    team: String,
    authentication: String,
    provider: String,
    image: String,
    agents: std::collections::BTreeMap<String, chapchoo_persistence::AgentSummary>,
    executions: std::collections::BTreeMap<String, String>,
}
impl FixtureIds {
    fn agent(&self, name: &str) -> Result<&chapchoo_persistence::AgentSummary, TestError> {
        self.agents
            .get(name)
            .ok_or_else(|| "Missing migrated Agent".into())
    }
}
fn draft(ids: &FixtureIds, request: &str) -> CreateCloudAgent {
    CreateCloudAgent {
        team_id: ids.team.clone(),
        authentication_id: ids.authentication.clone(),
        request_id: request.into(),
        provider_id: ids.provider.clone(),
        name: "My Agent".into(),
        profile: chapchoo_persistence::CreateAgentProfile {
            handle: format!("agent-{}", request.chars().take(10).collect::<String>()),
            image_id: ids.image.clone(),
            palette_key: "sage".into(),
        },
    }
}
fn mutation(request: &str, expected: &str, action: LifecycleAction) -> LifecycleRequest {
    LifecycleRequest {
        request_id: request.into(),
        expected_request_id: expected.into(),
        action,
    }
}
fn present(outcome: CreationOutcome) -> Result<chapchoo_persistence::AgentSummary, TestError> {
    match outcome {
        CreationOutcome::Present(v) => Ok(*v),
        _ => Err("Expected current Agent".into()),
    }
}

async fn exercise(store: Store, users: &[BearerSecret]) -> Result<(), TestError> {
    let owner = users[0].digest();
    let foreign = users[1].digest();
    let team = store.teams(&owner, NOW).await?.remove(0).id;
    let mut ids = FixtureIds {
        authentication: fixture_auth::personal(&store, &owner, NOW).await?,
        provider: store
            .provider_inventory(&owner, None, 20, NOW)
            .await?
            .remove(0)
            .id,
        image: store
            .agent_profile_images(&owner, None, 1, NOW)
            .await?
            .remove(0)
            .id,
        agents: store
            .agents(&owner, &team, None, 20, NOW)
            .await?
            .into_iter()
            .map(|agent| (agent.name.clone(), agent))
            .collect(),
        team,
        executions: Default::default(),
    };
    for name in ["running", "stale"] {
        let agent = ids.agent(name)?;
        let snapshot = store
            .delivery_snapshot(
                None,
                chapchoo_persistence::ComputerRequest {
                    computer_id: &agent.computer.id,
                    request_id: &agent.computer.current_request_id,
                },
                None,
            )
            .await?;
        ids.executions.insert(
            name.into(),
            snapshot
                .computer
                .current_execution_id
                .ok_or("Missing migrated execution")?,
        );
    }
    let running = ids.agent("running")?;
    let stale = ids.agent("stale")?;
    let stopped = ids.agent("stopped")?;
    let missing = ids.agent("missing")?;
    assert_eq!(
        store
            .agent_creation_result(&owner, "historical-auth-free", NOW)
            .await?,
        Some(chapchoo_persistence::AgentCreationResult::Present {
            assigned_agent_id: running.id.clone(),
            team_id: ids.team.clone(),
        })
    );
    assert!(
        store
            .agent_creation_result(&foreign, "historical-auth-free", NOW)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .create_cloud_agent(&owner, &draft(&ids, "historical-auth-free"), NOW)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        ids.agents
            .values()
            .all(|agent| agent.authentication.is_none())
    );
    store
        .select_agent_authentication(
            &owner,
            &chapchoo_persistence::SelectAgentAuthentication {
                request_id: "migrated-agent-selection".into(),
                agent_id: stopped.id.clone(),
                authentication_id: ids.authentication.clone(),
                expected: None,
            },
            NOW,
        )
        .await?;
    let selected_legacy = store.agent(&owner, &stopped.id, NOW).await?;
    assert_eq!(
        selected_legacy
            .authentication
            .ok_or("Missing migrated Agent selection")?
            .authentication_id,
        ids.authentication
    );
    assert_eq!(
        selected_legacy.computer.current_request_id,
        stopped.computer.current_request_id
    );
    assert!(matches!(
        store.provider_inventory(&foreign, None, 20, NOW).await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .computer_inventory(&foreign, &ids.provider, None, 20, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .resource_inventory(&foreign, &running.computer.id, None, 20, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert_eq!(
        store.provider_inventory(&owner, None, 20, NOW).await?.len(),
        1
    );
    let inventory = store
        .computer_inventory(&owner, &ids.provider, None, 2, NOW)
        .await?;
    assert_eq!(inventory.len(), 2);
    let later = store
        .computer_inventory(
            &owner,
            &ids.provider,
            Some(&inventory[1].computer.id),
            20,
            NOW,
        )
        .await?;
    assert!(later.iter().all(|row| {
        inventory
            .iter()
            .all(|previous| row.computer.id != previous.computer.id)
    }));
    assert!(
        later
            .iter()
            .all(|row| !row.residual && row.cleanup_completed_at.is_none())
    );
    let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
    let caps = ProviderCapabilities {
        create: true,
        start: true,
        stop: true,
        restart: true,
    };
    store
        .configure_system_provider(
            &proof,
            "Configured",
            caps,
            &chapchoo_domain::ProviderSubstrate::Kubernetes,
            NOW,
        )
        .await?;
    assert_eq!(
        store.available_providers(&owner, None, 10, NOW).await?[0].name,
        "Configured"
    );
    assert!(
        store
            .available_providers(&owner, None, 0, NOW)
            .await
            .is_err()
    );
    let created = present(
        store
            .create_cloud_agent(&owner, &draft(&ids, "create"), NOW)
            .await?,
    )?;
    assert!(!created.id.is_empty());
    assert_eq!(created.computer.power, ObservedPower::Unknown);
    let replay = present(
        store
            .create_cloud_agent(&owner, &draft(&ids, "create"), NOW)
            .await?,
    )?;
    assert_eq!(replay.id, created.id);
    let mut changed = draft(&ids, "create");
    changed.name = "Different".into();
    assert!(matches!(
        store.create_cloud_agent(&owner, &changed, NOW).await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.agent(&foreign, &created.id, NOW).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                &foreign,
                &created.id,
                &mutation("foreign-delete", "create", LifecycleAction::Delete),
                NOW
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                &owner,
                &created.id,
                &mutation("bad-stop", "create", LifecycleAction::Stop),
                NOW
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store.agent(&owner, &stale.id, NOW).await?.computer.power,
        ObservedPower::Unknown
    );
    assert_eq!(
        store
            .agent(&owner, &stale.id, NOW)
            .await?
            .computer
            .last_known_power,
        ObservedPower::Running
    );
    assert!(matches!(
        store
            .request_lifecycle(
                &owner,
                &stale.id,
                &mutation("stop-stale", "prior-stale", LifecycleAction::Stop),
                NOW
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                &owner,
                &missing.id,
                &mutation("start-missing", "prior-missing", LifecycleAction::Start),
                NOW
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let started = store
        .request_lifecycle(
            &owner,
            &stopped.id,
            &mutation("start", "prior-stopped", LifecycleAction::Start),
            NOW,
        )
        .await?;
    assert_eq!(started.computer.current_request_id, "start");
    let stopped = store
        .request_lifecycle(
            &owner,
            &running.id,
            &mutation("stop", "prior-running", LifecycleAction::Stop),
            NOW,
        )
        .await?;
    assert_eq!(stopped.computer.desired, DesiredState::Stopped);
    let repeat = store
        .request_lifecycle(
            &owner,
            &running.id,
            &mutation("stop", "prior-running", LifecycleAction::Stop),
            NOW,
        )
        .await?;
    assert_eq!(repeat.computer.current_request_id, "stop");
    assert!(matches!(
        store
            .request_lifecycle(
                &owner,
                &running.id,
                &mutation("stop", "prior-running", LifecycleAction::Restart),
                NOW
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .request_lifecycle(
                &owner,
                &running.id,
                &mutation("stale", "prior-running", LifecycleAction::Restart),
                NOW
            )
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .request_lifecycle(
            &owner,
            &running.id,
            &mutation("delete-running", "stop", LifecycleAction::Delete),
            NOW,
        )
        .await?;
    let deleted = store
        .request_lifecycle(
            &owner,
            &created.id,
            &mutation("delete", "create", LifecycleAction::Delete),
            NOW,
        )
        .await?;
    assert_eq!(deleted.computer.desired, DesiredState::Deleting);
    assert_eq!(
        store
            .force_remove_agent(&owner, &created.id, "delete", NOW)
            .await?,
        AgentRemoval::Removed
    );
    assert_eq!(
        store
            .force_remove_agent(&owner, &created.id, "delete", NOW)
            .await?,
        AgentRemoval::AlreadyRemoved
    );
    assert!(matches!(
        store
            .force_remove_agent(&owner, &created.id, "wrong-delete", NOW)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.agent(&owner, &created.id, NOW).await,
        Err(StoreError::NotFound)
    ));
    let residual = store
        .computer_inventory(&owner, &ids.provider, None, 20, NOW)
        .await?;
    assert!(
        residual
            .iter()
            .any(|row| row.computer.id == created.computer.id
                && row.residual
                && row.computer.desired == DesiredState::Deleting)
    );
    assert!(
        matches!(store.create_cloud_agent(&owner,&draft(&ids,"create"),NOW).await?,CreationOutcome::Removed{assigned_agent_id} if assigned_agent_id==created.id)
    );
    assert!(matches!(
        store
            .force_remove_agent(&foreign, &created.id, "delete", NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    let a = store.clone();
    let b = store.clone();
    let da = owner.clone();
    let db = owner.clone();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let ba = barrier.clone();
    let bb = barrier;
    let draft_a = draft(&ids, "race");
    let draft_b = draft(&ids, "race");
    let one = tokio::spawn(async move {
        ba.wait().await;
        a.create_cloud_agent(&da, &draft_a, NOW).await
    });
    let two = tokio::spawn(async move {
        bb.wait().await;
        b.create_cloud_agent(&db, &draft_b, NOW).await
    });
    let (first, second) = tasks::join_pair(one, two, None).await?;
    let (first, second) = support::combine(first.map_err(Into::into), second.map_err(Into::into))?;
    let first = present(first)?;
    let second = present(second)?;
    assert_eq!(first.id, second.id);
    let force_race = present(
        store
            .create_cloud_agent(&owner, &draft(&ids, "force-race-create"), NOW)
            .await?,
    )?;
    store
        .request_lifecycle(
            &owner,
            &force_race.id,
            &mutation(
                "force-race-delete",
                "force-race-create",
                LifecycleAction::Delete,
            ),
            NOW,
        )
        .await?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let a = store.clone();
    let da = owner.clone();
    let ba = barrier.clone();
    let correct_id = force_race.id.clone();
    let correct = tokio::spawn(async move {
        ba.wait().await;
        a.force_remove_agent(&da, &correct_id, "force-race-delete", NOW)
            .await
    });
    let b = store.clone();
    let db = owner.clone();
    let wrong = tokio::spawn(async move {
        barrier.wait().await;
        b.force_remove_agent(&db, &force_race.id, "unaccepted-delete", NOW)
            .await
    });
    let (correct, wrong) = tasks::join_pair(correct, wrong, None).await?;
    assert_eq!(correct?, AgentRemoval::Removed);
    assert!(matches!(
        wrong,
        Err(StoreError::Conflict | StoreError::InvalidTransition)
    ));
    exercise_provider_keys(&store, &ids, &proof).await?;
    exercise_connector_keys(&store, &ids, &proof).await?;
    Ok(())
}

fn provider_key_draft(request: &str) -> Result<ComponentKeyDraft, TestError> {
    key_draft(&format!("{NOW}:{request}"))
}
fn key_draft(request: &str) -> Result<ComponentKeyDraft, TestError> {
    Ok(ComponentKeyDraft {
        request_id: request.into(),
        token: BearerSecret::generate()?,
    })
}

async fn exercise_provider_keys(
    store: &Store,
    ids: &FixtureIds,
    proof: &chapchoo_crypto::VerifiedSystemOwner,
) -> Result<(), TestError> {
    let policy = ComponentKeyPolicy {
        validity_ms: 1_000,
        proactive_after_ms: 500,
        overlap_ms: 100,
    };
    let first = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-one")?,
            policy,
            NOW,
        )
        .await?;
    let repeated = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-one")?,
            policy,
            NOW + 10,
        )
        .await?;
    assert_eq!(first.provider_id, ids.provider);
    assert_eq!(first.provider_id, repeated.provider_id);
    let first = first.key;
    let repeated = repeated.key;
    assert!(first.token.expose() == repeated.token.expose());
    assert_eq!(first.expires_at, repeated.expires_at);
    assert_eq!(first.issued_at, repeated.issued_at);
    store
        .provider_key_identity(&ids.provider, &first.token.digest(), NOW + 10)
        .await?;
    let second = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-two")?,
            policy,
            NOW + 950,
        )
        .await?;
    let second = second.key;
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::System(proof),
                "provider",
                &provider_key_draft("provider-one")?,
                policy,
                NOW + 951
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .provider_key_identity(&ids.provider, &first.token.digest(), NOW + 999)
        .await?;
    assert!(matches!(
        store
            .provider_key_identity(&ids.provider, &first.token.digest(), NOW + 1000)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    store
        .provider_key_identity(&ids.provider, &second.token.digest(), NOW + 1000)
        .await?;
    let third = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-three")?,
            policy,
            NOW + 1100,
        )
        .await?;
    store
        .provider_key_identity(&ids.provider, &second.token.digest(), NOW + 1100)
        .await?;
    store
        .revoke_provider_key(
            ProviderOwner::System(proof),
            "provider",
            "1000:provider-revoke",
            NOW + 1101,
        )
        .await?;
    store
        .revoke_provider_key(
            ProviderOwner::System(proof),
            "provider",
            "1000:provider-revoke",
            NOW + 1102,
        )
        .await?;
    let third = third.key;
    for digest in [second.token.digest(), third.token.digest()] {
        assert!(matches!(
            store
                .provider_key_identity(&ids.provider, &digest, NOW + 1102)
                .await,
            Err(StoreError::Unauthenticated)
        ));
    }
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::System(proof),
                "provider",
                &provider_key_draft("provider-three")?,
                policy,
                NOW + 1102
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let fourth = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-four")?,
            policy,
            NOW + 1200,
        )
        .await?;
    let fourth = fourth.key;
    assert!(matches!(
        store
            .revoke_provider_key(
                ProviderOwner::System(proof),
                "provider",
                "1000:provider-revoke",
                NOW + 1201
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .revoke_provider_key(
                ProviderOwner::System(proof),
                "provider",
                "1000:provider-four",
                NOW + 1201
            )
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .provider_key_identity(&ids.provider, &fourth.token.digest(), NOW + 1201)
        .await?;
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::User(&fourth.token.digest()),
                &ids.provider,
                &provider_key_draft("self-rotation")?,
                policy,
                NOW + 1201
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .provider_key_identity(&ids.provider, &fourth.token.digest(), fourth.expires_at)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::System(proof),
                "provider",
                &provider_key_draft("provider-four")?,
                policy,
                fourth.expires_at
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    Ok(())
}

fn session_draft(request: &str) -> Result<ConnectorSessionDraft, TestError> {
    Ok(ConnectorSessionDraft {
        request_id: request.into(),
        initial_key: BearerSecret::generate()?,
    })
}
async fn exercise_connector_keys(
    store: &Store,
    ids: &FixtureIds,
    proof: &chapchoo_crypto::VerifiedSystemOwner,
) -> Result<(), TestError> {
    let time = NOW + 4_000;
    let provider = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-connector-setup")?,
            ComponentKeyPolicy::default(),
            time,
        )
        .await?;
    let provider = provider.key;
    let stale = ids.agent("stale")?;
    let stale_execution = ids.executions.get("stale").ok_or("Stale execution")?;
    let running_execution = ids.executions.get("running").ok_or("Running execution")?;
    let first = store
        .issue_connector_session(
            &ids.provider,
            &provider.token.digest(),
            stale_execution,
            &session_draft("issue-one")?,
            time,
        )
        .await?;
    let recovered = store
        .issue_connector_session(
            &ids.provider,
            &provider.token.digest(),
            stale_execution,
            &session_draft("issue-two")?,
            time + 1,
        )
        .await?;
    assert_eq!(first.id, recovered.id);
    assert_eq!(first.accepted_request_id, recovered.accepted_request_id);
    assert!(first.initial_key.expose() == recovered.initial_key.expose());
    assert!(matches!(
        store
            .issue_connector_session(
                &ids.provider,
                &provider.token.digest(),
                running_execution,
                &session_draft("terminal-issue")?,
                time
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .connector_key_identity(&first.id, &first.initial_key.digest(), time)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let policy = ComponentKeyPolicy {
        validity_ms: 1_000,
        proactive_after_ms: 500,
        overlap_ms: 100,
    };
    let operational = store
        .exchange_connector_key(
            &first.id,
            &first.initial_key.digest(),
            &key_draft("exchange-one")?,
            policy,
            time + 2,
        )
        .await?;
    let identity = store
        .connector_key_identity(&first.id, &operational.token.digest(), time + 3)
        .await?;
    assert_eq!(identity.computer_id, stale.computer.id);
    assert_eq!(&identity.execution_id, stale_execution);
    let repeated = store
        .exchange_connector_key(
            &first.id,
            &first.initial_key.digest(),
            &key_draft("exchange-one")?,
            policy,
            time + 3,
        )
        .await?;
    assert!(operational.token.expose() == repeated.token.expose());
    assert_eq!(operational.expires_at, repeated.expires_at);
    assert!(matches!(
        store
            .exchange_connector_key(
                &first.id,
                &operational.token.digest(),
                &key_draft("wrong-role")?,
                policy,
                time + 3
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let refreshed = store
        .refresh_connector_key(
            &first.id,
            &operational.token.digest(),
            &key_draft("refresh-one")?,
            policy,
            time + 10,
        )
        .await?;
    let lost_response = store
        .refresh_connector_key(
            &first.id,
            &operational.token.digest(),
            &key_draft("refresh-one")?,
            policy,
            time + 11,
        )
        .await?;
    assert!(refreshed.token.expose() == lost_response.token.expose());
    assert_eq!(refreshed.expires_at, lost_response.expires_at);
    assert!(matches!(
        store
            .refresh_connector_key(
                &first.id,
                &operational.token.digest(),
                &key_draft("previous-new-successor")?,
                policy,
                time + 12
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .exchange_connector_key(
                &first.id,
                &first.initial_key.digest(),
                &key_draft("exchange-one")?,
                policy,
                time + 13
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .connector_key_identity(&first.id, &refreshed.token.digest(), time + 13)
        .await?;
    assert!(matches!(
        store
            .connector_key_identity(&first.id, &operational.token.digest(), time + 110)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let exchanged = store
        .exchange_connector_key(
            &first.id,
            &first.initial_key.digest(),
            &key_draft("exchange-two")?,
            policy,
            time + 130,
        )
        .await?;
    assert!(matches!(
        store
            .connector_key_identity(&first.id, &refreshed.token.digest(), time + 131)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .refresh_connector_key(
                &first.id,
                &exchanged.token.digest(),
                &key_draft("exchange-two")?,
                policy,
                time + 131
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let bootstrap_again = store
        .issue_connector_session(
            &ids.provider,
            &provider.token.digest(),
            stale_execution,
            &session_draft("issue-three")?,
            time + 135,
        )
        .await?;
    assert!(bootstrap_again.initial_key.expose() == first.initial_key.expose());
    store
        .connector_key_identity(&first.id, &exchanged.token.digest(), time + 136)
        .await?;
    store
        .revoke_provider_key(
            ProviderOwner::System(proof),
            "provider",
            "1000:provider-offline",
            time + 140,
        )
        .await?;
    assert!(matches!(
        store
            .provider_key_identity(&ids.provider, &provider.token.digest(), time + 140)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let independent = store
        .refresh_connector_key(
            &first.id,
            &exchanged.token.digest(),
            &key_draft("refresh-independent")?,
            policy,
            time + 150,
        )
        .await?;
    store
        .connector_key_identity(&first.id, &independent.token.digest(), time + 151)
        .await?;
    // Same-request concurrent refresh must recover exactly one successor.
    let a = store.clone();
    let b = store.clone();
    let sa = first.id.clone();
    let sb = first.id.clone();
    let ka = independent.token.digest();
    let kb = ka.clone();
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let ba = barrier.clone();
    let bb = barrier;
    let da = key_draft("refresh-race")?;
    let db = key_draft("refresh-race")?;
    let one = tokio::spawn(async move {
        ba.wait().await;
        a.refresh_connector_key(&sa, &ka, &da, policy, time + 152)
            .await
    });
    let two = tokio::spawn(async move {
        bb.wait().await;
        b.refresh_connector_key(&sb, &kb, &db, policy, time + 152)
            .await
    });
    let (one, two) = tasks::join_pair(one, two, None).await?;
    let (raced_one, raced_two) =
        support::combine(one.map_err(Into::into), two.map_err(Into::into))?;
    assert!(raced_one.token.expose() == raced_two.token.expose());
    assert_eq!(raced_one.expires_at, raced_two.expires_at);
    let a = store.clone();
    let b = store.clone();
    let sa = first.id.clone();
    let sb = first.id.clone();
    let ka = raced_one.token.digest();
    let kb = ka.clone();
    let ba = Arc::new(tokio::sync::Barrier::new(2));
    let bb = ba.clone();
    let da = key_draft("refresh-distinct-one")?;
    let db = key_draft("refresh-distinct-two")?;
    let one = tokio::spawn(async move {
        ba.wait().await;
        a.refresh_connector_key(&sa, &ka, &da, policy, time + 153)
            .await
    });
    let two = tokio::spawn(async move {
        bb.wait().await;
        b.refresh_connector_key(&sb, &kb, &db, policy, time + 153)
            .await
    });
    let mut accepted = None;
    let mut rejected = 0;
    let (one, two) = tasks::join_pair(one, two, None).await?;
    for outcome in [one, two] {
        match outcome {
            Ok(key) => assert!(accepted.replace(key).is_none()),
            Err(StoreError::InvalidTransition) => rejected += 1,
            Err(error) => return Err(error.into()),
        }
    }
    assert_eq!(rejected, 1);
    let newest = accepted.ok_or("Missing accepted refresh")?;
    store
        .connector_key_identity(&first.id, &newest.token.digest(), time + 154)
        .await?;
    let management = store
        .rotate_provider_key(
            ProviderOwner::System(proof),
            "provider",
            &provider_key_draft("provider-for-session-revoke")?,
            ComponentKeyPolicy::default(),
            time + 160,
        )
        .await?;
    let management = management.key;
    store
        .revoke_connector_session(
            &ids.provider,
            &management.token.digest(),
            &first.id,
            "session-revoke",
            time + 170,
        )
        .await?;
    store
        .revoke_connector_session(
            &ids.provider,
            &management.token.digest(),
            &first.id,
            "session-revoke",
            time + 171,
        )
        .await?;
    for digest in [
        independent.token.digest(),
        raced_one.token.digest(),
        newest.token.digest(),
    ] {
        assert!(matches!(
            store
                .connector_key_identity(&first.id, &digest, time + 171)
                .await,
            Err(StoreError::Unauthenticated)
        ));
    }
    assert!(matches!(
        store
            .exchange_connector_key(
                &first.id,
                &first.initial_key.digest(),
                &key_draft("no-revival")?,
                policy,
                time + 171
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .refresh_connector_key(
                &first.id,
                &raced_one.token.digest(),
                &key_draft("refresh-race")?,
                policy,
                time + 171
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .issue_connector_session(
                &ids.provider,
                &management.token.digest(),
                stale_execution,
                &session_draft("no-replacement-request")?,
                time + 171
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .revoke_connector_session(
                &ids.provider,
                &management.token.digest(),
                &first.id,
                "refresh-race",
                time + 171
            )
            .await,
        Err(StoreError::Conflict)
    ));
    Ok(())
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[test]
fn computer_operations_sqlite() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("operations.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    let users = (0..2)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    legacy::sqlite(&mut connection, &cipher)?;
    connection.batch_execute(&seed(&cipher, &users, &provider, false)?)?;
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?
        .block_on(async { exercise(Store::connect_sqlite(path, cipher).await?, &users).await })?;
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE current_request_id='delete' AND agent_removed_at IS NOT NULL AND desired_state='DELETING'").get_result::<Count>(&mut connection)?.n,1);
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM connector_sessions WHERE initial_request_id='initial-session-key' AND revoked_at IS NOT NULL AND initial_nonce IS NULL AND current_hash IS NULL AND recovery_nonce IS NULL").get_result::<Count>(&mut connection)?.n,1);
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM connector_connections")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    Ok(())
}

#[test]
fn computer_operations_postgres() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = hex(&BearerSecret::generate()?.digest().as_bytes()[..8]);
    let schema = format!("computer_operations_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_in_schema(&url);
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
fn postgres_in_schema(url: &str) -> Result<(), TestError> {
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    let users = (0..2)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    legacy::postgres(&mut connection, &cipher)?;
    connection.batch_execute(&seed(&cipher, &users, &provider, true)?)?;
    prepare_postgres(url, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_postgres(url.to_owned(), cipher).await?,
            &users,
        )
        .await
    })?;
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE current_request_id='delete' AND agent_removed_at IS NOT NULL AND desired_state='DELETING'").get_result::<Count>(&mut connection)?.n,1);
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM connector_sessions WHERE initial_request_id='initial-session-key' AND revoked_at IS NOT NULL AND initial_nonce IS NULL AND current_hash IS NULL AND recovery_nonce IS NULL").get_result::<Count>(&mut connection)?.n,1);
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM connector_connections")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    Ok(())
}
