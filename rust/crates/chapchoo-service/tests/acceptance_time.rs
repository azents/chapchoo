use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, BearerSecret, DomainReason, EmailAddress,
    Password, PasswordCost, PasswordWork, SessionPolicy,
};
use std::{error::Error, sync::Arc};
use zeroize::Zeroizing;

type TestError = Box<dyn Error + Send + Sync>;

#[test]
fn proofs_use_the_final_acceptance_instant_and_login_uses_that_same_instant()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("acceptance.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let bootstrap = BearerSecret::generate()?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let app = AccountsService::new(
            store,
            PasswordWork::new(1, PasswordCost::new(10)?)?,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".to_owned(),
                setup_proof: Some(bootstrap.digest()),
                session_policy: SessionPolicy::default(),
            },
        )?;
        let issued_at = 1_000_000;
        let admin = app
            .initialize(
                &bootstrap,
                EmailAddress::parse("admin@example.invalid")?,
                Password::parse("Aa1!admin".to_owned())?,
                AcceptanceTime::Fixed(issued_at),
            )
            .await?;
        assert_eq!(admin.created_at, issued_at);
        let email = EmailAddress::parse("member@example.invalid")?;
        let expired = app
            .issue_signup_grant(&admin.token.digest(), &email, issued_at)
            .await?;
        let acceptance = expired.grant.expires_at;
        assert_eq!(
            app.redeem_signup(
                &expired.proof,
                email.clone(),
                Password::parse("Aa1!member".to_owned())?,
                AcceptanceTime::Fixed(acceptance)
            )
            .await
            .err()
            .map(|error| error.reason()),
            Some(DomainReason::InvalidProof)
        );
        assert!(
            app.preview_signup(&expired.proof, acceptance)
                .await?
                .is_none()
        );
        let fresh = app
            .issue_signup_grant(&admin.token.digest(), &email, acceptance)
            .await?;
        let member = app
            .redeem_signup(
                &fresh.proof,
                email.clone(),
                Password::parse("Aa1!member".to_owned())?,
                AcceptanceTime::Fixed(acceptance),
            )
            .await?;
        assert_eq!(member.created_at, acceptance);
        let reset = app
            .issue_password_reset(&admin.token.digest(), &email, acceptance)
            .await?;
        let reset_acceptance = reset.grant.expires_at;
        assert_eq!(
            app.redeem_password_reset(
                &reset.proof,
                Password::parse("Bb2!changed".to_owned())?,
                AcceptanceTime::Fixed(reset_acceptance)
            )
            .await
            .err()
            .map(|error| error.reason()),
            Some(DomainReason::InvalidProof)
        );
        let login = app
            .password_login(
                &email,
                Zeroizing::new("Aa1!member".to_owned()),
                AcceptanceTime::Fixed(reset_acceptance),
            )
            .await?;
        assert_eq!(login.created_at, reset_acceptance);
        assert_eq!(login.user.id, member.user.id);
        Ok::<(), TestError>(())
    })
}
