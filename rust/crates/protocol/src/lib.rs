//! Generated transport contract, independent of persistence and application code.
//!
//! Debug output deliberately omits every message field, including nested values.
//! Wire encoding and explicit field access remain the credential boundaries.
//!
//! ```compile_fail
//! use chapchoo_persistence::Store;
//! ```

/// Generated Protobuf messages and Tonic clients/servers.
pub mod accounts {
    tonic::include_proto!("chapchoo.accounts");

    macro_rules! redacted_debug {
        ($($message:ident),+ $(,)?) => {
            $(
                impl std::fmt::Debug for $message {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(stringify!($message)).finish_non_exhaustive()
                    }
                }
            )+
        };
    }

    // Adding a message requires explicitly choosing this secret-safe formatting
    // policy; the inherited missing_debug_implementations gate enforces it.
    redacted_debug!(
        User,
        LoginMethods,
        AuthSession,
        PasswordLoginRequest,
        SendCodeRequest,
        CodeChallenge,
        VerifyCodeRequest,
        RenewRequest,
        SignupEmailRequest,
        DeliveryAccepted,
        ProofRequest,
        ProofPreview,
        RedeemSignupRequest,
        RedeemPasswordResetRequest,
        SetupStatusResult,
        InitializeRequest,
        Team,
        Teams,
        TeamRequest,
        CreateTeamRequest,
        UpdateTeamRequest,
        Member,
        Members,
        MemberRequest,
        SetMemberRoleRequest,
        Invitation,
        Invitations,
        InviteMemberRequest,
        ProcessInvitationRequest,
        IssueGrantRequest,
        GrantRequest,
        ListGrantsRequest,
        Grant,
        Grants,
        IssuedGrant,
    );
}

#[cfg(test)]
mod tests {
    use super::accounts::*;
    use prost::Message;

    #[test]
    fn credential_debug_never_exposes_fields() {
        let credential = "secret-that-must-not-appear";
        let messages: Vec<Box<dyn std::fmt::Debug>> = vec![
            Box::new(PasswordLoginRequest {
                email: credential.into(),
                password: credential.into(),
            }),
            Box::new(AuthSession {
                token: credential.into(),
                user: Some(User {
                    email: credential.into(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            Box::new(CodeChallenge {
                binding: credential.into(),
                ..Default::default()
            }),
            Box::new(VerifyCodeRequest {
                email: credential.into(),
                binding: credential.into(),
                code: credential.into(),
            }),
            Box::new(RenewRequest {
                token: credential.into(),
            }),
            Box::new(ProofRequest {
                proof: credential.into(),
            }),
            Box::new(RedeemSignupRequest {
                proof: credential.into(),
                email: credential.into(),
                password: credential.into(),
            }),
            Box::new(RedeemPasswordResetRequest {
                proof: credential.into(),
                password: credential.into(),
            }),
            Box::new(InitializeRequest {
                installation_proof: credential.into(),
                email: credential.into(),
                password: credential.into(),
            }),
            Box::new(IssuedGrant {
                proof: credential.into(),
                url: credential.into(),
                ..Default::default()
            }),
        ];
        for message in messages {
            let rendered = format!("{message:?}");
            assert!(!rendered.contains(credential));
            assert!(rendered.ends_with("{ .. }"));
        }
    }

    #[test]
    fn optional_scalars_and_unknown_enum_values_survive_wire_roundtrip()
    -> Result<(), prost::DecodeError> {
        let omitted = UpdateTeamRequest {
            team_id: "team".into(),
            name: None,
            handle: Some(String::new()),
        };
        assert_eq!(
            omitted.encode_to_vec(),
            [0x0a, 0x04, b't', b'e', b'a', b'm', 0x1a, 0x00]
        );
        let decoded = UpdateTeamRequest::decode(omitted.encode_to_vec().as_slice())?;
        assert_eq!(decoded.name, None);
        assert_eq!(decoded.handle, Some(String::new()));
        let unknown = SetMemberRoleRequest {
            team_id: "team".into(),
            user_id: "user".into(),
            role: 12345,
        };
        let decoded = SetMemberRoleRequest::decode(unknown.encode_to_vec().as_slice())?;
        assert_eq!(decoded.role, 12345);
        assert!(MembershipRole::try_from(decoded.role).is_err());
        Ok(())
    }

    #[test]
    fn int64_and_timestamps_keep_exact_precision() -> Result<(), prost::DecodeError> {
        let request = ListGrantsRequest {
            offset: Some(i64::MAX),
            limit: Some(0),
        };
        assert_eq!(
            request.encode_to_vec(),
            [
                0x08, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0x10, 0x00
            ]
        );
        let decoded = ListGrantsRequest::decode(request.encode_to_vec().as_slice())?;
        assert_eq!(decoded.offset, Some(i64::MAX));
        assert_eq!(decoded.limit, Some(0));
        let grant = Grant {
            issued_at: Some(prost_types::Timestamp {
                seconds: 9_007_199_254_740_993,
                nanos: 123_456_789,
            }),
            ..Default::default()
        };
        let decoded = Grant::decode(grant.encode_to_vec().as_slice())?;
        assert_eq!(decoded.issued_at, grant.issued_at);
        Ok(())
    }
}
