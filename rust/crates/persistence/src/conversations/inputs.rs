use super::*;

#[derive(Clone)]
pub struct AcceptConversationInput {
    pub agent_id: String,
    pub conversation_id: Option<String>,
    pub request_id: String,
    pub target: ConversationTarget,
    pub text: ConversationText,
}
#[derive(Clone)]
pub struct ConversationInputReceipt {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub target: ConversationTarget,
    pub text: ConversationText,
    pub accepted_at: i64,
}
/// Bounded acceptance metadata; status reads never load the submitted text.
pub struct ConversationInputReference {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub target: ConversationTarget,
    pub accepted_at: i64,
    pub deleted: bool,
}
impl fmt::Debug for ConversationInputReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationInputReference([REDACTED])")
    }
}
#[derive(diesel::QueryableByName)]
pub(super) struct InputReferenceRow {
    #[diesel(sql_type=IdentitySql)]
    pub(super) id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    pub(super) conversation_id: EntityId,
    #[diesel(sql_type=Text)]
    pub(super) request_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub(super) target_snapshot: Option<String>,
    #[diesel(sql_type=BigInt)]
    pub(super) accepted_at: i64,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    pub(super) deleted: bool,
}
impl InputReferenceRow {
    pub(super) fn reference(self) -> Result<ConversationInputReference, StoreError> {
        Ok(ConversationInputReference {
            id: self.id.to_dto(EntityKind::ConversationOperation),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            request_id: self.request_id,
            target: match self.target_snapshot {
                Some(snapshot) => ConversationTarget::Snapshot(
                    SnapshotId::parse(snapshot).map_err(|_| StoreError::Integrity)?,
                ),
                None => ConversationTarget::NotStarted,
            },
            accepted_at: self.accepted_at,
            deleted: self.deleted,
        })
    }
}
pub enum ConversationInputAdmission {
    Accepted(ConversationInputReceipt),
    Existing(ConversationInputReceipt),
}
impl ConversationInputAdmission {
    pub fn into_receipt(self) -> ConversationInputReceipt {
        match self {
            Self::Accepted(receipt) | Self::Existing(receipt) => receipt,
        }
    }
}
impl fmt::Debug for ConversationInputAdmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationInputAdmission([REDACTED])")
    }
}
impl fmt::Debug for AcceptConversationInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AcceptConversationInput([REDACTED])")
    }
}
impl fmt::Debug for ConversationInputReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConversationInputReceipt([REDACTED])")
    }
}
#[derive(diesel::QueryableByName)]
pub(super) struct InputRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    target_snapshot: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    submitted_text: Option<String>,
    #[diesel(sql_type=BigInt)]
    accepted_at: i64,
}
impl InputRow {
    pub(super) fn receipt(self) -> Result<ConversationInputReceipt, StoreError> {
        Ok(ConversationInputReceipt {
            id: self.id.to_dto(EntityKind::ConversationOperation),
            conversation_id: self.conversation_id.to_dto(EntityKind::Conversation),
            request_id: self.request_id,
            target: match self.target_snapshot {
                Some(snapshot) => ConversationTarget::Snapshot(
                    SnapshotId::parse(snapshot).map_err(|_| StoreError::Integrity)?,
                ),
                None => ConversationTarget::NotStarted,
            },
            text: ConversationText::parse(
                self.submitted_text.ok_or(StoreError::InvalidTransition)?,
            )
            .map_err(|_| StoreError::Integrity)?,
            accepted_at: self.accepted_at,
        })
    }
}

