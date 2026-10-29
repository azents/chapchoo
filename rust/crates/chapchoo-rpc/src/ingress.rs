use std::time::{SystemTime, UNIX_EPOCH};

use chapchoo_crypto::{BearerSecret, EmailCode, SecretDigest};
use chapchoo_domain::{EmailAddress, MembershipRole, Password};
use chapchoo_protocol::accounts as wire;
use chapchoo_service::{DomainReason, ServiceError};
use tonic::{Request, Status};

use crate::status::{reject, status};

pub(crate) fn now() -> Result<i64, Status> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| status(ServiceError::infrastructure(e)))?;
    i64::try_from(elapsed.as_millis()).map_err(|e| status(ServiceError::infrastructure(e)))
}

pub(crate) fn actor<T>(request: &Request<T>) -> Result<SecretDigest, Status> {
    let mut values = request.metadata().get_all("authorization").iter();
    let value = values
        .next()
        .ok_or_else(|| reject(DomainReason::InvalidSession))?;
    if values.next().is_some() {
        return Err(reject(DomainReason::InvalidSession));
    }
    let raw = value
        .to_str()
        .map_err(|_| reject(DomainReason::InvalidSession))?;
    let token = raw
        .strip_prefix("Bearer ")
        .ok_or_else(|| reject(DomainReason::InvalidSession))?;
    Ok(bearer(token.to_owned(), DomainReason::InvalidSession)?.digest())
}

pub(crate) fn bearer(value: String, reason: DomainReason) -> Result<BearerSecret, Status> {
    BearerSecret::parse(value).map_err(|_| reject(reason))
}

pub(crate) fn email(value: &str) -> Result<EmailAddress, Status> {
    if value.len() > 320 {
        return Err(reject(DomainReason::InvalidInput));
    }
    EmailAddress::parse(value).map_err(|_| reject(DomainReason::InvalidInput))
}

pub(crate) fn password(value: String) -> Result<Password, Status> {
    Password::parse(value).map_err(|_| reject(DomainReason::WeakPassword))
}

pub(crate) fn code(value: String) -> Result<EmailCode, Status> {
    EmailCode::parse(value).map_err(|_| reject(DomainReason::InvalidCode))
}

pub(crate) fn id(value: &str) -> Result<(), Status> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        Err(reject(DomainReason::InvalidInput))
    } else {
        Ok(())
    }
}

pub(crate) fn member_role(value: i32) -> Result<MembershipRole, Status> {
    match wire::MembershipRole::try_from(value) {
        Ok(wire::MembershipRole::Manager) => Ok(MembershipRole::Manager),
        Ok(wire::MembershipRole::Member) => Ok(MembershipRole::Member),
        Ok(wire::MembershipRole::Unspecified) | Err(_) => Err(reject(DomainReason::InvalidInput)),
    }
}

pub(crate) fn invitation_action(value: i32) -> Result<bool, Status> {
    match wire::InvitationAction::try_from(value) {
        Ok(wire::InvitationAction::Accept) => Ok(true),
        Ok(wire::InvitationAction::Decline) => Ok(false),
        Ok(wire::InvitationAction::Unspecified) | Err(_) => Err(reject(DomainReason::InvalidInput)),
    }
}

pub(crate) fn pagination(value: wire::ListGrantsRequest) -> Result<(i64, i64), Status> {
    let offset = value.offset.unwrap_or(0);
    let limit = value.limit.unwrap_or(50);
    if offset < 0 || !(1..=1000).contains(&limit) {
        Err(reject(DomainReason::InvalidInput))
    } else {
        Ok((offset, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_and_unspecified_enum_values() {
        for value in [0, -1, 3, i32::MAX] {
            assert!(member_role(value).is_err());
            assert!(invitation_action(value).is_err());
        }
        assert_eq!(member_role(1).ok(), Some(MembershipRole::Manager));
        assert_eq!(invitation_action(2).ok(), Some(false));
    }

    #[test]
    fn rejects_ambiguous_authorization_and_never_logs_the_value()
    -> Result<(), Box<dyn std::error::Error>> {
        let token = BearerSecret::generate()?;
        let mut request = Request::new(());
        request.metadata_mut().append(
            "authorization",
            format!("Bearer {}", token.expose()).parse()?,
        );
        assert!(actor(&request).is_ok());
        request.metadata_mut().append(
            "authorization",
            format!("Bearer {}", token.expose()).parse()?,
        );
        let failure = actor(&request)
            .err()
            .ok_or("Expected duplicate rejection")?;
        assert!(!failure.message().contains(token.expose()));
        Ok(())
    }
}
