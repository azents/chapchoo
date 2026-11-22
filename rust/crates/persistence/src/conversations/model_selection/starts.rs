use super::*;

macro_rules! require_model_handoff {
    ($connection:expr, $dialect:expr, $conversation:expr, $guarded:expr) => {
        if !$guarded {
            let governed = diesel::sql_query($dialect.query(
                "SELECT (EXISTS(SELECT 1 FROM conversation_model_scopes WHERE conversation_id=? AND saved_request_id IS NOT NULL)
                   OR EXISTS(SELECT 1 FROM conversation_model_starts WHERE conversation_id=? AND settlement IS NULL)) AS present",
            ))
            .bind::<IdentitySql, _>($conversation)
            .bind::<IdentitySql, _>($conversation)
            .get_result::<$crate::conversations::model_selection::Present>($connection)
            .await?
            .present;
            if governed {
                return Err(StoreError::InvalidTransition);
            }
        }
    };
}
pub(in crate::conversations) use require_model_handoff;

impl Store {
    /// This reservation is subordinate to an existing exact forwarding grant.
    pub async fn reserve_conversation_model_start(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationModelStart,
    ) -> Result<ConversationModelStartAdmission, StoreError> {
        let (conversation, launch) = draft.target.ids()?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, &draft.operation_id)?;
        let connection_id = EntityId::from_dto(EntityKind::Connection, source.connection_id)?;
        operation!(self, |connection, dialect| {
            let route = connector!(connection, dialect, source);
            let binding = applied_binding!(
                connection,
                dialect,
                conversation,
                route.computer_id,
                launch,
                draft.target
            )
            .ok_or(StoreError::InvalidTransition)?;
            let authorised = diesel::sql_query(dialect.query(
                "SELECT (EXISTS(SELECT 1 FROM conversation_input_delivery d JOIN conversation_inputs i ON i.id=d.operation_id
                    WHERE d.operation_id=? AND i.conversation_id=? AND d.execution_id=? AND d.source_snapshot=?
                      AND d.native_session_id=? AND d.delivery_state='POSSIBLE') OR
                  EXISTS(SELECT 1 FROM conversation_continuation_inputs q
                    JOIN conversation_ending_decisions e USING(conversation_id,execution_id,source_snapshot,occurrence_id)
                    WHERE q.id=? AND q.conversation_id=? AND q.execution_id=? AND q.source_snapshot=?
                      AND e.native_session_id=? AND e.launch_operation_id=? AND q.delivery_state='POSSIBLE')) AS present",
            ))
            .bind::<IdentitySql, _>(operation)
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(route._execution_id)
            .bind::<Text, _>(draft.target.snapshot.as_str())
            .bind::<Text, _>(draft.target.native_session.as_str())
            .bind::<IdentitySql, _>(operation)
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(route._execution_id)
            .bind::<Text, _>(draft.target.snapshot.as_str())
            .bind::<Text, _>(draft.target.native_session.as_str())
            .bind::<IdentitySql, _>(launch)
            .get_result::<Present>(connection)
            .await?
            .present;
            if !authorised || pending!(connection, dialect, conversation) {
                return Err(StoreError::InvalidTransition);
            }
            let scope = scope!(connection, dialect, conversation);
            let current = scope.as_ref().filter(|row| {
                row.matches(&draft.target, launch)
                    && row.authentication_id == binding.authentication_id
            });
            if !draft.capable || current.is_none() {
                if scope
                    .as_ref()
                    .is_some_and(|row| row.saved_request_id.is_some())
                {
                    return Err(StoreError::InvalidTransition);
                }
                return Ok(ConversationModelStartAdmission::Ungoverned);
            }
            let retained_tuning = current.and_then(|row| row.saved_tuning.as_deref());
            let tuning = tuning_from_storage(retained_tuning)?;
            if tuning.permission_mode.is_some() && !draft.permission_mode_capable {
                return Err(StoreError::InvalidTransition);
            }
            let model_id = current.and_then(|row| row.saved_model_id.clone());
            let inserted = diesel::sql_query(dialect.query(
                "INSERT INTO conversation_model_starts
                 (conversation_id,operation_id,source_snapshot,native_session_id,execution_id,launch_operation_id,connection_id,model_id,tuning)
                 VALUES(?,?,?,?,?,?,?,?,?) ON CONFLICT (conversation_id,operation_id) DO NOTHING",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(operation)
            .bind::<Text, _>(draft.target.snapshot.as_str())
            .bind::<Text, _>(draft.target.native_session.as_str())
            .bind::<IdentitySql, _>(route._execution_id)
            .bind::<IdentitySql, _>(launch)
            .bind::<IdentitySql, _>(connection_id)
            .bind::<Nullable<Text>, _>(&model_id)
            .bind::<Nullable<Text>, _>(retained_tuning)
            .execute(connection)
            .await?;
            if inserted != 1 {
                return Err(StoreError::Conflict);
            }
            Ok(ConversationModelStartAdmission::Reserved { model_id, tuning })
        })
    }

    /// A current route can carry the same physical owner's retained outcome,
    /// but it does not grant another publication.
    pub async fn settle_conversation_model_start(
        &self,
        source: &ConnectorSource<'_>,
        draft: &ConversationModelStart,
        outcome: &ConversationModelStartOutcome,
    ) -> Result<(), StoreError> {
        let (conversation, launch) = draft.target.ids()?;
        let operation = EntityId::from_dto(EntityKind::ConversationOperation, &draft.operation_id)?;
        let connection_id = EntityId::from_dto(EntityKind::Connection, source.connection_id)?;
        let (settled, message) = match outcome {
            ConversationModelStartOutcome::Inserted(message) => {
                ("INSERTED", Some(message.as_str()))
            }
            ConversationModelStartOutcome::NotPublished => ("NOT_PUBLISHED", None),
            ConversationModelStartOutcome::ModelUnavailableBeforeStart(model) => {
                if !valid_model_id(model) {
                    return Err(StoreError::InvalidInput);
                }
                ("MODEL_UNAVAILABLE", None)
            }
        };
        operation!(self, |connection, dialect| {
            let route = connector!(connection, dialect, source);
            let _ = applied_binding!(
                connection,
                dialect,
                conversation,
                route.computer_id,
                launch,
                draft.target
            )
            .ok_or(StoreError::InvalidTransition)?;
            let row = diesel::sql_query(dialect.query(
                "SELECT * FROM conversation_model_starts WHERE conversation_id=? AND operation_id=?",
            ))
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(operation)
            .get_result::<StartRow>(connection)
            .await
            .optional()?;
            let Some(row) = row else {
                if settled != "NOT_PUBLISHED" {
                    return Err(StoreError::NotFound);
                }
                // Seal exact non-publication before a delayed Reserve can
                // commit after the original caller has lost its response.
                diesel::sql_query(dialect.query(
                    "INSERT INTO conversation_model_starts
                     (conversation_id,operation_id,source_snapshot,native_session_id,execution_id,launch_operation_id,connection_id,settlement)
                     VALUES(?,?,?,?,?,?,?,'NOT_PUBLISHED')",
                ))
                .bind::<IdentitySql,_>(conversation).bind::<IdentitySql,_>(operation)
                .bind::<Text,_>(draft.target.snapshot.as_str()).bind::<Text,_>(draft.target.native_session.as_str())
                .bind::<IdentitySql,_>(route._execution_id).bind::<IdentitySql,_>(launch)
                .bind::<IdentitySql,_>(connection_id)
                .execute(connection).await?;
                return Ok(());
            };
            if row.execution_id != route._execution_id
                || row.launch_operation_id != launch
                || row.source_snapshot != draft.target.snapshot.as_str()
                || row.native_session_id != draft.target.native_session.as_str()
            {
                return Err(StoreError::Conflict);
            }
            if let ConversationModelStartOutcome::ModelUnavailableBeforeStart(model) = outcome
                && row.model_id.as_deref() != Some(model)
            {
                return Err(StoreError::Conflict);
            }
            if let Some(existing) = row.settlement {
                if existing == settled && row.native_message_id.as_deref() == message {
                    return Ok(());
                }
                return Err(StoreError::Conflict);
            }
            diesel::sql_query(dialect.query(
                "UPDATE conversation_model_starts SET settlement=?,native_message_id=?
                 WHERE conversation_id=? AND operation_id=? AND settlement IS NULL",
            ))
            .bind::<Text, _>(settled)
            .bind::<Nullable<Text>, _>(message)
            .bind::<IdentitySql, _>(conversation)
            .bind::<IdentitySql, _>(operation)
            .execute(connection)
            .await?;
            if matches!(settled, "MODEL_UNAVAILABLE" | "INSERTED") {
                diesel::sql_query(dialect.query(
                    "UPDATE conversation_model_scopes SET recovery_operation_id=?
                     WHERE conversation_id=? AND launch_operation_id=? AND saved_model_id=?",
                ))
                .bind::<Nullable<IdentitySql>, _>(
                    (settled == "MODEL_UNAVAILABLE").then_some(operation),
                )
                .bind::<IdentitySql, _>(conversation)
                .bind::<IdentitySql, _>(launch)
                .bind::<Nullable<Text>, _>(&row.model_id)
                .execute(connection)
                .await?;
            }
            Ok(())
        })
    }
}
