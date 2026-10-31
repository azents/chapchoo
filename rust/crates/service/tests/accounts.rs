use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{CredentialCipher, RootKey};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, BearerSecret, DomainReason, EmailAddress,
    EmailCode, MailFailure, MailFuture, MailMessage, MailPurpose, Mailer, Password, PasswordCost,
    PasswordWork, SessionPolicy,
};
use std::{
    error::Error,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tempfile::TempDir;
use zeroize::Zeroizing;

type TestError = Box<dyn Error + Send + Sync>;
const NOW: i64 = 1_000_000;

#[derive(Default)]
struct CaptureMailer {
    messages: Mutex<Vec<MailMessage>>,
    fail: AtomicBool,
}

impl Mailer for CaptureMailer {
    fn send(&self, message: MailMessage) -> MailFuture<'_> {
        Box::pin(async move {
            self.messages
                .lock()
                .map_err(|_| MailFailure::new(std::io::Error::other("Capture lock failed")))?
                .push(message);
            if self.fail.load(Ordering::SeqCst) {
                Err(MailFailure::new(std::io::Error::other(
                    "Synthetic uncertain provider outcome",
                )))
            } else {
                Ok(())
            }
        })
    }
}

fn prepare() -> Result<
    (
        TempDir,
        std::path::PathBuf,
        Arc<CredentialCipher>,
        BearerSecret,
    ),
    TestError,
> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("service.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    Ok((directory, path, cipher, BearerSecret::generate()?))
}

fn service(
    store: Store,
    proof: &BearerSecret,
    mailer: Option<Arc<dyn Mailer>>,
) -> Result<AccountsService, TestError> {
    Ok(AccountsService::new(
        store,
        PasswordWork::new(2, PasswordCost::new(10)?)?,
        mailer,
        AccountsConfig {
            general_web_url: "http://localhost:3000/product".to_owned(),
            setup_proof: Some(proof.digest()),
            session_policy: SessionPolicy::default(),
        },
    )?)
}

#[test]
fn manual_links_and_password_work_are_composed_without_mail() -> Result<(), TestError> {
    let (_directory, path, cipher, bootstrap) = prepare()?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let app = service(store, &bootstrap, None)?;
        let member_email = EmailAddress::parse("Member@Example.Invalid")?;
        assert!(app.methods().password);
        assert!(!app.methods().email_code);
        assert!(!app.methods().signup_email);
        let admin = app
            .initialize(
                &bootstrap,
                EmailAddress::parse("Admin@Example.Invalid")?,
                Password::parse("Aa1!admin".to_owned())?,
                AcceptanceTime::Fixed(NOW),
            )
            .await?;
        assert_eq!(
            app.request_signup_email(&member_email, NOW)
                .await
                .err()
                .map(|error| error.reason()),
            Some(DomainReason::MailUnavailable)
        );
        assert_eq!(
            app.send_code(&member_email, NOW)
                .await
                .err()
                .map(|error| error.reason()),
            Some(DomainReason::MailUnavailable)
        );
        let issued = app
            .issue_signup_grant(&admin.token.digest(), &member_email, NOW)
            .await?;
        assert_eq!(
            issued.expose_url(),
            format!(
                "http://localhost:3000/product/signup?token={}",
                issued.proof.expose()
            )
        );
        assert_eq!(issued.grant.expires_at, NOW + 7 * 24 * 60 * 60 * 1000);
        assert!(!format!("{issued:?}").contains(issued.proof.expose()));
        let member = app
            .redeem_signup(
                &issued.proof,
                member_email.clone(),
                Password::parse("Aa1!member".to_owned())?,
                AcceptanceTime::Fixed(NOW),
            )
            .await?;
        // This verifies service hashing/account construction/verification wiring,
        // not store membership, proof consumption or session revocation rules.
        let login = app
            .password_login(
                &member_email,
                Zeroizing::new("Aa1!member".to_owned()),
                AcceptanceTime::Fixed(NOW),
            )
            .await?;
        assert_eq!(login.user.id, member.user.id);
        assert_eq!(
            app.password_login(
                &member_email,
                Zeroizing::new("wrong-password".to_owned()),
                AcceptanceTime::Fixed(NOW)
            )
            .await
            .err()
            .map(|error| error.reason()),
            Some(DomainReason::InvalidCredentials)
        );
        let reset = app
            .issue_password_reset(&admin.token.digest(), &member_email, NOW)
            .await?;
        assert_eq!(
            reset.expose_url(),
            format!(
                "http://localhost:3000/product/reset-password?token={}",
                reset.proof.expose()
            )
        );
        assert_eq!(reset.grant.expires_at, NOW + 24 * 60 * 60 * 1000);
        Ok::<(), TestError>(())
    })
}

