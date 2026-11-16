//! Durable readiness waiting and exact processing ownership on the outgoing row.
use super::*;
use diesel::sql_types::Bool;

#[derive(Clone)]
pub struct ConversationInputCandidate {
    pub id: String,
    pub agent_id: String,
    pub conversation_id: String,
}
/// Internal references to an issued claim, never standalone account authority.
#[derive(Clone)]
pub struct ConversationInputAuthority {
    pub agent_id: String,
    pub conversation_id: String,
    pub input_id: String,
    pub claim_id: String,
}
pub struct ClaimedConversationInput {
    pub agent_id: String,
    pub receipt: ConversationInputReceipt,
    pub claim_id: String,
}
pub struct ConversationInputWake {
    pub agent_id: String,
    pub input_id: String,
    pub predecessor: String,
}
macro_rules! redacted { ($($t:ty),+) => {$(impl fmt::Debug for $t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($t),"([REDACTED])")) }
})+}; }
redacted!(
    ConversationInputCandidate,
    ConversationInputAuthority,
    ClaimedConversationInput,
    ConversationInputWake
);

macro_rules! insert_waiting_input {
    ($connection:expr,$dialect:expr,$id:expr,$conversation:expr,$actor:expr,$computer:expr,$request:expr,$target:expr,$text:expr,$now:expr,$replaces:expr) => {{
        #[derive(diesel::QueryableByName)]
        struct Context {
            #[diesel(sql_type=IdentitySql)] session_id: EntityId,
            #[diesel(sql_type=Text)] kind: String,
            #[diesel(sql_type=Text)] desired_state: String,
            #[diesel(sql_type=Text)] current_request_id: String,
        }
        let context=diesel::sql_query($dialect.query("SELECT s.id AS session_id,c.kind,c.desired_state,c.current_request_id FROM auth_sessions s CROSS JOIN computers c WHERE c.id=? AND s.revoked_at IS NULL AND s.expires_at>? AND (s.current_hash=? OR (s.previous_hash=? AND s.previous_valid_until>?))"))
            .bind::<IdentitySql,_>($computer).bind::<BigInt,_>($now)
            .bind::<Binary,_>($actor.as_bytes().as_slice()).bind::<Binary,_>($actor.as_bytes().as_slice()).bind::<BigInt,_>($now)
            .get_result::<Context>($connection).await?;
        #[derive(diesel::QueryableByName)]
        struct Order { #[diesel(sql_type=BigInt)] next_order:i64 }
        let order=diesel::sql_query($dialect.query("SELECT CAST(COALESCE(MAX(input_order),0)+1 AS BIGINT) AS next_order FROM conversation_inputs WHERE conversation_id=?"))
            .bind::<IdentitySql,_>($conversation).get_result::<Order>($connection).await?;
        let wake=(context.kind=="CLOUD" && context.desired_state=="STOPPED").then_some(context.current_request_id);
        diesel::sql_query($dialect.query("INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at,replaces_input_id,queue_state,consumer_expires_at,readiness_waiting,submitting_session_id,input_order,wake_predecessor) VALUES(?,?,?,?,?,?,?,'PENDING',0,TRUE,?,?,?)"))
            .bind::<IdentitySql,_>($id).bind::<IdentitySql,_>($conversation).bind::<Text,_>($request)
            .bind::<Nullable<Text>,_>($target).bind::<Text,_>($text).bind::<BigInt,_>($now)
            .bind::<Nullable<IdentitySql>,_>($replaces).bind::<IdentitySql,_>(context.session_id)
            .bind::<BigInt,_>(order.next_order).bind::<Nullable<Text>,_>(wake.as_deref()).execute($connection).await?;
    }};
}
pub(super) use insert_waiting_input;

#[derive(diesel::QueryableByName)]
struct CandidateRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    agent_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
}
#[derive(diesel::QueryableByName)]
pub(super) struct ActorRow {
    #[diesel(sql_type=Binary)]
    pub(super) current_hash: Vec<u8>,
}
#[derive(diesel::QueryableByName)]
pub(super) struct PendingBinding {
    #[diesel(sql_type=IdentitySql)]
    pub(super) agent_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    pub(super) conversation_id: EntityId,
    #[diesel(sql_type=Text)]
    pub(super) queue_state: String,
}
#[derive(diesel::QueryableByName)]
struct WaitingRow {
    #[diesel(sql_type=Text)]
    queue_state: String,
    #[diesel(sql_type=Bool)]
    readiness_waiting: bool,
    #[diesel(sql_type=Nullable<BigInt>)]
    input_order: Option<i64>,
    #[diesel(sql_type=Nullable<Text>)]
    wake_predecessor: Option<String>,
}
macro_rules! queued_actor {
    ($connection:expr,$dialect:expr,$input:expr) => {{
        let bound = diesel::sql_query($dialect.query("SELECT s.user_id FROM conversation_inputs i JOIN auth_sessions s ON s.id=i.submitting_session_id WHERE i.id=?"))
            .bind::<IdentitySql,_>($input).get_result::<crate::models::UserId>($connection).await.optional()?;
        if let Some(bound) = bound {
            // Session mutation takes User before Session. Load the current
            // digest after that lock so concurrent rotation is not withdrawal.
            let user = diesel::sql_query($dialect.query($dialect.user_lock()))
                .bind::<IdentitySql,_>(bound.user_id).get_result::<crate::models::UserRow>($connection).await.optional()?;
            if user.is_some() {
                let now = crate::directory::database_now!($connection, $dialect);
                let row = diesel::sql_query($dialect.query("SELECT s.current_hash FROM conversation_inputs i JOIN auth_sessions s ON s.id=i.submitting_session_id WHERE i.id=? AND s.user_id=? AND s.revoked_at IS NULL AND s.expires_at>?"))
                    .bind::<IdentitySql,_>($input).bind::<IdentitySql,_>(bound.user_id).bind::<BigInt,_>(now)
                    .get_result::<$crate::conversations::waiting::ActorRow>($connection).await.optional()?;
                row.map(|r| r.current_hash.as_slice().try_into().map(SecretDigest::from_bytes).map_err(|_|StoreError::Integrity)).transpose()?
            } else { None }
        } else { None }
    }};
}
pub(super) use queued_actor;

