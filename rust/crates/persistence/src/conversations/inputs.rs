use super::*;

pub struct AcceptConversationInput {
    pub agent_id: String,
    pub conversation_id: Option<String>,
    pub request_id: String,
    pub target: ConversationTarget,
    pub text: ConversationText,
}
pub struct ConversationInputReceipt {
    pub id: String,
    pub conversation_id: String,
    pub request_id: String,
    pub target: ConversationTarget,
    pub text: ConversationText,
    pub accepted_at: i64,
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
struct InputRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    conversation_id: EntityId,
    #[diesel(sql_type=Text)]
    request_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    target_snapshot: Option<String>,
    #[diesel(sql_type=Text)]
    submitted_text: String,
    #[diesel(sql_type=BigInt)]
    accepted_at: i64,
}
impl InputRow {
    fn receipt(self) -> Result<ConversationInputReceipt, StoreError> {
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
            text: ConversationText::parse(self.submitted_text)
                .map_err(|_| StoreError::Integrity)?,
            accepted_at: self.accepted_at,
        })
    }
}

impl Store {
    /// Persist product acceptance only. No forwarding/native success is implied.
    pub async fn accept_conversation_input(
        &self,
        actor: &SecretDigest,
        draft: &AcceptConversationInput,
        now: i64,
    ) -> Result<ConversationInputReceipt, StoreError> {
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
                return Ok(previous);
            }
            if owner.desired_state == "DELETING" || !draft.target.matches(&row.source()?) {
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
            Ok(ConversationInputReceipt {
                id: id.to_dto(EntityKind::ConversationOperation),
                conversation_id: row.id.to_dto(EntityKind::Conversation),
                request_id: draft.request_id.clone(),
                target: draft.target.clone(),
                text: draft.text.clone(),
                accepted_at: now,
            })
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
