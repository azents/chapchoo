//! Generated transport contract, independent of persistence and application code.
//!
//! Debug output deliberately omits every message field, including nested values.
//! Wire encoding and explicit field access remain the credential boundaries.
//!
//! ```compile_fail
//! use chapchoo_persistence::Store;
//! ```

/// Captured observations contain private conversation content, not Debug fields.
pub mod conversations {
    tonic::include_proto!("chapchoo.conversations");
    macro_rules! redacted_debug {
        ($($message:ty),+ $(,)?) => {$(
            impl std::fmt::Debug for $message {
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
                    f.debug_struct(stringify!($message)).finish_non_exhaustive()
                }
            }
        )+};
    }
    redacted_debug!(
        NativeTarget,
        NativeStateReport,
        WorkTarget,
        StopCommand,
        StopCheck,
        StopReport,
        LiveRequestReport,
        InputCommand,
        InputChunkRequest,
        InputChunk,
        InputReport,
        AnswerCommand,
        AnswerChunkRequest,
        AnswerChunk,
        AnswerReport,
        ExecutionAuthenticationReference,
        BeginAuthenticationRequest,
        AppliedAuthentication,
        OptionalAppliedAuthentication,
        ExecutionCredentialDescriptor,
        AuthenticationPending,
        AuthenticationSettled,
        ExecutionAuthenticationProgress,
        execution_authentication_progress::State,
        AuthenticationReadRequest,
        AuthenticationPiece,
        AuthenticationHandoffCheck,
        input_report::Result,
        InitialNativeContext,
        HarnessSupply,
        harness_supply::Choice,
        SetupCommand,
        setup_command::Context,
        SetupCheck,
        SetupReport,
        setup_report::Result,
        CaptureSource,
        OwnerConversationRequest,
        OwnerConversation,
        ObservedNative,
        SavedSupply,
        InputTarget,
        input_target::Target,
        OwnerInputPiece,
        InputAcceptance,
        OwnerInputProgress,
        owner_input_progress::Outcome,
        OwnerInputResultRequest,
        InputDelivery,
        input_delivery::Observation,
        OwnerInputStatus,
        InputPreparation,
        input_preparation::Observation,
        AnswerStrings,
        AnswerValue,
        answer_value::Value,
        AnswerField,
        AnswerFields,
        FormAcceptance,
        TypedAnswer,
        typed_answer::Answer,
        OwnerAnswerPiece,
        AnswerAcceptance,
        OwnerAnswerProgress,
        owner_answer_progress::Outcome,
        OwnerAnswerResultRequest,
        OwnerAnswerStatus,
        OwnerAnswerRead,
        OwnerLiveRequests,
        LiveRequest,
        LiveRequestList,
        RecordSource,
        RecordAnchor,
        OwnerRecordsRequest,
        RecordMetadata,
        RecordRegion,
        OwnerFieldsRequest,
        FieldMetadata,
        RecordFields,
        OwnerFieldRead,
        FieldChunk,
        SaveSupplyRequest,
        OwnerStopRequest,
        OwnerStopResultRequest,
        StopReceipt,
        OptionalStopReceipt,
        SourcePosition,
        CaptureRequest,
        CurrentCaptureRequest,
        CurrentCaptureState,
        CaptureState,
        BindCaptureRequest,
        CapturePiece,
        PieceAccepted,
        PairPublished,
        CapturePieceResult,
        capture_piece_result::Outcome,
        CompleteCaptureRequest,
    );
}
/// Backend catalogue and device acquisition; all messages omit fields in Debug.
pub mod authentication {
    tonic::include_proto!("chapchoo.authentication");
    macro_rules! redacted_debug {
        ($($message:ty),+ $(,)?) => {$(
            impl std::fmt::Debug for $message {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!($message)).finish_non_exhaustive()
                }
            }
        )+};
    }
    redacted_debug!(
        Scope,
        scope::Owner,
        Authentication,
        ListRequest,
        AuthenticationList,
        RecordRequest,
        RegisterKeyRequest,
        ReplaceKeyRequest,
        RenameRequest,
        DeleteRequest,
        RecordIdentity,
        SelectionIdentity,
        DeviceIdentity,
        AcceptedResult,
        accepted_result::Result,
        DeleteResult,
        delete_result::Outcome,
        RequestResultRequest,
        HistoricalResult,
        ReauthenticateTarget,
        StartDeviceRequest,
        DeviceRequest,
        CancelDeviceRequest,
        PendingDevice,
        DeviceProgress,
        device_progress::State,
        ExpectedSelection,
        SelectAgentRequest,
        AgentSelection,
    );
}

/// Computer domain contracts carry private names and identities; formatting is
/// field-free just like account and component messages.
pub mod computers {
    tonic::include_proto!("chapchoo.computers");
    macro_rules! redacted_debug {
        ($($message:ty),+ $(,)?) => {$(
            impl std::fmt::Debug for $message {
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
                    f.debug_struct(stringify!($message)).finish_non_exhaustive()
                }
            }
        )+};
    }
    redacted_debug!(
        Page,
        AgentRequest,
        TeamAgentRequest,
        UpdateAgentProfileRequest,
        CreationResultRequest,
        HistoricalCreationResult,
        PresentCreation,
        historical_creation_result::Outcome,
        AgentListRequest,
        CreateAgentRequest,
        CreateAgentResult,
        RemovedCreation,
        Agent,
        AgentList,
        AgentProfileImage,
        AgentProfileImages,
        AgentHandleRequest,
        HandleAvailability,
        ProviderChoice,
        ProviderChoices,
        ActionAvailability,
        Computer,
        LifecycleRequest,
        PreparationRetryRequest,
        ForceRemoveRequest,
        ProviderCapabilities,
        ProviderSummary,
        ProviderList,
        InventoryRequest,
        InventoryComputer,
        ComputerInventory,
        ResourceCursor,
        ResourcePage,
        InventoryResource,
        ResourceInventory,
        create_agent_result::Outcome,
    );
}

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
        DisplaySettings,
        DisplayPatch,
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
        TeamHandleRequest,
        TeamHandleAvailability,
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

