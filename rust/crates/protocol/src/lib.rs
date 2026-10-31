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
}
