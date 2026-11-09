//! Execution-local native preparation and physical ownership. No server authority.
mod preparation;
mod runtime;
mod storage;

pub use preparation::{AppliedSupply, Freshness, Preparer};
pub use runtime::{
    InteractiveRequest, InteractiveResponse, NativeOwner, NativePromptAcceptance, Retirement,
    ScopeEvidence, SessionEventRecorder,
};
pub use storage::NativeStorage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolStage {
    Initialize,
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
    #[error("Native predecessor retirement is unconfirmed")]
    RetirementUnknown,
    #[error("Native start failed and predecessor retirement is unconfirmed")]
    StartAndRetirement { primary: Box<Error> },
    #[error("Native process operation failed")]
    Process,
}
