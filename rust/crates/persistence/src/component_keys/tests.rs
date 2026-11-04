use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BootstrapProviderPermit, RootKey};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{
    AsyncPgConnection, RunQueryDsl, async_connection_wrapper::AsyncConnectionWrapper,
};
use std::sync::Arc;

type TestError = Box<dyn std::error::Error + Send + Sync>;
const NOW: i64 = 1_000_000;
const REGISTRATION: &str = "installed-provider";

fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([53; 32]),
    )?)?))
}
fn draft(nonce: &str) -> Result<ComponentKeyDraft, TestError> {
    Ok(ComponentKeyDraft {
        request_id: ProviderKeyRequestId::new(NOW, nonce)?.as_str().to_owned(),
        token: BearerSecret::generate()?,
    })
}
fn authority<'a>(
    proof: &'a VerifiedSystemOwner,
    name: &'a str,
    substrate: &'a ProviderSubstrate,
) -> ProviderOwner<'a> {
    ProviderOwner::Bootstrap {
        proof,
        name,
        substrate,
        capabilities: ProviderCapabilities {
            create: true,
            start: true,
            stop: true,
            restart: true,
        },
    }
}

async fn exercise(store: Store) -> Result<(), TestError> {
    let proof = BootstrapProviderPermit::new(REGISTRATION.to_owned())?.authenticated();
    let substrate = ProviderSubstrate::Kubernetes;
    let policy = ComponentKeyPolicy {
        validity_ms: 1000,
        proactive_after_ms: 500,
        overlap_ms: 20,
    };
    let first_draft = draft("first")?;
    let first = store
        .rotate_provider_key(
            authority(&proof, "Fixture", &substrate),
            REGISTRATION,
            &first_draft,
            policy,
            NOW,
        )
        .await?;
    let assigned = EntityId::from_dto(EntityKind::Provider, &first.provider_id)?;
    assert_ne!(first.provider_id, REGISTRATION);
    assert_eq!(first.key.expires_at, NOW + 1000);
    assert_eq!(first.key.token.expose(), first_draft.token.expose());
    let retry = store
        .rotate_provider_key(
            authority(&proof, "Fixture", &substrate),
            REGISTRATION,
            &ComponentKeyDraft {
                request_id: first_draft.request_id.clone(),
                token: BearerSecret::generate()?,
            },
            policy,
            NOW + 1,
        )
        .await?;
    assert_eq!(retry.provider_id, first.provider_id);
    assert_eq!(retry.key.token.expose(), first.key.token.expose());
    assert!(matches!(
        store
            .rotate_provider_key(
                authority(&proof, "Changed", &substrate),
                REGISTRATION,
                &first_draft,
                policy,
                NOW + 1
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .rotate_provider_key(
                authority(&proof, "Fixture", &substrate),
                &first.provider_id,
                &draft("wrong-target")?,
                policy,
                NOW + 1
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    assert_eq!(
        store
            .provider_key_identity(&first.provider_id, &first.key.token.digest(), NOW + 1)
            .await?
            .id,
        first.provider_id
    );
    assert!(matches!(
        store
            .provider_key_identity(REGISTRATION, &first.key.token.digest(), NOW + 1)
            .await,
        Err(StoreError::InvalidInput)
    ));

    let row = operation!(store, |connection, dialect| {
        let ids = diesel::sql_query(
            dialect.query("SELECT id FROM computer_providers WHERE registration_key=?"),
        )
        .bind::<Text, _>(REGISTRATION)
        .load::<OwnerIdRow>(connection)
        .await?;
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].id, assigned);
        let row =
            diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                .bind::<IdentitySql, _>(assigned)
                .get_result::<ProviderKeyRow>(connection)
                .await?;
        Ok(row)
    })?;
    let recovered = recover_key(
        &store.cipher,
        &format!("computer-provider:{}", first.provider_id),
        &row.current_hash,
        row.recovery_nonce.as_deref(),
        row.recovery_ciphertext.as_deref(),
    )?;
    assert_eq!(recovered.expose(), first.key.token.expose());
    assert!(
        recover_key(
            &store.cipher,
            &format!("computer-provider:{REGISTRATION}"),
            &row.current_hash,
            row.recovery_nonce.as_deref(),
            row.recovery_ciphertext.as_deref()
        )
        .is_err()
    );

    // A configured key is a target, never an adoption grant for another owner.
    operation!(store, |connection, dialect| {
        diesel::sql_query(dialect.query(
            "UPDATE computer_providers SET system_owner_id='another-installation' WHERE id=?",
        ))
        .bind::<IdentitySql, _>(assigned)
        .execute(connection)
        .await?;
        Ok(())
    })?;
    assert!(matches!(
        store
            .rotate_provider_key(
                authority(&proof, "Fixture", &substrate),
                REGISTRATION,
                &draft("wrong-owner")?,
                policy,
                NOW + 2
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    operation!(store, |connection, dialect| {
        diesel::sql_query(
            dialect
                .query("UPDATE computer_providers SET system_owner_id='installation' WHERE id=?"),
        )
        .bind::<IdentitySql, _>(assigned)
        .execute(connection)
        .await?;
        Ok(())
    })?;
    let second = store
        .rotate_provider_key(
            authority(&proof, "Fixture", &substrate),
            REGISTRATION,
            &draft("second")?,
            policy,
            NOW + 2,
        )
        .await?;
    assert_eq!(second.provider_id, first.provider_id);
    assert_ne!(second.key.token.expose(), first.key.token.expose());
    store
        .provider_key_identity(&first.provider_id, &first.key.token.digest(), NOW + 3)
        .await?;
    assert!(matches!(
        store
            .provider_key_identity(&first.provider_id, &first.key.token.digest(), NOW + 22)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let revoke = ProviderKeyRequestId::new(NOW, "revoke")?;
    store
        .revoke_provider_key(
            ProviderOwner::System(&proof),
            REGISTRATION,
            revoke.as_str(),
            NOW + 23,
        )
        .await?;
    assert!(matches!(
        store
            .provider_key_identity(&first.provider_id, &second.key.token.digest(), NOW + 24)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    exercise_registration(&store).await?;
    exercise_concurrent_bootstrap(&store).await?;
    exercise_directory(&store).await?;
    Ok(())
}

async fn exercise_registration(store: &Store) -> Result<(), TestError> {
    const TARGET: &str = "registered-provider";
    let proof = BootstrapProviderPermit::new(TARGET.to_owned())?.authenticated();
    let substrate = ProviderSubstrate::Kubernetes;
    let capabilities = ProviderCapabilities {
        create: true,
        start: true,
        stop: false,
        restart: false,
    };
    store
        .configure_system_provider(&proof, "Registered", capabilities, &substrate, NOW)
        .await?;
    // Reconfiguration resolves the registration key without issuing a second row.
    store
        .configure_system_provider(&proof, "Renamed", capabilities, &substrate, NOW + 1)
        .await?;
    let token = BearerSecret::generate()?;
    let digest = token.digest();
    let canonical = operation!(store, |connection, dialect| {
        let row = diesel::sql_query(
            dialect.query("SELECT * FROM computer_providers WHERE registration_key=?"),
        )
        .bind::<Text, _>(TARGET)
        .get_result::<ProviderRow>(connection)
        .await?;
        assert_eq!(row.name, "Renamed");
        let canonical = row.id.to_dto(EntityKind::Provider);
        let context = provider_registration_context(
            &canonical,
            ProviderRegistrationOwner::System(proof.owner_id()),
        )?;
        let envelope = store.cipher.protect_latest(&context, &token)?;
        diesel::sql_query(dialect.query("INSERT INTO provider_registration_credentials(provider_id,credential_hash,recovery_nonce,recovery_ciphertext) VALUES(?,?,?,?)"))
            .bind::<IdentitySql,_>(row.id).bind::<Binary,_>(digest.as_bytes().as_slice())
            .bind::<Binary,_>(envelope.nonce().as_slice()).bind::<Binary,_>(envelope.ciphertext())
            .execute(connection).await?;
        Ok(canonical)
    })?;
    let key_draft = draft("registration-first")?;
    let policy = ComponentKeyPolicy::default();
    let accepted = store
        .rotate_provider_key(
            ProviderOwner::Registration {
                credential: &digest,
                substrate: &substrate,
            },
            TARGET,
            &key_draft,
            policy,
            NOW + 2,
        )
        .await?;
    assert_eq!(accepted.provider_id, canonical);
    let recovered = store
        .rotate_provider_key(
            ProviderOwner::Registration {
                credential: &digest,
                substrate: &substrate,
            },
            TARGET,
            &key_draft,
            policy,
            NOW + 3,
        )
        .await?;
    assert_eq!(recovered.key.token.expose(), accepted.key.token.expose());
    let wrong_digest = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::Registration {
                    credential: &wrong_digest,
                    substrate: &substrate
                },
                TARGET,
                &draft("registration-denied")?,
                policy,
                NOW + 4,
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::Registration {
                    credential: &digest,
                    substrate: &substrate
                },
                &canonical,
                &key_draft,
                policy,
                NOW + 4,
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    // Registration recovery is authenticated on every retry, not only issuance.
    operation!(store, |connection, dialect| {
        diesel::sql_query(dialect.query(
            "UPDATE provider_registration_credentials SET recovery_ciphertext=? WHERE provider_id=?",
        ))
        .bind::<Binary,_>(vec![0; 59])
        .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Provider, &canonical)?)
        .execute(connection).await?;
        Ok(())
    })?;
    assert!(matches!(
        store
            .rotate_provider_key(
                ProviderOwner::Registration {
                    credential: &digest,
                    substrate: &substrate
                },
                TARGET,
                &key_draft,
                policy,
                NOW + 5,
            )
            .await,
        Err(StoreError::Crypto(_))
    ));
    Ok(())
}

async fn exercise_concurrent_bootstrap(store: &Store) -> Result<(), TestError> {
    const TARGET: &str = "racing-registration";
    let proof = BootstrapProviderPermit::new(TARGET.to_owned())?.authenticated();
    let substrate = ProviderSubstrate::Kubernetes;
    let left = draft("same-concurrent-request")?;
    let right = draft("same-concurrent-request")?;
    let (left, right) = tokio::join!(
        store.rotate_provider_key(
            authority(&proof, "Concurrent", &substrate),
            TARGET,
            &left,
            ComponentKeyPolicy::default(),
            NOW,
        ),
        store.rotate_provider_key(
            authority(&proof, "Concurrent", &substrate),
            TARGET,
            &right,
            ComponentKeyPolicy::default(),
            NOW,
        ),
    );
    let (left, right) = (left?, right?);
    assert_eq!(left.provider_id, right.provider_id);
    assert_eq!(left.key.token.expose(), right.key.token.expose());
    Ok(())
}

async fn exercise_directory(store: &Store) -> Result<(), TestError> {
    use crate::directory::{DirectoryPolicy, database_now};
    const TARGET: &str = "directory-provider";
    let proof = BootstrapProviderPermit::new(TARGET.to_owned())?.authenticated();
    let substrate = ProviderSubstrate::Kubernetes;
    let now = operation!(store, |connection, dialect| {
        Ok(database_now!(connection, dialect))
    })?;
    let issued = store
        .rotate_provider_key(
            authority(&proof, "Directory", &substrate),
            TARGET,
            &ComponentKeyDraft {
                request_id: ProviderKeyRequestId::new(now, "directory")?.as_str().into(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now,
        )
        .await?;
    let provider = issued.provider_id.as_str();
    let digest = issued.key.token.digest();
    let policy = DirectoryPolicy::default();
    let first = store.register_control("http://127.0.0.1:1", policy).await?;
    let second = store.register_control("http://127.0.0.1:1", policy).await?;
    EntityId::from_dto(EntityKind::ControlInstance, &first.id)?;
    assert_ne!(first.id, second.id);
    store.renew_control(&first.id, policy).await?;
    let left = store
        .prepare_provider_connection(provider, &digest, &first.id)
        .await?;
    let right = store
        .prepare_provider_connection(provider, &digest, &second.id)
        .await?;
    assert_eq!(left.expected, None);
    assert_eq!(right.expected, None);
    EntityId::from_dto(EntityKind::Connection, &left.owner.connection_id)?;
    assert_ne!(left.owner.connection_id, right.owner.connection_id);
    let accepted = store
        .admit_provider_connection(
            provider,
            &digest,
            left.expected.as_ref(),
            &left.owner,
            policy,
        )
        .await?;
    assert_eq!(accepted.owner(), left.owner);
    assert!(matches!(
        store
            .admit_provider_connection(
                provider,
                &digest,
                right.expected.as_ref(),
                &right.owner,
                policy,
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let resolved = store
        .resolve_provider_route(provider)
        .await?
        .ok_or("No live route")?;
    assert_eq!(resolved.component.owner(), left.owner);
    assert_eq!(resolved.control.id, first.id);
    store
        .renew_provider_connection(provider, &digest, &left.owner, policy)
        .await?;
    exercise_connector(store, provider, &digest, now, &first.id).await?;
    let replacement = store
        .prepare_provider_connection(provider, &digest, &second.id)
        .await?;
    assert_eq!(replacement.expected, Some(left.owner.clone()));
    store
        .admit_provider_connection(
            provider,
            &digest,
            replacement.expected.as_ref(),
            &replacement.owner,
            policy,
        )
        .await?;
    assert!(
        !store
            .close_provider_connection(provider, &digest, &left.owner)
            .await?
    );
    store.drain_control(&first.id).await?;
    assert_eq!(
        store
            .withdraw_drained_control_connections(&first.id)
            .await?,
        0
    );
    assert_eq!(
        store
            .resolve_provider_route(provider)
            .await?
            .ok_or("Replacement missing")?
            .component
            .owner(),
        replacement.owner
    );
    store.drain_control(&second.id).await?;
    assert!(store.resolve_provider_route(provider).await?.is_none());
    assert!(matches!(
        store
            .prepare_provider_connection(provider, &digest, &second.id)
            .await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store.renew_control(&second.id, policy).await,
        Err(StoreError::InvalidTransition)
    ));
    assert_eq!(
        store
            .withdraw_drained_control_connections(&second.id)
            .await?,
        1
    );
    assert_eq!(
        store
            .withdraw_drained_control_connections(&second.id)
            .await?,
        0
    );
    Ok(())
}

async fn exercise_connector(
    store: &Store,
    provider: &str,
    credential: &SecretDigest,
    now: i64,
    control: &str,
) -> Result<(), TestError> {
    use crate::{ConnectorSessionDraft, DirectoryPolicy};
    let computer = EntityId::generate(now)?;
    let execution = EntityId::generate(now)?;
    let user = EntityId::generate(now)?;
    operation!(store, |connection, dialect| {
        diesel::sql_query(dialect.query("INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at) VALUES(?,'connector@example.invalid','connector@example.invalid','fixture',FALSE,?)"))
            .bind::<IdentitySql,_>(user).bind::<BigInt,_>(now).execute(connection).await?;
        diesel::sql_query(dialect.query("INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES(?,?,'CLOUD',?,'RUNNING','create',?,'CREATE','STARTING',?)"))
            .bind::<IdentitySql,_>(computer).bind::<IdentitySql,_>(user)
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Provider, provider)?)
            .bind::<Binary,_>([7_u8;32].as_slice()).bind::<BigInt,_>(now)
            .execute(connection).await?;
        diesel::sql_query(dialect.query("INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at) VALUES(?,?,TRUE,?)"))
            .bind::<IdentitySql,_>(execution).bind::<IdentitySql,_>(computer)
            .bind::<BigInt,_>(now).execute(connection).await?;
        diesel::sql_query(dialect.query("UPDATE computers SET current_execution_id=? WHERE id=?"))
            .bind::<IdentitySql, _>(execution)
            .bind::<IdentitySql, _>(computer)
            .execute(connection)
            .await?;
        Ok(())
    })?;
    let execution_id = execution.to_dto(EntityKind::Execution);
    let left = ConnectorSessionDraft {
        request_id: "connector-accept-left".into(),
        initial_key: BearerSecret::generate()?,
    };
    let right = ConnectorSessionDraft {
        request_id: "connector-accept-right".into(),
        initial_key: BearerSecret::generate()?,
    };
    let (left, right) = tokio::join!(
        store.issue_connector_session(provider, credential, &execution_id, &left, now),
        store.issue_connector_session(provider, credential, &execution_id, &right, now),
    );
    let (session, retry) = (left?, right?);
    EntityId::from_dto(EntityKind::ConnectorSession, &session.id)?;
    assert_eq!(session.id, retry.id);
    assert_eq!(session.initial_key.expose(), retry.initial_key.expose());
    assert_eq!(session.accepted_request_id, retry.accepted_request_id);
    assert!(matches!(
        store
            .prepare_connector_connection(&session.id, &session.initial_key.digest(), control)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let policy = ComponentKeyPolicy::default();
    let exchange = ComponentKeyDraft {
        request_id: "connector-exchange".into(),
        token: BearerSecret::generate()?,
    };
    let key = store
        .exchange_connector_key(
            &session.id,
            &session.initial_key.digest(),
            &exchange,
            policy,
            now,
        )
        .await?;
    let retry = store
        .exchange_connector_key(
            &session.id,
            &session.initial_key.digest(),
            &exchange,
            policy,
            now + 1,
        )
        .await?;
    assert_eq!(retry.token.expose(), key.token.expose());
    let identity = store
        .connector_key_identity(&session.id, &key.token.digest(), now + 1)
        .await?;
    assert_eq!(identity.execution_id, execution_id);
    assert_eq!(identity.computer_id, computer.to_dto(EntityKind::Computer));
    let connection = store
        .prepare_connector_connection(&session.id, &key.token.digest(), control)
        .await?;
    assert_eq!(connection.expected, None);
    let directory = DirectoryPolicy::default();
    store
        .admit_connector_connection(
            &session.id,
            &key.token.digest(),
            connection.expected.as_ref(),
            &connection.owner,
            directory,
        )
        .await?;
    let source = crate::ConnectorSource {
        session_id: &session.id,
        control_id: &connection.owner.control_id,
        connection_id: &connection.owner.connection_id,
        credential: &key.token.digest(),
    };
    store.verify_connector_connection(&source).await?;
    let route = store
        .resolve_connector_route(&identity.computer_id)
        .await?
        .ok_or("Connector route missing")?;
    assert_eq!(route.component.owner(), connection.owner);
    store
        .renew_connector_connection(
            &session.id,
            &key.token.digest(),
            &connection.owner,
            directory,
        )
        .await?;
    let replacement = store
        .prepare_connector_connection(&session.id, &key.token.digest(), control)
        .await?;
    assert_eq!(replacement.expected, Some(connection.owner.clone()));
    store
        .admit_connector_connection(
            &session.id,
            &key.token.digest(),
            replacement.expected.as_ref(),
            &replacement.owner,
            directory,
        )
        .await?;
    assert!(matches!(
        store.verify_connector_connection(&source).await,
        Err(StoreError::Conflict)
    ));
    assert!(
        !store
            .close_connector_connection(&session.id, &key.token.digest(), &connection.owner)
            .await?
    );
    assert!(
        store
            .close_connector_connection(&session.id, &key.token.digest(), &replacement.owner)
            .await?
    );
    let refresh = ComponentKeyDraft {
        request_id: "connector-refresh".into(),
        token: BearerSecret::generate()?,
    };
    let fresh = store
        .refresh_connector_key(&session.id, &key.token.digest(), &refresh, policy, now + 2)
        .await?;
    assert_ne!(fresh.token.expose(), key.token.expose());
    store
        .connector_key_identity(&session.id, &key.token.digest(), now + 3)
        .await?;
    assert!(matches!(
        store
            .connector_key_identity(&session.id, &key.token.digest(), now + 300_002)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    store
        .revoke_connector_session(
            provider,
            credential,
            &session.id,
            "connector-revoke",
            now + 4,
        )
        .await?;
    assert!(matches!(
        store
            .connector_key_identity(&session.id, &fresh.token.digest(), now + 5)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .issue_connector_session(
                provider,
                credential,
                &execution_id,
                &ConnectorSessionDraft {
                    request_id: "connector-after-revoke".into(),
                    initial_key: BearerSecret::generate()?
                },
                now + 5,
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    Ok(())
}

#[test]
fn sqlite_component_credentials_and_directory_preserve_canonical_identity() -> Result<(), TestError>
{
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("provider.sqlite");
    let root = cipher()?;
    crate::prepare_sqlite(&path, &root)?;
    tokio::runtime::Runtime::new()?
        .block_on(async { exercise(Store::connect_sqlite(path, root).await?).await })
}

#[test]
fn postgres_component_credentials_and_directory_preserve_canonical_identity()
-> Result<(), TestError> {
    let _guard = crate::preparation::POSTGRES_PREPARATION_TEST_LOCK
        .lock()
        .map_err(|_| "PostgreSQL preparation test lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let secret = BearerSecret::generate()?;
    let suffix = secret.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("provider_identity_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let root = cipher()?;
        crate::prepare_postgres(&url, &root)?;
        tokio::runtime::Runtime::new()?
            .block_on(async { exercise(Store::connect_postgres(url, root).await?).await })
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
