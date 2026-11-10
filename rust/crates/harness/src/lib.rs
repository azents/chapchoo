//! Execution-local native preparation and physical ownership. No server authority.
mod authentication;
mod capture;
mod preparation;
mod runtime;
mod storage;

pub use authentication::{
    NativeAuthentication, NativeAuthenticationBinding, NativeAuthenticationFuture,
    NativeAuthenticationProvider, NativeCredentials,
};
pub use capture::{CaptureBoundary, CaptureLog, CapturePass, CapturedEvent};
pub use preparation::{
    AppliedSupply, Freshness, NativeComponentVersions, NativePreparationObservation, Preparer,
};
pub use runtime::{
    CaptureReplay, InteractiveRequest, InteractiveResponse, NativeAnswerObservation,
    NativeAnswerOperation, NativeCaptureReader, NativeContext, NativeForeground, NativeHandle,
    NativeInputObservation, NativeInputOperation, NativeObservation, NativeOwner,
    NativePromptAcceptance, NativeReportedOperation, NativeSetupFailure, NativeSetupIntent,
    NativeSetupObservation, NativeSetupOperation, NativeStopObservation, NativeTextTarget,
    NativeWorkTarget, Retirement, ScopeEvidence,
};
pub use storage::NativeStorage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolStage {
    Initialize,
    Authentication,
    Session,
}

/// Body-safe classifications. Native output and paths are not error messages.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Native storage is unavailable or outside its allowed boundary")]
    Storage,
    #[error("Native runtime does not satisfy the supported restricted profile")]
    UnsupportedRuntime,
    #[error("Native acquisition temporarily failed")]
    Transient,
    #[error("Native artifact integrity or layout is invalid")]
    Integrity,
    #[error("Native supply is incompatible")]
    Incompatible,
    #[error("Native protocol operation failed")]
    Protocol,
    #[error("Native protocol rejected {stage:?} (code {code})")]
    ProtocolRejected { stage: ProtocolStage, code: i32 },
    #[error("Native operation was not admitted")]
    NotAdmitted,
    #[error("Native authentication material is invalid or unavailable")]
    Authentication,
    #[error("The native operation conflicts with its original payload or target")]
    OperationConflict,
    #[error("Native delivery may have occurred and cannot be replayed")]
    DeliveryUncertain,
    #[error("Native predecessor retirement is unconfirmed")]
    RetirementUnknown,
    #[error("Native start failed and predecessor retirement is unconfirmed")]
    StartAndRetirement { primary: Box<Error> },
    #[error("Native process operation failed")]
    Process,
    #[error("The captured source or requested cursor is unavailable")]
    CaptureUnavailable,
    #[error("The captured source is incomplete or malformed")]
    CaptureCorrupt,
    #[error("The captured source conflicts with its exact binding or occurrence")]
    CaptureConflict,
}
