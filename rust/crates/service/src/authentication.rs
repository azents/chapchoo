//! Backend authentication acquisition and process-owned external work.

use crate::{AcceptanceTime, ServiceError};
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    CheckDeviceAdmission, DeviceAuthenticationState, DeviceAuthenticationStatus,
    DeviceAuthenticationStep, DeviceStepFailure, DeviceStepSettlement, ExchangeDeviceAdmission,
    StartDeviceAdmission, StartDeviceAuthentication, Store, StoreError,
};
use std::{fmt, sync::Arc};
use work::{AuthenticationWork, combined};

mod catalogue;
mod provider;
mod work;
pub use provider::{
    DeviceApproval, DeviceAuthenticationProvider, DevicePollOutcome, DeviceProviderFailure,
    DeviceProviderFuture,
};
pub const DEVICE_VERIFICATION_URI: &str = "https://auth.openai.com/codex/device";

#[derive(Debug)]
pub enum AuthenticationProgress {
    Current(DeviceAuthenticationStatus),
    ProviderFailure {
        status: DeviceAuthenticationStatus,
        reason: DeviceProviderFailure,
    },
}

#[derive(Clone)]
pub struct AuthenticationService {
    store: Store,
    provider: Arc<dyn DeviceAuthenticationProvider>,
    work: Arc<AuthenticationWork>,
}
impl fmt::Debug for AuthenticationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticationService([REDACTED])")
    }
}
impl AuthenticationService {
    pub fn new(
        store: Store,
        provider: Arc<dyn DeviceAuthenticationProvider>,
        capacity: usize,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            store,
            provider,
            work: Arc::new(AuthenticationWork::new(capacity)?),
        })
    }
    async fn current(
        &self,
        actor: &SecretDigest,
        mut status: DeviceAuthenticationStatus,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationProgress, ServiceError> {
        while matches!(
            &status.state,
            DeviceAuthenticationState::Starting
                | DeviceAuthenticationState::Checking
                | DeviceAuthenticationState::Exchanging
        ) && !self
            .work
            .owns_after_admission(&status.attempt_id, status.step_id.as_deref())
            .await
        {
            // Work may have completed and been reaped since the original read
            // or while admission ownership was handed off. Re-read only after
            // the ownerless observation, so terminal success is not downgraded.
            let previous_step = status.step_id.clone();
            status = self
                .store
                .external_device_authentication_status(actor, &status.attempt_id, clock.now()?)
                .await?;
            if status.step_id == previous_step
                && matches!(
                    &status.state,
                    DeviceAuthenticationState::Starting
                        | DeviceAuthenticationState::Checking
                        | DeviceAuthenticationState::Exchanging
                )
            {
                // Retained in-flight state without an owner never grants replay.
                status.state = DeviceAuthenticationState::Uncertain;
            }
        }
        Ok(AuthenticationProgress::Current(status))
    }
    async fn acceptance(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        result: Result<DeviceAuthenticationStatus, StoreError>,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationProgress, ServiceError> {
        match result {
            Ok(status) => self.current(actor, status, clock).await,
            Err(primary) => {
                let failure = if matches!(
                    &primary,
                    StoreError::Unauthenticated
                        | StoreError::Forbidden
                        | StoreError::NotFound
                        | StoreError::InvalidTransition
                ) {
                    DeviceStepFailure::Failed
                } else {
                    DeviceStepFailure::Uncertain
                };
                let primary = ServiceError::from(primary);
                let now = match clock.now() {
                    Ok(now) => now,
                    Err(cleanup) => return Err(combined(primary, cleanup)),
                };
                match self
                    .store
                    .fail_external_device_step(step, failure, now)
                    .await
                {
                    Ok(_) => Err(primary),
                    Err(cleanup) => Err(combined(primary, cleanup.into())),
                }
            }
        }
    }
    async fn provider_failure(
        &self,
        actor: &SecretDigest,
        step: &DeviceAuthenticationStep,
        reason: DeviceProviderFailure,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationProgress, ServiceError> {
        let now = clock.now()?;
        let failure = if reason == DeviceProviderFailure::Rejected {
            DeviceStepFailure::Failed
        } else {
            DeviceStepFailure::Uncertain
        };
        let settlement = self
            .store
            .fail_external_device_step(step, failure, now)
            .await?;
        // Narrow cleanup authority does not grant a code/record read. Recheck
        // the current caller even when cancellation or role withdrawal won.
        let status = self
            .store
            .external_device_authentication_status(actor, &step.attempt_id(), now)
            .await?;
        if settlement == DeviceStepSettlement::Recorded {
            Ok(AuthenticationProgress::ProviderFailure { status, reason })
        } else {
            self.current(actor, status, clock).await
        }
    }
    pub async fn start_device(
        &self,
        actor: &SecretDigest,
        draft: StartDeviceAuthentication,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationProgress, ServiceError> {
        let service = self.clone();
        let actor = SecretDigest::from_bytes(*actor.as_bytes());
        self.work
            .run(move |work| async move {
                let admission = work.admission();
                let admitted = service
                    .store
                    .start_external_device_authentication(&actor, &draft, clock.now()?)
                    .await?;
                if let StartDeviceAdmission::Issue(step) = &admitted {
                    work.mark(step.attempt_id(), step.step_id());
                }
                drop(admission);
                match admitted {
                    StartDeviceAdmission::Current(status) => {
                        service.current(&actor, status, clock).await
                    }
                    StartDeviceAdmission::Issue(step) => {
                        match service.provider.issue(clock.now()?).await {
                            Ok(issued) => {
                                let result = service
                                    .store
                                    .accept_external_device_authorization(
                                        &actor,
                                        &step,
                                        &issued,
                                        clock.now()?,
                                    )
                                    .await;
                                service.acceptance(&actor, &step, result, clock).await
                            }
                            Err(reason) => {
                                service.provider_failure(&actor, &step, reason, clock).await
                            }
                        }
                    }
                }
            })
            .await
    }
    pub async fn poll_device(
        &self,
        actor: &SecretDigest,
        attempt: String,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationProgress, ServiceError> {
        let service = self.clone();
        let actor = SecretDigest::from_bytes(*actor.as_bytes());
        self.work
            .run(move |work| async move {
                let admission = work.admission();
                let admitted = service
                    .store
                    .check_external_device_authentication(&actor, &attempt, clock.now()?)
                    .await?;
                if let CheckDeviceAdmission::Check(check) = &admitted {
                    work.mark(check.step.attempt_id(), check.step.step_id());
                }
                drop(admission);
                let check = match admitted {
                    CheckDeviceAdmission::Current(status) => {
                        return service.current(&actor, status, clock).await;
                    }
                    CheckDeviceAdmission::Check(check) => check,
                };
                let approval = match service
                    .provider
                    .poll(&check.device_auth_id, &check.user_code)
                    .await
                {
                    Ok(DevicePollOutcome::Pending) => {
                        let result = service
                            .store
                            .accept_external_device_pending(
                                &actor,
                                &check.step,
                                check.interval_ms,
                                clock.now()?,
                            )
                            .await;
                        return service.acceptance(&actor, &check.step, result, clock).await;
                    }
                    Ok(DevicePollOutcome::Approved(approval)) => approval,
                    Err(reason) => {
                        return service
                            .provider_failure(&actor, &check.step, reason, clock)
                            .await;
                    }
                };
                let admission = work.admission();
                let admitted = service
                    .store
                    .exchange_external_device_authentication(&actor, &check.step, clock.now()?)
                    .await;
                if let Ok(ExchangeDeviceAdmission::Exchange(step)) = &admitted {
                    work.mark(step.attempt_id(), step.step_id());
                }
                drop(admission);
                let step = match admitted {
                    Ok(ExchangeDeviceAdmission::Current(status)) => {
                        return service.current(&actor, status, clock).await;
                    }
                    Ok(ExchangeDeviceAdmission::Exchange(step)) => step,
                    Err(primary) => {
                        return service
                            .acceptance(&actor, &check.step, Err(primary), clock)
                            .await;
                    }
                };
                match service.provider.exchange(&approval, clock.now()?).await {
                    Ok(tokens) => {
                        let result = service
                            .store
                            .complete_external_device_authentication(
                                &actor,
                                &step,
                                &tokens,
                                clock.now()?,
                            )
                            .await;
                        service.acceptance(&actor, &step, result, clock).await
                    }
                    Err(reason) => service.provider_failure(&actor, &step, reason, clock).await,
                }
            })
            .await
    }
    pub async fn device_status(
        &self,
        actor: &SecretDigest,
        attempt: &str,
        now: i64,
    ) -> Result<AuthenticationProgress, ServiceError> {
        self.work.reap_available()?;
        self.current(
            actor,
            self.store
                .external_device_authentication_status(actor, attempt, now)
                .await?,
            AcceptanceTime::Fixed(now),
        )
        .await
    }
    pub async fn cancel_device(
        &self,
        actor: &SecretDigest,
        attempt: &str,
        request: &str,
        now: i64,
    ) -> Result<AuthenticationProgress, ServiceError> {
        // Persist the cancellation fence independently of work capacity,
        // bookkeeping contention and unrelated retained work failures. Their
        // existing owner still collects them on ordinary reaping or finish.
        self.current(
            actor,
            self.store
                .cancel_external_device_authentication(actor, attempt, request, now)
                .await?,
            AcceptanceTime::Fixed(now),
        )
        .await
    }
    /// Stop admission and await every owned call. Cancellation of this finish
    /// preserves the JoinSet and already-collected causes for a later re-entry.
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.work.finish().await
    }
    /// Close external-work admission before draining RPC handlers and effects.
    pub fn close(&self) {
        self.work.close();
    }
}

#[cfg(test)]
mod tests;
