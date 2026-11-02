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
fn draft(id: &str, team: &str) -> CreateCloudAgent {
    CreateCloudAgent {
        agent_id: format!("agent-{id}"),
        computer_id: id.into(),
        team_id: team.into(),
        request_id: format!("create-{id}"),
        provider_id: "provider".into(),
        name: "Private".into(),
    }
}
async fn create(
    store: &Store,
    actor: &BearerSecret,
    id: &str,
    team: &str,
) -> Result<(), TestError> {
    let CreationOutcome::Present(agent) = store
        .create_cloud_agent(&actor.digest(), &draft(id, team), now()?)
        .await?
    else {
        return Err("Expected Agent".into());
    };
    assert_eq!(agent.team_id, team);
    Ok(())
}
async fn clean(store: &Store, provider: &BearerSecret, computer: &str) -> Result<(), TestError> {
    let digest = provider.digest();
    let source = ProviderSource {
        provider_id: "provider",
        control_id: "control",
        connection_id: "connection",
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
    Ok(())
}
async fn exercise(
    store: Store,
    users: &[BearerSecret],
    provider: &BearerSecret,
) -> Result<(), TestError> {
    create(&store, &users[0], "personal", "personal-0").await?;
    create(&store, &users[0], "owner", "shared").await?;
    create(&store, &users[1], "leaving", "shared").await?;
    create(&store, &users[2], "kicked", "shared").await?;
    for (index, expected) in [
        (0, "agent-owner"),
        (1, "agent-leaving"),
        (2, "agent-kicked"),
    ] {
        let agents = store
            .agents(&users[index].digest(), "shared", None, 20, now()?)
            .await?;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, expected);
    }
    assert!(matches!(
        store
            .agent(&users[0].digest(), "agent-leaving", now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .agents(&users[1].digest(), "personal-0", None, 20, now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[1].digest(), &draft("foreign", "personal-0"), now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[0].digest(), &draft("missing", "missing"), now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[0].digest(), &draft("owner", "personal-0"), now()?)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store.agents(&users[0].digest(), "", None, 20, now()?).await,
        Err(StoreError::InvalidInput)
    ));

    store
        .leave_team(&users[1].digest(), "shared", now()?)
        .await?;
    assert!(matches!(
        store
            .delivery_snapshot(
                Some(&users[1].digest()),
                ComputerRequest {
                    computer_id: "leaving",
                    request_id: "create-leaving",
                },
                None
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .agent(&users[1].digest(), "agent-leaving", now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[1].digest(), &draft("late", "shared"), now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .agent(&users[0].digest(), "agent-owner", now()?)
            .await?
            .computer
            .desired,
        DesiredState::Running
    );
    clean(&store, provider, "leaving").await?;

    store
        .remove_team_member(&users[0].digest(), "shared", "user-2", now()?)
        .await?;
    assert!(matches!(
        store
            .request_lifecycle(
                &users[2].digest(),
                "agent-kicked",
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
    clean(&store, provider, "kicked").await?;

    store
        .delete_team(&users[0].digest(), "shared", now()?)
        .await?;
    assert!(matches!(
        store.team(&users[0].digest(), "shared", now()?).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .teams(&users[0].digest(), now()?)
            .await?
            .iter()
            .all(|t| t.id != "shared")
    );
    assert!(matches!(
        store
            .agents(&users[0].digest(), "shared", None, 20, now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .create_cloud_agent(&users[0].digest(), &draft("after-delete", "shared"), now()?)
            .await,
        Err(StoreError::NotFound)
    ));
    // Provider cleanup remains authoritative after membership or Team access ends.
    // A hidden cleanup row must not retain the deleted public handle. A new
    // Team with that handle has a distinct identity and cannot adopt old Agents.
    store
        .create_team(
            &users[0].digest(),
            &CreateTeam {
                id: "reused-handle".into(),
                name: "Replacement".into(),
                handle: "shared".into(),
            },
            now()?,
        )
        .await?;
    assert!(
        store
            .agents(&users[0].digest(), "reused-handle", None, 20, now()?)
            .await?
            .is_empty()
    );
    clean(&store, provider, "owner").await?;
    assert_eq!(
        store
            .team(&users[0].digest(), "reused-handle", now()?)
            .await?
            .handle,
        "shared"
    );
    assert_eq!(
        store
            .agent(&users[0].digest(), "agent-personal", now()?)
            .await?
            .team_id,
        "personal-0"
    );
    concurrent_team_deletion(&store, &users[0], provider).await?;
    Ok(())
}

async fn concurrent_team_deletion(
    store: &Store,
    user: &BearerSecret,
    provider: &BearerSecret,
) -> Result<(), TestError> {
    store
        .create_team(
            &user.digest(),
            &CreateTeam {
                id: "race".into(),
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
    let create = tokio::spawn(async move {
        started.wait().await;
        creation_store
            .create_cloud_agent(
                &creation_actor,
                &draft("race", "race"),
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
                "race",
                now().map_err(|_| StoreError::Integrity)?,
            )
            .await
    });
    let (created, deleted) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        (create.await, delete.await)
    })
    .await?;
    deleted??;
    match created? {
        Ok(CreationOutcome::Present(_)) => clean(store, provider, "race").await?,
        Err(StoreError::NotFound) => {}
        other => return Err(format!("Unexpected concurrent outcome: {other:?}").into()),
    }
    assert!(matches!(
        store.team(&user.digest(), "race", now()?).await,
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
    prepare_sqlite(&path, &cipher)?;
    let users = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&seed(&cipher, &users, &provider, false)?)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_sqlite(path, cipher).await?,
            &users,
            &provider,
        )
        .await
    })?;
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM teams WHERE id='shared'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM agent_creation_receipts WHERE team_id='shared'"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        3
    );
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE id IN ('leaving','kicked','owner') AND desired_state='DELETING' AND cleanup_completed_at IS NOT NULL").get_result::<Count>(&mut connection)?.n,3);
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
    result?;
    cleanup?;
    Ok(())
}
fn postgres_in_schema(url: &str) -> Result<(), TestError> {
    let cipher = cipher()?;
    prepare_postgres(url, &cipher)?;
    let users = (0..3)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let provider = BearerSecret::generate()?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(url)?;
    connection.batch_execute(&seed(&cipher, &users, &provider, true)?)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_postgres(url.into(), cipher).await?,
            &users,
            &provider,
        )
        .await
    })?;
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM teams WHERE id='shared'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query(
            "SELECT count(*) AS n FROM agent_creation_receipts WHERE team_id='shared'"
        )
        .get_result::<Count>(&mut connection)?
        .n,
        3
    );
    assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM computers WHERE id IN ('leaving','kicked','owner') AND desired_state='DELETING' AND cleanup_completed_at IS NOT NULL").get_result::<Count>(&mut connection)?.n,3);
    Ok(())
}
