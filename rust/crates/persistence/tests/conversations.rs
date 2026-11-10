#[path = "conversations/answers.rs"]
mod answers;
#[path = "conversations/authentication.rs"]
mod authentication;
#[path = "conversations/delivery.rs"]
mod delivery;
#[path = "conversations/input_reads.rs"]
mod input_reads;
#[path = "conversations/native_state.rs"]
mod native_state;
#[path = "conversations/preparation.rs"]
mod preparation;
#[path = "conversations/projections.rs"]
mod projections;
mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, ExternalSecret, PasswordCost,
    PasswordHasher, RootKey,
};
use chapchoo_domain::{
    ConversationSource, ConversationTarget, ConversationText, EmailAddress, EventOccurrenceId,
    ExistingCodexPath, HarnessSupply, MembershipRole, NativeConversationId, Password,
    ProviderKeyRequestId, ProviderSubstrate, SnapshotId, SourcePosition,
};
use chapchoo_persistence::{
    AcceptConversationInput, AuthenticationAcceptedResult, AuthenticationScope, CapturedEventPair,
    ComponentKeyDraft, ComponentKeyPolicy, ComputerRequest, ConnectorSessionDraft, ConnectorSource,
    CreateAgentProfile, CreateCloudAgent, CreateTeam, CreationOutcome, DirectoryPolicy,
    EventChunkRead, EventMetadataRead, EventPayloadKind, ExecutionDraft, ExecutionResources,
    GrantDraft, InviteMember, NewAccount, NewSession, POSTGRES_MIGRATIONS, ProviderCapabilities,
    ProviderOwner, ProviderSource, RegisterApiKey, SQLITE_MIGRATIONS, SaveHarnessSupply,
    SessionPolicy, SourceBinding, Store, StoreError, prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::Binary};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;
