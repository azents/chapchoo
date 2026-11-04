#[path = "support/legacy.rs"]
mod legacy;
mod support;
#[path = "support/tasks.rs"]
mod tasks;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_domain::{DesiredState, LifecycleAction};
use chapchoo_persistence::{
    ComputerRequest, CreateCloudAgent, CreateTeam, CreationOutcome, LifecycleRequest,
    ProviderSource, Store, StoreError, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type TestError = Box<dyn Error + Send + Sync>;
fn now() -> Result<i64, TestError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
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
    let expiry = now()? + 600_000;
    let mut sql = String::from("BEGIN;");
    for (index, token) in users.iter().enumerate() {
        let session = format!("session-{index}");
        let protected = cipher.protect_latest(&session, token)?;
        sql.push_str(&format!("INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region) VALUES('user-{index}','{index}@example.invalid','{index}@example.invalid','synthetic',TRUE,0,'en','UTC','en-US'); INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal-{index}','PERSONAL','Personal','personal-{index}','user-{index}',0); INSERT INTO memberships VALUES('personal-{index}','user-{index}','MEMBER'); INSERT INTO personal_contexts VALUES('user-{index}','personal-{index}'); INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('{session}','user-{index}',{},{},{},0,0,{expiry});", binary(token.digest().as_bytes()), binary(protected.nonce()), binary(protected.ciphertext())));
    }
    sql.push_str("INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('shared','COLLABORATIVE','Shared','shared','user-0',0); INSERT INTO memberships VALUES('shared','user-0','MEMBER'),('shared','user-1','MEMBER'),('shared','user-2','MEMBER');");
    let protected = cipher.protect_latest("computer-provider:provider", provider)?;
    sql.push_str(&format!("INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at) VALUES('provider','SYSTEM','owner','Fixture','KUBERNETES',TRUE,TRUE,TRUE,TRUE,0); INSERT INTO provider_keys(provider_id,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('provider',{},0,{expiry},'key',{},{}); INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://127.0.0.1:1',0,{expiry}); INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES('provider','control','connection',{}, {expiry}); COMMIT;",binary(provider.digest().as_bytes()),binary(protected.nonce()),binary(protected.ciphertext()),binary(provider.digest().as_bytes())));
    Ok(sql)
}
struct FixtureIds {
    provider: String,
    image: String,
    admin: chapchoo_crypto::SecretDigest,
}
fn draft(ids: &FixtureIds, id: &str, team: &str) -> CreateCloudAgent {
    CreateCloudAgent {
        team_id: team.into(),
        request_id: format!("create-{id}"),
        provider_id: ids.provider.clone(),
        name: "Private".into(),
        profile: chapchoo_persistence::CreateAgentProfile {
            handle: id.into(),
            image_id: ids.image.clone(),
            palette_key: "sage".into(),
        },
    }
}
async fn create(
    store: &Store,
    ids: &FixtureIds,
    actor: &BearerSecret,
    id: &str,
    team: &str,
) -> Result<chapchoo_persistence::AgentSummary, TestError> {
    let CreationOutcome::Present(agent) = store
        .create_cloud_agent(&actor.digest(), &draft(ids, id, team), now()?)
        .await
        .map_err(|error| format!("Create {id}: {error}"))?
    else {
        return Err("Expected Agent".into());
    };
    assert_eq!(agent.team_id, team);
    Ok(*agent)
}
async fn clean(
    store: &Store,
    ids: &FixtureIds,
    provider: &BearerSecret,
    computer: &str,
) -> Result<(), TestError> {
    let digest = provider.digest();
    let route = store
        .resolve_provider_route(&ids.provider)
        .await?
        .ok_or("Provider route")?;
    let source = ProviderSource {
        provider_id: &ids.provider,
        control_id: &route.component.control_id,
        connection_id: &route.component.connection_id,
        credential: &digest,
    };
    let managed = store.managed_computers(&source, None, 100).await?;
    let current = managed
        .iter()
        .find(|c| c.id == computer)
        .ok_or("Missing managed Computer")?;
    assert_eq!(current.desired_state, "DELETING");
    store
        .confirm_computer_cleanup(
            &source,
            ComputerRequest {
                computer_id: computer,
                request_id: &current.current_request_id,
            },
        )
        .await?;
    let completed = store
        .computer_inventory(&ids.admin, &ids.provider, None, 50, now()?)
        .await
        .map_err(|error| format!("Inventory after cleanup: {error}"))?;
    assert!(
        completed
            .iter()
            .any(|row| row.computer.id == computer && row.cleanup_completed_at.is_some())
    );
    Ok(())
}
async fn exercise(
    store: Store,
    users: &[BearerSecret],
    provider: &BearerSecret,
) -> Result<(), TestError> {
    let ids = FixtureIds {
        admin: users[0].digest(),
        provider: store
            .provider_inventory(&users[0].digest(), None, 1, now()?)
            .await?
            .remove(0)
            .id,
        image: store
            .agent_profile_images(&users[0].digest(), None, 1, now()?)
            .await?
            .remove(0)
            .id,
    };
    let personal = store
        .teams(&users[0].digest(), now()?)
        .await?
        .into_iter()
        .find(|t| t.kind == chapchoo_domain::TeamKind::Personal)
        .ok_or("Personal Team")?
        .id;
    let shared = store
        .team_by_handle(&users[0].digest(), "shared", now()?)
        .await?
        .id;
    let missing = store
        .teams(&users[1].digest(), now()?)
        .await?
        .into_iter()
        .find(|t| t.kind == chapchoo_domain::TeamKind::Personal)
        .ok_or("Other personal Team")?
        .id;
    let kicked_user = store.current_identity(&users[2].digest(), now()?).await?.id;
    let personal_agent = create(&store, &ids, &users[0], "personal", &personal).await?;
    let owner_agent = create(&store, &ids, &users[0], "owner", &shared).await?;
    let leaving = create(&store, &ids, &users[1], "leaving", &shared).await?;
    let kicked = create(&store, &ids, &users[2], "kicked", &shared).await?;
    for (index, expected) in [(0, &owner_agent.id), (1, &leaving.id), (2, &kicked.id)] {
        let agents = store
            .agents(&users[index].digest(), &shared, None, 20, now()?)
            .await?;
        assert_eq!(agents.len(), 1);
        assert_eq!(&agents[0].id, expected);
    }
    assert!(matches!(
        store.agent(&users[0].digest(), &leaving.id, now()?).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .agents(&users[1].digest(), &personal, None, 20, now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(
                &users[1].digest(),
                &draft(&ids, "foreign", &personal),
                now()?
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(
                &users[0].digest(),
                &draft(&ids, "missing", &missing),
                now()?
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[0].digest(), &draft(&ids, "owner", &personal), now()?)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.agents(&users[0].digest(), "", None, 20, now()?).await,
        Err(StoreError::InvalidInput)
    ));

    store
        .leave_team(&users[1].digest(), &shared, now()?)
        .await?;
    assert!(matches!(
        store
            .delivery_snapshot(
                Some(&users[1].digest()),
                ComputerRequest {
                    computer_id: &leaving.computer.id,
                    request_id: "create-leaving",
                },
                None
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store.agent(&users[1].digest(), &leaving.id, now()?).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[1].digest(), &draft(&ids, "late", &shared), now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .agent(&users[0].digest(), &owner_agent.id, now()?)
            .await?
            .computer
            .desired,
        DesiredState::Running
    );
    clean(&store, &ids, provider, &leaving.computer.id).await?;

    store
        .remove_team_member(&users[0].digest(), &shared, &kicked_user, now()?)
        .await?;
    assert!(matches!(
        store
            .request_lifecycle(
                &users[2].digest(),
                &kicked.id,
                &LifecycleRequest {
                    request_id: "after-kick".into(),
                    expected_request_id: "create-kicked".into(),
                    action: LifecycleAction::Delete
                },
                now()?
            )
            .await,
        Err(StoreError::NotFound)
    ));
    clean(&store, &ids, provider, &kicked.computer.id).await?;

    store
        .delete_team(&users[0].digest(), &shared, now()?)
        .await?;
    assert!(matches!(
        store.team(&users[0].digest(), &shared, now()?).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .teams(&users[0].digest(), now()?)
            .await?
            .iter()
            .all(|t| t.id != shared)
    );
    assert!(matches!(
        store
            .agents(&users[0].digest(), &shared, None, 20, now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(
                &users[0].digest(),
                &draft(&ids, "after-delete", &shared),
                now()?
            )
            .await,
        Err(StoreError::NotFound)
    ));
    // Provider cleanup remains authoritative after membership or Team access ends.
    // A hidden cleanup row must not retain the deleted public handle. A new
    // Team with that handle has a distinct identity and cannot adopt old Agents.
    let replacement = store
        .create_team(
            &users[0].digest(),
            &CreateTeam {
                name: "Replacement".into(),
                handle: "shared".into(),
            },
            now()?,
        )
        .await?;
    assert!(
        store
            .agents(&users[0].digest(), &replacement.id, None, 20, now()?)
            .await?
            .is_empty()
    );
    clean(&store, &ids, provider, &owner_agent.computer.id).await?;
    assert_eq!(
        store
            .team(&users[0].digest(), &replacement.id, now()?)
            .await?
            .handle
            .as_deref(),
        Some("shared")
    );
    assert_eq!(
        store
            .agent(&users[0].digest(), &personal_agent.id, now()?)
            .await?
            .team_id,
        personal
    );
    concurrent_team_deletion(&store, &ids, &users[0], provider).await?;
    Ok(())
}

async fn concurrent_team_deletion(
    store: &Store,
    ids: &FixtureIds,
    user: &BearerSecret,
    provider: &BearerSecret,
) -> Result<(), TestError> {
    let team = store
        .create_team(
            &user.digest(),
            &CreateTeam {
                name: "Race".into(),
                handle: "race".into(),
            },
            now()?,
        )
        .await?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let creation_store = store.clone();
    let creation_actor = user.digest();
    let started = barrier.clone();
    let creation = draft(ids, "race", &team.id);
    let team_id = team.id.clone();
    let create = tokio::spawn(async move {
        started.wait().await;
        creation_store
            .create_cloud_agent(
                &creation_actor,
                &creation,
                now().map_err(|_| StoreError::Integrity)?,
            )
            .await
    });
    let deletion_store = store.clone();
    let deletion_actor = user.digest();
    let delete = tokio::spawn(async move {
        barrier.wait().await;
        deletion_store
            .delete_team(
                &deletion_actor,
                &team_id,
                now().map_err(|_| StoreError::Integrity)?,
            )
            .await
    });
    let (created, deleted) = tasks::join_pair(
        create,
        delete,
        Some(tokio::time::Instant::now() + std::time::Duration::from_secs(10)),
    )
    .await?;
    deleted?;
    match created {
        Ok(CreationOutcome::Present(agent)) => {
            clean(store, ids, provider, &agent.computer.id).await?
        }
        Err(StoreError::NotFound) => {}
        other => return Err(format!("Unexpected concurrent outcome: {other:?}").into()),
    }
    assert!(matches!(
        store.team(&user.digest(), &team.id, now()?).await,
        Err(StoreError::NotFound)
    ));
    Ok(())
}
#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}
fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([46; 32]),
    )?)?))
}
#[test]
fn sqlite_private_team_agents_and_departure_cleanup() -> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("agent-teams.sqlite");
    let cipher = cipher()?;
    let users = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    legacy::sqlite(&mut connection, &cipher)?;
    connection.batch_execute(&seed(&cipher, &users, &provider, false)?)?;
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_sqlite(path, cipher).await?,
            &users,
            &provider,
        )
        .await
    })?;
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM teams WHERE name='Shared'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM agent_creation_receipts WHERE request_id IN ('create-owner','create-leaving','create-kicked')"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        3
    );
    assert!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE desired_state='DELETING' AND cleanup_completed_at IS NOT NULL").get_result::<Count>(&mut connection)?.n >= 3);
    Ok(())
}
#[test]
fn postgres_private_team_agents_and_departure_cleanup() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let schema = format!(
        "agent_teams_{}",
        hex(&BearerSecret::generate()?.digest().as_bytes()[..8])
    );
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = postgres_in_schema(&url);
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
fn postgres_in_schema(url: &str) -> Result<(), TestError> {
    let cipher = cipher()?;
    let users = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    legacy::postgres(&mut connection, &cipher)?;
    connection.batch_execute(&seed(&cipher, &users, &provider, true)?)?;
    prepare_postgres(url, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_postgres(url.into(), cipher).await?,
            &users,
            &provider,
        )
        .await
    })?;
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM teams WHERE name='Shared'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM agent_creation_receipts WHERE request_id IN ('create-owner','create-leaving','create-kicked')"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        3
    );
    assert!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE desired_state='DELETING' AND cleanup_completed_at IS NOT NULL").get_result::<Count>(&mut connection)?.n >= 3);
    Ok(())
}
