use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, CredentialCipher, EmailCode, PasswordCost, PasswordHasher, RootKey,
};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{
    EmailLoginOutcome, GrantDraft, GrantKind, NewAccount, NewChallenge, NewSession, SessionPolicy,
    Store, StoreError, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{error::Error, sync::Arc};

type TestError = Box<dyn Error + Send + Sync>;
const NOW: i64 = 1_000_000;

fn account(id: &str, email: &str, password: &str) -> Result<NewAccount, TestError> {
    Ok(NewAccount {
        user_id: id.to_owned(),
        email: EmailAddress::parse(email)?,
        password: PasswordHasher::new(PasswordCost::new(10)?)
            .hash(&Password::parse(password.to_owned())?)?,
        personal_team_id: format!("personal-{id}"),
        personal_handle: format!("personal-{id}"),
        session: new_session(&format!("session-{id}"))?,
    })
}

fn new_session(id: &str) -> Result<NewSession, TestError> {
    Ok(NewSession {
        id: id.to_owned(),
        token: BearerSecret::generate()?,
    })
}

async fn exercise(store: Store) -> Result<(), TestError> {
    let policy = SessionPolicy::default();
    let setup = BearerSecret::generate()?;
    let admin_draft = account("admin", "Admin@Example.Invalid", "Aa1!admin")?;
    assert!(store.setup_available().await?);
    assert!(matches!(
        store
            .initialize_admin(
                &setup.digest(),
                &BearerSecret::generate()?,
                &admin_draft,
                NOW,
                policy
            )
            .await,
        Err(StoreError::InvalidProof)
    ));
    let admin = store
        .initialize_admin(&setup.digest(), &setup, &admin_draft, NOW, policy)
        .await?;
    assert!(admin.user.is_admin);
    assert!(!store.setup_available().await?);
    assert!(
        store
            .initialize_admin(&setup.digest(), &setup, &admin_draft, NOW, policy)
            .await
            .is_err()
    );
    let proof = BearerSecret::generate()?;
    let email = EmailAddress::parse(" Member@Example.Invalid ")?;
    store
        .issue_signup_grant(
            &admin.token.digest(),
            &email,
            &GrantDraft {
                id: "grant-signup".to_owned(),
                proof: proof.digest(),
                expires_at: NOW + 86_400_000,
            },
            NOW,
        )
        .await?;
    assert!(
        store
            .preview_grant(&proof.digest(), GrantKind::Signup, NOW)
            .await?
            .is_some()
    );
    let member_draft = account("member", "member@example.invalid", "Aa1!member")?;
    let member = store
        .redeem_signup(&proof.digest(), &member_draft, NOW, policy)
        .await?;
    assert!(!member.user.is_admin);
    assert_eq!(store.teams(&member.token.digest(), NOW).await?.len(), 1);
    assert!(
        store
            .preview_grant(&proof.digest(), GrantKind::Signup, NOW)
            .await?
            .is_none()
    );
    assert!(
        store
            .redeem_signup(&proof.digest(), &member_draft, NOW, policy)
            .await
            .is_err()
    );
    assert!(matches!(
        store
            .issue_signup_grant(
                &member.token.digest(),
                &email,
                &GrantDraft {
                    id: "forbidden".to_owned(),
                    proof: BearerSecret::generate()?.digest(),
                    expires_at: NOW + 1000
                },
                NOW
            )
            .await,
        Err(StoreError::Forbidden)
    ));

    // Configured-mail issuance has no administrator issuer. Failure in session
    // insertion must roll back the user, personal context and grant consumption.
    let mail_proof = BearerSecret::generate()?;
    let mut mail_account = account("mail-user", "mail@example.invalid", "Cc3!mail")?;
    store
        .issue_email_signup_grant(
            &mail_account.email,
            &GrantDraft {
                id: "mail-grant".to_owned(),
                proof: mail_proof.digest(),
                expires_at: NOW + 86_400_000,
            },
            NOW,
        )
        .await?;
    mail_account.session.id = admin_draft.session.id.clone();
    assert!(matches!(
        store
            .redeem_signup(&mail_proof.digest(), &mail_account, NOW, policy)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .password_snapshot(&mail_account.email)
            .await?
            .is_none()
    );
    assert!(
        store
            .preview_grant(&mail_proof.digest(), GrantKind::Signup, NOW)
            .await?
            .is_some()
    );
    mail_account.session.id = "mail-user-session".to_owned();
    let mail_user = store
        .redeem_signup(&mail_proof.digest(), &mail_account, NOW, policy)
        .await?;
    assert_eq!(store.teams(&mail_user.token.digest(), NOW).await?.len(), 1);

    let race_proof = BearerSecret::generate()?;
    let race_email = EmailAddress::parse("signup-race@example.invalid")?;
    store
        .issue_signup_grant(
            &admin.token.digest(),
            &race_email,
            &GrantDraft {
                id: "race-grant".to_owned(),
                proof: race_proof.digest(),
                expires_at: NOW + 10000,
            },
            NOW,
        )
        .await?;
    let first_account = account("signup-race-a", race_email.delivery(), "Dd4!race")?;
    let second_account = account("signup-race-b", race_email.delivery(), "Dd4!race")?;
    let barrier = tokio::sync::Barrier::new(2);
    let first = async {
        barrier.wait().await;
        store
            .redeem_signup(&race_proof.digest(), &first_account, NOW, policy)
            .await
    };
    let second = async {
        barrier.wait().await;
        store
            .redeem_signup(&race_proof.digest(), &second_account, NOW, policy)
            .await
    };
    let (first, second) = tokio::join!(first, second);
    assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
    let winning_id = store
        .password_snapshot(&race_email)
        .await?
        .ok_or("Missing signup race winner")?
        .user
        .id;
    assert!(winning_id == "signup-race-a" || winning_id == "signup-race-b");

    // Race the same initial token: every caller must receive one committed value.
    let rotation_at = NOW + policy.rotation_ms;
    let barrier = Arc::new(tokio::sync::Barrier::new(4));
    let mut calls = tokio::task::JoinSet::new();
    for _ in 0..4 {
        let store = store.clone();
        let token = BearerSecret::parse(member.token.expose().to_owned())?;
        let barrier = barrier.clone();
        calls.spawn(async move {
            barrier.wait().await;
            store.renew_session(&token, rotation_at, policy).await
        });
    }
    let mut latest = None;
    let mut expiry = None;
    while let Some(result) = calls.join_next().await {
        let result = result??;
        if let Some(token) = &latest {
            assert_eq!(result.token.expose(), token);
        } else {
            latest = Some(result.token.expose().to_owned());
        }
        if let Some(expires) = expiry {
            assert_eq!(result.expires_at, expires);
        } else {
            expiry = Some(result.expires_at);
        }
    }
    let latest = BearerSecret::parse(latest.ok_or("Missing renewal result")?)?;
    assert_ne!(latest.expose(), member.token.expose());
    assert!(matches!(
        store
            .renew_session(&member.token, rotation_at + policy.overlap_ms, policy)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    let unchanged = store
        .renew_session(&latest, rotation_at + 1, policy)
        .await?;
    assert_eq!(Some(unchanged.expires_at), expiry);

    let reset = BearerSecret::generate()?;
    store
        .issue_password_reset(
            &admin.token.digest(),
            &email,
            &GrantDraft {
                id: "reset-a".to_owned(),
                proof: reset.digest(),
                expires_at: rotation_at + 86_400_000,
            },
            rotation_at,
        )
        .await?;
    let old = store
        .password_snapshot(&email)
        .await?
        .ok_or("Missing credential")?
        .verify("Aa1!member", &PasswordHasher::default())?;
    let reset_hash = PasswordHasher::new(PasswordCost::new(10)?)
        .hash(&Password::parse("Bb2!changed".to_owned())?)?;
    let racing_session = new_session("racing-login")?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let first = async {
        barrier.wait().await;
        store
            .redeem_password_reset(&reset.digest(), &reset_hash, rotation_at + 2)
            .await
    };
    let second = async {
        barrier.wait().await;
        store
            .accept_password_login(&old, &racing_session, rotation_at + 2, policy)
            .await
    };
    let (reset_result, login_result) = tokio::join!(first, second);
    reset_result?;
    if let Ok(session) = login_result {
        assert!(matches!(
            store
                .current_identity(&session.token.digest(), rotation_at + 3)
                .await,
            Err(StoreError::Unauthenticated)
        ));
    }
    assert!(matches!(
        store
            .current_identity(&latest.digest(), rotation_at + 3)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(
        store
            .redeem_password_reset(&reset.digest(), &reset_hash, rotation_at + 3)
            .await
            .is_err()
    );
    let snapshot = store
        .password_snapshot(&email)
        .await?
        .ok_or("Missing credential")?;
    assert!(!PasswordHasher::default().verify("Aa1!member", &snapshot.password)?);
    let verified = snapshot.verify("Bb2!changed", &PasswordHasher::default())?;
    let fresh = store
        .accept_password_login(
            &verified,
            &new_session("fresh-login")?,
            rotation_at + 3,
            policy,
        )
        .await?;
    assert_eq!(fresh.user.id, "member");
    assert_eq!(
        store
            .teams(&fresh.token.digest(), rotation_at + 3)
            .await?
            .len(),
        1
    );
    store
        .logout_session(&fresh.token.digest(), rotation_at + 4)
        .await?;
    assert!(matches!(
        store
            .current_identity(&fresh.token.digest(), rotation_at + 5)
            .await,
        Err(StoreError::Unauthenticated)
    ));

    // Email verification is single-use and never unrestricted signup.
    for (id, target, registered) in [
        ("known-code", "member@example.invalid", true),
        ("unknown-code", "unknown@example.invalid", false),
    ] {
        let draft = NewChallenge {
            id: id.to_owned(),
            email: EmailAddress::parse(target)?,
            binding: BearerSecret::generate()?,
            code: EmailCode::parse("Ab12cd".to_owned())?,
            expires_at: rotation_at + 600_000,
        };
        store.create_login_challenge(&draft, rotation_at).await?;
        assert!(matches!(
            store
                .verify_login_challenge(
                    &draft.email,
                    &draft.binding,
                    &EmailCode::parse("WRONG1".to_owned())?,
                    &new_session(&format!("{id}-wrong"))?,
                    rotation_at + 1,
                    policy
                )
                .await,
            Err(StoreError::InvalidProof)
        ));
        let outcome = store
            .verify_login_challenge(
                &draft.email,
                &draft.binding,
                &EmailCode::parse("aB12cD".to_owned())?,
                &new_session(&format!("{id}-correct"))?,
                rotation_at + 2,
                policy,
            )
            .await?;
        match outcome {
            EmailLoginOutcome::Authenticated(value) => {
                assert!(registered);
                assert_eq!(value.user.id, "member");
            }
            EmailLoginOutcome::RegistrationRequired => assert!(!registered),
        }
        assert!(
            store
                .verify_login_challenge(
                    &draft.email,
                    &draft.binding,
                    &draft.code,
                    &new_session(&format!("{id}-repeat"))?,
                    rotation_at + 3,
                    policy
                )
                .await
                .is_err()
        );
    }
    let promoted = store.grant_administrator(&email).await?;
    assert!(promoted.is_admin);
    assert_eq!(promoted.id, "member");
    assert!(
        store
            .grant_administrator(&EmailAddress::parse("absent@example.invalid")?)
            .await
            .is_err()
    );
    Ok(())
}

#[test]
fn authentication_on_sqlite() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("auth.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?
        .block_on(async { exercise(Store::connect_sqlite(path, cipher).await?).await })
}

#[test]
fn authentication_on_postgres() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("authentication_test_{suffix}");
    connection.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([1; 32]),
        )?)?);
        prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?
            .block_on(async { exercise(Store::connect_postgres(url, cipher).await?).await })
    })();
    let cleanup = connection.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}