impl Store {
    pub async fn conversation_input_reference(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        request: &str,
        now: i64,
    ) -> Result<Option<ConversationInputReference>, StoreError> {
        request_id(request)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            diesel::sql_query(dialect.query(
                "SELECT id,conversation_id,request_id,target_snapshot,accepted_at,(deleted_at IS NOT NULL) AS deleted FROM conversation_inputs WHERE conversation_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(id)
            .bind::<Text, _>(request)
            .get_result::<InputReferenceRow>(connection)
            .await
            .optional()?
            .map(InputReferenceRow::reference)
            .transpose()
        })
    }
    /// Persist product acceptance only. No forwarding/native success is implied.
    pub async fn accept_conversation_input(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationInput,
        now: i64,
    ) -> Result<ConversationInputReceipt, StoreError> {
        Ok(self
            .admit_conversation_input(actor, draft, now)
            .await?
            .into_receipt())
    }
    /// Distinguish a new product operation from receipt recovery in the same
    /// completed transaction. Only a new operation enters its processing owner.
    pub async fn admit_conversation_input(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationInput,
        now: i64,
    ) -> Result<ConversationInputAdmission, StoreError> {
        request_id(&draft.request_id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, &draft.agent_id)?;
        let expected = draft
            .conversation_id
            .as_deref()
            .map(|id| EntityId::from_dto(EntityKind::Conversation, id))
            .transpose()?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let current = diesel::sql_query(dialect.query("SELECT c.* FROM conversations c JOIN agent_conversations a ON a.conversation_id=c.id AND a.agent_id=c.agent_id WHERE a.agent_id=?"))
                .bind::<IdentitySql,_>(agent).get_result::<ConversationRow>(connection).await.optional()?;
            let row = match current {
                Some(row) => row,
                None => {
                    if expected.is_some() || draft.target != ConversationTarget::NotStarted {
                        return Err(StoreError::NotFound);
                    }
                    if owner.desired_state == "DELETING" {
                        return Err(StoreError::InvalidTransition);
                    }
                    let id = EntityId::generate(now)?;
                    diesel::sql_query(dialect.query("INSERT INTO conversations(id,agent_id,source_state,created_at) VALUES(?,?,'NOT_STARTED',?)"))
                        .bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(agent).bind::<BigInt,_>(now).execute(connection).await?;
                    diesel::sql_query(dialect.query(
                        "INSERT INTO agent_conversations(agent_id,conversation_id) VALUES(?,?)",
                    ))
                    .bind::<IdentitySql, _>(agent)
                    .bind::<IdentitySql, _>(id)
                    .execute(connection)
                    .await?;
                    conversation!(connection, dialect, id)
                }
            };
            if expected.is_some_and(|expected| expected != row.id) {
                return Err(StoreError::Conflict);
            }
            let previous = diesel::sql_query(dialect.query(
                "SELECT * FROM conversation_inputs WHERE conversation_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(row.id)
            .bind::<Text, _>(&draft.request_id)
            .get_result::<InputRow>(connection)
            .await
            .optional()?;
            if let Some(previous) = previous {
                let previous = previous.receipt()?;
                if previous.target != draft.target || previous.text != draft.text {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationInputAdmission::Existing(previous));
            }
            let source = row.source()?;
            let joins_startup = if draft.target.matches(&source) {
                true
            } else if draft.target != ConversationTarget::NotStarted
                || matches!(source, ConversationSource::NotStarted)
            {
                false
            } else {
                #[derive(diesel::QueryableByName)]
                struct InputCount {
                    #[diesel(sql_type=BigInt)]
                    n: i64,
                }
                let inputs = diesel::sql_query(dialect.query(
                    "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_inputs WHERE conversation_id=?",
                ))
                .bind::<IdentitySql, _>(row.id)
                .get_result::<InputCount>(connection)
                .await?;
                #[derive(diesel::QueryableByName)]
                struct CurrentExecution {
                    #[diesel(sql_type = Nullable<IdentitySql>)]
                    current_execution_id: Option<EntityId>,
                }
                let execution = diesel::sql_query(
                    dialect.query("SELECT current_execution_id FROM computers WHERE id=?"),
                )
                .bind::<IdentitySql, _>(owner.computer_id)
                .get_result::<CurrentExecution>(connection)
                .await?;
                let Some(execution_id) = execution.current_execution_id else {
                    return Err(StoreError::InvalidTransition);
                };
                let snapshot = row.source_snapshot.clone();
                let native = row.native_session_id.clone();
                let startup = diesel::sql_query(dialect.query(
                    "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_startups WHERE conversation_id=? AND ((preparation_state='READY' AND ready_snapshot=? AND ready_native_session=?) OR (preparation_state='POSSIBLE' AND execution_id=? AND ? IN ('LOADING','READY') AND (fresh_native_allowed=FALSE OR ?='LOADING')))",
                ))
                .bind::<IdentitySql, _>(row.id)
                .bind::<Nullable<Text>, _>(snapshot.as_deref())
                .bind::<Nullable<Text>, _>(native.as_deref())
                .bind::<IdentitySql, _>(execution_id)
                .bind::<Text, _>(&row.source_state)
                .bind::<Text, _>(&row.source_state)
                .get_result::<InputCount>(connection)
                .await?;
                inputs.n == 0 && startup.n > 0
            };
            if owner.desired_state == "DELETING" || !joins_startup {
                return Err(StoreError::InvalidTransition);
            }
            if expected.is_none() && draft.target != ConversationTarget::NotStarted {
                return Err(StoreError::InvalidInput);
            }
            let id = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("INSERT INTO conversation_inputs(id,conversation_id,request_id,target_snapshot,submitted_text,accepted_at) VALUES(?,?,?,?,?,?)"))
                .bind::<IdentitySql,_>(id).bind::<IdentitySql,_>(row.id).bind::<Text,_>(&draft.request_id)
                .bind::<Nullable<Text>,_>(draft.target.snapshot().map(SnapshotId::as_str))
                .bind::<Text,_>(draft.text.as_str()).bind::<BigInt,_>(now).execute(connection).await?;
            Ok(ConversationInputAdmission::Accepted(
                ConversationInputReceipt {
                    id: id.to_dto(EntityKind::ConversationOperation),
                    conversation_id: row.id.to_dto(EntityKind::Conversation),
                    request_id: draft.request_id.clone(),
                    target: draft.target.clone(),
                    text: draft.text.clone(),
                    accepted_at: now,
                },
            ))
        })
    }

    /// Absence is not evidence of native non-delivery and never resubmits work.
    pub async fn conversation_input(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        request: &str,
        now: i64,
    ) -> Result<Option<ConversationInputReceipt>, StoreError> {
        request_id(request)?;
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            diesel::sql_query(dialect.query(
                "SELECT * FROM conversation_inputs WHERE conversation_id=? AND request_id=?",
            ))
            .bind::<IdentitySql, _>(id)
            .bind::<Text, _>(request)
            .get_result::<InputRow>(connection)
            .await
            .optional()?
            .map(InputRow::receipt)
            .transpose()
        })
    }
}
