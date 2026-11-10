//! Bounded execution-only application, renewal and short-lived material reads.
use super::*;
use crate::DomainReason;
use chapchoo_crypto::ExternalSecret;
use chapchoo_persistence::{
    AppliedExecutionAuthentication, AuthenticationCredentialSource, AuthenticationKind,
    AuthenticationRefreshState, ExecutionAuthenticationMaterial,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub struct ExecutionAuthenticationDescriptor {
    pub applied: AppliedExecutionAuthentication,
    pub source: AuthenticationCredentialSource,
    pub kind: AuthenticationKind,
    pub account_id: Option<String>,
    pub expires_at: Option<i64>,
    pub total_bytes: usize,
}
pub struct ExecutionAuthenticationChunk {
    pub descriptor: ExecutionAuthenticationDescriptor,
    pub offset: usize,
    pub content: ExternalSecret,
    _permit: OwnedSemaphorePermit,
}
pub enum ExecutionAuthenticationProgress {
    Current(Box<ExecutionAuthenticationDescriptor>),
    Pending {
        attempt_id: String,
    },
    Settled {
        attempt_id: String,
        state: AuthenticationRefreshState,
    },
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    ExecutionAuthenticationDescriptor,
    ExecutionAuthenticationChunk,
    ExecutionAuthenticationProgress
);
#[derive(Clone)]
pub struct ExecutionAuthenticationService {
    store: Store,
    refresh: AuthenticationRefreshService,
    encoder: Arc<dyn NativeTokenEncoder>,
    reads: Arc<Semaphore>,
}
impl fmt::Debug for ExecutionAuthenticationService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExecutionAuthenticationService([REDACTED])")
    }
}
impl ExecutionAuthenticationService {
    pub fn new(
        store: Store,
        provider: Arc<dyn ChatGptRefreshProvider>,
        encoder: Arc<dyn NativeTokenEncoder>,
        capacity: usize,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            refresh: AuthenticationRefreshService::new(store.clone(), provider, capacity)?,
            store,
            encoder,
            reads: Arc::new(Semaphore::new(16)),
        })
    }
    fn permit(&self) -> Result<OwnedSemaphorePermit, ServiceError> {
        self.reads
            .clone()
            .try_acquire_owned()
            .map_err(|_| ServiceError::domain(DomainReason::AuthenticationCapacityExhausted))
    }
    /// Pure metadata recovery; neither selects nor refreshes an authentication.
    pub async fn binding(
        &self,
        consumer: &ExecutionAuthenticationConsumer,
    ) -> Result<Option<AppliedExecutionAuthentication>, ServiceError> {
        let _permit = self.permit()?;
        Ok(self
            .store
            .execution_authentication_binding(&consumer.source(), &consumer.application())
            .await?)
    }
    /// Only the actual native launch calls this saved-to-applied admission.
    pub async fn begin(
        &self,
        consumer: &ExecutionAuthenticationConsumer,
        expected_previous: Option<&str>,
    ) -> Result<AppliedExecutionAuthentication, ServiceError> {
        let _permit = self.permit()?;
        Ok(self
            .store
            .begin_execution_authentication(
                &consumer.source(),
                &consumer.application(),
                expected_previous,
            )
            .await?)
    }
    fn native(
        &self,
        material: ExecutionAuthenticationMaterial,
    ) -> Result<(ExecutionAuthenticationDescriptor, ExternalSecret), ServiceError> {
        let payload = match material.kind {
            AuthenticationKind::OpenAiApiKey => {
                ExternalSecret::new(material.secret.expose().to_vec())
                    .map_err(|_| ServiceError::domain(DomainReason::InvalidTransition))?
            }
            AuthenticationKind::ChatGpt => self
                .encoder
                .encode(
                    &material.secret,
                    material
                        .account_id
                        .as_deref()
                        .ok_or_else(|| ServiceError::domain(DomainReason::InvalidTransition))?,
                )
                .map_err(|_| ServiceError::domain(DomainReason::InvalidTransition))?,
        };
        let descriptor = ExecutionAuthenticationDescriptor {
            applied: material.applied,
            source: material.source,
            kind: material.kind,
            account_id: material.account_id,
            expires_at: material.expires_at,
            total_bytes: payload.expose().len(),
        };
        Ok((descriptor, payload))
    }
    /// Explicit eligible-consumer renewal. An ordinary metadata read never calls it.
    pub async fn acquire(
        &self,
        consumer: ExecutionAuthenticationConsumer,
        clock: AcceptanceTime,
    ) -> Result<ExecutionAuthenticationProgress, ServiceError> {
        {
            let _permit = self.permit()?;
            let material = self
                .store
                .execution_authentication_material(
                    &consumer.source(),
                    &consumer.application(),
                    None,
                )
                .await?;
            if material.kind == AuthenticationKind::OpenAiApiKey {
                return Ok(ExecutionAuthenticationProgress::Current(Box::new(
                    self.native(material)?.0,
                )));
            }
        }
        match self
            .refresh
            .current_execution(consumer.clone(), clock)
            .await?
        {
            AuthenticationRefreshProgress::Current(current) => {
                let _permit = self.permit()?;
                let material = self
                    .store
                    .execution_authentication_material(
                        &consumer.source(),
                        &consumer.application(),
                        Some(&current.source),
                    )
                    .await?;
                Ok(ExecutionAuthenticationProgress::Current(Box::new(
                    self.native(material)?.0,
                )))
            }
            AuthenticationRefreshProgress::Pending { attempt_id } => {
                Ok(ExecutionAuthenticationProgress::Pending { attempt_id })
            }
            AuthenticationRefreshProgress::Settled { attempt_id, state } => {
                Ok(ExecutionAuthenticationProgress::Settled { attempt_id, state })
            }
        }
    }
    /// Each piece rechecks the exact current application and protected source.
    /// The complete server ChatGPT bundle is never returned to this consumer.
    pub async fn read(
        &self,
        consumer: &ExecutionAuthenticationConsumer,
        source: &AuthenticationCredentialSource,
        offset: usize,
        limit: usize,
    ) -> Result<ExecutionAuthenticationChunk, ServiceError> {
        if !(1..=32 * 1024).contains(&limit) {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        let permit = self.permit()?;
        let material = self
            .store
            .execution_authentication_material(
                &consumer.source(),
                &consumer.application(),
                Some(source),
            )
            .await?;
        let (descriptor, payload) = self.native(material)?;
        let start = offset;
        if start >= descriptor.total_bytes {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        let end = start.saturating_add(limit).min(descriptor.total_bytes);
        Ok(ExecutionAuthenticationChunk {
            descriptor,
            offset: start,
            content: ExternalSecret::new(payload.expose()[start..end].to_vec())
                .map_err(|_| ServiceError::domain(DomainReason::InvalidTransition))?,
            _permit: permit,
        })
    }
    /// Final handoff revalidation. It does not refresh or force native adoption.
    pub async fn check(
        &self,
        consumer: &ExecutionAuthenticationConsumer,
        source: &AuthenticationCredentialSource,
    ) -> Result<(), ServiceError> {
        let _permit = self.permit()?;
        self.store
            .execution_authentication_material(
                &consumer.source(),
                &consumer.application(),
                Some(source),
            )
            .await?;
        Ok(())
    }
    pub fn close(&self) {
        self.reads.close();
        self.refresh.close();
    }
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.refresh.finish().await
    }
}