/// Computer component and trusted per-call peer contracts.
pub mod control {
    tonic::include_proto!("chapchoo.control");
    macro_rules! redacted_debug {
        ($($message:path),+ $(,)?) => {
            $(impl std::fmt::Debug for $message {
                fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(stringify!($message)).finish_non_exhaustive()
                }
            })+
        };
    }
    redacted_debug!(
        ProviderKeyRequest,
        ProviderAuthenticationRequest,
        provider_authentication_request::Evidence,
        ProviderConnectionCheck,
        ConnectorConnectionCheck,
        ProviderCommandCheck,
        ProviderCommandAdmission,
        ProviderReport,
        ConnectorCommandCheck,
        ProviderCapabilities,
        ProviderConfiguration,
        ComponentKey,
        ProviderCredentials,
        ConnectorSessionRequest,
        ConnectorSession,
        RevokeConnectorSessionRequest,
        ConnectorKeyRequest,
        ProviderHello,
        ConnectorHello,
        Connected,
        Heartbeat,
        HeartbeatAccepted,
        CallIdentity,
        CancelCall,
        ComputerContext,
        ManagedResource,
        NativeAllocationWitness,
        DockerContainerNative,
        DockerVolumeNative,
        DockerVolumeIdentity,
        managed_resource::Native,
        OwnedPresent,
        owned_present::NativeIdentity,
        DefinitivelyAbsent,
        definitively_absent::ExpectedNativeIdentity,
        UnknownResource,
        ResourceEvidence,
        resource_evidence::Evidence,
        ResourceObservation,
        ExecutionObservation,
        ConnectorObservation,
        ObserveResources,
        PrepareExecution,
        DockerPreparation,
        RetireExecution,
        CleanupResources,
        ProviderCommand,
        provider_command::Operation,
        ObserveConnector,
        ConnectorCommand,
        connector_command::Operation,
        ProviderResult,
        ConnectorResult,
        ProviderFrame,
        provider_frame::Payload,
        ConnectorFrame,
        connector_frame::Payload,
        ControlFrame,
        control_frame::Payload,
        UserAuthority,
        ReconciliationAuthority,
        PeerAuthority,
        peer_authority::Context,
        ExpectedOwner,
        ProviderInvocation,
        ProviderConvergence,
        ConnectorInvocation,
        ProviderInvocationResult,
        ConnectorInvocationResult,
    );
}

#[cfg(test)]
mod tests {
    use super::accounts::*;
    #[test]
    fn execution_credential_wire_debug_omits_bytes_and_nested_metadata() {
        use super::conversations as c;
        let descriptor = c::ExecutionCredentialDescriptor {
            account_id: Some("private-account-marker".into()),
            credential_source: vec![91; 24],
            ..Default::default()
        };
        let values: Vec<Box<dyn std::fmt::Debug>> = vec![
            Box::new(c::AuthenticationPiece {
                descriptor: Some(descriptor.clone()),
                content: b"private-key-marker".to_vec(),
                offset: 0,
            }),
            Box::new(c::execution_authentication_progress::State::Current(
                descriptor.clone(),
            )),
            Box::new(c::ExecutionAuthenticationProgress {
                state: Some(c::execution_authentication_progress::State::Current(
                    descriptor,
                )),
            }),
        ];
        for value in values {
            let rendered = format!("{value:?}");
            assert!(rendered.ends_with("{ .. }"));
            for field in [
                "content",
                "credential_source",
                "private-account-marker",
                "private-key-marker",
            ] {
                assert!(!rendered.contains(field));
            }
        }
    }

    #[test]
    fn component_and_peer_oneof_debug_never_exposes_fields() {
        use super::control as c;
        let secret = "component-secret-that-must-not-appear";
        let prepare = c::PrepareExecution {
            connector_initial_key: secret.into(),
            ..Default::default()
        };
        let command = c::ProviderCommand {
            operation: Some(c::provider_command::Operation::Prepare(prepare.clone())),
            ..Default::default()
        };
        let values: Vec<Box<dyn std::fmt::Debug>> = vec![
            Box::new(c::ComponentKey {
                token: secret.into(),
                ..Default::default()
            }),
            Box::new(c::ConnectorSession {
                initial_key: secret.into(),
                ..Default::default()
            }),
            Box::new(prepare.clone()),
            Box::new(c::provider_command::Operation::Prepare(prepare)),
            Box::new(command.clone()),
            Box::new(c::control_frame::Payload::Provider(command.clone())),
            Box::new(c::ControlFrame {
                payload: Some(c::control_frame::Payload::Provider(command)),
            }),
            Box::new(c::peer_authority::Context::User(c::UserAuthority {
                session_credential_digest: secret.as_bytes().to_vec(),
            })),
        ];
        for value in values {
            let rendered = format!("{value:?}");
            assert!(!rendered.contains(secret));
            assert!(!rendered.contains("session_credential_digest"));
            assert!(rendered.ends_with("{ .. }"));
        }
    }

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
                display: None,
            }),
            Box::new(RedeemPasswordResetRequest {
                proof: credential.into(),
                password: credential.into(),
            }),
            Box::new(InitializeRequest {
                installation_proof: credential.into(),
                email: credential.into(),
                password: credential.into(),
                display: None,
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
