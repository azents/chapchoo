use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, CredentialCipher, ExternalSecret, PasswordCost, PasswordHasher, RootKey,
};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{
    AuthenticationAcceptedResult, AuthenticationScope, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, NewAccount, NewSession, SessionPolicy, prepare_sqlite,
};
use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::sync::{Mutex, oneshot};

type TestError = Box<dyn Error + Send + Sync>;
#[derive(Debug)]
struct FixtureFailures(Vec<TestError>);
impl fmt::Display for FixtureFailures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixture retained {} failure(s)", self.0.len())
    }
}
impl Error for FixtureFailures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0
            .first()
            .map(|error| error.as_ref() as &(dyn Error + 'static))
    }
}
struct Fixture {
    _directory: tempfile::TempDir,
    store: Store,
    actor: SecretDigest,
}
async fn fixture() -> Result<Fixture, TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("service.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([31; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let store = Store::connect_sqlite(path, cipher).await?;
    let setup = BearerSecret::generate()?;
    let account = NewAccount {
        email: EmailAddress::parse("authentication@example.invalid")?,
        password: PasswordHasher::new(PasswordCost::new(10)?)
            .hash(&Password::parse("Aa1!fixture".to_owned())?)?,
        session: NewSession {
            token: BearerSecret::generate()?,
        },
        display: None,
    };
    let result = store
        .initialize_admin(
            &setup.digest(),
            &setup,
            &account,
            1000,
            SessionPolicy::default(),
        )
        .await?;
    Ok(Fixture {
        _directory: directory,
        store,
        actor: result.token.digest(),
    })
}
struct Gate {
    entered: parking_lot::Mutex<Option<oneshot::Sender<()>>>,
    release: Mutex<Option<oneshot::Receiver<()>>>,
}
impl Gate {
    async fn wait(&self) -> Result<(), DeviceProviderFailure> {
        self.entered
            .lock()
            .take()
            .ok_or(DeviceProviderFailure::InvalidResponse)?
            .send(())
            .map_err(|_| DeviceProviderFailure::Uncertain)?;
        self.release
            .lock()
            .await
            .take()
            .ok_or(DeviceProviderFailure::InvalidResponse)?
            .await
            .map_err(|_| DeviceProviderFailure::Uncertain)
    }
}
struct Provider {
    issue: AtomicUsize,
    poll: AtomicUsize,
    exchange: AtomicUsize,
    gate: Option<Gate>,
    exchange_gate: Option<Gate>,
    failure: Option<DeviceProviderFailure>,
    pending_once: bool,
}
impl Provider {
    fn immediate() -> Self {
        Self {
            issue: AtomicUsize::new(0),
            poll: AtomicUsize::new(0),
            exchange: AtomicUsize::new(0),
            gate: None,
            exchange_gate: None,
            failure: None,
            pending_once: true,
        }
    }
}
impl DeviceAuthenticationProvider for Provider {
    fn issue(&self, now: i64) -> DeviceProviderFuture<'_, DeviceAuthorizationDraft> {
        Box::pin(async move {
            self.issue.fetch_add(1, Ordering::SeqCst);
            if let Some(gate) = &self.gate {
                gate.wait().await?;
            }
            if let Some(failure) = self.failure {
                return Err(failure);
            }
            Ok(DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(b"synthetic-device".to_vec())
                    .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
                user_code: "SYNTHETIC-CODE".to_owned(),
                interval_ms: 1000,
                expires_at: now + 60000,
            })
        })
    }
    fn poll<'a>(
        &'a self,
        _device: &'a ExternalSecret,
        _code: &'a str,
    ) -> DeviceProviderFuture<'a, DevicePollOutcome> {
        Box::pin(async move {
            let count = self.poll.fetch_add(1, Ordering::SeqCst);
            if self.pending_once && count == 0 {
                return Ok(DevicePollOutcome::Pending);
            }
            Ok(DevicePollOutcome::Approved(DeviceApproval {
                authorization_code: ExternalSecret::new(b"synthetic-approval".to_vec())
                    .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
                code_verifier: ExternalSecret::new(b"synthetic-verifier".to_vec())
                    .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
            }))
        })
    }
    fn exchange<'a>(
        &'a self,
        _approval: &'a DeviceApproval,
        now: i64,
    ) -> DeviceProviderFuture<'a, DeviceAuthenticationTokens> {
        Box::pin(async move {
            self.exchange.fetch_add(1, Ordering::SeqCst);
            if let Some(gate) = &self.exchange_gate {
                gate.wait().await?;
            }
            Ok(DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"synthetic-oauth-bundle".to_vec())
                    .map_err(|_| DeviceProviderFailure::InvalidResponse)?,
                account_id: Some("account-a".to_owned()),
                email: None,
                plan_type: None,
                expires_at: Some(now + 3600000),
            })
        })
    }
}
fn draft(request: &str) -> StartDeviceAuthentication {
    StartDeviceAuthentication {
        scope: AuthenticationScope::Personal,
        request_id: request.to_owned(),
        name: "ChatGPT".to_owned(),
        target: None,
    }
}
fn state(progress: AuthenticationProgress) -> DeviceAuthenticationStatus {
    match progress {
        AuthenticationProgress::Current(status)
        | AuthenticationProgress::ProviderFailure { status, .. } => status,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn live_exchange_keeps_exact_ownership_for_status_and_duplicate_poll() -> Result<(), TestError>
{
    let fixture = fixture().await?;
    let (entered, exchanging) = oneshot::channel();
    let (release, released) = oneshot::channel();
    let provider = Arc::new(Provider {
        exchange_gate: Some(Gate {
            entered: parking_lot::Mutex::new(Some(entered)),
            release: Mutex::new(Some(released)),
        }),
        pending_once: false,
        ..Provider::immediate()
    });
    let service = AuthenticationService::new(fixture.store.clone(), provider.clone(), 4)?;
    let result = async {
        let first = state(
            service
                .start_device(
                    &fixture.actor,
                    draft("live-exchange"),
                    AcceptanceTime::Fixed(1000),
                )
                .await?,
        );
        let attempt = first.attempt_id;
        let worker = service.clone();
        let actor = SecretDigest::from_bytes(*fixture.actor.as_bytes());
        let owned_attempt = attempt.clone();
        let call = tokio::spawn(async move {
            worker
                .poll_device(&actor, owned_attempt, AcceptanceTime::Fixed(2000))
                .await
        });
        exchanging.await.map_err(ServiceError::infrastructure)?;
        assert!(matches!(
            state(
                service
                    .device_status(&fixture.actor, &attempt, 2001)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Exchanging
        ));
        assert!(matches!(
            state(
                service
                    .poll_device(&fixture.actor, attempt, AcceptanceTime::Fixed(2001))
                    .await?
            )
            .state,
            DeviceAuthenticationState::Exchanging
        ));
        assert_eq!(provider.poll.load(Ordering::SeqCst), 1);
        assert_eq!(provider.exchange.load(Ordering::SeqCst), 1);
        release
            .send(())
            .map_err(|_| ServiceError::domain(crate::DomainReason::Internal))?;
        assert!(matches!(
            state(call.await.map_err(ServiceError::infrastructure)??).state,
            DeviceAuthenticationState::Registered { .. }
        ));
        assert_eq!(provider.exchange.load(Ordering::SeqCst), 1);
        Ok::<_, ServiceError>(())
    }
    .await;
    let cleanup = service.finish().await;
    match (result, cleanup) {
        (Err(primary), Err(cleanup)) => Err(TestError::from(combined(primary, cleanup))),
        (Err(error), _) | (_, Err(error)) => Err(TestError::from(error)),
        _ => Ok(()),
    }
}

#[tokio::test]
async fn service_polls_once_per_due_call_and_publishes_without_second_registration()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let provider = Arc::new(Provider::immediate());
    let service = AuthenticationService::new(fixture.store.clone(), provider.clone(), 4)?;
    let result = async {
        let first = state(
            service
                .start_device(
                    &fixture.actor,
                    draft("service-start"),
                    AcceptanceTime::Fixed(1000),
                )
                .await?,
        );
        assert!(matches!(
            first.state,
            DeviceAuthenticationState::Pending { .. }
        ));
        let pending = state(
            service
                .poll_device(
                    &fixture.actor,
                    first.attempt_id.clone(),
                    AcceptanceTime::Fixed(2000),
                )
                .await?,
        );
        assert!(matches!(
            pending.state,
            DeviceAuthenticationState::Pending {
                next_check_at: 3000,
                ..
            }
        ));
        let complete = state(
            service
                .poll_device(
                    &fixture.actor,
                    first.attempt_id,
                    AcceptanceTime::Fixed(3000),
                )
                .await?,
        );
        assert!(matches!(
            complete.state,
            DeviceAuthenticationState::Registered { .. }
        ));
        let recovered = state(
            service
                .start_device(
                    &fixture.actor,
                    draft("service-start"),
                    AcceptanceTime::Fixed(3001),
                )
                .await?,
        );
        assert!(matches!(
            recovered.state,
            DeviceAuthenticationState::Registered { .. }
        ));
        assert_eq!(provider.issue.load(Ordering::SeqCst), 1);
        assert_eq!(provider.poll.load(Ordering::SeqCst), 2);
        assert_eq!(provider.exchange.load(Ordering::SeqCst), 1);
        Ok::<_, ServiceError>(())
    }
    .await;
    let cleanup = service.finish().await;
    match (result, cleanup) {
        (Err(primary), Err(cleanup)) => Err(TestError::from(combined(primary, cleanup))),
        (Err(error), _) | (_, Err(error)) => Err(TestError::from(error)),
        _ => Ok(()),
    }
}
#[tokio::test]
async fn cancelled_waiter_keeps_started_capacity_and_cancellation_can_still_fence_publication()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let (entered, ready) = oneshot::channel();
    let (release, waiting) = oneshot::channel();
    let provider = Arc::new(Provider {
        gate: Some(Gate {
            entered: parking_lot::Mutex::new(Some(entered)),
            release: Mutex::new(Some(waiting)),
        }),
        ..Provider::immediate()
    });
    let service = AuthenticationService::new(fixture.store.clone(), provider.clone(), 1)?;
    let actor = fixture.actor.clone();
    let worker = service.clone();
    let caller = tokio::spawn(async move {
        worker
            .start_device(
                &actor,
                draft("cancelled-waiter"),
                AcceptanceTime::Fixed(1000),
            )
            .await
    });
    ready.await?;
    caller.abort();
    assert!(caller.await.is_err_and(|error| error.is_cancelled()));
    let result = async {
        let busy = service
            .start_device(
                &fixture.actor,
                draft("not-admitted"),
                AcceptanceTime::Fixed(1000),
            )
            .await;
        assert!(busy.is_err_and(
            |error| error.reason() == crate::DomainReason::AuthenticationCapacityExhausted
        ));
        assert!(
            service
                .request_result(&fixture.actor, "not-admitted", 1000)
                .await?
                .is_none()
        );
        let receipt = service
            .request_result(&fixture.actor, "cancelled-waiter", 1000)
            .await?
            .ok_or("Missing accepted request")?;
        let AuthenticationAcceptedResult::Device { attempt_id } = receipt.accepted else {
            return Err("Expected attempt result".into());
        };
        assert!(matches!(
            state(
                service
                    .device_status(&fixture.actor, &attempt_id, 1000)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Starting
        ));
        let mut drain = Box::pin(service.finish());
        std::future::poll_fn(|context| {
            assert!(drain.as_mut().poll(context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        // Finish holds the JoinSet bookkeeping mutex while physical work is
        // blocked. Cancellation must still commit its fence before release.
        assert!(matches!(
            state(
                service
                    .device_status(&fixture.actor, &attempt_id, 1000)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Starting
        ));
        assert!(matches!(
            state(
                service
                    .cancel_device(&fixture.actor, &attempt_id, "cancel-owned", 1000)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Cancelled
        ));
        drop(drain);
        Ok::<_, TestError>(())
    }
    .await;
    let released = release
        .send(())
        .map_err(|_| "Provider release receiver disappeared");
    let cleanup = service.finish().await;
    let mut failures = Vec::new();
    if let Err(error) = result {
        failures.push(error);
    }
    if let Err(error) = released {
        failures.push(TestError::from(error));
    }
    if let Err(error) = cleanup {
        failures.push(Box::new(error));
    }
    if !failures.is_empty() {
        return Err(TestError::from(FixtureFailures(failures)));
    }
    assert!(
        service
            .list(
                &fixture.actor,
                &AuthenticationScope::Personal,
                None,
                100,
                1000
            )
            .await?
            .is_empty()
    );
    assert_eq!(provider.issue.load(Ordering::SeqCst), 1);
    assert!(
        service
            .start_device(
                &fixture.actor,
                draft("after-finish"),
                AcceptanceTime::Fixed(1000)
            )
            .await
            .is_err_and(|error| error.reason() == crate::DomainReason::InvalidTransition)
    );
    Ok(())
}
#[tokio::test]
async fn retained_exchange_without_process_owner_reports_uncertainty_and_never_replays()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let provider = Arc::new(Provider::immediate());
    let StartDeviceAdmission::Issue(step) = fixture
        .store
        .start_external_device_authentication(&fixture.actor, &draft("retained"), 1000)
        .await?
    else {
        return Err("Missing issuance step".into());
    };
    let issued = provider.issue(1000).await?;
    fixture
        .store
        .accept_external_device_authorization(&fixture.actor, &step, &issued, 1000)
        .await?;
    let CheckDeviceAdmission::Check(check) = fixture
        .store
        .check_external_device_authentication(&fixture.actor, &step.attempt_id(), 2000)
        .await?
    else {
        return Err("Missing check step".into());
    };
    fixture
        .store
        .exchange_external_device_authentication(&fixture.actor, &check.step, 2000)
        .await?;
    let service = AuthenticationService::new(fixture.store.clone(), provider.clone(), 4)?;
    let result = async {
        assert!(matches!(
            state(
                service
                    .device_status(&fixture.actor, &step.attempt_id(), 2000)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Uncertain
        ));
        assert!(matches!(
            state(
                service
                    .poll_device(
                        &fixture.actor,
                        step.attempt_id(),
                        AcceptanceTime::Fixed(2001)
                    )
                    .await?
            )
            .state,
            DeviceAuthenticationState::Uncertain
        ));
        assert_eq!(provider.poll.load(Ordering::SeqCst), 0);
        assert_eq!(provider.exchange.load(Ordering::SeqCst), 0);
        let stale = fixture
            .store
            .external_device_authentication_status(&fixture.actor, &step.attempt_id(), 2001)
            .await?;
        assert!(matches!(
            state(
                service
                    .cancel_device(&fixture.actor, &step.attempt_id(), "cancel-retained", 2001)
                    .await?
            )
            .state,
            DeviceAuthenticationState::Cancelled
        ));
        assert!(matches!(
            state(
                service
                    .current(&fixture.actor, stale, AcceptanceTime::Fixed(2001))
                    .await?
            )
            .state,
            DeviceAuthenticationState::Cancelled
        ));
        Ok::<_, ServiceError>(())
    }
    .await;
    let cleanup = service.finish().await;
    match (result, cleanup) {
        (Err(primary), Err(cleanup)) => Err(TestError::from(combined(primary, cleanup))),
        (Err(error), _) | (_, Err(error)) => Err(TestError::from(error)),
        _ => Ok(()),
    }
}
#[tokio::test]
async fn provider_unknown_is_retained_and_start_replay_does_not_issue_again()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let provider = Arc::new(Provider {
        failure: Some(DeviceProviderFailure::Uncertain),
        ..Provider::immediate()
    });
    let service = AuthenticationService::new(fixture.store.clone(), provider.clone(), 4)?;
    let result = async {
        assert!(matches!(
            service
                .start_device(
                    &fixture.actor,
                    draft("provider-unknown"),
                    AcceptanceTime::Fixed(1000)
                )
                .await?,
            AuthenticationProgress::ProviderFailure {
                reason: DeviceProviderFailure::Uncertain,
                ..
            }
        ));
        assert!(matches!(
            state(
                service
                    .start_device(
                        &fixture.actor,
                        draft("provider-unknown"),
                        AcceptanceTime::Fixed(2000)
                    )
                    .await?
            )
            .state,
            DeviceAuthenticationState::Uncertain
        ));
        assert_eq!(provider.issue.load(Ordering::SeqCst), 1);
        Ok::<_, ServiceError>(())
    }
    .await;
    let cleanup = service.finish().await;
    match (result, cleanup) {
        (Err(primary), Err(cleanup)) => Err(TestError::from(combined(primary, cleanup))),
        (Err(error), _) | (_, Err(error)) => Err(TestError::from(error)),
        _ => Ok(()),
    }
}