#[test]
fn delivery_failure_keeps_committed_invitation_and_signup_grant() -> Result<(), TestError> {
    let (_directory, path, cipher, bootstrap) = prepare()?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let capture = Arc::new(CaptureMailer::default());
        let app = service(store, &bootstrap, Some(capture.clone()))?;
        let admin = app
            .initialize(
                &bootstrap,
                EmailAddress::parse("admin@example.invalid")?,
                Password::parse("Aa1!admin".to_owned())?,
                AcceptanceTime::Fixed(NOW),
            )
            .await?;
        let team = app
            .create_team(
                &admin.token.digest(),
                "Team".to_owned(),
                "team".to_owned(),
                NOW,
            )
            .await?;
        capture.fail.store(true, Ordering::SeqCst);
        let failure = app
            .invite_member(
                &admin.token.digest(),
                &team.id,
                EmailAddress::parse("new@example.invalid")?,
                NOW,
            )
            .await
            .err()
            .ok_or("Expected delivery failure")?;
        assert_eq!(failure.reason(), DomainReason::Internal);
        assert!(failure.delivery_uncertain());
        assert!(failure.report().is_some());
        assert_eq!(
            failure
                .report()
                .ok_or("Missing report")?
                .backtrace()
                .status(),
            std::backtrace::BacktraceStatus::Captured
        );
        assert!(failure.source().is_some());
        assert_eq!(
            failure.to_string(),
            "Email delivery could not be confirmed after commit"
        );
        assert_eq!(
            app.team_invitations(&admin.token.digest(), &team.id, NOW)
                .await?
                .len(),
            1
        );
        assert_eq!(
            app.list_signup_grants(&admin.token.digest(), 0, 50, NOW)
                .await?
                .total,
            1
        );
        let messages = capture.messages.lock().map_err(|_| "Capture lock failed")?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].purpose, MailPurpose::TeamInvitation);
        assert!(messages[0].expose_body().contains("/product/signup?token="));
        assert!(!format!("{:?}", messages[0]).contains("token="));
        assert!(!format!("{failure:?}").contains("provider outcome"));
        Ok::<(), TestError>(())
    })
}

#[test]
fn code_delivery_and_unregistered_verification_do_not_bypass_signup() -> Result<(), TestError> {
    let (_directory, path, cipher, bootstrap) = prepare()?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let capture = Arc::new(CaptureMailer::default());
        let app = service(store, &bootstrap, Some(capture.clone()))?;
        let email = EmailAddress::parse("admin@example.invalid")?;
        app.initialize(
            &bootstrap,
            email.clone(),
            Password::parse("Aa1!admin".to_owned())?,
            AcceptanceTime::Fixed(NOW),
        )
        .await?;
        let challenge = app.send_code(&email, NOW).await?;
        let code = {
            let messages = capture.messages.lock().map_err(|_| "Capture lock failed")?;
            let body = messages[0].expose_body();
            let code = body
                .split("code is ")
                .nth(1)
                .and_then(|tail| tail.split('.').next())
                .ok_or("Missing code")?;
            EmailCode::parse(code.to_ascii_lowercase())?
        };
        let session = app
            .verify_code(&email, &challenge.binding, &code, NOW)
            .await?;
        assert!(session.user.is_admin);
        assert_eq!(
            app.verify_code(&email, &challenge.binding, &code, NOW)
                .await
                .err()
                .map(|error| error.reason()),
            Some(DomainReason::InvalidCode)
        );
        let unregistered = EmailAddress::parse("unregistered@example.invalid")?;
        let challenge = app.send_code(&unregistered, NOW).await?;
        let code = {
            let messages = capture.messages.lock().map_err(|_| "Capture lock failed")?;
            let body = messages[1].expose_body();
            let code = body
                .split("code is ")
                .nth(1)
                .and_then(|tail| tail.split('.').next())
                .ok_or("Missing code")?;
            EmailCode::parse(code.to_owned())?
        };
        assert_eq!(
            app.verify_code(&unregistered, &challenge.binding, &code, NOW)
                .await
                .err()
                .map(|error| error.reason()),
            Some(DomainReason::RegistrationRequired)
        );
        assert_eq!(
            app.verify_code(&unregistered, &challenge.binding, &code, NOW)
                .await
                .err()
                .map(|error| error.reason()),
            Some(DomainReason::InvalidCode)
        );
        assert!(
            app.password_login(
                &unregistered,
                Zeroizing::new("Aa1!admin".to_owned()),
                AcceptanceTime::Fixed(NOW)
            )
            .await
            .is_err()
        );
        Ok::<(), TestError>(())
    })
}
