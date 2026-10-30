use std::{backtrace::Backtrace, error::Error, fmt};

use chapchoo_persistence::StoreError;

/// Transport-independent canonical rejection reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainReason {
    InvalidInput,
    InvalidCredentials,
    InvalidSession,
    PermissionDenied,
    NotFound,
    Conflict,
    InvalidTransition,
    InvalidProof,
    EmailMismatch,
    EmailRegistered,
    WeakPassword,
    InvalidCode,
    RegistrationRequired,
    AlreadyMember,
    AlreadyProcessed,
    OwnerTransferRequired,
    HandleConflict,
    MailUnavailable,
    CapacityExhausted,
    Internal,
}

pub struct InfrastructureReport {
    source: Box<dyn Error + Send + Sync>,
    backtrace: Backtrace,
}

impl InfrastructureReport {
    pub fn source_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self.source.as_ref()
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Debug for InfrastructureReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InfrastructureReport([REDACTED])")
    }
}

pub struct ServiceError {
    reason: DomainReason,
    report: Option<InfrastructureReport>,
    delivery_uncertain: bool,
}

impl ServiceError {
    pub fn domain(reason: DomainReason) -> Self {
        Self {
            reason,
            report: None,
            delivery_uncertain: false,
        }
    }

    pub fn infrastructure(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            reason: DomainReason::Internal,
            report: Some(InfrastructureReport {
                source: Box::new(source),
                backtrace: Backtrace::force_capture(),
            }),
            delivery_uncertain: false,
        }
    }

    pub(crate) fn delivery(source: impl Error + Send + Sync + 'static) -> Self {
        let mut error = Self::infrastructure(source);
        error.delivery_uncertain = true;
        error
    }

    pub fn reason(&self) -> DomainReason {
        self.reason
    }
    pub fn report(&self) -> Option<&InfrastructureReport> {
        self.report.as_ref()
    }
    pub fn delivery_uncertain(&self) -> bool {
        self.delivery_uncertain
    }
}

impl fmt::Debug for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServiceError")
            .field("reason", &self.reason)
            .field("has_report", &self.report.is_some())
            .field("delivery_uncertain", &self.delivery_uncertain)
            .finish()
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = if self.delivery_uncertain {
            "Email delivery could not be confirmed after commit"
        } else {
            match self.reason {
                DomainReason::Internal => "Service operation failed",
                DomainReason::InvalidCredentials => "Invalid credentials",
                DomainReason::InvalidSession => "Invalid session",
                DomainReason::PermissionDenied => "Permission denied",
                DomainReason::NotFound => "Object not found",
                DomainReason::InvalidInput => "Invalid input",
                DomainReason::Conflict => "Conflicting existing resource",
                DomainReason::InvalidTransition => "Invalid current transition",
                DomainReason::InvalidProof => "Invalid or unusable proof",
                DomainReason::EmailMismatch => "Email does not match the proof",
                DomainReason::EmailRegistered => "Email is already registered",
                DomainReason::WeakPassword => "Password does not meet the policy",
                DomainReason::InvalidCode => "Invalid verification code",
                DomainReason::RegistrationRequired => {
                    "Registration through a signup link is required"
                }
                DomainReason::AlreadyMember => "User is already a member",
                DomainReason::AlreadyProcessed => "Invitation was already processed",
                DomainReason::OwnerTransferRequired => {
                    "Ownership transfer is required before leaving"
                }
                DomainReason::HandleConflict => "Team handle is already in use",
                DomainReason::MailUnavailable => "Email delivery is not configured",
                DomainReason::CapacityExhausted => "Password processing capacity is exhausted",
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.report
            .as_ref()
            .map(|report| report.source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<StoreError> for ServiceError {
    fn from(error: StoreError) -> Self {
        let reason = match error {
            StoreError::Unauthenticated => DomainReason::InvalidSession,
            StoreError::Forbidden => DomainReason::PermissionDenied,
            StoreError::NotFound => DomainReason::NotFound,
            StoreError::Conflict => DomainReason::Conflict,
            StoreError::InvalidTransition => DomainReason::InvalidTransition,
            StoreError::InvalidInput => DomainReason::InvalidInput,
            StoreError::InvalidProof => DomainReason::InvalidProof,
            StoreError::RegistrationRequired => DomainReason::RegistrationRequired,
            StoreError::EmailMismatch => DomainReason::EmailMismatch,
            StoreError::EmailRegistered => DomainReason::EmailRegistered,
            StoreError::AlreadyMember => DomainReason::AlreadyMember,
            StoreError::AlreadyProcessed => DomainReason::AlreadyProcessed,
            StoreError::OwnerTransferRequired => DomainReason::OwnerTransferRequired,
            StoreError::HandleConflict => DomainReason::HandleConflict,
            infrastructure @ (StoreError::Pool(_)
            | StoreError::Database(_)
            | StoreError::Preparation(_)
            | StoreError::Task(_)
            | StoreError::Crypto(_)
            | StoreError::Integrity) => return Self::infrastructure(infrastructure),
        };
        Self::domain(reason)
    }
}
