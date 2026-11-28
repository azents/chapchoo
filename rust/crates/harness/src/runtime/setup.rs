//! Setup is execution-owned and idempotent independently of Control call IDs.
use super::*;
#[derive(Clone, PartialEq, Eq)]
pub enum NativeSetupIntent {
    Initial {
        allow_fresh: bool,
        require_retained_source: bool,
    },
    Existing(NativeTextTarget),
}
/// The sealed command fields available before reading preparation inputs.
#[derive(Clone, PartialEq, Eq)]
pub struct NativeSetupRequest {
    pub operation_id: String,
    pub scope: crate::NativeToolScope,
    pub supply: HarnessSupply,
    pub intent: NativeSetupIntent,
}
#[derive(Clone, PartialEq, Eq)]
pub struct NativeSetupOperation {
    pub operation_id: String,
    pub scope: crate::NativeToolScope,
    pub supply: HarnessSupply,
    pub intent: NativeSetupIntent,
    pub instructions: chapchoo_domain::ProjectInstructions,
    pub tool_catalog: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSetupFailure {
    NotAdmitted,
    SourceUnavailable,
    BeforeNativeLaunch,
    NativeRejected,
    RetirementUnknown,
}
#[derive(Clone, PartialEq, Eq)]
pub enum NativeSetupObservation {
    Unseen,
    PossibleForwarding,
    Ready(NativeTextTarget),
    Failed(NativeSetupFailure),
}
pub struct NativeSetupRecovery {
    pub outcome: NativeSetupObservation,
    pub connection_id: Option<String>,
    pub external_tools_applied: bool,
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}})+};}
safe_debug!(
    NativeSetupIntent,
    NativeSetupRequest,
    NativeSetupOperation,
    NativeSetupObservation,
    NativeSetupRecovery
);
impl NativeSetupOperation {
    pub(super) fn check_request(&self, request: &NativeSetupRequest) -> Result<(), Error> {
        if self.operation_id != request.operation_id
            || self.scope != request.scope
            || self.supply != request.supply
            || self.intent != request.intent
        {
            return Err(Error::OperationConflict);
        }
        Ok(())
    }
}
#[derive(Default)]
pub(super) struct SetupRegistry {
    entries: BTreeMap<String, SetupEntry>,
}
struct SetupEntry {
    operation: NativeSetupOperation,
    outcome: NativeSetupObservation,
    connection_id: Option<String>,
    external_tools_applied: bool,
}
impl SetupRegistry {
    pub(super) fn expire_unstarted(
        &mut self,
        operation: &NativeSetupOperation,
    ) -> Result<NativeSetupObservation, Error> {
        if let Some(previous) = self.begin(operation, None)? {
            return Ok(previous);
        }
        let failure = NativeSetupObservation::Failed(NativeSetupFailure::NotAdmitted);
        self.completed(&operation.operation_id, failure.clone())?;
        Ok(failure)
    }
    pub(super) fn release_reported(
        &mut self,
        id: &str,
        expected: &NativeSetupObservation,
    ) -> Result<(), Error> {
        valid_id(id)?;
        if !matches!(
            expected,
            NativeSetupObservation::Ready(_) | NativeSetupObservation::Failed(_)
        ) {
            return Err(Error::OperationConflict);
        }
        if self
            .entries
            .get(id)
            .is_some_and(|entry| &entry.outcome != expected)
        {
            return Err(Error::OperationConflict);
        }
        self.entries.remove(id);
        Ok(())
    }
    pub(super) fn begin(
        &mut self,
        operation: &NativeSetupOperation,
        connection_id: Option<&str>,
    ) -> Result<Option<NativeSetupObservation>, Error> {
        valid_id(&operation.operation_id)?;
        if let Some(previous) = self.recover(operation)? {
            return Ok(Some(previous));
        }
        if self.entries.len() >= 256 {
            return Err(Error::NotAdmitted);
        }
        self.entries.insert(
            operation.operation_id.clone(),
            SetupEntry {
                operation: operation.clone(),
                outcome: NativeSetupObservation::PossibleForwarding,
                connection_id: connection_id.map(str::to_owned),
                external_tools_applied: false,
            },
        );
        Ok(None)
    }
    pub(super) fn completed(
        &mut self,
        id: &str,
        outcome: NativeSetupObservation,
    ) -> Result<(), Error> {
        self.entries
            .get_mut(id)
            .ok_or(Error::OperationConflict)?
            .outcome = outcome;
        Ok(())
    }
    pub(super) fn recover(
        &self,
        operation: &NativeSetupOperation,
    ) -> Result<Option<NativeSetupObservation>, Error> {
        let Some(entry) = self.entries.get(&operation.operation_id) else {
            return Ok(None);
        };
        if &entry.operation != operation {
            return Err(Error::OperationConflict);
        }
        Ok(Some(entry.outcome.clone()))
    }
    pub(super) fn observe(
        &self,
        request: &NativeSetupRequest,
    ) -> Result<NativeSetupRecovery, Error> {
        valid_id(&request.operation_id)?;
        let Some(entry) = self.entries.get(&request.operation_id) else {
            return Ok(NativeSetupRecovery {
                outcome: NativeSetupObservation::Unseen,
                connection_id: None,
                external_tools_applied: false,
            });
        };
        entry.operation.check_request(request)?;
        Ok(NativeSetupRecovery {
            outcome: entry.outcome.clone(),
            connection_id: entry.connection_id.clone(),
            external_tools_applied: entry.external_tools_applied,
        })
    }
}
fn valid_id(id: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(Error::NotAdmitted);
    }
    Ok(())
}
impl State {
    /// Preserve any existing relay and record only setup evidence.
    pub(super) async fn try_reuse_setup(
        &mut self,
        operation: &NativeSetupOperation,
        tools: &dyn crate::NativeToolsProvider,
        observe_only: bool,
    ) -> Result<Option<NativeSetupObservation>, Error> {
        let (Some(route), Some(connection)) = (tools.setup_route(), tools.setup_connection_id())
        else {
            return Ok(None);
        };
        if !route.is_open()
            || !tools.has_pending_external()
            || (!observe_only && !self.can_reuse_setup(operation))
        {
            return Ok(None);
        }
        let NativeSetupIntent::Existing(target) = &operation.intent else {
            return Ok(None);
        };
        let applied = if observe_only {
            None
        } else {
            let provision = self
                .live
                .as_ref()
                .and_then(|live| live.tools.as_ref())
                .ok_or(Error::NotAdmitted)?;
            match (provision.external_supply(), provision.applied_external()) {
                (None, None) => None,
                (Some(supply), Some(preparation)) if preparation.connection_id == connection => {
                    Some((preparation, supply))
                }
                _ => return Ok(None),
            }
        };
        if !tools
            .confirm_external_reuse(target, applied, observe_only)
            .await
            .map_err(|_| Error::DeliveryUncertain)?
        {
            return Ok(None);
        }
        route.publish(|| {
            if let Some(previous) = self.setups.begin(operation, Some(connection))? {
                return Ok(Some(previous));
            }
            let ready = NativeSetupObservation::Ready(target.clone());
            self.setups
                .completed(&operation.operation_id, ready.clone())?;
            Ok(Some(ready))
        })
    }

