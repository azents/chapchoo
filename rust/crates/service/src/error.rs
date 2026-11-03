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

    fn capture(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl fmt::Debug for InfrastructureReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InfrastructureReport([REDACTED])")
    }
}

pub struct ServiceError {
    failure: ServiceFailure,
}

enum ServiceFailure {
    Rejected(DomainReason),
    Infrastructure(InfrastructureReport),
    DeliveryUncertain(InfrastructureReport),
}

impl ServiceError {
    pub fn domain(reason: DomainReason) -> Self {
        Self {
            failure: ServiceFailure::Rejected(reason),
        }
    }

    pub fn infrastructure(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            failure: ServiceFailure::Infrastructure(InfrastructureReport::capture(source)),
        }
    }

    pub(crate) fn delivery(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            failure: ServiceFailure::DeliveryUncertain(InfrastructureReport::capture(source)),
        }
    }

    pub fn reason(&self) -> DomainReason {
        match &self.failure {
            ServiceFailure::Rejected(reason) => *reason,
            ServiceFailure::Infrastructure(_) | ServiceFailure::DeliveryUncertain(_) => {
                DomainReason::Internal
            }
        }
    }
    pub fn report(&self) -> Option<&InfrastructureReport> {
        match &self.failure {
            ServiceFailure::Rejected(_) => None,
            ServiceFailure::Infrastructure(report) | ServiceFailure::DeliveryUncertain(report) => {
                Some(report)
            }
        }
    }
    pub fn delivery_uncertain(&self) -> bool {
        matches!(self.failure, ServiceFailure::DeliveryUncertain(_))
    }
}

impl fmt::Debug for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServiceError")
            .field("reason", &self.reason())
            .field("has_report", &self.report().is_some())
            .field("delivery_uncertain", &self.delivery_uncertain())
            .finish()
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = if self.delivery_uncertain() {
            "Email delivery could not be confirmed after commit"
        } else {
            match self.reason() {
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
        self.report()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_rejections_keep_their_reason_without_an_incident_or_delivery_claim() {
        for reason in [
            DomainReason::InvalidInput,
            DomainReason::InvalidCredentials,
            DomainReason::InvalidSession,
            DomainReason::PermissionDenied,
            DomainReason::NotFound,
            DomainReason::Conflict,
            DomainReason::InvalidTransition,
            DomainReason::InvalidProof,
            DomainReason::EmailMismatch,
            DomainReason::EmailRegistered,
            DomainReason::WeakPassword,
            DomainReason::InvalidCode,
            DomainReason::RegistrationRequired,
            DomainReason::AlreadyMember,
            DomainReason::AlreadyProcessed,
            DomainReason::OwnerTransferRequired,
            DomainReason::HandleConflict,
            DomainReason::MailUnavailable,
            DomainReason::CapacityExhausted,
            DomainReason::Internal,
        ] {
            let error = ServiceError::domain(reason);
            assert_eq!(error.reason(), reason);
            assert!(error.report().is_none());
            assert!(error.source().is_none());
            assert!(!error.delivery_uncertain());
            assert_eq!(
                format!("{error:?}"),
                format!(
                    "ServiceError {{ reason: {reason:?}, has_report: false, delivery_uncertain: false }}"
                )
            );
        }
        assert_eq!(
            ServiceError::domain(DomainReason::Internal).to_string(),
            "Service operation failed"
        );
    }

    #[test]
    fn incident_variants_retain_private_causes_but_preserve_safe_public_observations()
    -> Result<(), Box<dyn Error>> {
        let private = "Synthetic private native failure";
        for delivery in [false, true] {
            let source = std::io::Error::new(std::io::ErrorKind::ConnectionReset, private);
            let error = if delivery {
                ServiceError::delivery(source)
            } else {
                ServiceError::infrastructure(source)
            };
            assert_eq!(error.reason(), DomainReason::Internal);
            assert_eq!(error.delivery_uncertain(), delivery);
            assert_eq!(
                error.to_string(),
                if delivery {
                    "Email delivery could not be confirmed after commit"
                } else {
                    "Service operation failed"
                }
            );
            let report = error.report().ok_or("Missing incident report")?;
            assert_eq!(
                report.backtrace().status(),
                std::backtrace::BacktraceStatus::Captured
            );
            assert_eq!(report.source_error().to_string(), private);
            assert_eq!(
                error
                    .source()
                    .and_then(|source| source.downcast_ref::<std::io::Error>())
                    .ok_or("Missing original error")?
                    .kind(),
                std::io::ErrorKind::ConnectionReset
            );
            assert_eq!(
                format!("{error:?}"),
                format!(
                    "ServiceError {{ reason: Internal, has_report: true, delivery_uncertain: {delivery} }}"
                )
            );
            assert!(!error.to_string().contains(private));
            assert_eq!(format!("{report:?}"), "InfrastructureReport([REDACTED])");
        }
        Ok(())
    }
}
