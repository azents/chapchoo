use super::*;
use chapchoo_persistence::{
    AuthenticationRefreshState as RefreshState, CheckDeviceAdmission, ExchangeDeviceAdmission,
    StartDeviceAdmission,
};
async fn authentication(fixture: &Fixture) -> Result<String, TestError> {
    let StartDeviceAdmission::Issue(step) = fixture
        .store
        .start_external_device_authentication(&fixture.actor, &draft("refresh-fixture"), 1000)
        .await?
    else {
        return Err("Missing device admission".into());
    };
    fixture
        .store
        .accept_external_device_authorization(
            &fixture.actor,
            &step,
            &DeviceAuthorizationDraft {
                device_auth_id: ExternalSecret::new(b"synthetic-device".to_vec())?,
                user_code: "CODE".into(),
                expires_at: 60000,
                interval_ms: 1000,
            },
            1000,
        )
        .await?;
    let CheckDeviceAdmission::Check(check) = fixture
        .store
        .check_external_device_authentication(&fixture.actor, &step.attempt_id(), 2000)
        .await?
    else {
        return Err("Missing check".into());
    };
    let ExchangeDeviceAdmission::Exchange(step) = fixture
        .store
        .exchange_external_device_authentication(&fixture.actor, &check.step, 2000)
        .await?
    else {
        return Err("Missing exchange".into());
    };
    let status = fixture
        .store
        .complete_external_device_authentication(
            &fixture.actor,
            &step,
            &DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"original-refresh-bundle".to_vec())?,
                account_id: Some("account-a".into()),
                email: None,
                plan_type: None,
                expires_at: Some(2000),
            },
            2000,
        )
        .await?;
    let DeviceAuthenticationState::Registered { authentication_id } = status.state else {
        return Err("Missing authentication record".into());
    };
    Ok(authentication_id)
}
struct Refresh {
    calls: AtomicUsize,
    entered: tokio::sync::Semaphore,
    release: tokio::sync::Semaphore,
    failure: Option<RefreshProviderFailure>,
}
impl ChatGptRefreshProvider for Refresh {
    fn refresh<'a>(
        &'a self,
        bundle: &'a ExternalSecret,
        account: &'a str,
        now: i64,
    ) -> RefreshProviderFuture<'a> {
        Box::pin(async move {
            assert_eq!(bundle.expose(), b"original-refresh-bundle");
            assert_eq!(account, "account-a");
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.entered.add_permits(1);
            self.release
                .acquire()
                .await
                .map_err(|_| RefreshProviderFailure::Uncertain)?
                .forget();
            if let Some(failure) = self.failure {
                return Err(failure);
            }
            Ok(DeviceAuthenticationTokens {
                secret: ExternalSecret::new(b"renewed-refresh-bundle".to_vec())
                    .map_err(|_| RefreshProviderFailure::Uncertain)?,
                account_id: Some(account.into()),
                email: None,
                plan_type: None,
                expires_at: Some(now + 3_600_000),
            })
        })
    }
}
#[tokio::test]
async fn concurrent_consumers_share_the_owned_refresh_beyond_a_cancelled_waiter()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let authentication = authentication(&fixture).await?;
    let provider = Arc::new(Refresh {
        calls: AtomicUsize::new(0),
        entered: tokio::sync::Semaphore::new(0),
        release: tokio::sync::Semaphore::new(0),
        failure: None,
    });
    let service = AuthenticationRefreshService::new(fixture.store.clone(), provider.clone(), 4)?;
    let owned = service.clone();
    let actor = fixture.actor.clone();
    let id = authentication.clone();
    let waiter = tokio::spawn(async move {
        owned
            .current(
                actor,
                AuthenticationScope::Personal,
                id,
                AcceptanceTime::Fixed(2000),
            )
            .await
    });
    provider.entered.acquire().await?.forget();
    waiter.abort();
    let _ = waiter.await;
    let result = async {
        let progress = service
            .current(
                fixture.actor.clone(),
                AuthenticationScope::Personal,
                authentication.clone(),
                AcceptanceTime::Fixed(2000),
            )
            .await?;
        assert!(matches!(
            progress,
            AuthenticationRefreshProgress::Pending { .. }
        ));
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        provider.release.add_permits(1);
        service.finish().await?;
        let fresh = AuthenticationRefreshService::new(fixture.store.clone(), provider.clone(), 4)?;
        let value = fresh
            .current(
                fixture.actor.clone(),
                AuthenticationScope::Personal,
                authentication,
                AcceptanceTime::Fixed(2001),
            )
            .await?;
        let AuthenticationRefreshProgress::Current(material) = value else {
            return Err("Refreshed source not current".into());
        };
        assert_eq!(material.secret.expose(), b"renewed-refresh-bundle");
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        fresh.finish().await?;
        Ok::<(), TestError>(())
    }
    .await;
    provider.release.add_permits(4);
    let cleanup = service
        .finish()
        .await
        .map_err(|error| Box::new(error) as TestError);
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(primary), Err(cleanup)) => {
            Err(Box::new(FixtureFailures(vec![primary, cleanup])) as TestError)
        }
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}
#[tokio::test]
async fn lost_refresh_outcome_remains_uncertain_for_a_new_server_owner() -> Result<(), TestError> {
    let fixture = fixture().await?;
    let authentication = authentication(&fixture).await?;
    let provider = Arc::new(Refresh {
        calls: AtomicUsize::new(0),
        entered: tokio::sync::Semaphore::new(0),
        release: tokio::sync::Semaphore::new(1),
        failure: Some(RefreshProviderFailure::Uncertain),
    });
    for _ in 0..2 {
        let service =
            AuthenticationRefreshService::new(fixture.store.clone(), provider.clone(), 4)?;
        let result = service
            .current(
                fixture.actor.clone(),
                AuthenticationScope::Personal,
                authentication.clone(),
                AcceptanceTime::Fixed(2000),
            )
            .await;
        let cleanup = service.finish().await;
        let value = result?;
        cleanup?;
        assert!(matches!(
            value,
            AuthenticationRefreshProgress::Settled {
                state: RefreshState::Uncertain,
                ..
            }
        ));
    }
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    Ok(())
}
#[tokio::test]
async fn retained_possible_exchange_without_process_ownership_does_not_redispatch()
-> Result<(), TestError> {
    let fixture = fixture().await?;
    let authentication = authentication(&fixture).await?;
    let admitted = fixture
        .store
        .begin_external_authentication_refresh(
            &fixture.actor,
            &AuthenticationScope::Personal,
            &authentication,
            2000,
        )
        .await?;
    let chapchoo_persistence::AuthenticationRefreshAdmission::Granted { step, .. } = admitted
    else {
        return Err("Expected sealed refresh attempt".into());
    };
    let provider = Arc::new(Refresh {
        calls: AtomicUsize::new(0),
        entered: tokio::sync::Semaphore::new(0),
        release: tokio::sync::Semaphore::new(0),
        failure: None,
    });
    let service = AuthenticationRefreshService::new(fixture.store.clone(), provider.clone(), 4)?;
    let result = service
        .current(
            fixture.actor,
            AuthenticationScope::Personal,
            authentication,
            AcceptanceTime::Fixed(2000),
        )
        .await;
    let cleanup = service.finish().await;
    assert!(matches!(result?, AuthenticationRefreshProgress::Settled {
        state: RefreshState::Uncertain, attempt_id,
    } if attempt_id == step.id()));
    cleanup?;
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    Ok(())
}
