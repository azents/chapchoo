//! Durable evidence gates native forwarding; lookup never grants a second effect.

use super::*;

#[derive(Clone, PartialEq, Eq)]
pub struct ConversationDeliveryTarget {
    pub snapshot: SnapshotId,
    pub native_session: NativeConversationId,
}
#[derive(Clone, PartialEq, Eq)]
pub enum ConversationInputOutcome {
    Inserted { native_message_id: String },
    Steered,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationInputReportAcknowledgement {
    Recorded,
    Closed,
}
#[derive(Clone, PartialEq, Eq)]
pub enum ConversationInputDelivery {
    PossibleForwarding,
    Known(ConversationInputOutcome),
}
pub enum ConversationForwardingAdmission {
    /// Only this result authorizes the operation owner to cross the boundary.
    Granted(ConversationForwardingGrant),
    Existing(ConversationInputDelivery),
}
pub struct ConversationForwardingGrant {
    pub execution_id: String,
    pub connector_session_id: String,
}
pub struct ConversationOperationForwarding<'a> {
    pub agent_id: &'a str,
    pub conversation_id: &'a str,
    pub operation_id: &'a str,
    pub target: &'a ConversationDeliveryTarget,
}
pub struct ConversationInputChunk {
    pub bytes: Vec<u8>,
    pub total_bytes: u32,
    pub offset: u32,
}
#[derive(diesel::QueryableByName)]
struct InputChunkRow {
    #[diesel(sql_type=Nullable<Binary>)]
    bytes: Option<Vec<u8>>,
    #[diesel(sql_type=BigInt)]
    total_bytes: i64,
}
macro_rules! safe_debug {
    ($($name:ty),+)=>{$(impl fmt::Debug for $name {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { f.write_str(concat!(stringify!($name),"([REDACTED])")) }
    })+};
}
safe_debug!(
    ConversationDeliveryTarget,
    ConversationInputOutcome,
    ConversationInputDelivery,
    ConversationForwardingAdmission,
    ConversationForwardingGrant,
    ConversationOperationForwarding<'_>,
    ConversationInputChunk
);

