//! Deterministic regression for durable admission followed by delayed activation.
use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, RootKey,
};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_service::{
    AccountsConfig, AccountsService, ComputerRequest, CreationOutcome, PasswordWork,
    ProviderCapabilities, SessionPolicy,
};
use tokio::sync::oneshot;
type TestError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::test]
async fn drain_waits_for_later_owned_work_after_an_earlier_worker_fails() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("drain.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([30; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let store = Store::connect_sqlite(path, cipher).await?;
    let service = Arc::new(ComputersService::new(store));
    let admission = Arc::new(AtomicBool::new(true));
    let services = ComponentConnectionServices::new(service, "draining".into(), admission.clone());
    let (cancelled, cancellation) = oneshot::channel::<()>();
    {
        let mut workers = services
            .inner
            .workers
            .lock()
            .map_err(|_| "Workers unavailable")?;
        workers.spawn(async move {
            let _cancelled = cancelled;
            std::future::pending::<()>().await;
        });
        workers.abort_all();
    }
    assert!(cancellation.await.is_err());
    let (release, released) = oneshot::channel();
    let (finished, completion) = oneshot::channel();
    services
        .inner
        .workers
        .lock()
        .map_err(|_| "Workers unavailable")?
        .spawn(async move {
            let _ = released.await;
            let _ = finished.send(7);
        });
    let mut drain = Box::pin(services.drain());
    std::future::poll_fn(|context| {
        assert!(
            drain.as_mut().poll(context).is_pending(),
            "A worker failure must not abandon other retained work"
        );
        Poll::Ready(())
    })
    .await;
    assert!(!admission.load(Ordering::Acquire));
    release.send(()).map_err(|_| "Worker was abandoned")?;
    let error = drain.await.err().ok_or("Missing worker failure")?;
    assert_eq!(error.code(), tonic::Code::Internal);
    assert_eq!(completion.await?, 7);
    Ok(())
}

pub(super) async fn entry(
    inner: &Inner,
    credential: SecretDigest,
    connector: Option<(&str, &str, &str)>,
) -> Result<(Arc<Connection>, OutboundReceivers), TestError> {
    let (prepared, target) = if let Some((session, computer, execution)) = connector {
        (
            inner
                .service
                .prepare_connector_connection(session, &credential, &inner.instance_id)
                .await?,
            Target::Connector {
                computer: computer.into(),
                execution: execution.into(),
            },
        )
    } else {
        (
            inner
                .service
                .prepare_provider_connection("provider", &credential, &inner.instance_id)
                .await?,
            Target::Provider("provider".into()),
        )
    };
    let prepared = Arc::new(prepared);
    let (calls, receivers) = ConnectionCalls::new(
        prepared.owner().connection_id.clone(),
        COMMAND_QUEUE,
        LOCAL_PENDING,
        inner.pending.clone(),
    )
    .map_err(|_| "Call preparation failed")?;
    Ok((
        Arc::new(Connection {
            target,
            prepared,
            credential,
            calls,
            _permit: inner.permit()?,
            policy: None,
            convergence: Arc::new(Semaphore::new(1)),
            cursor: Mutex::new(None),
            cleanup: Mutex::new(HashMap::new()),
            native: Mutex::new(HashMap::new()),
        }),
        receivers,
    ))
}