/// Qualify only this issued claim inside the caller's completed operation.
/// The resolved digest never leaves persistence or survives these authority locks.
macro_rules! queued_owned_agent {
    ($connection:expr,$dialect:expr,$authority:expr) => {{
        'qualified: {
            let authority = $authority;
            let agent = EntityId::from_dto(EntityKind::Agent, &authority.agent_id)?;
            let conversation = EntityId::from_dto(EntityKind::Conversation, &authority.conversation_id)?;
            let input = EntityId::from_dto(EntityKind::ConversationOperation, &authority.input_id)?;
            let read = "SELECT c.agent_id,i.conversation_id,i.queue_state FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.id=?";
            let binding = diesel::sql_query($dialect.query(read))
                .bind::<IdentitySql,_>(input)
                .get_result::<$crate::conversations::waiting::PendingBinding>($connection).await.optional()?;
            let Some(binding) = binding else { break 'qualified None; };
            if binding.agent_id != agent || binding.conversation_id != conversation {
                return Err(StoreError::NotFound);
            }
            if binding.queue_state != "PENDING" { break 'qualified None; }
            let actor = $crate::conversations::waiting::queued_actor!($connection, $dialect, input);
            let Some(actor) = actor else {
                let retained = diesel::sql_query($dialect.query(read)).bind::<IdentitySql,_>(input)
                    .get_result::<$crate::conversations::waiting::PendingBinding>($connection).await.optional()?;
                if retained.is_none_or(|row| row.queue_state != "PENDING") { break 'qualified None; }
                return Err(StoreError::Unauthenticated);
            };
            let now = crate::directory::database_now!($connection, $dialect);
            let owned: Result<_, StoreError> = async {
                Ok($crate::conversations::authority::owned_agent!($connection, $dialect, &actor, agent, now))
            }.await;
            let retained = diesel::sql_query($dialect.query(read)).bind::<IdentitySql,_>(input)
                .get_result::<$crate::conversations::waiting::PendingBinding>($connection).await.optional()?;
            if retained.is_none_or(|row| row.queue_state != "PENDING") {
                match owned {
                    Ok(_) | Err(StoreError::Unauthenticated | StoreError::NotFound) => break 'qualified None,
                    Err(error) => return Err(error),
                }
            }
            let owner = owned?;
            $crate::conversations::input_queue::require_consumer!($connection, $dialect, input, &authority.claim_id);
            let now = crate::directory::database_now!($connection, $dialect);
            if $crate::conversations::waiting::queued_actor!($connection, $dialect, input).is_none() {
                return Err(StoreError::Unauthenticated);
            }
            break 'qualified Some((owner, now));
        }
    }};
}
pub(super) use queued_owned_agent;

macro_rules! waiting_owner {
    ($connection:expr,$dialect:expr,$input:expr,$actor:expr,$agent:expr,$now:expr) => {{
        let owner: Result<_, StoreError> = async {
            Ok(owned_agent!($connection, $dialect, $actor, $agent, $now))
        }.await;
        match owner {
            Ok(owner) => owner,
            Err(StoreError::Unauthenticated | StoreError::NotFound) => {
                diesel::sql_query($dialect.query("UPDATE conversation_inputs SET queue_state='ERROR',wake_predecessor=NULL WHERE id=? AND readiness_waiting=TRUE AND queue_state='PENDING' AND conversation_id IN (SELECT id FROM conversations WHERE agent_id=?)"))
                    .bind::<IdentitySql,_>($input).bind::<IdentitySql,_>($agent).execute($connection).await?;
                return Ok(None);
            }
            Err(error) => return Err(error),
        }
    }};
}
impl Store {
    /// A projection qualified by the exact issued claim and current bound session.
    pub async fn queued_conversation_agent(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<crate::AgentSummary, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, &authority.agent_id)?;
        operation!(self, |connection, dialect| {
            let (_, now) = queued_owned_agent!(connection, dialect, authority)
                .ok_or(StoreError::InvalidTransition)?;
            let query = format!("{} WHERE a.id=?", crate::computers::agent_query());
            diesel::sql_query(dialect.query(&query))
                .bind::<IdentitySql, _>(agent)
                .get_result::<crate::computers::AgentComputerRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?
                .summary(now)
        })
    }

    /// Admit queue-owned Start through the shared lifecycle transaction with
    /// final bound-input/session qualification under its ownership locks.
    pub async fn start_waiting_conversation_input(
        &self,
        wake: &ConversationInputWake,
    ) -> Result<crate::AgentSummary, StoreError> {
        self.admit_lifecycle(
            &wake.agent_id,
            &crate::LifecycleRequest {
                request_id: wake.input_id.clone(),
                expected_request_id: wake.predecessor.clone(),
                action: chapchoo_domain::LifecycleAction::Start,
            },
            crate::computers::LifecycleAuthority::WaitingInput(&wake.input_id),
        )
        .await
    }

    /// Freeze an observational scan frontier so arrivals cannot postpone its wrap.
    pub async fn waiting_conversation_input_frontier(&self) -> Result<Option<String>, StoreError> {
        operation!(self, |connection, dialect| {
            let row = diesel::sql_query(dialect.query("SELECT i.id,c.agent_id,i.conversation_id FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.queue_state='PENDING' AND (i.readiness_waiting=TRUE OR i.wake_predecessor IS NOT NULL) ORDER BY i.id DESC LIMIT 1"))
                .get_result::<CandidateRow>(connection).await.optional()?;
            Ok(row.map(|row| row.id.to_dto(EntityKind::ConversationOperation)))
        })
    }

    /// Bounded metadata within one finite scan; neither bound grants processing authority.
    pub async fn waiting_conversation_inputs(
        &self,
        after: Option<&str>,
        through: &str,
    ) -> Result<Vec<ConversationInputCandidate>, StoreError> {
        let after = after
            .map(|v| EntityId::from_dto(EntityKind::ConversationOperation, v))
            .transpose()?;
        let through = EntityId::from_dto(EntityKind::ConversationOperation, through)?;
        operation!(self, |connection, dialect| {
            let rows = diesel::sql_query(dialect.query("SELECT i.id,c.agent_id,i.conversation_id FROM conversation_inputs i JOIN conversations c ON c.id=i.conversation_id WHERE i.queue_state='PENDING' AND (i.readiness_waiting=TRUE OR i.wake_predecessor IS NOT NULL) AND (? IS NULL OR i.id>?) AND i.id<=? ORDER BY i.id LIMIT 32"))
                .bind::<Nullable<IdentitySql>,_>(after).bind::<Nullable<IdentitySql>,_>(after).bind::<IdentitySql,_>(through).load::<CandidateRow>(connection).await?;
            Ok(rows
                .into_iter()
                .map(|r| ConversationInputCandidate {
                    id: r.id.to_dto(EntityKind::ConversationOperation),
                    agent_id: r.agent_id.to_dto(EntityKind::Agent),
                    conversation_id: r.conversation_id.to_dto(EntityKind::Conversation),
                })
                .collect())
        })
    }
    pub async fn conversation_input_wake(
        &self,
        candidate: &ConversationInputCandidate,
    ) -> Result<Option<ConversationInputWake>, StoreError> {
        let id = EntityId::from_dto(EntityKind::ConversationOperation, &candidate.id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, &candidate.agent_id)?;
        operation!(self, |connection, dialect| {
            let now = crate::directory::database_now!(connection, dialect);
            let Some(actor) = queued_actor!(connection, dialect, id) else {
                return Ok(None);
            };
            let owner = waiting_owner!(connection, dialect, id, &actor, agent, now);
            #[derive(diesel::QueryableByName)]
            struct Power {
                #[diesel(sql_type=Text)]
                kind: String,
                #[diesel(sql_type=Text)]
                current_request_id: String,
            }
            let power = diesel::sql_query(
                dialect.query("SELECT kind,current_request_id FROM computers WHERE id=?"),
            )
            .bind::<IdentitySql, _>(owner.computer_id)
            .get_result::<Power>(connection)
            .await?;
            let row=diesel::sql_query(dialect.query("SELECT queue_state,readiness_waiting,input_order,wake_predecessor FROM conversation_inputs WHERE id=?"))
                .bind::<IdentitySql,_>(id).get_result::<WaitingRow>(connection).await.optional()?;
            let Some(row) = row else { return Ok(None) };
            let Some(predecessor) = row.wake_predecessor else {
                return Ok(None);
            };
            if row.queue_state != "PENDING"
                || power.kind != "CLOUD"
                || owner.desired_state == "DELETING"
                || (power.current_request_id != predecessor
                    && power.current_request_id != candidate.id)
            {
                diesel::sql_query(
                    dialect
                        .query("UPDATE conversation_inputs SET wake_predecessor=NULL WHERE id=?"),
                )
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                return Ok(None);
            }
            Ok(Some(ConversationInputWake {
                agent_id: candidate.agent_id.clone(),
                input_id: candidate.id.clone(),
                predecessor,
            }))
        })
    }
    pub async fn settle_conversation_input_wake(
        &self,
        input: &str,
        predecessor: &str,
    ) -> Result<(), StoreError> {
        let id = EntityId::from_dto(EntityKind::ConversationOperation, input)?;
        operation!(self, |connection, dialect| {
            diesel::sql_query(dialect.query("UPDATE conversation_inputs SET wake_predecessor=NULL WHERE id=? AND wake_predecessor=?"))
                .bind::<IdentitySql,_>(id).bind::<Text,_>(predecessor).execute(connection).await?;
            Ok(())
        })
    }
    /// Claim one FIFO head under current owner/session authority before loading its body.
    pub async fn claim_conversation_input(
        &self,
        candidate: &ConversationInputCandidate,
        policy: crate::DirectoryPolicy,
    ) -> Result<Option<ClaimedConversationInput>, StoreError> {
        let id = EntityId::from_dto(EntityKind::ConversationOperation, &candidate.id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, &candidate.agent_id)?;
        let conversation =
            EntityId::from_dto(EntityKind::Conversation, &candidate.conversation_id)?;
        operation!(self, |connection, dialect| {
            let now = crate::directory::database_now!(connection, dialect);
            let Some(actor) = queued_actor!(connection, dialect, id) else {
                diesel::sql_query(dialect.query("UPDATE conversation_inputs SET queue_state='ERROR',wake_predecessor=NULL WHERE id=? AND readiness_waiting=TRUE AND queue_state='PENDING'"))
                    .bind::<IdentitySql,_>(id).execute(connection).await?;
                return Ok(None);
            };
            let owner = waiting_owner!(connection, dialect, id, &actor, agent, now);
            if !super::authority::input_ready!(connection, dialect, owner.computer_id) {
                return Ok(None);
            };
            let current = conversation!(connection, dialect, conversation);
            if current.agent_id != agent {
                return Err(StoreError::NotFound);
            };
            let sql = match dialect {
                Dialect::Postgres => {
                    "SELECT queue_state,readiness_waiting,input_order,wake_predecessor FROM conversation_inputs WHERE id=? AND conversation_id=? FOR UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT queue_state,readiness_waiting,input_order,wake_predecessor FROM conversation_inputs WHERE id=? AND conversation_id=?"
                }
            };
            let Some(row) = diesel::sql_query(dialect.query(sql))
                .bind::<IdentitySql, _>(id)
                .bind::<IdentitySql, _>(conversation)
                .get_result::<WaitingRow>(connection)
                .await
                .optional()?
            else {
                return Ok(None);
            };
            if row.queue_state != "PENDING" || !row.readiness_waiting {
                return Ok(None);
            };
            let order = row.input_order.ok_or(StoreError::Integrity)?;
            #[derive(diesel::QueryableByName)]
            struct Count {
                #[diesel(sql_type=BigInt)]
                n: i64,
            }
            let prior=diesel::sql_query(dialect.query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_inputs WHERE conversation_id=? AND queue_state='PENDING' AND (input_order IS NULL OR input_order<?)"))
                .bind::<IdentitySql,_>(conversation).bind::<BigInt,_>(order).get_result::<Count>(connection).await?;
            if prior.n != 0 {
                return Ok(None);
            };
            let now = crate::directory::database_now!(connection, dialect);
            // Ownership locks can wait beyond session expiry or rotation.
            // Qualify the current session again before loading original text.
            let Some(actor) = queued_actor!(connection, dialect, id) else {
                diesel::sql_query(dialect.query("UPDATE conversation_inputs SET queue_state='ERROR',wake_predecessor=NULL WHERE id=? AND readiness_waiting=TRUE AND queue_state='PENDING'"))
                    .bind::<IdentitySql,_>(id).execute(connection).await?;
                return Ok(None);
            };
            let owner = waiting_owner!(connection, dialect, id, &actor, agent, now);
            if !super::authority::input_ready!(connection, dialect, owner.computer_id) {
                return Ok(None);
            }
            let claim = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("UPDATE conversation_inputs SET readiness_waiting=FALSE,consumer_claim_id=?,consumer_expires_at=? WHERE id=?"))
                .bind::<IdentitySql,_>(claim).bind::<BigInt,_>(policy.expiry(now,true)?).bind::<IdentitySql,_>(id).execute(connection).await?;
            let receipt =
                diesel::sql_query(dialect.query("SELECT * FROM conversation_inputs WHERE id=?"))
                    .bind::<IdentitySql, _>(id)
                    .get_result::<super::inputs::InputRow>(connection)
                    .await?
                    .receipt()?;
            Ok(Some(ClaimedConversationInput {
                agent_id: candidate.agent_id.clone(),
                receipt,
                claim_id: claim.to_dto(EntityKind::ConversationOperation),
            }))
        })
    }
    /// Observe retained work under its actual submitting session. Consumption
    /// ends this observation before a vanished binding could look like withdrawal.
    pub async fn queued_conversation_input_pending(
        &self,
        authority: &ConversationInputAuthority,
    ) -> Result<bool, StoreError> {
        operation!(self, |connection, dialect| {
            Ok(queued_owned_agent!(connection, dialect, authority).is_some())
        })
    }
    /// Stale workers cannot finalize or release a successor's ownership. Native uncertainty is never reclaimable.
    pub async fn release_conversation_input_claim(
        &self,
        conversation: &str,
        input: &str,
        claim: &str,
        failed: bool,
    ) -> Result<super::input_queue::ConversationInputQueueState, StoreError> {
        let conversation = EntityId::from_dto(EntityKind::Conversation, conversation)?;
        let input = EntityId::from_dto(EntityKind::ConversationOperation, input)?;
        let claim = EntityId::from_dto(EntityKind::ConversationOperation, claim)?;
        operation!(self, |connection, dialect| {
            let changed = diesel::sql_query(dialect.query("UPDATE conversation_inputs SET readiness_waiting=CASE WHEN ?=FALSE AND NOT EXISTS(SELECT 1 FROM conversation_input_preparation p WHERE p.operation_id=conversation_inputs.id) AND NOT EXISTS(SELECT 1 FROM conversation_input_delivery d WHERE d.operation_id=conversation_inputs.id) THEN TRUE ELSE FALSE END,queue_state=CASE WHEN ?=TRUE OR EXISTS(SELECT 1 FROM conversation_input_preparation p WHERE p.operation_id=conversation_inputs.id) OR EXISTS(SELECT 1 FROM conversation_input_delivery d WHERE d.operation_id=conversation_inputs.id) THEN 'ERROR' ELSE 'PENDING' END,consumer_claim_id=NULL,consumer_expires_at=0 WHERE id=? AND conversation_id=? AND consumer_claim_id=? AND queue_state='PENDING'"))
                .bind::<Bool,_>(failed).bind::<Bool,_>(failed).bind::<IdentitySql,_>(input).bind::<IdentitySql,_>(conversation).bind::<IdentitySql,_>(claim).execute(connection).await?;
            if changed == 0 {
                return Ok(super::input_queue::ConversationInputQueueState::Absent);
            }
            let row = diesel::sql_query(dialect.query("SELECT queue_state,consumer_expires_at,readiness_waiting,consumer_claim_id FROM conversation_inputs WHERE id=?"))
                .bind::<IdentitySql,_>(input).get_result::<super::input_queue::QueueRow>(connection).await?;
            row.state()
        })
    }
}
