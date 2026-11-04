use super::*;
use crate::{GrantDraft, GrantKind, NewAccount, NewSession, SessionPolicy, Store};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey};
use chapchoo_domain::Password;
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::sync::Arc;

type TestError = Box<dyn std::error::Error + Send + Sync>;
const NOW: i64 = 1000;

fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([23; 32]),
    )?)?))
}
fn account(email: &str) -> Result<NewAccount, TestError> {
    Ok(NewAccount {
        email: EmailAddress::parse(email)?,
        password: PasswordHasher::new(PasswordCost::new(10)?)
            .hash(&Password::parse("Aa1!fixture".to_owned())?)?,
        session: NewSession {
            token: BearerSecret::generate()?,
        },
        display: None,
    })
}

async fn exercise(store: Store) -> Result<String, TestError> {
    let policy = SessionPolicy {
        validity_ms: 1000,
        rotation_ms: 10,
        overlap_ms: 5,
        absolute_lifetime_ms: Some(500),
    };
    let proof = BearerSecret::generate()?;
    let draft = account("admin@example.invalid")?;
    let admin = store
        .initialize_admin(&proof.digest(), &proof, &draft, NOW, policy)
        .await?;
    assert!(admin.user.is_admin);
    let user_id = EntityId::from_dto(EntityKind::User, &admin.user.id)?;
    assert_eq!(user_id.to_dto(EntityKind::User), admin.user.id);
    assert_eq!(
        store.current_identity(&admin.token.digest(), NOW).await?.id,
        admin.user.id
    );
    let catalogue = store
        .agent_profile_images(&admin.token.digest(), None, 100, NOW)
        .await?;
    assert_eq!(catalogue.len(), 6);
    for image in catalogue {
        let id = EntityId::from_dto(EntityKind::ProfileImage, &image.id)?;
        assert_eq!(id.to_dto(EntityKind::ProfileImage), image.id);
    }
    assert!(matches!(
        store
            .agent_profile_images(&BearerSecret::generate()?.digest(), None, 100, NOW)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let member = account("member@example.invalid")?;
    let signup = BearerSecret::generate()?;
    let grant = store
        .issue_signup_grant(
            &admin.token.digest(),
            &member.email,
            &GrantDraft {
                proof: signup.digest(),
                expires_at: NOW + 400,
            },
            NOW,
        )
        .await?;
    assert!(EntityId::from_dto(EntityKind::Grant, &grant.id).is_ok());
    assert!(matches!(
        store
            .revoke_grant(
                &admin.token.digest(),
                &admin.user.id,
                GrantKind::Signup,
                NOW
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    let created = store
        .redeem_signup(&signup.digest(), &member, NOW, policy)
        .await?;
    assert_ne!(created.user.id, admin.user.id);
    assert!(EntityId::from_dto(EntityKind::User, &created.user.id).is_ok());
    // Retained result evidence is recoverable only by its original owner. The
    // read does not need or reinterpret an old creation payload fingerprint.
    let assigned = EntityId::generate(NOW)?;
    crate::store::operation!(store, |connection, dialect| {
        let query = diesel::sql_query(dialect.query(
            "INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id)
             SELECT user_id,team_id,?,?,? FROM personal_contexts WHERE user_id=?",
        ))
        .bind::<Text,_>("accepted-before-cutover")
        .bind::<diesel::sql_types::Binary,_>([1_u8;32].as_slice())
        .bind::<IdentitySql,_>(assigned)
        .bind::<IdentitySql,_>(user_id);
        diesel_async::RunQueryDsl::execute(query, connection).await?;
        Ok(())
    })?;
    assert_eq!(
        store
            .agent_creation_result(&admin.token.digest(), "accepted-before-cutover", NOW)
            .await?,
        Some(crate::AgentCreationResult::Removed {
            assigned_agent_id: assigned.to_dto(EntityKind::Agent)
        }),
    );
    assert!(
        store
            .agent_creation_result(&created.token.digest(), "accepted-before-cutover", NOW)
            .await?
            .is_none()
    );
    Ok(admin.user.id)
}

macro_rules! verify_personal_context {
    ($connection:expr,$id:expr) => {{
        #[derive(diesel::QueryableByName)]
        struct Count { #[diesel(sql_type=diesel::sql_types::BigInt)] n: i64 }
        let count=diesel::sql_query("SELECT count(*) AS n FROM personal_contexts p JOIN teams t ON t.id=p.team_id JOIN users u ON u.id=p.user_id WHERE t.kind='PERSONAL' AND t.handle IS NULL AND t.owner_user_id=u.id")
            .get_result::<Count>($connection)?.n;
        assert_eq!(count,2);
        assert!(EntityId::from_dto(EntityKind::User,$id).is_ok());
    }};
}

#[test]
fn compact_sqlite_accounts_sessions_and_catalogue_are_repository_owned() -> Result<(), TestError> {
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("accounts.sqlite");
    let root = cipher()?;
    crate::prepare_sqlite(&path, &root)?;
    let user_id = tokio::runtime::Runtime::new()?
        .block_on(async { exercise(Store::connect_sqlite(path.clone(), root).await?).await })?;
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    verify_personal_context!(&mut connection, &user_id);
    Ok(())
}

#[test]
fn compact_postgres_accounts_sessions_and_catalogue_are_repository_owned() -> Result<(), TestError>
{
    let _guard = crate::preparation::POSTGRES_PREPARATION_TEST_LOCK
        .lock()
        .map_err(|_| "PostgreSQL preparation test lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let token = BearerSecret::generate()?;
    let suffix = token.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("profile_accounts_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let root = cipher()?;
        crate::prepare_postgres(&url, &root)?;
        let user_id = tokio::runtime::Runtime::new()?.block_on(async {
            exercise(Store::connect_postgres(url.clone(), root).await?).await
        })?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        verify_personal_context!(&mut connection, &user_id);
        Ok(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (Err(primary), Err(cleanup)) => {
            Err(format!("{primary}; fixture cleanup also failed: {cleanup}").into())
        }
    }
}