#[test]
fn delayed_admission_cannot_displace_a_new_provider_or_connector() -> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("activation.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([29; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path, cipher).await?;
        let service = Arc::new(ComputersService::new(store.clone()));
        let proof = BootstrapProviderPermit::new("provider".into())?.authenticated();
        let key = service
            .exchange_bootstrap_provider(
                &proof,
                "Fixture",
                ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
                &chapchoo_domain::ProviderSubstrate::Kubernetes,
                "key",
                AcceptanceTime::System,
            )
            .await?;
        let control = service.register_control("http://127.0.0.1:1").await?;
        let services = ComponentConnectionServices::new(
            service.clone(),
            control.id.clone(),
            Arc::new(AtomicBool::new(true)),
        );
        let inner = services.inner.clone();
        // First exercise the Provider path, then use its actual admitted source
        // to prepare a real execution-bound Connector for the same regression.
        let (provider_a, _rx_a) = entry(&inner, key.token.digest(), None).await?;
        exercise_activation(inner.clone(), provider_a, key.token.digest(), None).await?;
        let setup = BearerSecret::generate()?;
        let accounts = AccountsService::new(
            store,
            PasswordWork::new(1, PasswordCost::new(10)?)?,
            None,
            AccountsConfig {
                general_web_url: "http://localhost:3000".into(),
                setup_proof: Some(setup.digest()),
                session_policy: SessionPolicy::default(),
            },
        )?;
        let user = accounts
            .initialize(
                &setup,
                EmailAddress::parse("owner@example.invalid")?,
                Password::parse("Aa1!fixture-password".into())?,
                AcceptanceTime::System,
                None,
            )
            .await?
            .token;
        let created = service
            .create_cloud_agent(
                &user.digest(),
                &accounts
                    .list_teams(&user.digest(), AcceptanceTime::System.now()?)
                    .await?
                    .remove(0)
                    .id,
                "create",
                "provider",
                "Private fixture",
                AcceptanceTime::System,
            )
            .await?;
        let CreationOutcome::Present(agent) = created else {
            return Err("Agent missing".into());
        };
        let current = inner
            .entries
            .lock()
            .map_err(|_| "Registry unavailable")?
            .providers
            .get("provider")
            .cloned()
            .ok_or("Provider missing")?;
        let source = inner.source(&current).ok_or("Provider source missing")?;
        let reservation = service
            .reserve_execution(
                &source,
                ComputerRequest {
                    computer_id: &agent.computer.id,
                    request_id: "create",
                },
                &chapchoo_domain::NativeScope::kubernetes("fixture".into())?,
            )
            .await?;
        let session = service
            .issue_connector_session_from_source(
                &source,
                &reservation.execution_id,
                "bootstrap",
                AcceptanceTime::System,
            )
            .await?;
        let operational = service
            .exchange_connector_key(
                &session.id,
                &session.initial_key.digest(),
                "exchange",
                AcceptanceTime::System,
            )
            .await?;
        let connector = Some((
            session.id.as_str(),
            agent.computer.id.as_str(),
            reservation.execution_id.as_str(),
        ));
        let (connector_a, _connector_rx) =
            entry(&inner, operational.token.digest(), connector).await?;
        exercise_activation(inner, connector_a, operational.token.digest(), connector).await?;
        Ok(())
    })
}

async fn exercise_activation(
    inner: Arc<Inner>,
    a: Arc<Connection>,
    credential: SecretDigest,
    connector: Option<(&str, &str, &str)>,
) -> Result<(), TestError> {
    let (admitted, admitted_rx) = oneshot::channel();
    let (resume, resume_rx) = oneshot::channel();
    *inner
        .activation_checkpoint
        .lock()
        .map_err(|_| "Checkpoint unavailable")? = Some((admitted, resume_rx));
    let worker = inner.clone();
    let old = a.clone();
    let first = tokio::spawn(async move { worker.activate(old).await });
    admitted_rx.await?;
    // At this exact post-commit/pre-install boundary the activation guard must
    // still be held. B captures the newly committed A, never the old predecessor.
    assert!(inner.activation.try_lock().is_err());
    let (b, _rx_b) = entry(&inner, credential, connector).await?;
    let worker = inner.clone();
    let successor = b.clone();
    let second = tokio::spawn(async move { worker.activate(successor).await });
    resume.send(()).map_err(|_| "Activation ended early")?;
    first.await??;
    second.await??;
    assert!(a.calls.is_closed());
    assert!(!b.calls.is_closed());
    inner.finish(&a).await;
    inner.verify(&b).await?;
    assert!(!b.calls.is_closed());
    Ok(())
}