use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type TestError = Box<dyn Error + Send + Sync>;
static POSTGRES_CONVERSATION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([57; 32]),
    )?)?))
}
fn now() -> Result<i64, TestError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}
fn snapshot(value: &str) -> Result<SnapshotId, TestError> {
    Ok(SnapshotId::parse(value.to_owned())?)
}
fn position(ordinal: i64, id: &str) -> Result<SourcePosition, TestError> {
    Ok(SourcePosition::new(
        ordinal,
        EventOccurrenceId::parse(id.to_owned())?,
    )?)
}
struct Fixture {
    owner: BearerSecret,
    other: BearerSecret,
    agent: chapchoo_persistence::AgentSummary,
    other_agent: String,
    team_id: String,
    session: String,
    control: String,
    connection: String,
    connector: BearerSecret,
    now: i64,
}
impl Fixture {
    fn source<'a>(&'a self, key: &'a chapchoo_crypto::SecretDigest) -> ConnectorSource<'a> {
        ConnectorSource {
            session_id: &self.session,
            control_id: &self.control,
            connection_id: &self.connection,
            credential: key,
        }
    }
}
async fn fixture(store: &Store) -> Result<Fixture, TestError> {
    let now = now()?;
    let mut tokens = Vec::new();
    for index in 0..2 {
        let token = BearerSecret::generate()?;
        let proof = BearerSecret::generate()?;
        let account = NewAccount {
            email: EmailAddress::parse(&format!("conversation{index}@example.invalid"))?,
            password: PasswordHasher::new(PasswordCost::new(10)?)
                .hash(&Password::parse("Aa1!conversation".to_owned())?)?,
            session: NewSession {
                token: BearerSecret::parse(token.expose().to_owned())?,
            },
            display: None,
        };
        if index == 0 {
            store
                .initialize_admin(
                    &proof.digest(),
                    &proof,
                    &account,
                    now,
                    SessionPolicy::default(),
                )
                .await?;
        } else {
            store
                .issue_email_signup_grant(
                    &account.email,
                    &GrantDraft {
                        proof: proof.digest(),
                        expires_at: now + 60_000,
                    },
                    now,
                )
                .await?;
            store
                .redeem_signup(&proof.digest(), &account, now, SessionPolicy::default())
                .await?;
        }
        tokens.push(token);
    }
    let other = tokens.pop().ok_or("Missing other caller")?;
    let owner = tokens.pop().ok_or("Missing owner")?;
    let team = store
        .create_team(
            &owner.digest(),
            &CreateTeam {
                name: "Conversation".to_owned(),
                handle: "conversation".to_owned(),
            },
            now,
        )
        .await?;
    let invitation = store
        .invite_member(
            &owner.digest(),
            &InviteMember {
                team_id: team.id.clone(),
                recipient: EmailAddress::parse("conversation1@example.invalid")?,
                signup: None,
            },
            now,
        )
        .await?;
    store
        .process_invitation(&other.digest(), &invitation.invitation.id, true, now)
        .await?;
    let other_user = store.current_identity(&other.digest(), now).await?.id;
    store
        .set_member_role(
            &owner.digest(),
            &team.id,
            &other_user,
            MembershipRole::Manager,
            now,
        )
        .await?;
    let authentication = match store
        .register_external_api_key(
            &owner.digest(),
            &RegisterApiKey {
                scope: AuthenticationScope::Team {
                    team_id: team.id.clone(),
                },
                request_id: "conversation-auth".to_owned(),
                name: String::new(),
                key: ExternalSecret::new(b"synthetic-never-used-model-key".to_vec())?,
            },
            now,
        )
        .await?
    {
        AuthenticationAcceptedResult::Record { authentication_id } => authentication_id,
        _ => return Err("Expected authentication record".into()),
    };
    let permit = BootstrapProviderPermit::new("conversation-provider".to_owned())?.authenticated();
    let provider = store
        .rotate_provider_key(
            ProviderOwner::Bootstrap {
                proof: &permit,
                name: "Conversation fixture",
                substrate: &ProviderSubstrate::Kubernetes,
                capabilities: ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
            },
            "conversation-provider",
            &ComponentKeyDraft {
                request_id: ProviderKeyRequestId::new(now, "conversation-provider-key")?
                    .as_str()
                    .to_owned(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now,
        )
        .await?;
    let control = store
        .register_control("http://127.0.0.1:1", DirectoryPolicy::default())
        .await?;
    let prepared = store
        .prepare_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            &control.id,
        )
        .await?;
    store
        .admit_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            prepared.expected.as_ref(),
            &prepared.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let image = store
        .agent_profile_images(&owner.digest(), None, 1, now)
        .await?
        .pop()
        .ok_or("Missing profile image")?
        .id;
    let agent = match store
        .create_cloud_agent(
            &owner.digest(),
            &CreateCloudAgent {
                team_id: team.id.clone(),
                request_id: "conversation-create".to_owned(),
                provider_id: provider.provider_id.clone(),
                name: "Conversation fixture".to_owned(),
                authentication_id: authentication.clone(),
                profile: CreateAgentProfile {
                    handle: "chat-fixture".to_owned(),
                    image_id: image.clone(),
                    palette_key: "sage".to_owned(),
                },
            },
            now,
        )
        .await?
    {
        CreationOutcome::Present(agent) => *agent,
        _ => return Err("Expected Agent".into()),
    };
    let other_agent = match store
        .create_cloud_agent(
            &other.digest(),
            &CreateCloudAgent {
                team_id: team.id.clone(),
                request_id: "conversation-other".to_owned(),
                provider_id: provider.provider_id.clone(),
                name: "Other private Agent".to_owned(),
                authentication_id: authentication,
                profile: CreateAgentProfile {
                    handle: "other-chat".to_owned(),
                    image_id: image,
                    palette_key: "sage".to_owned(),
                },
            },
            now,
        )
        .await?
    {
        CreationOutcome::Present(agent) => agent.id,
        _ => return Err("Expected other private Agent".into()),
    };
    let provider_key = provider.key.token.digest();
    let source = ProviderSource {
        provider_id: &provider.provider_id,
        control_id: &control.id,
        connection_id: &prepared.owner.connection_id,
        credential: &provider_key,
    };
    let reservation = store
        .reserve_execution(
            &source,
            ComputerRequest {
                computer_id: &agent.computer.id,
                request_id: &agent.computer.current_request_id,
            },
            &ExecutionDraft {
                native: ExecutionResources::Kubernetes {
                    namespace: "fixture".to_owned(),
                    pod_name: "conversation-pod".to_owned(),
                    bootstrap_secret_name: "conversation-bootstrap".to_owned(),
                    saved_pvc_name: "conversation-pvc".to_owned(),
                },
            },
        )
        .await?;
    let initial = store
        .issue_connector_session_from_source(
            &source,
            &reservation.execution_id,
            &ConnectorSessionDraft {
                request_id: "conversation-session".to_owned(),
                initial_key: BearerSecret::generate()?,
            },
            now,
        )
        .await?;
    let key = store
        .exchange_connector_key(
            &initial.id,
            &initial.initial_key.digest(),
            &ComponentKeyDraft {
                request_id: "conversation-key".to_owned(),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now,
        )
        .await?;
    let prepared = store
        .prepare_connector_connection(&initial.id, &key.token.digest(), &control.id)
        .await?;
    store
        .admit_connector_connection(
            &initial.id,
            &key.token.digest(),
            prepared.expected.as_ref(),
            &prepared.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    Ok(Fixture {
        owner,
        other,
        agent,
        other_agent,
        team_id: team.id,
        session: initial.id,
        control: control.id,
        connection: prepared.owner.connection_id,
        connector: key.token,
        now,
    })
}

async fn exercise(store: &Store, f: &Fixture) -> Result<(), TestError> {
    let owner = f.owner.digest();
    let other = f.other.digest();
    let key = f.connector.digest();
    let source = f.source(&key);
    assert!(
        store
            .connector_current_conversation(&source)
            .await?
            .is_none()
    );
    assert!(
        store
            .current_conversation(&owner, &f.agent.id, f.now)
            .await?
            .is_none()
    );
    assert!(matches!(
        store.current_conversation(&other, &f.agent.id, f.now).await,
        Err(StoreError::NotFound)
    ));
    let default = store.harness_supply(&owner, &f.agent.id, f.now).await?;
    assert_eq!(default.supply, HarnessSupply::Managed);
    assert!(default.saved.is_none());
    let saved = SaveHarnessSupply {
        agent_id: f.agent.id.clone(),
        request_id: "existing-supply".to_owned(),
        expected_request_id: None,
        supply: HarnessSupply::Existing(ExistingCodexPath::parse("/tools/owner-codex".to_owned())?),
    };
    let (first, again) = tokio::join!(
        store.save_harness_supply(&owner, &saved, f.now),
        store.save_harness_supply(&owner, &saved, f.now)
    );
    assert_eq!(first?.request_id, again?.request_id);
    assert!(matches!(
        store.harness_supply(&other, &f.agent.id, f.now).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store.save_harness_supply(&other, &saved, f.now).await,
        Err(StoreError::NotFound)
    ));
    store
        .save_harness_supply(
            &owner,
            &SaveHarnessSupply {
                agent_id: f.agent.id.clone(),
                request_id: "managed-supply".to_owned(),
                expected_request_id: Some("existing-supply".to_owned()),
                supply: HarnessSupply::Managed,
            },
            f.now,
        )
        .await?;
    store.save_harness_supply(&owner, &saved, f.now).await?;
    assert_eq!(
        store
            .harness_supply(&owner, &f.agent.id, f.now)
            .await?
            .supply,
        HarnessSupply::Managed
    );
    assert_eq!(
        store
            .agent(&owner, &f.agent.id, f.now)
            .await?
            .computer
            .current_request_id,
        f.agent.computer.current_request_id
    );

    let draft = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: None,
        request_id: "first-input".to_owned(),
        target: ConversationTarget::NotStarted,
        text: ConversationText::parse("  first input\n한국어  ".to_owned())?,
    };
    let (one, two) = tokio::join!(
        store.admit_conversation_input(&owner, &draft, f.now),
        store.admit_conversation_input(&owner, &draft, f.now)
    );
    use chapchoo_persistence::ConversationInputAdmission as Admission;
    let (first, existing) = match (one?, two?) {
        (Admission::Accepted(first), Admission::Existing(existing))
        | (Admission::Existing(existing), Admission::Accepted(first)) => (first, existing),
        _ => return Err("Concurrent duplicate received another processing admission".into()),
    };
    assert_eq!(
        store
            .connector_current_conversation(&source)
            .await?
            .ok_or("Missing current conversation")?
            .id,
        first.conversation_id
    );
    assert_eq!(first.id, existing.id);
    assert_eq!(first.text.as_str(), draft.text.as_str());
    input_reads::exercise(
        store,
        f,
        &first.conversation_id,
        &ConversationTarget::NotStarted,
        first.text.as_str(),
    )
    .await?;
    let changed = AcceptConversationInput {
        text: ConversationText::parse("changed".to_owned())?,
        ..AcceptConversationInput {
            agent_id: draft.agent_id.clone(),
            conversation_id: None,
            request_id: draft.request_id.clone(),
            target: draft.target.clone(),
            text: draft.text.clone(),
        }
    };
    assert!(matches!(
        store
            .accept_conversation_input(&owner, &changed, f.now)
            .await,
        Err(StoreError::Conflict)
    ));
    let s1 = snapshot("snapshot-one")?;
    let native = NativeConversationId::parse("native-context-one".to_owned())?;
    preparation::initial(store, f, &first, &s1, &native).await?;
    let binding = SourceBinding {
        conversation_id: first.conversation_id.clone(),
        expected_previous: None,
        snapshot: s1.clone(),
        native_session: native.clone(),
    };
    assert!(matches!(
        store
            .bind_conversation_source(&source, &binding)
            .await?
            .source,
        ConversationSource::Loading { .. }
    ));
    // Initial source binding is not replacement: accepted input survives it.
    assert!(
        store
            .conversation_input(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                "first-input",
                f.now
            )
            .await?
            .is_some()
    );
    assert_eq!(
        store
            .accept_conversation_input(&owner, &draft, f.now)
            .await?
            .id,
        first.id
    );
    let fresh = AcceptConversationInput {
        agent_id: f.agent.id.clone(),
        conversation_id: Some(first.conversation_id.clone()),
        request_id: "during-loading".to_owned(),
        target: ConversationTarget::Snapshot(s1.clone()),
        text: ConversationText::parse("next".to_owned())?,
    };
    assert!(matches!(
        store.accept_conversation_input(&owner, &fresh, f.now).await,
        Err(StoreError::InvalidTransition)
    ));
    let pair = CapturedEventPair {
        snapshot: s1.clone(),
        position: position(1, "z-first")?,
        raw: vec![b'r'; 70_001],
        normalized: vec![b'n'; 80_003],
    };
    store
        .ingest_conversation_pair(&source, &first.conversation_id, &pair)
        .await?;
    assert_eq!(
        store
            .ingest_conversation_pair(&source, &first.conversation_id, &pair)
            .await?
            .source
            .position()
            .ok_or("Missing cursor")?
            .ordinal(),
        1
    );
    let conflict = CapturedEventPair {
        snapshot: s1.clone(),
        position: pair.position.clone(),
        raw: b"changed".to_vec(),
        normalized: pair.normalized.clone(),
    };
    assert!(matches!(
        store
            .ingest_conversation_pair(&source, &first.conversation_id, &conflict)
            .await,
        Err(StoreError::Conflict)
    ));
    let gap = CapturedEventPair {
        snapshot: s1.clone(),
        position: position(3, "gap")?,
        raw: b"raw".to_vec(),
        normalized: b"normalized".to_vec(),
    };
    assert!(matches!(
        store
            .ingest_conversation_pair(&source, &first.conversation_id, &gap)
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .complete_conversation_source(
                &source,
                &first.conversation_id,
                &s1,
                Some(&position(2, "missing")?)
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    store
        .complete_conversation_source(&source, &first.conversation_id, &s1, Some(&pair.position))
        .await?;
    input_reads::exercise(
        store,
        f,
        &first.conversation_id,
        &ConversationTarget::Snapshot(s1.clone()),
        first.text.as_str(),
    )
    .await?;
    assert!(matches!(
        store
            .bind_conversation_source(&source, &binding)
            .await?
            .source,
        ConversationSource::Ready { .. }
    ));
    store
        .accept_conversation_input(&owner, &fresh, f.now)
        .await?;
    let second = CapturedEventPair {
        snapshot: s1.clone(),
        position: position(2, "a-second")?,
        raw: b"second".to_vec(),
        normalized: b"second-normalized".to_vec(),
    };
    store
        .ingest_conversation_pair(&source, &first.conversation_id, &second)
        .await?;
    let metadata = store
        .conversation_event_metadata(
            &owner,
            &EventMetadataRead {
                agent_id: &f.agent.id,
                conversation_id: &first.conversation_id,
                snapshot: &s1,
                after: None,
                limit: 10,
            },
            f.now,
        )
        .await?;
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0].position.occurrence().as_str(), "z-first");
    assert_eq!(metadata[1].position.occurrence().as_str(), "a-second");
    for (kind, expected) in [
        (EventPayloadKind::Raw, &pair.raw),
        (EventPayloadKind::Normalized, &pair.normalized),
    ] {
        let mut bytes = Vec::new();
        let mut offset = 0;
        while bytes.len() < expected.len() {
            let chunk = store
                .conversation_event_chunk(
                    &owner,
                    &EventChunkRead {
                        agent_id: &f.agent.id,
                        conversation_id: &first.conversation_id,
                        snapshot: &s1,
                        occurrence: pair.position.occurrence(),
                        kind,
                        offset,
                        length: 32768,
                    },
                    f.now,
                )
                .await?;
            assert_eq!(chunk.offset, offset);
            assert_eq!(chunk.total_bytes, i64::try_from(expected.len())?);
            assert!(!chunk.bytes.is_empty());
            offset += u32::try_from(chunk.bytes.len())?;
            bytes.extend(chunk.bytes);
        }
        assert_eq!(&bytes, expected);
    }
    assert!(matches!(
        store
            .conversation_event_metadata(
                &other,
                &EventMetadataRead {
                    agent_id: &f.agent.id,
                    conversation_id: &first.conversation_id,
                    snapshot: &s1,
                    after: None,
                    limit: 10
                },
                f.now
            )
            .await,
        Err(StoreError::NotFound)
    ));
    let wrong = BearerSecret::generate()?.digest();
    assert!(matches!(
        store
            .bind_conversation_source(&f.source(&wrong), &binding)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    delivery::exercise(store, f, &first, &s1, &native).await?;
    answers::exercise(store, f, &first.conversation_id, &s1, &native).await?;
    native_state::exercise(store, f, &first.conversation_id, &s1, &native).await?;
    let s2 = snapshot("snapshot-two")?;
    let replacement = SourceBinding {
        conversation_id: first.conversation_id.clone(),
        expected_previous: Some(s1.clone()),
        snapshot: s2.clone(),
        native_session: native.clone(),
    };
    store
        .bind_conversation_source(&source, &replacement)
        .await?;
    delivery::replaced(store, f, &first, &s1, &native).await?;
    preparation::replaced(store, f, &first).await?;
    native_state::replaced(store, f, &first.conversation_id).await?;
    answers::replaced(store, f, &first.conversation_id).await?;
    assert!(
        store
            .conversation_input(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                "first-input",
                f.now
            )
            .await?
            .is_none()
    );
    assert!(
        store
            .conversation_input(
                &owner,
                &f.agent.id,
                &first.conversation_id,
                "during-loading",
                f.now
            )
            .await?
            .is_none()
    );
    assert!(matches!(
        store
            .conversation_event_metadata(
                &owner,
                &EventMetadataRead {
                    agent_id: &f.agent.id,
                    conversation_id: &first.conversation_id,
                    snapshot: &s1,
                    after: None,
                    limit: 10
                },
                f.now
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(matches!(
        store
            .ingest_conversation_pair(&source, &first.conversation_id, &pair)
            .await,
        Err(StoreError::Conflict)
    ));
    store
        .complete_conversation_source(&source, &first.conversation_id, &s2, None)
        .await?;
    assert!(matches!(
        store.accept_conversation_input(&owner, &draft, f.now).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store.accept_conversation_input(&owner, &fresh, f.now).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(
        store
            .conversation_event_metadata(
                &owner,
                &EventMetadataRead {
                    agent_id: &f.agent.id,
                    conversation_id: &first.conversation_id,
                    snapshot: &s2,
                    after: None,
                    limit: 10
                },
                f.now
            )
            .await?
            .is_empty()
    );
    assert_eq!(
        store
            .harness_supply(&owner, &f.agent.id, f.now)
            .await?
            .supply,
        HarnessSupply::Managed
    );
    let wrong_native = SourceBinding {
        native_session: NativeConversationId::parse("new-context-is-not-restoration".to_owned())?,
        ..replacement
    };
    assert!(matches!(
        store.bind_conversation_source(&source, &wrong_native).await,
        Err(StoreError::InvalidTransition)
    ));
    projections::exercise(store, f, &first.conversation_id, &s2).await?;
    store
        .close_connector_connection(
            &f.session,
            &key,
            &chapchoo_persistence::ConnectionOwner {
                control_id: f.control.clone(),
                connection_id: f.connection.clone(),
            },
        )
        .await?;
    assert!(matches!(
        store
            .complete_conversation_source(&source, &first.conversation_id, &s2, None)
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store.harness_supply(&owner, &f.other_agent, f.now).await,
        Err(StoreError::NotFound)
    ));
    store
        .save_harness_supply(
            &other,
            &SaveHarnessSupply {
                agent_id: f.other_agent.clone(),
                request_id: "other-managed".to_owned(),
                expected_request_id: None,
                supply: HarnessSupply::Managed,
            },
            f.now,
        )
        .await?;
    store.leave_team(&other, &f.team_id, f.now).await?;
    assert!(matches!(
        store.harness_supply(&other, &f.other_agent, f.now).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .current_conversation(&other, &f.other_agent, f.now)
            .await,
        Err(StoreError::NotFound)
    ));
    Ok(())
}

#[derive(PartialEq, Eq, diesel::QueryableByName)]
struct RetainedBytes {
    #[diesel(sql_type=Binary)]
    bytes: Vec<u8>,
}

#[derive(diesel::QueryableByName, PartialEq, Eq)]
struct RetainedProjectedField {
    #[diesel(sql_type=diesel::sql_types::Text)]
    metadata: String,
    #[diesel(sql_type=Binary)]
    bytes: Vec<u8>,
}

#[derive(diesel::QueryableByName)]
struct BackendPid {
    #[diesel(sql_type=diesel::sql_types::Integer)]
    pid: i32,
}
#[derive(diesel::QueryableByName)]
struct LockWait {
    #[diesel(sql_type=diesel::sql_types::Bool)]
    blocked: bool,
}

async fn source_write_holds_control_lock(
    store: &Store,
    fixture: &Fixture,
    url: &str,
) -> Result<(), TestError> {
    use diesel_async::{AsyncConnection, SimpleAsyncConnection};
    let key = fixture.connector.digest();
    let prepared = store
        .prepare_connector_connection(&fixture.session, &key, &fixture.control)
        .await?;
    store
        .admit_connector_connection(
            &fixture.session,
            &key,
            prepared.expected.as_ref(),
            &prepared.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let current = store
        .current_conversation(&fixture.owner.digest(), &fixture.agent.id, fixture.now)
        .await?
        .ok_or("Missing conversation for lock regression")?;
    let binding = SourceBinding {
        conversation_id: current.id,
        expected_previous: current.source.snapshot().cloned(),
        snapshot: snapshot("control-lock-proof")?,
        native_session: current.native_session_id.ok_or("Missing native context")?,
    };
    let mut blocker = AsyncPgConnection::establish(url).await?;
    let pid: BackendPid = diesel_async::RunQueryDsl::get_result(
        diesel::sql_query("SELECT pg_backend_pid() AS pid"),
        &mut blocker,
    )
    .await?;
    blocker.batch_execute("BEGIN; SELECT c.id FROM conversations c JOIN agents a ON a.id=c.agent_id WHERE a.handle='chat-fixture' FOR UPDATE OF c").await?;
    let store = store.clone();
    let session = fixture.session.clone();
    let control = fixture.control.clone();
    let connection = prepared.owner.connection_id;
    let writer = tokio::spawn(async move {
        store
            .bind_conversation_source(
                &ConnectorSource {
                    session_id: &session,
                    control_id: &control,
                    connection_id: &connection,
                    credential: &key,
                },
                &binding,
            )
            .await
    });
    let mut observer = AsyncPgConnection::establish(url).await?;
    // An authoritative database lock witness, not a sleep or scheduling guess.
    let queued=tokio::time::timeout(std::time::Duration::from_secs(5),async {
        loop {
            let blocked:LockWait=diesel_async::RunQueryDsl::get_result(
                diesel::sql_query("SELECT EXISTS (SELECT 1 FROM pg_stat_activity WHERE $1=ANY(pg_blocking_pids(pid))) AS blocked")
                    .bind::<diesel::sql_types::Integer,_>(pid.pid),&mut observer,
            ).await?;
            if blocked.blocked { return Ok::<(),TestError>(()); }
        }
    }).await;
    // drain_control requires this same exclusive row lock. NOWAIT gives a
    // deterministic observation while the source writer waits on its final row.
    let probe=observer.batch_execute("SELECT id FROM control_instances WHERE endpoint='http://127.0.0.1:1' FOR UPDATE NOWAIT").await;
    let release = blocker.batch_execute("ROLLBACK").await;
    let completed = writer.await;
    let queued = queued
        .map_err(|_| TestError::from("Source write never reached its locked conversation row"))
        .and_then(|result| result);
    let validation = support::combine(queued, release.map_err(Into::into)).map(|_| ());
    let completed = completed
        .map_err(TestError::from)
        .and_then(|result| result.map_err(Into::into))
        .map(|_| ());
    support::combine(validation, completed)?;
    assert!(
        probe.as_ref().is_err_and(|error| error
            .to_string()
            .contains("could not obtain lock on row in relation \"control_instances\"")),
        "Source write did not retain Control authority until commit: {probe:?}"
    );
    Ok(())
}

macro_rules! preserving_upgrade {
    ($connection:expr,$migrations:expr) => {{
        let connection=&mut *$connection;
        let receipts=diesel::sql_query("SELECT payload_hash AS bytes FROM agent_creation_receipts ORDER BY request_id")
            .load::<RetainedBytes>(connection)?;
        let credentials=diesel::sql_query("SELECT ciphertext AS bytes FROM external_authentications ORDER BY current_request_id")
            .load::<RetainedBytes>(connection)?;
        assert!(!receipts.is_empty() && !credentials.is_empty());
        // Restore the pre-feature schema with real Agent/account/authentication state.
        while connection.applied_migrations()?.iter().any(|version| version.to_string().as_str() > "00000000000011") {
            connection.revert_last_migration($migrations)?;
        }
        connection.run_pending_migrations($migrations)?;
        assert!(receipts==diesel::sql_query("SELECT payload_hash AS bytes FROM agent_creation_receipts ORDER BY request_id")
            .load::<RetainedBytes>(connection)?, "Creation fingerprint changed during conversation migration");
        assert!(credentials==diesel::sql_query("SELECT ciphertext AS bytes FROM external_authentications ORDER BY current_request_id")
            .load::<RetainedBytes>(connection)?, "Protected authentication changed during conversation migration");
    }};
}

macro_rules! preserving_tool_io_upgrade {
    ($connection:expr,$migrations:expr) => {{
        let connection=&mut *$connection;
        while connection.applied_migrations()?.iter().any(|version| version.to_string().as_str() > "00000000000016") {
            connection.revert_last_migration($migrations)?;
        }
        assert!(connection.revert_last_migration($migrations).is_err(), "New tool fields must prevent a destructive downgrade");
        // Explicitly remove only the new synthetic test fields after proving
        // the guard; the migration itself may never discard those facts.
        connection.batch_execute("DELETE FROM conversation_record_fields WHERE field_name IN ('TOOL_INPUT','TOOL_OUTPUT')")?;
        let query="SELECT record_kind || ':' || native_id || ':' || field_name || ':' || CAST(updated_ordinal AS TEXT) AS metadata, content AS bytes FROM conversation_record_fields ORDER BY conversation_id,record_kind,native_id,field_name";
        let retained=diesel::sql_query(query).load::<RetainedProjectedField>(connection)?;
        assert!(!retained.is_empty(), "Preservation requires existing projected data");
        connection.revert_last_migration($migrations)?;
        assert!(retained == diesel::sql_query(query).load::<RetainedProjectedField>(connection)?, "Downgrade changed existing projected fields");
        connection.run_pending_migrations($migrations)?;
        assert!(retained == diesel::sql_query(query).load::<RetainedProjectedField>(connection)?, "Upgrade changed existing projected fields");
    }};
}

macro_rules! preserving_deleted_inputs_upgrade {
    ($connection:expr,$migrations:expr,$down:expr) => {{
        let connection = &mut *$connection;
        connection.batch_execute("CREATE TEMP TABLE conversation_inputs (id TEXT PRIMARY KEY,submitted_text TEXT,deleted_at BIGINT CONSTRAINT input_deleted_body CHECK((submitted_text IS NULL)=(deleted_at IS NOT NULL))); INSERT INTO conversation_inputs VALUES('guard-fixture',NULL,1)")?;
        assert!(connection.transaction::<(),diesel::result::Error,_>(|connection| {
            connection.batch_execute($down)
        }).is_err(),
            "Deleted input bodies cannot be restored by downgrade");
        connection.batch_execute("UPDATE conversation_inputs SET submitted_text='synthetic restored fixture',deleted_at=NULL")?;
        connection.batch_execute($down)?;
        connection.batch_execute("DROP TABLE conversation_inputs")?;
        connection.revert_last_migration($migrations)?;
        connection.run_pending_migrations($migrations)?;
    }};
}
macro_rules! preserving_setup_source_upgrade {
    ($connection:expr,$migrations:expr) => {{
        let connection = &mut *$connection;
        // Exercise the actual guard SQL with synthetic active fences. The
        // completed conversation fixture has already retired its source facts.
        connection.batch_execute("CREATE TEMP TABLE conversation_input_preparation (require_retained_source BOOLEAN NOT NULL, payload TEXT NOT NULL); INSERT INTO conversation_input_preparation VALUES(TRUE,'preserved')")?;
        let down = include_str!("../migrations/sqlite/00000000000019_setup_source_fence/down.sql");
        assert!(connection.transaction::<(), diesel::result::Error, _>(|connection| {
            connection.batch_execute(down)
        }).is_err(), "A retained setup-source fence must prevent a destructive downgrade");
        connection.batch_execute("UPDATE conversation_input_preparation SET require_retained_source=FALSE")?;
        connection.batch_execute(down)?;
        #[derive(diesel::QueryableByName)]
        struct GuardPayload {
            #[diesel(sql_type=diesel::sql_types::Text)]
            payload: String,
        }
        assert_eq!(diesel::sql_query("SELECT payload FROM conversation_input_preparation").get_result::<GuardPayload>(connection)?.payload, "preserved");
        connection.batch_execute("DROP TABLE conversation_input_preparation")?;
        let query = "SELECT request_id AS metadata, submitted_text AS bytes FROM conversation_inputs ORDER BY request_id";
        #[derive(diesel::QueryableByName, PartialEq)]
        struct RetainedInputText {
            #[diesel(sql_type=diesel::sql_types::Text)]
            metadata: String,
            #[diesel(sql_type=diesel::sql_types::Text)]
            bytes: String,
        }
        let retained = diesel::sql_query(query).load::<RetainedInputText>(connection)?;
        connection.revert_last_migration($migrations)?;
        connection.run_pending_migrations($migrations)?;
        assert!(retained == diesel::sql_query(query).load::<RetainedInputText>(connection)?);
    }};
}
macro_rules! preserving_extent_upgrade {
    ($connection:expr,$migrations:expr) => {{
        let connection = &mut *$connection;
        while connection.applied_migrations()?.iter().any(|version| version.to_string().as_str() > "00000000000018") {
            connection.revert_last_migration($migrations)?;
        }
        let query="SELECT record_kind || ':' || native_id || ':' || field_name || ':' || CAST(updated_ordinal AS TEXT) AS metadata, content AS bytes FROM conversation_record_fields ORDER BY conversation_id,record_kind,native_id,field_name";
        let retained=diesel::sql_query(query).load::<RetainedProjectedField>(connection)?;
        assert!(!retained.is_empty());
        connection.revert_last_migration($migrations)?;
        assert!(retained==diesel::sql_query(query).load::<RetainedProjectedField>(connection)?, "Extent downgrade changed original field bytes/ordinals");
        connection.run_pending_migrations($migrations)?;
        assert!(retained==diesel::sql_query(query).load::<RetainedProjectedField>(connection)?, "Extent upgrade changed original field bytes/ordinals");
        #[derive(diesel::QueryableByName)]
        struct InvalidExtents { #[diesel(sql_type=diesel::sql_types::BigInt)] n: i64 }
        assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM conversation_record_fields f LEFT JOIN conversation_field_extents e USING(conversation_id,record_kind,native_id,field_name) WHERE e.ordinal IS NULL OR e.ordinal<>f.updated_ordinal OR e.total_bytes<>length(f.content)").get_result::<InvalidExtents>(connection)?.n,0);
    }};
}

macro_rules! preserving_native_details_upgrade {
    ($connection:expr,$migrations:expr) => {{
        let connection = &mut *$connection;
        while connection.applied_migrations()?.iter().any(|version| version.to_string().as_str() > "00000000000017") {
            connection.revert_last_migration($migrations)?;
        }
        connection.batch_execute("INSERT INTO conversation_native_state(conversation_id,execution_id,source_snapshot,native_session_id,ordinal,occurrence_id,foreground,work_id) SELECT c.id,p.current_execution_id,c.source_snapshot,c.native_session_id,1,'migration-native-state','IDLE',NULL FROM conversations c JOIN agents a ON a.id=c.agent_id JOIN computers p ON p.id=a.computer_id WHERE a.handle='chat-fixture' AND c.source_snapshot IS NOT NULL ON CONFLICT(conversation_id) DO NOTHING")?;
        // Existing private BLOBs witness byte preservation; no credential row is referenced.
        connection.batch_execute("INSERT INTO conversation_native_details(conversation_id,preparation_payload,authentication_id) SELECT s.conversation_id,substr(f.content,1,32768),NULL FROM conversation_native_state s JOIN conversation_record_fields f ON f.conversation_id=s.conversation_id WHERE length(f.content)>0 LIMIT 1")?;
        let query = "SELECT preparation_payload AS bytes FROM conversation_native_details ORDER BY conversation_id";
        let retained = diesel::sql_query(query).load::<RetainedBytes>(connection)?;
        assert!(!retained.is_empty(), "Downgrade guard requires retained details");
        assert!(connection.revert_last_migration($migrations).is_err(), "Native details must prevent destructive downgrade");
        assert!(retained == diesel::sql_query(query).load::<RetainedBytes>(connection)?, "Rejected downgrade changed native details");
        connection.batch_execute("DELETE FROM conversation_native_details")?;
        let fields = "SELECT record_kind || ':' || native_id || ':' || field_name || ':' || CAST(updated_ordinal AS TEXT) AS metadata, content AS bytes FROM conversation_record_fields ORDER BY conversation_id,record_kind,native_id,field_name";
        let old = diesel::sql_query(fields).load::<RetainedProjectedField>(connection)?;
        connection.revert_last_migration($migrations)?;
        connection.run_pending_migrations($migrations)?;
        assert!(old == diesel::sql_query(fields).load::<RetainedProjectedField>(connection)?, "Native detail migration changed retained fields");
    }};
}

#[test]
fn sqlite_conversation_operations_and_preserving_downgrade_guard() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("conversation.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let runtime = tokio::runtime::Runtime::new()?;
    let store = runtime.block_on(Store::connect_sqlite(path.clone(), cipher.clone()))?;
    let fixture = runtime.block_on(fixture(&store))?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    preserving_upgrade!(&mut connection, SQLITE_MIGRATIONS);
    prepare_sqlite(&path, &cipher)?;
    assert_eq!(
        runtime
            .block_on(store.agent(&fixture.owner.digest(), &fixture.agent.id, fixture.now))?
            .id,
        fixture.agent.id
    );
    runtime.block_on(exercise(&store, &fixture))?;
    preserving_deleted_inputs_upgrade!(
        &mut connection,
        SQLITE_MIGRATIONS,
        include_str!("../migrations/sqlite/00000000000020_deleted_inputs/down.sql")
    );
    preserving_setup_source_upgrade!(&mut connection, SQLITE_MIGRATIONS);
    preserving_extent_upgrade!(&mut connection, SQLITE_MIGRATIONS);
    preserving_native_details_upgrade!(&mut connection, SQLITE_MIGRATIONS);
    preserving_tool_io_upgrade!(&mut connection, SQLITE_MIGRATIONS);
    Ok(())
}
#[test]
fn postgres_conversation_operations_and_preserving_downgrade_guard() -> Result<(), TestError> {
    let _fixture_lock = POSTGRES_CONVERSATION_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")
        .map_err(|_| "Disposable PostgreSQL is required")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("conversation_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let runtime = tokio::runtime::Runtime::new()?;
        let store = runtime.block_on(Store::connect_postgres(url.clone(), cipher.clone()))?;
        let fixture = runtime.block_on(fixture(&store))?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        preserving_upgrade!(&mut connection, POSTGRES_MIGRATIONS);
        prepare_postgres(&url, &cipher)?;
        assert_eq!(
            runtime
                .block_on(store.agent(&fixture.owner.digest(), &fixture.agent.id, fixture.now))?
                .id,
            fixture.agent.id
        );
        runtime.block_on(exercise(&store, &fixture))?;
        preserving_deleted_inputs_upgrade!(
            &mut connection,
            POSTGRES_MIGRATIONS,
            include_str!("../migrations/postgres/00000000000020_deleted_inputs/down.sql")
        );
        preserving_setup_source_upgrade!(&mut connection, POSTGRES_MIGRATIONS);
        preserving_extent_upgrade!(&mut connection, POSTGRES_MIGRATIONS);
        preserving_native_details_upgrade!(&mut connection, POSTGRES_MIGRATIONS);
        preserving_tool_io_upgrade!(&mut connection, POSTGRES_MIGRATIONS);
        runtime.block_on(source_write_holds_control_lock(&store, &fixture, &url))?;
        while connection
            .applied_migrations()?
            .iter()
            .any(|version| version.to_string().as_str() > "00000000000012")
        {
            connection.revert_last_migration(POSTGRES_MIGRATIONS)?;
        }
        assert!(
            connection
                .revert_last_migration(POSTGRES_MIGRATIONS)
                .is_err()
        );
        Ok(())
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