    pub(super) fn can_reuse_setup(&self, operation: &NativeSetupOperation) -> bool {
        let NativeSetupIntent::Existing(target) = &operation.intent else {
            return false;
        };
        !self.blocked
            && self.live.as_ref().is_some_and(|live| {
                !live.ended.is_cancelled()
                    && live.instructions == operation.instructions
                    && live.tool_catalog == operation.tool_catalog
                    && live.requested_supply == operation.supply
                    && operation.scope.matches(&live._authentication)
                    && live
                        .capture
                        .snapshot()
                        .is_ok_and(|snapshot| snapshot == target.snapshot)
                    && live
                        .capture
                        .native_session()
                        .is_ok_and(|native| native.as_ref() == Some(&target.native_session))
                    && live.session.as_ref().is_some_and(|session| {
                        matches!(session, Session::AppServer(_))
                            && !session.is_closed()
                            && session.id() == target.native_session.as_str()
                    })
                    && live.tools.as_ref().is_some()
                    && live.observations.current().is_ok_and(|observed| {
                        matches!(
                            observed.foreground,
                            NativeForeground::Idle | NativeForeground::Unknown
                        )
                    })
                    && self.refresh_effects_settled(live)
            })
    }

    pub(super) async fn setup(
        &mut self,
        operation: NativeSetupOperation,
        authentication: Arc<dyn NativeAuthenticationProvider>,
        tools: Arc<dyn crate::NativeToolsProvider>,
    ) -> Result<NativeSetupObservation, Error> {
        valid_id(&operation.scope.agent_id)?;
        valid_id(&operation.scope.conversation_id)?;
        operation
            .instructions
            .validate()
            .map_err(|_| Error::NotAdmitted)?;
        if operation.tool_catalog.len() > 48 * 1024 {
            return Err(Error::NotAdmitted);
        }
        if let Some(previous) = self.setups.recover(&operation)? {
            return Ok(previous);
        }
        if !setup_route_open(tools.as_ref())
            || self.requires_idle_application(&operation, tools.as_ref()) && !self.refresh_safe()
        {
            return Err(Error::DeliveryUncertain);
        }
        // Do not resolve optional settings while Setup is deferred behind
        // another turn. Only the configuration owner selects at its cut.
        let tools = if self.live.is_some() && self.optional_setup_allowed() {
            tools
                .resolve_external()
                .await
                .map_err(Error::ToolsProvision)?
                .unwrap_or(tools)
        } else {
            tools
        };
        let started = match tools.setup_route() {
            Some(route) => {
                route.publish(|| self.setups.begin(&operation, tools.setup_connection_id()))
            }
            None => self.setups.begin(&operation, None),
        };
        if let Some(previous) = started? {
            return Ok(previous);
        }
        let outcome = match self.setup_once(&operation, authentication, tools).await {
            Ok(target) => {
                let entry = self
                    .setups
                    .entries
                    .get_mut(&operation.operation_id)
                    .ok_or(Error::OperationConflict)?;
                entry.external_tools_applied = self
                    .live
                    .as_ref()
                    .and_then(|live| live.tools.as_ref())
                    .and_then(crate::NativeToolProvision::applied_external)
                    .is_some_and(|applied| {
                        applied.operation_id == operation.operation_id
                            && entry.connection_id.as_deref()
                                == Some(applied.connection_id.as_str())
                    });
                NativeSetupObservation::Ready(target)
            }
            Err(error) => NativeSetupObservation::Failed(match error {
                Error::NotAdmitted
                | Error::UnsupportedRuntime
                | Error::Authentication
                | Error::OperationConflict => NativeSetupFailure::NotAdmitted,
                Error::Storage
                | Error::CaptureUnavailable
                | Error::CaptureCorrupt
                | Error::CaptureConflict => NativeSetupFailure::SourceUnavailable,
                Error::Transient | Error::Integrity | Error::Incompatible => {
                    NativeSetupFailure::BeforeNativeLaunch
                }
                Error::ToolsProvision(_) => NativeSetupFailure::BeforeNativeLaunch,
                Error::ToolsVerification(_) => NativeSetupFailure::NativeRejected,
                Error::ToolsCleanup(_) | Error::CleanupFailed { .. } => {
                    NativeSetupFailure::RetirementUnknown
                }
                Error::ProtocolRejected { .. } => NativeSetupFailure::NativeRejected,
                Error::RetirementUnknown | Error::StartAndRetirement { .. } => {
                    NativeSetupFailure::RetirementUnknown
                }
                _ => {
                    return Err(Error::DeliveryUncertain);
                }
            }),
        };
        self.setups
            .completed(&operation.operation_id, outcome.clone())?;
        Ok(outcome)
    }
    async fn setup_once(
        &mut self,
        operation: &NativeSetupOperation,
        authentication: Arc<dyn NativeAuthenticationProvider>,
        tools: Arc<dyn crate::NativeToolsProvider>,
    ) -> Result<NativeTextTarget, Error> {
        if tools
            .external_preparation()
            .is_some_and(|preparation| preparation.operation_id != operation.operation_id)
        {
            return Err(Error::OperationConflict);
        }
        let safe_retirement = self.needs_refresh(operation, tools.as_ref());
        if self.requires_idle_application(operation, tools.as_ref()) && !self.refresh_safe() {
            return Err(Error::NotAdmitted);
        }
        if safe_retirement {
            let live = self.live.as_ref().ok_or(Error::NotAdmitted)?;
            if !operation.scope.matches(&live._authentication) {
                return Err(Error::OperationConflict);
            }
            let _ = retained_context(&operation.intent, &live.capture)?;
            self.retire().await?;
        }
        let retained = self.captured_source().await?;
        let source = match retained {
            Some(source) => source,
            None => {
                if !matches!(
                    operation.intent,
                    NativeSetupIntent::Initial {
                        allow_fresh: true,
                        require_retained_source: false
                    }
                ) {
                    return Err(Error::CaptureUnavailable);
                }
                let storage = self.storage.clone();
                let source =
                    tokio::task::spawn_blocking(move || crate::CaptureLog::create(&storage))
                        .await
                        .map_err(|_| Error::Process)??;
                self.retained_capture = Some(source.clone());
                source
            }
        };
        let snapshot = source.snapshot()?;
        let context = retained_context(&operation.intent, &source)?;
        let empty_scope = matches!(&context, NativeContext::New) || safe_retirement;
        if self.live.is_none() {
            let storage = self.storage.clone();
            let document = operation.instructions.document();
            self.project_write = Some(tokio::task::spawn_blocking(move || {
                storage.publish_project_instructions(&document)
            }));
            let published = self.project_write.as_mut().ok_or(Error::Storage)?.await;
            self.project_write = None;
            published.map_err(|_| Error::Storage)??;
        }
        let native_session = self
            .start(
                operation.supply.clone(),
                context,
                source,
                authentication,
                tools,
                StartConfiguration {
                    scope: &operation.scope,
                    instructions: &operation.instructions,
                    tool_catalog: &operation.tool_catalog,
                    empty_scope,
                },
            )
            .await?;
        Ok(NativeTextTarget {
            snapshot,
            native_session,
        })
    }
    pub(super) fn needs_refresh(
        &self,
        operation: &NativeSetupOperation,
        tools: &dyn crate::NativeToolsProvider,
    ) -> bool {
        self.live.as_ref().is_some_and(|live| {
            live.instructions != operation.instructions
                || live.tool_catalog != operation.tool_catalog
                || live.requested_supply != operation.supply
                || self.optional_setup_allowed()
                    && live
                        .tools
                        .as_ref()
                        .and_then(crate::NativeToolProvision::external_supply)
                        != tools.external_supply()
        })
    }
    pub(super) fn requires_idle_application(
        &self,
        operation: &NativeSetupOperation,
        tools: &dyn crate::NativeToolsProvider,
    ) -> bool {
        self.needs_refresh(operation, tools)
            || self.optional_setup_allowed()
                && (self.live.is_some() && tools.has_pending_external()
                    || self.has_live_external_supply() && tools.external_preparation().is_some())
    }
    pub(super) fn optional_setup_allowed(&self) -> bool {
        !matches!(
            self.live.as_ref().and_then(|live| live.session.as_ref()),
            Some(Session::Acp(_))
        )
    }
    pub(super) fn has_live_external_supply(&self) -> bool {
        self.live
            .as_ref()
            .and_then(|live| live.tools.as_ref())
            .and_then(crate::NativeToolProvision::external_supply)
            .is_some()
    }
    pub(super) fn refresh_safe(&self) -> bool {
        self.live.as_ref().is_none_or(|live| {
            live.observations
                .current()
                .is_ok_and(|observed| observed.foreground == NativeForeground::Idle)
                && live.observations.quiescent()
                && self.refresh_effects_settled(live)
        })
    }

