use chapchoo_domain::{TeamKind, TeamRole};
use chapchoo_protocol::accounts as wire;
use chapchoo_service::{
    DomainReason, GrantKind, GrantSummary, GrantsPage, InvitationStatus, InvitationSummary,
    IssuedGrant, MemberSummary, ProofPreview, SessionResult, TeamSummary, UserSummary,
};
use prost_types::Timestamp;
use tonic::Status;

use crate::status::reject;

pub(crate) fn timestamp(milliseconds: i64) -> Result<Timestamp, Status> {
    let seconds = milliseconds.div_euclid(1000);
    if !(-62_135_596_800..=253_402_300_799).contains(&seconds) {
        return Err(reject(DomainReason::Internal));
    }
    let nanos = i32::try_from(milliseconds.rem_euclid(1000) * 1_000_000)
        .map_err(|_| reject(DomainReason::Internal))?;
    Ok(Timestamp { seconds, nanos })
}

pub(crate) fn user(value: UserSummary) -> Result<wire::User, Status> {
    Ok(wire::User {
        id: value.id,
        email: value.email.delivery().to_owned(),
        is_admin: value.is_admin,
        created_at: Some(timestamp(value.created_at)?),
        display: Some(display(value.display)),
    })
}

pub(crate) fn display(value: chapchoo_domain::DisplaySettings) -> wire::DisplaySettings {
    wire::DisplaySettings {
        language: value.language().to_owned(),
        time_zone: value.time_zone().to_owned(),
        region: value.region().to_owned(),
    }
}

pub(crate) fn session(value: SessionResult) -> Result<wire::AuthSession, Status> {
    Ok(wire::AuthSession {
        user: Some(user(value.user)?),
        token: value.token.expose().to_owned(),
        expires_at: Some(timestamp(value.expires_at)?),
        created_at: Some(timestamp(value.created_at)?),
        absolute_expires_at: value.absolute_expires_at.map(timestamp).transpose()?,
    })
}

pub(crate) fn preview(value: Option<ProofPreview>) -> Result<wire::ProofPreview, Status> {
    match value {
        Some(value) => Ok(wire::ProofPreview {
            usable: true,
            email_hint: Some(value.email_hint),
            expires_at: Some(timestamp(value.expires_at)?),
        }),
        None => Ok(wire::ProofPreview {
            usable: false,
            email_hint: None,
            expires_at: None,
        }),
    }
}

fn role(value: TeamRole) -> i32 {
    match value {
        TeamRole::Owner => wire::TeamRole::Owner.into(),
        TeamRole::Manager => wire::TeamRole::Manager.into(),
        TeamRole::Member => wire::TeamRole::Member.into(),
    }
}

pub(crate) fn team(value: TeamSummary) -> wire::Team {
    wire::Team {
        id: value.id,
        kind: match value.kind {
            TeamKind::Personal => wire::TeamKind::Personal.into(),
            TeamKind::Collaborative => wire::TeamKind::Collaborative.into(),
        },
        name: value.name,
        handle: value.handle,
        owner_user_id: value.owner_user_id,
        caller_role: role(value.caller_role),
    }
}

pub(crate) fn member(value: MemberSummary) -> wire::Member {
    wire::Member {
        user_id: value.user_id,
        email: value.email.delivery().to_owned(),
        role: role(value.role),
    }
}

pub(crate) fn invitation(value: InvitationSummary) -> Result<wire::Invitation, Status> {
    Ok(wire::Invitation {
        id: value.id,
        team_id: value.team_id,
        team_name: value.team_name,
        team_handle: value.team_handle,
        recipient_email: value.recipient_email.delivery().to_owned(),
        status: match value.status {
            InvitationStatus::Pending => wire::InvitationStatus::Pending.into(),
            InvitationStatus::Accepted => wire::InvitationStatus::Accepted.into(),
            InvitationStatus::Declined => wire::InvitationStatus::Declined.into(),
        },
        created_at: Some(timestamp(value.created_at)?),
        processed_at: value.processed_at.map(timestamp).transpose()?,
    })
}

pub(crate) fn grant(value: GrantSummary) -> Result<wire::Grant, Status> {
    Ok(wire::Grant {
        id: value.id,
        kind: match value.kind {
            GrantKind::Signup => wire::GrantKind::Signup.into(),
            GrantKind::PasswordReset => wire::GrantKind::PasswordReset.into(),
        },
        email: value.email.delivery().to_owned(),
        issued_at: Some(timestamp(value.issued_at)?),
        expires_at: Some(timestamp(value.expires_at)?),
        consumed_at: value.consumed_at.map(timestamp).transpose()?,
        revoked_at: value.revoked_at.map(timestamp).transpose()?,
    })
}

pub(crate) fn issued(value: IssuedGrant) -> Result<wire::IssuedGrant, Status> {
    let url = value.expose_url().to_owned();
    Ok(wire::IssuedGrant {
        grant: Some(grant(value.grant)?),
        proof: value.proof.expose().to_owned(),
        url,
    })
}

pub(crate) fn grants(value: GrantsPage) -> Result<wire::Grants, Status> {
    Ok(wire::Grants {
        items: value
            .items
            .into_iter()
            .map(grant)
            .collect::<Result<_, _>>()?,
        total: value.total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_use_euclidean_milliseconds_and_validate_protocol_range() -> Result<(), Status> {
        assert_eq!(
            timestamp(-1)?,
            Timestamp {
                seconds: -1,
                nanos: 999_000_000
            }
        );
        assert_eq!(
            timestamp(1234)?,
            Timestamp {
                seconds: 1,
                nanos: 234_000_000
            }
        );
        assert!(timestamp(i64::MAX).is_err());
        let preview = preview(None)?;
        assert!(!preview.usable);
        assert!(preview.email_hint.is_none());
        assert!(preview.expires_at.is_none());
        Ok(())
    }
}
