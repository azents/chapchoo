//! Explicit application orchestration without persistence handles or transport types.
mod authentication;
mod computers;
mod conversation;
mod error;
mod mail;
mod password;
mod service;
mod time;

pub use authentication::{
    AuthenticationProgress, AuthenticationRefreshProgress, AuthenticationRefreshService,
    AuthenticationService, ChatGptRefreshProvider, DEVICE_VERIFICATION_URI, DeviceApproval,
    DeviceAuthenticationProvider, DevicePollOutcome, DeviceProviderFailure, DeviceProviderFuture,
    ExecutionAuthenticationChunk, ExecutionAuthenticationConsumer,
    ExecutionAuthenticationDescriptor, ExecutionAuthenticationProgress,
    ExecutionAuthenticationService, NativeTokenEncoder, NativeTokenEncodingFailure,
    RefreshProviderFailure, RefreshProviderFuture,
};
pub use chapchoo_crypto::{BearerSecret, EmailCode, ExternalSecret, PasswordCost, SecretDigest};
pub use chapchoo_domain::{
    DisplaySettings, DockerDaemonId, EmailAddress, MembershipRole, Password, ProviderKind,
    ProviderSubstrate, TeamKind, TeamRole,
};
pub use chapchoo_persistence::{
    AcceptConversationInput, AgentAuthenticationSelection, AgentCreationResult, AgentProfileImage,
    AgentRemoval, AgentSummary, AppliedExecutionAuthentication, AuthenticationAcceptedResult,
    AuthenticationCredentialSource, AuthenticationKind, AuthenticationOperation,
    AuthenticationRefreshState, AuthenticationRequestResult, AuthenticationScope,
    AuthenticationSummary, ComponentKeyPolicy, ComponentKeyResult, ComponentRoute,
    ComputerInventorySummary, ComputerRequest, ComputerSummary, ConnectionOwner, ConnectorIdentity,
    ConnectorSessionResult, ConnectorSource, ControlRegistration, ConversationAnswer,
    ConversationAnswerChunk, ConversationAnswerDelivery, ConversationAnswerForwardingAdmission,
    ConversationAnswerResult, ConversationAnswerValue, ConversationDeliveryTarget,
    ConversationFieldChunkRead, ConversationFieldMetadata, ConversationFieldRead,
    ConversationForeground, ConversationFormAnswer, ConversationForwardingAdmission,
    ConversationInputAnchor, ConversationInputDelivery, ConversationInputOutcome,
    ConversationInputPreparation, ConversationInputScope, ConversationNativeObservation,
    ConversationNativePreparation, ConversationOperationForwarding,
    ConversationPreparationAdmission, ConversationPreparationFailure,
    ConversationPreparationFreshness, ConversationPreparationOutcome, ConversationPreparationState,
    ConversationRecordAnchor, ConversationRecordField, ConversationRecordKind,
    ConversationRecordMetadata, ConversationRegionRead, ConversationRequestKind,
    ConversationStopReceipt, ConversationStopState, ConversationSummary, ConversationWorkTarget,
    CreateAgentProfile, CreateCloudAgent, CreationOutcome, DeleteAuthentication,
    DeleteAuthenticationOutcome, DeliveryComputer, DeliveryExecution, DeliverySnapshot,
    DeviceAuthenticationState, DeviceAuthenticationStatus, DeviceAuthenticationTokens,
    DeviceAuthorizationDraft, DirectoryPolicy, EventPayloadChunk, ExecutionReservation,
    ExpectedAgentAuthentication, GrantKind, GrantSummary, InvitationStatus, InvitationSummary,
    LifecycleRequest, LiveConversationRequest, MAX_AUTHENTICATION_EXPIRY_MILLIS, ManagedComputer,
    ManagedResource, ManagedResourceKind, MemberSummary, NativeRetirementAdmission, ProofPreview,
    ProviderCapabilities, ProviderChoice, ProviderCredentialResult, ProviderIdentity,
    ProviderInventorySummary, ProviderSource, ReauthenticateTarget, RegisterApiKey,
    RenameAuthentication, ReplaceApiKey, ResolvedRoute, ResourceCursor, ResourceEvidence,
    ResourceReport, RetainedConversationInput, SaveHarnessSupply, SelectAgentAuthentication,
    SessionPolicy, SessionResult, SourceBinding, StartDeviceAuthentication, TeamPatch, TeamSummary,
    UpdateAgentProfile, UserSummary,
};
pub use computers::{ComputerPolicy, ComputersService, PreparedComponentConnection};
pub use conversation::{
    AcceptedConversationAnswer, AcceptedConversationInput, AcceptedConversationStop,
    ConversationAnswerAcceptance, ConversationAnswerDraft, ConversationAnswerFuture,
    ConversationAnswerProcessor, ConversationAnswerService, ConversationInputAcceptance,
    ConversationInputFuture, ConversationInputOwnership, ConversationInputProcessor,
    ConversationInputService, ConversationInputStatus, ConversationService, ConversationStopFuture,
    ConversationStopProcessor, ConversationStopService, EncodedNativeAnswer, NativeAnswerError,
    NativeNormalizationError, NormalizedCapture, encode_native_answer, normalize_captured_event,
    validate_conversation_answer,
};
pub use error::{DomainReason, InfrastructureReport, ServiceError};
pub use mail::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
pub use password::PasswordWork;
pub use service::{
    AccountsConfig, AccountsService, CodeChallenge, GrantsPage, IssuedGrant, LoginMethods,
};
pub use time::AcceptanceTime;