#[derive(diesel::QueryableByName)]
struct DeliveryRow {
    #[diesel(sql_type=IdentitySql)]
    execution_id: EntityId,
    #[diesel(sql_type=Text)]
    source_snapshot: String,
    #[diesel(sql_type=Text)]
    native_session_id: String,
    #[diesel(sql_type=Text)]
    delivery_state: String,
    #[diesel(sql_type=Nullable<Text>)]
    native_message_id: Option<String>,
}
#[derive(diesel::QueryableByName)]
pub(super) struct RouteSession {
    #[diesel(sql_type=IdentitySql)]
    pub(super) session_id: EntityId,
}
macro_rules! dispatch_route {
    ($connection:expr,$dialect:expr,$expected:expr,$computer:expr)=>{{
        let expected=$expected;
        if expected.component.control_id!=expected.control.id { return Err(StoreError::InvalidInput); }
        let credential=SecretDigest::from_bytes(expected.component.credential_hash.as_slice().try_into().map_err(|_|StoreError::Integrity)?);
        let session=diesel::sql_query($dialect.query("SELECT session_id FROM connector_connections WHERE computer_id=? AND control_id=? AND connection_id=?"))
            .bind::<IdentitySql,_>($computer)
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::ControlInstance,&expected.component.control_id)?)
            .bind::<IdentitySql,_>(EntityId::from_dto(EntityKind::Connection,&expected.component.connection_id)?)
            .get_result::<$crate::conversations::delivery::RouteSession>($connection).await.optional()?.ok_or(StoreError::Conflict)?;
        let session_id=session.session_id.to_dto(EntityKind::ConnectorSession);
        // Dispatch uses a resolved route digest, never a Connector bearer secret.
        let source=$crate::ConnectorSource { session_id:&session_id,control_id:&expected.component.control_id,
            connection_id:&expected.component.connection_id,credential:&credential };
        let route=connector!($connection,$dialect,&source);
        if route.computer_id!=$computer { return Err(StoreError::NotFound); }
        (route,session_id)
    }};
}
pub(super) use dispatch_route;
impl DeliveryRow {
    fn delivery(&self) -> Result<ConversationInputDelivery, StoreError> {
        Ok(
            match (
                self.delivery_state.as_str(),
                self.native_message_id.as_ref(),
            ) {
                ("POSSIBLE", None) => ConversationInputDelivery::PossibleForwarding,
                ("INSERTED", Some(id)) if !id.is_empty() && id.len() <= 512 => {
                    ConversationInputDelivery::Known(ConversationInputOutcome::Inserted {
                        native_message_id: id.clone(),
                    })
                }
                ("STEERED", None) => {
                    ConversationInputDelivery::Known(ConversationInputOutcome::Steered)
                }
                ("FAILED", None) => {
                    ConversationInputDelivery::Known(ConversationInputOutcome::Failed)
                }
                _ => return Err(StoreError::Integrity),
            },
        )
    }
    fn matches(&self, execution: EntityId, target: &ConversationDeliveryTarget) -> bool {
        self.execution_id == execution
            && self.source_snapshot == target.snapshot.as_str()
            && self.native_session_id == target.native_session.as_str()
    }
}
macro_rules! input {
    ($connection:expr,$dialect:expr,$conversation:expr,$operation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT id,conversation_id,request_id,target_snapshot,accepted_at FROM conversation_inputs WHERE id=? AND conversation_id=?"),
        )
        .bind::<IdentitySql, _>($operation)
        .bind::<IdentitySql, _>($conversation)
        .get_result::<super::inputs::InputReferenceRow>($connection)
        .await
        .optional()?
        .ok_or(StoreError::NotFound)?
    }};
}
macro_rules! delivery {
    ($connection:expr,$dialect:expr,$operation:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT * FROM conversation_input_delivery WHERE operation_id=?"),
        )
        .bind::<IdentitySql, _>($operation)
        .get_result::<DeliveryRow>($connection)
        .await
        .optional()?
    }};
}
pub(super) fn ids(
    draft: &ConversationOperationForwarding<'_>,
) -> Result<(EntityId, EntityId, EntityId), StoreError> {
    Ok((
        EntityId::from_dto(EntityKind::Agent, draft.agent_id)?,
        EntityId::from_dto(EntityKind::Conversation, draft.conversation_id)?,
        EntityId::from_dto(EntityKind::ConversationOperation, draft.operation_id)?,
    ))
}
pub(super) fn current(
    row: &ConversationRow,
    agent: EntityId,
    target: &ConversationDeliveryTarget,
) -> Result<(), StoreError> {
    if row.agent_id != agent {
        return Err(StoreError::NotFound);
    }
    if row.source_snapshot.as_deref() != Some(target.snapshot.as_str())
        || row.native_session_id.as_deref() != Some(target.native_session.as_str())
    {
        return Err(StoreError::Conflict);
    }
    Ok(())
}
impl Store {
    /// Read only a sealed operation's exact input under current execution/source
    /// authority. A byte chunk may split a UTF-8 code point.
    pub async fn connector_input_chunk(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationOperationForwarding<'_>,
        offset: u32,
    ) -> Result<ConversationInputChunk, StoreError> {
        let (agent, id, operation) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            current(&row, agent, draft.target)?;
            let previous =
                delivery!(connection, dialect, operation).ok_or(StoreError::InvalidTransition)?;
            if !previous.matches(route._execution_id, draft.target)
                || !matches!(
                    previous.delivery()?,
                    ConversationInputDelivery::PossibleForwarding
                )
            {
                return Err(StoreError::Conflict);
            }
            let query = match dialect {
                Dialect::Postgres => {
                    "SELECT substring(convert_to(submitted_text,'UTF8') FROM CAST(? AS INTEGER) FOR 32768) AS bytes,CAST(octet_length(submitted_text) AS BIGINT) AS total_bytes FROM conversation_inputs WHERE conversation_id=? AND id=?"
                }
                Dialect::Sqlite => {
                    "SELECT substr(CAST(submitted_text AS BLOB),?,32768) AS bytes,CAST(length(CAST(submitted_text AS BLOB)) AS BIGINT) AS total_bytes FROM conversation_inputs WHERE conversation_id=? AND id=?"
                }
            };
            let chunk = diesel::sql_query(dialect.query(query))
                .bind::<BigInt, _>(i64::from(offset) + 1)
                .bind::<IdentitySql, _>(id)
                .bind::<IdentitySql, _>(operation)
                .get_result::<InputChunkRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let total = u32::try_from(chunk.total_bytes).map_err(|_| StoreError::Integrity)?;
            if total == 0 || total > 16 * 1024 * 1024 || offset > total {
                return Err(StoreError::InvalidInput);
            }
            let bytes = match chunk.bytes {
                Some(bytes) => bytes,
                None if offset == total => Vec::new(),
                None => return Err(StoreError::Integrity),
            };
            Ok(ConversationInputChunk {
                bytes,
                total_bytes: total,
                offset,
            })
        })
    }
    /// Commit possible-forwarding before native dispatch. Existing facts only recover.
    pub async fn begin_conversation_input_forwarding(
        &self,
        actor: &SecretDigest,
        expected: &crate::ResolvedRoute,
        draft: &ConversationOperationForwarding<'_>,
        now: i64,
    ) -> Result<ConversationForwardingAdmission, StoreError> {
        let (agent, id, operation_id) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let owner = owned_agent!(connection, dialect, actor, agent, now);
            let (route, session_id) =
                dispatch_route!(connection, dialect, expected, owner.computer_id);
            let row = conversation!(connection, dialect, id);
            current(&row, agent, draft.target)?;
            let input = input!(connection, dialect, id, operation_id).reference()?;
            if let Some(previous) = delivery!(connection, dialect, operation_id) {
                // A changed execution is not permission to retry an old operation.
                if !previous.matches(route._execution_id, draft.target) {
                    return Err(StoreError::Conflict);
                }
                return Ok(ConversationForwardingAdmission::Existing(
                    previous.delivery()?,
                ));
            }
            if row.source_state != "READY" || owner.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            // NotStarted input survived only the first binding; replacement deletes it.
            if input
                .target
                .snapshot()
                .is_some_and(|snapshot| snapshot != &draft.target.snapshot)
            {
                return Err(StoreError::Conflict);
            }
            super::input_queue::require_consumer!(connection, dialect, operation_id);
            diesel::sql_query(dialect.query("INSERT INTO conversation_input_delivery(operation_id,execution_id,source_snapshot,native_session_id,delivery_state) VALUES(?,?,?,?,'POSSIBLE')"))
                .bind::<IdentitySql,_>(operation_id).bind::<IdentitySql,_>(route._execution_id)
                .bind::<Text,_>(draft.target.snapshot.as_str()).bind::<Text,_>(draft.target.native_session.as_str())
                .execute(connection).await?;
            Ok(ConversationForwardingAdmission::Granted(
                ConversationForwardingGrant {
                    execution_id: route._execution_id.to_dto(EntityKind::Execution),
                    connector_session_id: session_id,
                },
            ))
        })
    }
    /// Successful native pickup ends pending-input ownership. The report ACK
    /// closes the operation; it is not a permanent delivery receipt.
    pub async fn complete_conversation_input_forwarding(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationOperationForwarding<'_>,
        outcome: &ConversationInputOutcome,
    ) -> Result<ConversationInputReportAcknowledgement, StoreError> {
        let (agent, id, operation_id) = ids(draft)?;
        let (state, message) = match outcome {
            ConversationInputOutcome::Inserted { native_message_id } => {
                if native_message_id.is_empty() || native_message_id.len() > 512 {
                    return Err(StoreError::InvalidInput);
                }
                ("INSERTED", Some(native_message_id.as_str()))
            }
            ConversationInputOutcome::Steered => ("STEERED", None),
            ConversationInputOutcome::Failed => ("FAILED", None),
        };
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            current(&row, agent, draft.target)?;
            let pending = diesel::sql_query(dialect.query(
                "SELECT id,conversation_id,request_id,target_snapshot,accepted_at FROM conversation_inputs WHERE id=? AND conversation_id=?",
            ))
            .bind::<IdentitySql, _>(operation_id)
            .bind::<IdentitySql, _>(id)
            .get_result::<super::inputs::InputReferenceRow>(connection)
            .await
            .optional()?;
            let consumed = !matches!(outcome, ConversationInputOutcome::Failed);
            if pending.is_none() {
                // A lost ACK may repeat a successful report after removal.
                // No pending operation remains to deliver, recover or replay.
                return if consumed {
                    #[derive(diesel::QueryableByName)]
                    struct ReceiptCount {
                        #[diesel(sql_type=BigInt)]
                        n: i64,
                    }
                    let receipts = diesel::sql_query(dialect.query(
                        "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_messages WHERE conversation_id=? AND source_input_id=?",
                    ))
                    .bind::<IdentitySql, _>(id)
                    .bind::<IdentitySql, _>(operation_id)
                    .get_result::<ReceiptCount>(connection).await?;
                    Ok(if receipts.n > 0 {
                        ConversationInputReportAcknowledgement::Recorded
                    } else {
                        ConversationInputReportAcknowledgement::Closed
                    })
                } else {
                    Err(StoreError::NotFound)
                };
            }
            let previous = delivery!(connection, dialect, operation_id)
                .ok_or(StoreError::InvalidTransition)?;
            if !previous.matches(route._execution_id, draft.target) {
                return Err(StoreError::Conflict);
            }
            match previous.delivery()? {
                ConversationInputDelivery::Known(previous) if &previous == outcome => {
                    if !consumed {
                        return Ok(ConversationInputReportAcknowledgement::Closed);
                    }
                }
                ConversationInputDelivery::Known(_) => return Err(StoreError::Conflict),
                ConversationInputDelivery::PossibleForwarding => {}
            }
            if consumed {
                let receipt =
                    diesel::sql_query(dialect.query(
                        "SELECT * FROM conversation_inputs WHERE id=? AND conversation_id=?",
                    ))
                    .bind::<IdentitySql, _>(operation_id)
                    .bind::<IdentitySql, _>(id)
                    .get_result::<super::inputs::InputRow>(connection)
                    .await?
                    .receipt()?;
                #[derive(diesel::QueryableByName)]
                struct ExistingMessage {
                    #[diesel(sql_type=BigInt)]
                    n: i64,
                }
                let existing = diesel::sql_query(dialect.query(
                    "SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_messages WHERE source_input_id=?",
                ))
                .bind::<IdentitySql, _>(operation_id)
                .get_result::<ExistingMessage>(connection)
                .await?;
                // Pre-queue installations already committed their accepted
                // receipts. Preserve those immutable identities and ordering.
                if existing.n == 0 {
                    let receipt_now = crate::directory::database_now!(connection, dialect);
                    let _ = super::messages::append_message!(
                        connection,
                        dialect,
                        id,
                        super::messages::MessageOrigin::User {
                            input: operation_id,
                            request_id: &receipt.request_id,
                            accepted_at: receipt.accepted_at
                        },
                        receipt.text,
                        receipt_now
                    );
                }
                diesel::sql_query(
                    dialect
                        .query("DELETE FROM conversation_inputs WHERE id=? AND conversation_id=?"),
                )
                .bind::<IdentitySql, _>(operation_id)
                .bind::<IdentitySql, _>(id)
                .execute(connection)
                .await?;
                return Ok(ConversationInputReportAcknowledgement::Recorded);
            }
            diesel::sql_query(dialect.query("UPDATE conversation_input_delivery SET delivery_state=?,native_message_id=? WHERE operation_id=?"))
                .bind::<Text,_>(state).bind::<Nullable<Text>,_>(message).bind::<IdentitySql,_>(operation_id)
                .execute(connection).await?;
            diesel::sql_query(
                dialect.query("UPDATE conversation_inputs SET queue_state='ERROR' WHERE id=?"),
            )
            .bind::<IdentitySql, _>(operation_id)
            .execute(connection)
            .await?;
            Ok(ConversationInputReportAcknowledgement::Closed)
        })
    }
    /// Queue-attempt failure is separate from native outcome evidence.
    pub async fn fail_conversation_input_attempt(
        &self,
        source: &crate::ConnectorSource<'_>,
        draft: &ConversationOperationForwarding<'_>,
    ) -> Result<(), StoreError> {
        let (agent, id, operation) = ids(draft)?;
        operation!(self, |connection, dialect| {
            let row = super::records::source_conversation!(connection, dialect, source, id);
            let route = connector!(connection, dialect, source);
            current(&row, agent, draft.target)?;
            let previous = delivery!(connection, dialect, operation);
            let Some(previous) = previous else {
                return if diesel::sql_query(dialect.query(
                    "SELECT id,conversation_id,request_id,target_snapshot,accepted_at FROM conversation_inputs WHERE id=? AND conversation_id=?",
                ))
                .bind::<IdentitySql, _>(operation).bind::<IdentitySql, _>(id)
                .get_result::<super::inputs::InputReferenceRow>(connection).await.optional()?.is_none() {
                    Ok(())
                } else { Err(StoreError::InvalidTransition) };
            };
            if !previous.matches(route._execution_id, draft.target) {
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query("UPDATE conversation_inputs SET queue_state='ERROR' WHERE id=? AND conversation_id=?"))
                .bind::<IdentitySql, _>(operation).bind::<IdentitySql, _>(id).execute(connection).await?;
            Ok(())
        })
    }
    /// None is missing evidence, never proof of non-delivery or authority to resend.
    pub async fn conversation_input_delivery(
        &self,
        actor: &SecretDigest,
        agent_id: &str,
        conversation_id: &str,
        operation_id: &str,
        now: i64,
    ) -> Result<Option<ConversationInputDelivery>, StoreError> {
        let agent = EntityId::from_dto(EntityKind::Agent, agent_id)?;
        let id = EntityId::from_dto(EntityKind::Conversation, conversation_id)?;
        let operation_id = EntityId::from_dto(EntityKind::ConversationOperation, operation_id)?;
        operation!(self, |connection, dialect| {
            let _ = owned_agent!(connection, dialect, actor, agent, now);
            let row = conversation!(connection, dialect, id);
            if row.agent_id != agent {
                return Err(StoreError::NotFound);
            }
            let _ = input!(connection, dialect, id, operation_id);
            delivery!(connection, dialect, operation_id)
                .map(|row| row.delivery())
                .transpose()
        })
    }
}
