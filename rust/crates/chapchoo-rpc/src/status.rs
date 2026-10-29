use chapchoo_protocol::accounts::DomainReason as WireReason;
use chapchoo_service::{DomainReason, ServiceError};
use tonic::{Code, Status, metadata::MetadataValue};

/// Canonical generated enum names, sent only on rejected RPCs.
pub const DOMAIN_REASON_METADATA: &str = "chapchoo-domain-reason";

pub(crate) fn reject(reason: DomainReason) -> Status {
    status(ServiceError::domain(reason))
}

pub(crate) fn status(error: ServiceError) -> Status {
    // Dependency Display/source messages can contain SQL, addresses or credentials.
    // Preserve the source in ServiceError, but report only the safe category and
    // captured stack here. Do not format the original error chain.
    if let Some(report) = error.report() {
        tracing::error!(
            reason = ?error.reason(),
            delivery_uncertain = error.delivery_uncertain(),
            backtrace = %report.backtrace(),
            "Application infrastructure failure"
        );
    }
    let reason = wire_reason(error.reason());
    let code = match error.reason() {
        DomainReason::InvalidCredentials
        | DomainReason::InvalidSession
        | DomainReason::InvalidCode => Code::Unauthenticated,
        DomainReason::PermissionDenied => Code::PermissionDenied,
        DomainReason::NotFound => Code::NotFound,
        DomainReason::InvalidInput
        | DomainReason::InvalidProof
        | DomainReason::EmailMismatch
        | DomainReason::WeakPassword => Code::InvalidArgument,
        DomainReason::Conflict
        | DomainReason::EmailRegistered
        | DomainReason::AlreadyMember
        | DomainReason::HandleConflict => Code::AlreadyExists,
        DomainReason::InvalidTransition
        | DomainReason::RegistrationRequired
        | DomainReason::AlreadyProcessed
        | DomainReason::OwnerTransferRequired => Code::FailedPrecondition,
        DomainReason::MailUnavailable => {
            if error.report().is_some() {
                Code::Unavailable
            } else {
                Code::FailedPrecondition
            }
        }
        DomainReason::CapacityExhausted => Code::ResourceExhausted,
        DomainReason::Internal => Code::Internal,
    };
    let mut result = Status::new(code, error.to_string());
    // The generated enum's fixed names are valid ASCII metadata values.
    result.metadata_mut().insert(
        DOMAIN_REASON_METADATA,
        MetadataValue::from_static(reason.as_str_name()),
    );
    result
}

fn wire_reason(reason: DomainReason) -> WireReason {
    match reason {
        DomainReason::InvalidInput => WireReason::InvalidInput,
        DomainReason::InvalidCredentials => WireReason::InvalidCredentials,
        DomainReason::InvalidSession => WireReason::InvalidSession,
        DomainReason::PermissionDenied => WireReason::PermissionDenied,
        DomainReason::NotFound => WireReason::NotFound,
        DomainReason::Conflict => WireReason::Conflict,
        DomainReason::InvalidTransition => WireReason::InvalidTransition,
        DomainReason::InvalidProof => WireReason::InvalidProof,
        DomainReason::EmailMismatch => WireReason::EmailMismatch,
        DomainReason::EmailRegistered => WireReason::EmailRegistered,
        DomainReason::WeakPassword => WireReason::WeakPassword,
        DomainReason::InvalidCode => WireReason::InvalidCode,
        DomainReason::RegistrationRequired => WireReason::RegistrationRequired,
        DomainReason::AlreadyMember => WireReason::AlreadyMember,
        DomainReason::AlreadyProcessed => WireReason::AlreadyProcessed,
        DomainReason::OwnerTransferRequired => WireReason::OwnerTransferRequired,
        DomainReason::HandleConflict => WireReason::HandleConflict,
        DomainReason::MailUnavailable => WireReason::MailUnavailable,
        DomainReason::CapacityExhausted => WireReason::CapacityExhausted,
        DomainReason::Internal => WireReason::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_has_typed_reason_and_sanitized_message() {
        let failure = status(ServiceError::infrastructure(std::io::Error::other(
            "password=secret SQL connection-string",
        )));
        assert_eq!(failure.code(), Code::Internal);
        assert!(!failure.message().contains("secret"));
        assert_eq!(
            failure
                .metadata()
                .get(DOMAIN_REASON_METADATA)
                .and_then(|v| v.to_str().ok()),
            Some("DOMAIN_REASON_INTERNAL")
        );
        assert_eq!(
            reject(DomainReason::OwnerTransferRequired).code(),
            Code::FailedPrecondition
        );
        assert_eq!(
            reject(DomainReason::InvalidCredentials).code(),
            Code::Unauthenticated
        );
    }
}
