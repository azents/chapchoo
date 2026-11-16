//! Peer admission qualifies an existing input-owned native fact, never a new grant.
use super::*;

pub enum QueuedSetupContext<'a> {
    Initial {
        allow_fresh: bool,
        require_retained_source: bool,
    },
    Existing(&'a ConversationDeliveryTarget),
}
pub enum QueuedConversationCommand<'a> {
    Setup {
        draft: ConversationInputPreparation<'a>,
        supply: &'a HarnessSupply,
        context: QueuedSetupContext<'a>,
    },
    Input(ConversationOperationForwarding<'a>),
}
impl fmt::Debug for QueuedSetupContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QueuedSetupContext([REDACTED])")
    }
}
impl fmt::Debug for QueuedConversationCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("QueuedConversationCommand([REDACTED])")
    }
}
#[derive(diesel::QueryableByName)]
struct SetupFact {
    #[diesel(sql_type=Text)]
    supply_kind: String,
    #[diesel(sql_type=Nullable<Text>)]
    executable_path: Option<String>,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    fresh_native_allowed: bool,
    #[diesel(sql_type=diesel::sql_types::Bool)]
    require_retained_source: bool,
    #[diesel(sql_type=Nullable<Text>)]
    target_snapshot: Option<String>,
}
#[derive(diesel::QueryableByName)]
struct Found {
    #[diesel(sql_type=BigInt)]
    n: i64,
}
impl Store {
    /// Restricted internal Connector Setup/Input admission. Every field is bound
    /// to this issued input, current submitting session and exact live claim in
    /// this completed operation. The receiver cannot infer a successor claim.
    pub async fn queued_connector_delivery_snapshot(
        &self,
        authority: &ConversationInputAuthority,
        scope: crate::ComputerRequest<'_>,
        execution: &str,
        command: &QueuedConversationCommand<'_>,
    ) -> Result<crate::DeliverySnapshot, StoreError> {
        let (command_agent, command_conversation, command_input) = match command {
            QueuedConversationCommand::Setup { draft, .. } => {
                (draft.agent_id, draft.conversation_id, draft.operation_id)
            }
            QueuedConversationCommand::Input(draft) => {
                (draft.agent_id, draft.conversation_id, draft.operation_id)
            }
        };
        if command_agent != authority.agent_id
            || command_conversation != authority.conversation_id
            || command_input != authority.input_id
        {
            return Err(StoreError::Forbidden);
        }
        let input = EntityId::from_dto(EntityKind::ConversationOperation, &authority.input_id)?;
        let conversation =
            EntityId::from_dto(EntityKind::Conversation, &authority.conversation_id)?;
        let execution_id = EntityId::from_dto(EntityKind::Execution, execution)?;
        operation!(self, |connection, dialect| {
            let (owner, _) = waiting::queued_owned_agent!(connection, dialect, authority)
                .ok_or(StoreError::InvalidTransition)?;
            if owner.computer_id != EntityId::from_dto(EntityKind::Computer, scope.computer_id)? {
                return Err(StoreError::NotFound);
            }
            let computer = crate::delivery::locked_computer!(connection, dialect, scope);
            if computer.current_execution_id.as_deref() != Some(execution)
                || !authority::input_ready!(connection, dialect, owner.computer_id)
            {
                return Err(StoreError::InvalidTransition);
            }
            let row = conversation!(connection, dialect, conversation);
            match command {
                QueuedConversationCommand::Setup {
                    supply, context, ..
                } => {
                    // The inner joins categorically exclude startup/refresh.
                    let fact = diesel::sql_query(dialect.query("SELECT p.supply_kind,p.executable_path,p.fresh_native_allowed,p.require_retained_source,i.target_snapshot FROM conversation_input_preparation p JOIN conversation_inputs i ON i.id=p.operation_id JOIN conversations c ON c.id=i.conversation_id AND c.current_preparation_operation_id=p.operation_id JOIN project_instruction_evidence e ON e.operation_id=p.operation_id AND e.conversation_id=c.id AND e.agent_id=c.agent_id AND e.execution_id=p.execution_id WHERE p.operation_id=? AND i.conversation_id=? AND p.execution_id=? AND p.preparation_state='POSSIBLE'"))
                        .bind::<IdentitySql,_>(input).bind::<IdentitySql,_>(conversation)
                        .bind::<IdentitySql,_>(execution_id).get_result::<SetupFact>(connection)
                        .await.optional()?.ok_or(StoreError::InvalidTransition)?;
                    let selected = match (fact.supply_kind.as_str(), fact.executable_path) {
                        ("MANAGED", None) => HarnessSupply::Managed,
                        ("EXISTING", Some(path)) => HarnessSupply::Existing(
                            ExistingCodexPath::parse(path).map_err(|_| StoreError::Integrity)?,
                        ),
                        _ => return Err(StoreError::Integrity),
                    };
                    if &selected != *supply {
                        return Err(StoreError::Conflict);
                    }
                    match (fact.target_snapshot.as_deref(), context) {
                        (
                            None,
                            QueuedSetupContext::Initial {
                                allow_fresh,
                                require_retained_source,
                            },
                        ) if *allow_fresh == fact.fresh_native_allowed
                            && *require_retained_source == fact.require_retained_source
                            && row.source_snapshot.is_none() => {}
                        (submitted, QueuedSetupContext::Existing(target))
                            if (submitted == Some(target.snapshot.as_str())
                                || (submitted.is_none() && !fact.fresh_native_allowed))
                                && row.source_state == "READY"
                                && row.source_snapshot.as_deref()
                                    == Some(target.snapshot.as_str())
                                && row.native_session_id.as_deref()
                                    == Some(target.native_session.as_str()) => {}
                        _ => return Err(StoreError::Conflict),
                    }
                }
                QueuedConversationCommand::Input(draft) => {
                    delivery::current(
                        &row,
                        EntityId::from_dto(EntityKind::Agent, &authority.agent_id)?,
                        draft.target,
                    )?;
                    if row.source_state != "READY" {
                        return Err(StoreError::InvalidTransition);
                    }
                    let found = diesel::sql_query(dialect.query("SELECT CAST(count(*) AS BIGINT) AS n FROM conversation_input_delivery d JOIN conversation_inputs i ON i.id=d.operation_id WHERE d.operation_id=? AND i.conversation_id=? AND d.execution_id=? AND d.delivery_state='POSSIBLE' AND d.source_snapshot=? AND d.native_session_id=?"))
                        .bind::<IdentitySql,_>(input).bind::<IdentitySql,_>(conversation)
                        .bind::<IdentitySql,_>(execution_id).bind::<Text,_>(draft.target.snapshot.as_str())
                        .bind::<Text,_>(draft.target.native_session.as_str()).get_result::<Found>(connection).await?;
                    if found.n != 1 {
                        return Err(StoreError::InvalidTransition);
                    }
                }
            }
            // Route/source locks and reads may wait; session and claim expiry are
            // checked at the final completed admission, not its entry instant.
            let snapshot =
                crate::delivery::snapshot!(connection, dialect, computer, scope, Some(execution));
            if snapshot
                .execution
                .as_ref()
                .is_none_or(|execution| execution.retired_at.is_some())
            {
                return Err(StoreError::InvalidTransition);
            }
            input_queue::require_consumer!(connection, dialect, input, &authority.claim_id);
            Ok(snapshot)
        })
    }
}