    fn refresh_effects_settled(&self, live: &Live) -> bool {
        (live.tools.is_none()
            || live
                .observations
                .pending_ending()
                .is_ok_and(|ending| ending.is_none()))
            && live
                .requests
                .try_lock()
                .is_ok_and(|requests| requests.is_empty())
            && self.inputs.is_empty()
            && live
                .tools
                .as_ref()
                .is_none_or(crate::NativeToolProvision::quiescent)
    }
}
fn retained_context(
    intent: &NativeSetupIntent,
    source: &crate::CaptureLog,
) -> Result<NativeContext, Error> {
    let snapshot = source.snapshot()?;
    let native = source.native_session()?;
    Ok(match (intent, native) {
        (NativeSetupIntent::Existing(expected), Some(session))
            if expected.snapshot == snapshot && expected.native_session == session =>
        {
            NativeContext::Resume {
                session,
                replay: CaptureReplay::AlreadyCaptured,
            }
        }
        (NativeSetupIntent::Initial { .. }, Some(session)) => NativeContext::Resume {
            session,
            replay: CaptureReplay::AlreadyCaptured,
        },
        (
            NativeSetupIntent::Initial {
                allow_fresh: true, ..
            },
            None,
        ) if source.is_empty()? => NativeContext::New,
        _ => return Err(Error::CaptureUnavailable),
    })
}
impl NativeHandle {
    /// Queue admission returns before preparation; later lookup reads actor-owned facts.
    pub fn admit_setup_with_authentication(
        &self,
        operation: NativeSetupOperation,
        authentication: Arc<dyn NativeAuthenticationProvider>,
        tools: Arc<dyn crate::NativeToolsProvider>,
    ) -> Result<tokio::sync::oneshot::Receiver<Result<NativeSetupObservation, Error>>, Error> {
        valid_id(&operation.operation_id)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::Setup(operation, authentication, tools, result))
            .map_err(|_| Error::NotAdmitted)?;
        Ok(rx)
    }
    /// Recover a retained result without touching authentication, tools, or the live owner.
    pub async fn setup_observation(
        &self,
        request: NativeSetupRequest,
    ) -> Result<NativeSetupRecovery, Error> {
        valid_id(&request.operation_id)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::SetupObservation(request, result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn request(operation: &NativeSetupOperation) -> NativeSetupRequest {
        NativeSetupRequest {
            operation_id: operation.operation_id.clone(),
            scope: operation.scope.clone(),
            supply: operation.supply.clone(),
            intent: operation.intent.clone(),
        }
    }
    fn tool_scope() -> crate::NativeToolScope {
        crate::NativeToolScope {
            agent_id: "agent".into(),
            conversation_id: "conversation".into(),
        }
    }

    struct UnusedAuthentication(Arc<AtomicUsize>);
    impl NativeAuthenticationProvider for UnusedAuthentication {
        fn acquire(
            &self,
            _: chapchoo_domain::LaunchCapture,
        ) -> crate::NativeAuthenticationFuture<'_, crate::NativeAuthentication> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(Error::Authentication) })
        }
        fn check<'a>(
            &'a self,
            _binding: &'a NativeAuthenticationBinding,
        ) -> crate::NativeAuthenticationFuture<'a, ()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Err(Error::Authentication) })
        }
    }
    #[test]
    fn only_the_exact_known_setup_result_can_release_retained_capacity() -> Result<(), Error> {
        let mut registry = SetupRegistry::default();
        let operation = NativeSetupOperation {
            operation_id: "operation".into(),
            scope: tool_scope(),
            supply: HarnessSupply::Managed,
            instructions: Default::default(),
            tool_catalog: Vec::new(),
            intent: NativeSetupIntent::Initial {
                allow_fresh: true,
                require_retained_source: false,
            },
        };
        registry.begin(&operation, None)?;
        let rejected = NativeSetupObservation::Failed(NativeSetupFailure::NotAdmitted);
        assert!(registry.release_reported("operation", &rejected).is_err());
        registry.completed("operation", rejected.clone())?;
        assert!(
            registry
                .release_reported(
                    "operation",
                    &NativeSetupObservation::Failed(NativeSetupFailure::NativeRejected)
                )
                .is_err()
        );
        registry.release_reported("operation", &rejected)?;
        registry.release_reported("operation", &rejected)?;
        assert!(matches!(
            registry.observe(&request(&operation))?.outcome,
            NativeSetupObservation::Unseen
        ));
        Ok(())
    }
    type TestError = Box<dyn std::error::Error>;
    fn operation(id: &str) -> NativeSetupOperation {
        NativeSetupOperation {
            operation_id: id.into(),
            scope: tool_scope(),
            supply: HarnessSupply::Managed,
            instructions: Default::default(),
            tool_catalog: Vec::new(),
            intent: NativeSetupIntent::Initial {
                allow_fresh: true,
                require_retained_source: false,
            },
        }
    }
    #[test]
    fn expired_setup_overflow_rejects_only_that_request_and_preserves_uncertainty()
    -> Result<(), TestError> {
        let mut registry = SetupRegistry::default();
        for index in 0..256 {
            registry.begin(&operation(&format!("pending-{index}")), None)?;
        }
        assert!(matches!(
            registry.expire_unstarted(&operation("expired")),
            Err(Error::NotAdmitted)
        ));
        assert_eq!(registry.entries.len(), 256);
        for index in 0..256 {
            assert!(matches!(
                registry
                    .observe(&request(&operation(&format!("pending-{index}"))))?
                    .outcome,
                NativeSetupObservation::PossibleForwarding
            ));
        }
        assert!(matches!(
            registry.expire_unstarted(&operation("pending-0"))?,
            NativeSetupObservation::PossibleForwarding
        ));
        let settled = NativeSetupObservation::Failed(NativeSetupFailure::BeforeNativeLaunch);
        registry.completed("pending-0", settled.clone())?;
        registry.release_reported("pending-0", &settled)?;
        assert!(matches!(
            registry.expire_unstarted(&operation("expired"))?,
            NativeSetupObservation::Failed(NativeSetupFailure::NotAdmitted)
        ));
        Ok(())
    }

    #[test]
    fn duplicate_setup_recovers_original_evidence_and_changed_intent_conflicts()
    -> Result<(), TestError> {
        let mut registry = SetupRegistry::default();
        let first = operation("operation");
        assert!(registry.begin(&first, Some("original"))?.is_none());
        assert!(matches!(
            registry.begin(&first, Some("replacement"))?,
            Some(NativeSetupObservation::PossibleForwarding)
        ));
        let changed = NativeSetupOperation {
            intent: NativeSetupIntent::Initial {
                allow_fresh: false,
                require_retained_source: false,
            },
            ..first.clone()
        };
        assert!(matches!(
            registry.begin(&changed, Some("replacement")),
            Err(Error::OperationConflict)
        ));
        let changed_scope = NativeSetupOperation {
            scope: crate::NativeToolScope {
                agent_id: "another-agent".into(),
                ..tool_scope()
            },
            ..first.clone()
        };
        assert!(matches!(
            registry.begin(&changed_scope, Some("replacement")),
            Err(Error::OperationConflict)
        ));
        registry.completed(
            &first.operation_id,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable),
        )?;
        assert!(matches!(
            registry.begin(&first, Some("replacement"))?,
            Some(NativeSetupObservation::Failed(
                NativeSetupFailure::SourceUnavailable
            ))
        ));
        assert_eq!(
            registry.observe(&request(&first))?.connection_id.as_deref(),
            Some("original")
        );
        assert!(matches!(
            registry.observe(&request(&operation("missing")))?.outcome,
            NativeSetupObservation::Unseen
        ));
        Ok(())
    }
    #[tokio::test]
    async fn missing_resume_source_does_not_create_a_fresh_context() -> Result<(), TestError> {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let owner = NativeOwner::new(storage.clone())?;
        let handle = owner.handle();
        let calls = Arc::new(AtomicUsize::new(0));
        let authentication: Arc<dyn NativeAuthenticationProvider> =
            Arc::new(UnusedAuthentication(calls.clone()));
        let operation = NativeSetupOperation {
            intent: NativeSetupIntent::Initial {
                allow_fresh: false,
                require_retained_source: false,
            },
            ..operation("resume-only")
        };
        let rx = handle.admit_setup_with_authentication(
            operation.clone(),
            authentication.clone(),
            crate::server_tools::unused_tools(),
        )?;
        drop(rx);
        assert!(matches!(
            handle.setup_observation(request(&operation)).await?.outcome,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable)
        ));
        assert!(!storage.capture_root().join("current").exists());
        assert!(matches!(
            handle
                .admit_setup_with_authentication(
                    operation,
                    authentication,
                    crate::server_tools::unused_tools()
                )?
                .await??,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable)
        ));
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        Ok(())
    }
    #[tokio::test]
    async fn recovery_grant_cannot_replace_a_missing_retained_capture() -> Result<(), TestError> {
        let saved = tempfile::tempdir()?;
        let storage = NativeStorage::open(saved.path())?;
        let capture = crate::CaptureLog::create(&storage)?;
        let intent = NativeSetupIntent::Initial {
            allow_fresh: true,
            require_retained_source: true,
        };
        assert!(matches!(
            retained_context(&intent, &capture)?,
            NativeContext::New
        ));
        let session = NativeConversationId::parse("existing-native-context".into())?;
        capture.bind_native(&session)?;
        assert!(
            matches!(retained_context(&intent, &capture)?, NativeContext::Resume { session: found, .. } if found == session)
        );
        drop(capture);
        fs::remove_dir_all(storage.capture_root().join("current"))?;
        let owner = NativeOwner::new(storage.clone())?;
        let calls = Arc::new(AtomicUsize::new(0));
        let authentication: Arc<dyn NativeAuthenticationProvider> =
            Arc::new(UnusedAuthentication(calls.clone()));
        let result = owner
            .handle()
            .admit_setup_with_authentication(
                NativeSetupOperation {
                    operation_id: "recovery-after-source-loss".into(),
                    scope: tool_scope(),
                    supply: HarnessSupply::Managed,
                    instructions: Default::default(),
                    tool_catalog: Vec::new(),
                    intent,
                },
                authentication,
                crate::server_tools::unused_tools(),
            )?
            .await??;
        assert!(matches!(
            result,
            NativeSetupObservation::Failed(NativeSetupFailure::SourceUnavailable)
        ));
        assert!(!storage.capture_root().join("current").exists());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(owner.finish().await?, Retirement::NoNativeStarted);
        Ok(())
    }
}
