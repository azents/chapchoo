//! One process-owned exchange per protected source; other consumers observe it.
use super::*;
use chapchoo_persistence::{
    AuthenticationCredentialMaterial, AuthenticationCredentialSource,
    AuthenticationRefreshAdmission as Admission, AuthenticationRefreshState as State,
    AuthenticationRefreshStep, AuthenticationScope, ConnectorSource, ConversationInputPreparation,
    DeviceAuthenticationTokens,
};
/// Owned internal execution authority retained beyond a cancelled RPC waiter.
/// Its credential is the authenticated operational key digest, not an account token.
#[derive(Clone)]
pub struct ExecutionAuthenticationConsumer {
    pub session_id: String,
    pub control_id: String,
    pub connection_id: String,
    pub credential: SecretDigest,
    pub agent_id: String,
    pub conversation_id: String,
    pub operation_id: String,
}
impl fmt::Debug for ExecutionAuthenticationConsumer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExecutionAuthenticationConsumer([REDACTED])")
    }
}
impl ExecutionAuthenticationConsumer {
    pub(super) fn source(&self) -> ConnectorSource<'_> {
        ConnectorSource {
            session_id: &self.session_id,
            control_id: &self.control_id,
            connection_id: &self.connection_id,
            credential: &self.credential,
        }
    }
    pub(super) fn application(&self) -> ConversationInputPreparation<'_> {
        ConversationInputPreparation {
            agent_id: &self.agent_id,
            conversation_id: &self.conversation_id,
            operation_id: &self.operation_id,
        }
    }
}
enum Consumer {
    Account {
        actor: SecretDigest,
        scope: AuthenticationScope,
        authentication: String,
    },
    Execution(ExecutionAuthenticationConsumer),
}
impl Consumer {
    async fn begin(&self, store: &Store, now: i64) -> Result<Admission, StoreError> {
        match self {
            Self::Account {
                actor,
                scope,
                authentication,
            } => {
                store
                    .begin_external_authentication_refresh(actor, scope, authentication, now)
                    .await
            }
            Self::Execution(consumer) => {
                store
                    .begin_execution_authentication_refresh(
                        &consumer.source(),
                        &consumer.application(),
                        now,
                    )
                    .await
            }
        }
    }
    async fn complete(
        &self,
        store: &Store,
        step: &AuthenticationRefreshStep,
        tokens: &DeviceAuthenticationTokens,
        now: i64,
    ) -> Result<AuthenticationCredentialSource, StoreError> {
        match self {
            Self::Account { actor, scope, .. } => {
                store
                    .complete_external_authentication_refresh(actor, scope, step, tokens, now)
                    .await
            }
            Self::Execution(consumer) => {
                store
                    .complete_execution_authentication_refresh(
                        &consumer.source(),
                        &consumer.application(),
                        step,
                        tokens,
                        now,
                    )
                    .await
            }
        }
    }
}
#[derive(Debug)]
pub enum AuthenticationRefreshProgress {
    Current(AuthenticationCredentialMaterial),
    Pending { attempt_id: String },
    Settled { attempt_id: String, state: State },
}
#[derive(Clone)]
pub struct AuthenticationRefreshService {
    store: Store,
    provider: Arc<dyn ChatGptRefreshProvider>,
    work: Arc<AuthenticationWork>,
}
impl fmt::Debug for AuthenticationRefreshService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthenticationRefreshService([REDACTED])")
    }
}
impl AuthenticationRefreshService {
    pub fn new(
        store: Store,
        provider: Arc<dyn ChatGptRefreshProvider>,
        capacity: usize,
    ) -> Result<Self, ServiceError> {
        Ok(Self {
            store,
            provider,
            work: Arc::new(AuthenticationWork::new(capacity)?),
        })
    }
    /// Internal eligible consumer operation, not a public credential read API.
    /// A cancelled waiter cannot stop the admitted refresh or release capacity.
    pub async fn current(
        &self,
        actor: SecretDigest,
        scope: AuthenticationScope,
        authentication: String,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationRefreshProgress, ServiceError> {
        self.current_for(
            Consumer::Account {
                actor,
                scope,
                authentication,
            },
            clock,
        )
        .await
    }
    pub async fn current_execution(
        &self,
        consumer: ExecutionAuthenticationConsumer,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationRefreshProgress, ServiceError> {
        self.current_for(Consumer::Execution(consumer), clock).await
    }
    async fn current_for(
        &self,
        consumer: Consumer,
        clock: AcceptanceTime,
    ) -> Result<AuthenticationRefreshProgress, ServiceError> {
        let service = self.clone();
        self.work
            .run(move |owner| async move {
                let handoff = owner.admission();
                let admission = consumer.begin(&service.store, clock.now()?).await?;
                let (step, credentials) = match admission {
                    Admission::Current(material) => {
                        return Ok(AuthenticationRefreshProgress::Current(material));
                    }
                    Admission::Existing {
                        attempt_id,
                        state: State::Possible,
                    } => {
                        drop(handoff);
                        let owned = owner
                            .owns_after_admission(&attempt_id, Some(&attempt_id))
                            .await;
                        return Ok(if owned {
                            AuthenticationRefreshProgress::Pending { attempt_id }
                        } else {
                            AuthenticationRefreshProgress::Settled {
                                attempt_id,
                                state: State::Uncertain,
                            }
                        });
                    }
                    Admission::Existing { attempt_id, state } => {
                        return Ok(AuthenticationRefreshProgress::Settled { attempt_id, state });
                    }
                    Admission::Granted { step, credentials } => (step, credentials),
                };
                let attempt_id = step.id();
                owner.mark(attempt_id.clone(), &attempt_id);
                drop(handoff);
                let result = service
                    .provider
                    .refresh(&credentials.secret, &credentials.account_id, clock.now()?)
                    .await;
                match result {
                    Ok(tokens) => {
                        let source = match consumer
                            .complete(&service.store, &step, &tokens, clock.now()?)
                            .await
                        {
                            Ok(source) => source,
                            Err(primary) => {
                                let primary = ServiceError::from(primary);
                                return match service
                                    .store
                                    .fail_external_authentication_refresh(
                                        &step,
                                        State::Uncertain,
                                        clock.now()?,
                                    )
                                    .await
                                {
                                    Ok(()) => Err(primary),
                                    Err(cleanup) => Err(combined(primary, cleanup.into())),
                                };
                            }
                        };
                        Ok(AuthenticationRefreshProgress::Current(
                            AuthenticationCredentialMaterial {
                                authentication_id: credentials.authentication_id,
                                source,
                                account_id: credentials.account_id,
                                expires_at: tokens.expires_at,
                                secret: tokens.secret,
                            },
                        ))
                    }
                    Err(reason) => {
                        let state = match reason {
                            RefreshProviderFailure::NotDispatched => State::NotDispatched,
                            RefreshProviderFailure::Rejected => State::Rejected,
                            RefreshProviderFailure::Uncertain => State::Uncertain,
                        };
                        service
                            .store
                            .fail_external_authentication_refresh(&step, state, clock.now()?)
                            .await?;
                        Ok(AuthenticationRefreshProgress::Settled { attempt_id, state })
                    }
                }
            })
            .await
    }
    pub fn close(&self) {
        self.work.close();
    }
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.work.finish().await
    }
}
