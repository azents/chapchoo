#[path = "external_authentication/device.rs"]
mod device;
#[path = "external_authentication/selection.rs"]
mod selection;
mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, CredentialCipher, ExternalSecret, ExternalSecretContext, ExternalSecretEnvelope,
    ExternalSecretKind, ExternalSecretOwner, PasswordCost, PasswordHasher, RootKey,
};
use chapchoo_domain::{EmailAddress, MembershipRole, Password};
use chapchoo_persistence::{
    AuthenticationAcceptedResult, AuthenticationScope, CreateTeam, DeleteAuthentication,
    DeleteAuthenticationOutcome, GrantDraft, InviteMember, NewAccount, NewSession, RegisterApiKey,
    RenameAuthentication, ReplaceApiKey, SessionPolicy, Store, StoreError, prepare_postgres,
    prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::Binary};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{error::Error, sync::Arc};

type TestError = Box<dyn Error + Send + Sync>;
const NOW: i64 = 1_000_000;
// Both schema fixtures share the database-scoped preparation advisory lock.
static POSTGRES_FIXTURE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([63; 32]),
    )?)?))
}
fn key(value: &str) -> Result<ExternalSecret, TestError> {
    Ok(ExternalSecret::new(value.as_bytes().to_vec())?)
}
fn record(result: AuthenticationAcceptedResult) -> Result<String, TestError> {
    match result {
        AuthenticationAcceptedResult::Record { authentication_id } => Ok(authentication_id),
        _ => Err("Expected a record result".into()),
    }
}
async fn seed(store: &Store) -> Result<Vec<BearerSecret>, TestError> {
    seed_at(store, NOW).await
}
async fn seed_at(store: &Store, now: i64) -> Result<Vec<BearerSecret>, TestError> {
    let mut tokens = Vec::new();
    for index in 0..4 {
        let token = BearerSecret::generate()?;
        let proof = BearerSecret::generate()?;
        let draft = NewAccount {
            email: EmailAddress::parse(&format!("auth{index}@example.invalid"))?,
            password: PasswordHasher::new(PasswordCost::new(10)?)
                .hash(&Password::parse("Aa1!fixture".to_owned())?)?,
            session: NewSession {
                token: BearerSecret::parse(token.expose().to_owned())?,
            },
            display: None,
        };
        if index == 0 {
            store
                .initialize_admin(
                    &proof.digest(),
                    &proof,
                    &draft,
                    now,
                    SessionPolicy::default(),
                )
                .await?;
        } else {
            store
                .issue_email_signup_grant(
                    &draft.email,
                    &GrantDraft {
                        proof: proof.digest(),
                        expires_at: now + 10000,
                    },
                    now,
                )
                .await?;
            store
                .redeem_signup(&proof.digest(), &draft, now, SessionPolicy::default())
                .await?;
        }
        tokens.push(token);
    }
    Ok(tokens)
}
async fn exercise(store: &Store) -> Result<(String, String), TestError> {
    let tokens = seed(store).await?;
    let owner = tokens[0].digest();
    let manager = tokens[1].digest();
    let member = tokens[2].digest();
    let stranger = tokens[3].digest();
    let personal = AuthenticationScope::Personal;
    let team = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Auth Team".to_owned(),
                handle: "auth-team".to_owned(),
            },
            NOW,
        )
        .await?;
    for (index, token) in tokens.iter().enumerate().take(3).skip(1) {
        let invitation = store
            .invite_member(
                &owner,
                &InviteMember {
                    team_id: team.id.clone(),
                    recipient: EmailAddress::parse(&format!("auth{index}@example.invalid"))?,
                    signup: None,
                },
                NOW,
            )
            .await?;
        store
            .process_invitation(&token.digest(), &invitation.invitation.id, true, NOW)
            .await?;
    }
    let manager_user = store.current_identity(&manager, NOW).await?.id;
    store
        .set_member_role(
            &owner,
            &team.id,
            &manager_user,
            MembershipRole::Manager,
            NOW,
        )
        .await?;
    let shared = AuthenticationScope::Team {
        team_id: team.id.clone(),
    };
    let original = RegisterApiKey {
        scope: personal.clone(),
        request_id: "personal-register".to_owned(),
        name: String::new(),
        key: key("synthetic-personal-key")?,
    };
    let whitespace = RegisterApiKey {
        scope: personal.clone(),
        request_id: "whitespace-register".to_owned(),
        name: "   ".to_owned(),
        key: key("synthetic-personal-key")?,
    };
    assert!(matches!(
        store
            .register_external_api_key(&owner, &whitespace, NOW)
            .await,
        Err(StoreError::InvalidInput)
    ));
    let id = record(
        store
            .register_external_api_key(&owner, &original, NOW)
            .await?,
    )?;
    assert_eq!(
        record(
            store
                .register_external_api_key(&owner, &original, NOW + 1)
                .await?
        )?,
        id
    );
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        1
    );
    assert!(
        store
            .external_authentications(&manager, &personal, None, 100, NOW)
            .await?
            .is_empty()
    );
    assert!(matches!(
        store
            .external_authentication(&manager, &personal, &id, NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .external_authentication_request_result(&manager, "personal-register", NOW)
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .external_authentications(&owner, &personal, None, 101, NOW)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        store
            .external_authentications(&owner, &personal, Some(&id), 100, NOW)
            .await?
            .is_empty()
    );
    let changed = RegisterApiKey {
        key: key("different-synthetic-key")?,
        ..original
    };
    assert!(matches!(
        store.register_external_api_key(&owner, &changed, NOW).await,
        Err(StoreError::Conflict)
    ));
    let detail = store
        .external_authentication(&owner, &personal, &id, NOW)
        .await?;
    assert_eq!(detail.current_request_id, "personal-register");
    assert!(
        detail.account_id.is_none()
            && detail.email.is_none()
            && detail.plan_type.is_none()
            && detail.expires_at.is_none()
    );
    assert_eq!(format!("{detail:?}"), "AuthenticationSummary([REDACTED])");
    let replace = ReplaceApiKey {
        scope: personal.clone(),
        request_id: "replace".to_owned(),
        authentication_id: id.clone(),
        expected_request_id: "personal-register".to_owned(),
        key: key("replacement-synthetic-key")?,
    };
    assert_eq!(
        record(
            store
                .replace_external_api_key(&owner, &replace, NOW)
                .await?
        )?,
        id
    );
    assert_eq!(
        record(
            store
                .replace_external_api_key(&owner, &replace, NOW)
                .await?
        )?,
        id
    );
    let rename = RenameAuthentication {
        scope: personal.clone(),
        request_id: "rename".to_owned(),
        authentication_id: id.clone(),
        expected_request_id: "replace".to_owned(),
        name: String::new(),
    };
    store
        .rename_external_authentication(&owner, &rename, NOW)
        .await?;
    assert_eq!(
        store
            .external_authentication(&owner, &personal, &id, NOW)
            .await?
            .name,
        ""
    );
    // Accepted replacement replay survives a subsequent mutation and cannot restore its old key.
    store
        .replace_external_api_key(&owner, &replace, NOW)
        .await?;
    assert_eq!(
        store
            .external_authentication(&owner, &personal, &id, NOW)
            .await?
            .current_request_id,
        "rename"
    );
    let stale = RenameAuthentication {
        request_id: "stale-rename".to_owned(),
        expected_request_id: "replace".to_owned(),
        ..rename
    };
    assert!(matches!(
        store
            .rename_external_authentication(&owner, &stale, NOW)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    let deletion = DeleteAuthentication {
        scope: personal.clone(),
        request_id: "delete".to_owned(),
        authentication_id: id.clone(),
        expected_request_id: "rename".to_owned(),
    };
    assert!(matches!(
        store
            .delete_external_authentication(&owner, &deletion, NOW)
            .await?,
        DeleteAuthenticationOutcome::Accepted(AuthenticationAcceptedResult::Deleted { .. })
    ));
    assert!(matches!(
        store
            .delete_external_authentication(&owner, &deletion, NOW)
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert!(matches!(
        store
            .external_authentication(&owner, &personal, &id, NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .external_authentication_request_result(&owner, "personal-register", NOW)
            .await?
            .ok_or("Missing original receipt")?
            .accepted,
        AuthenticationAcceptedResult::Record {
            authentication_id: id
        }
    );

    let registration = RegisterApiKey {
        scope: shared.clone(),
        request_id: "team-register".to_owned(),
        name: "Shared key".to_owned(),
        key: key("synthetic-team-key")?,
    };
    assert!(matches!(
        store
            .register_external_api_key(&member, &registration, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .register_external_api_key(&stranger, &registration, NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    let shared_id = record(
        store
            .register_external_api_key(&manager, &registration, NOW)
            .await?,
    )?;
    assert_eq!(
        store
            .external_authentications(&member, &shared, None, 100, NOW)
            .await?
            .len(),
        1
    );
    assert!(matches!(
        store
            .external_authentications(&stranger, &shared, None, 100, NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    // Current role, not original acceptance, authorizes mutation/result recovery.
    store
        .set_member_role(&owner, &team.id, &manager_user, MembershipRole::Member, NOW)
        .await?;
    assert!(matches!(
        store
            .register_external_api_key(&manager, &registration, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .external_authentication_request_result(&manager, "team-register", NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert_eq!(
        store
            .external_authentication(&manager, &shared, &shared_id, NOW)
            .await?
            .id,
        shared_id
    );
    store
        .set_member_role(
            &owner,
            &team.id,
            &manager_user,
            MembershipRole::Manager,
            NOW,
        )
        .await?;
    let shared_delete = DeleteAuthentication {
        scope: shared.clone(),
        request_id: "team-delete".to_owned(),
        authentication_id: shared_id.clone(),
        expected_request_id: "team-register".to_owned(),
    };
    assert!(matches!(
        store
            .delete_external_authentication(&owner, &shared_delete, NOW)
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));
    assert_eq!(
        store
            .external_authentication_request_result(&manager, "team-register", NOW)
            .await?
            .ok_or("Missing shared receipt")?
            .accepted,
        AuthenticationAcceptedResult::Record {
            authentication_id: shared_id
        }
    );
    store
        .set_member_role(&owner, &team.id, &manager_user, MembershipRole::Member, NOW)
        .await?;
    store
        .remove_team_member(&owner, &team.id, &manager_user, NOW)
        .await?;
    assert!(matches!(
        store
            .external_authentication_request_result(&manager, "team-register", NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    // Concurrent exact registration returns one identity and one row on each engine.
    let race = RegisterApiKey {
        scope: personal.clone(),
        request_id: "concurrent-register".to_owned(),
        name: "Concurrent".to_owned(),
        key: key("synthetic-concurrent-key")?,
    };
    let (first, second) = tokio::join!(
        store.register_external_api_key(&owner, &race, NOW),
        store.register_external_api_key(&owner, &race, NOW)
    );
    let concurrent_id = record(first?)?;
    assert_eq!(concurrent_id, record(second?)?);
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        1
    );
    let invalid = RegisterApiKey {
        scope: personal,
        request_id: "invalid-key".to_owned(),
        name: "Invalid".to_owned(),
        key: key("key\nwith-newline")?,
    };
    assert!(matches!(
        store.register_external_api_key(&owner, &invalid, NOW).await,
        Err(StoreError::InvalidInput)
    ));
    assert!(
        store
            .external_authentication_request_result(&owner, "invalid-key", NOW)
            .await?
            .is_none()
    );
    let cleanup = RegisterApiKey {
        scope: shared,
        request_id: "team-cleanup".to_owned(),
        name: "Removed with Team".to_owned(),
        key: key("synthetic-cleanup-key")?,
    };
    store
        .register_external_api_key(&owner, &cleanup, NOW)
        .await?;
    store.delete_team(&owner, &team.id, NOW).await?;
    assert!(matches!(
        store.team(&owner, &team.id, NOW).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .external_authentication_request_result(&owner, "team-cleanup", NOW)
            .await?
            .is_none()
    );
    assert_eq!(
        store
            .external_authentications(&owner, &AuthenticationScope::Personal, None, 100, NOW)
            .await?
            .len(),
        1
    );
    Ok((store.current_identity(&owner, NOW).await?.id, concurrent_id))
}

#[derive(diesel::QueryableByName)]
struct StoredSecret {
    #[diesel(sql_type=Binary)]
    nonce: Vec<u8>,
    #[diesel(sql_type=Binary)]
    ciphertext: Vec<u8>,
}
fn verify_stored_secret(
    cipher: &CredentialCipher,
    row: StoredSecret,
    owner: &str,
    id: &str,
) -> Result<(), TestError> {
    let envelope = ExternalSecretEnvelope::from_parts(
        row.nonce.try_into().map_err(|_| "Invalid nonce")?,
        row.ciphertext,
    )?;
    let context = ExternalSecretContext::new(
        ExternalSecretOwner::User(owner),
        id,
        ExternalSecretKind::OpenAiApiKey,
    )?;
    assert_eq!(
        cipher.recover_external(&context, &envelope)?.expose(),
        b"synthetic-concurrent-key"
    );
    Ok(())
}

#[test]
fn sqlite_catalogue_permissions_mutations_and_historical_recovery() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("external.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let (owner, id) = tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path.clone(), cipher.clone()).await?;
        exercise(&store).await
    })?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    let row = diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='concurrent-register'").get_result::<StoredSecret>(&mut connection)?;
    verify_stored_secret(&cipher, row, &owner, &id)
}
#[test]
fn postgres_catalogue_permissions_mutations_and_historical_recovery() -> Result<(), TestError> {
    let _fixture = POSTGRES_FIXTURE_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("external_catalogue_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let (owner, id) = tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url.clone(), cipher.clone()).await?;
            exercise(&store).await
        })?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        let row = diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='concurrent-register'").get_result::<StoredSecret>(&mut connection)?;
        verify_stored_secret(&cipher, row, &owner, &id)
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
