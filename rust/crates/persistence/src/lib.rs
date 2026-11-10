//! Engine-specific completed storage operations for one installation.

mod accounts;
mod agent_cleanup;
mod challenges;
mod component_keys;
mod computer_executions;
mod computers;
mod connector_keys;
mod conversations;
mod creation_results;
mod delivery;
mod directory;
mod display;
mod entity_id;
mod external_auth;
mod identity;
mod identity_transition;
mod models;
mod native_resources;
mod preparation;
mod profile_catalogue;
mod profile_initialization;
mod profile_preparation;
mod sessions;
mod store;
mod teams;

pub use accounts::{GrantDraft, GrantKind, GrantSummary, NewAccount, ProofPreview};
pub use challenges::{EmailLoginOutcome, NewChallenge};
pub use component_keys::{
    ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult, ProviderCredentialResult,
    ProviderOwner, ProviderRegistrationOwner, provider_registration_context,
};
pub use computer_executions::{
    ComputerRequest, ExecutionDraft, ExecutionReservation, ExecutionResources, ManagedComputer,
    ManagedResourceKind, NativeRetirementAdmission, ProviderSource,
};
pub use computers::{
    AgentRemoval, AgentSummary, ComputerInventorySummary, ComputerSummary, CreateAgentProfile,
    CreateCloudAgent, CreationOutcome, LifecycleRequest, ProviderCapabilities, ProviderChoice,
    ProviderIdentity, ProviderInventorySummary, ResourceCursor, UpdateAgentProfile,
};
pub use connector_keys::{
    ConnectorIdentity, ConnectorSessionDraft, ConnectorSessionResult, ConnectorSource,
};
pub use conversations::{
    AcceptConversationAnswer, AcceptConversationInput, AcceptConversationStop, CapturedEventPair,
    ConversationAnswer, ConversationAnswerAdmission, ConversationAnswerChunk,
    ConversationAnswerDelivery, ConversationAnswerForwardingAdmission, ConversationAnswerQuestion,
    ConversationAnswerReceipt, ConversationAnswerReference, ConversationAnswerResult,
    ConversationAnswerValue, ConversationDeliveryTarget, ConversationFieldChange,
    ConversationFieldChunkRead, ConversationFieldMetadata, ConversationFieldRead,
    ConversationFieldUpdate, ConversationForeground, ConversationFormAnswer,
    ConversationForwardingAdmission, ConversationForwardingGrant, ConversationInputAdmission,
    ConversationInputAnchor, ConversationInputChunk, ConversationInputDelivery,
    ConversationInputOutcome, ConversationInputPreparation, ConversationInputReceipt,
    ConversationInputReference, ConversationInputScope, ConversationLiveRequestReference,
    ConversationNativeObservation, ConversationNativePreparation, ConversationOperationForwarding,
    ConversationPreparationAdmission, ConversationPreparationFacts, ConversationPreparationFailure,
    ConversationPreparationFreshness, ConversationPreparationOutcome, ConversationPreparationState,
    ConversationRecordAnchor, ConversationRecordField, ConversationRecordKind,
    ConversationRecordMetadata, ConversationRecordUpdate, ConversationRegionRead,
    ConversationRequestKind, ConversationStopAdmission, ConversationStopReceipt,
    ConversationStopState, ConversationSummary, ConversationWorkTarget, EventChunkRead,
    EventMetadataRead, EventPairMetadata, EventPayloadChunk, EventPayloadKind,
    HarnessSupplyReceipt, HarnessSupplySelection, LiveConversationRequest,
    RetainedConversationInput, SaveHarnessSupply, SourceBinding,
};
pub use creation_results::AgentCreationResult;
pub use delivery::{DeliveryComputer, DeliveryExecution, DeliverySnapshot};
pub use directory::{
    ComponentRoute, ConnectionOwner, ConnectionPreparation, ControlRegistration, DirectoryPolicy,
    ResolvedRoute,
};
pub use external_auth::{
    AgentAuthenticationSelection, AppliedExecutionAuthentication, AuthenticationAcceptedResult,
    AuthenticationCredentialMaterial, AuthenticationCredentialSource, AuthenticationKind,
    AuthenticationOperation, AuthenticationRefreshAdmission, AuthenticationRefreshState,
    AuthenticationRefreshStep, AuthenticationRequestResult, AuthenticationScope,
    AuthenticationSummary, CheckDeviceAdmission, DeleteAuthentication, DeleteAuthenticationOutcome,
    DeviceAuthenticationState, DeviceAuthenticationStatus, DeviceAuthenticationStep,
    DeviceAuthenticationTokens, DeviceAuthorizationDraft, DeviceCheck, DeviceStepFailure,
    DeviceStepSettlement, ExchangeDeviceAdmission, ExecutionAuthenticationMaterial,
    ExpectedAgentAuthentication, MAX_AUTHENTICATION_EXPIRY_MILLIS, ReauthenticateTarget,
    RegisterApiKey, RenameAuthentication, ReplaceApiKey, SelectAgentAuthentication,
    StartDeviceAdmission, StartDeviceAuthentication,
};
pub use models::{CredentialSnapshot, UserSummary, VerifiedCredential};
pub use native_resources::{ManagedResource, ResourceEvidence, ResourceReport};
pub use preparation::{
    POSTGRES_MIGRATIONS, PreparationError, SQLITE_MIGRATIONS, postgres_ready, prepare_postgres,
    prepare_sqlite, sqlite_ready,
};
pub use profile_catalogue::AgentProfileImage;
pub use sessions::{NewSession, SessionPolicy, SessionResult};
pub use store::{Store, StoreError};
pub use teams::{
    CreateTeam, InvitationResult, InvitationStatus, InvitationSummary, InviteMember, MemberSummary,
    TeamPatch, TeamSignupGrant, TeamSummary,
};
