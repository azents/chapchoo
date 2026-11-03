//! Fresh domain authority for trusted peer dispatch; never an external API.
use crate::{
    ComputerRequest, ManagedResource, Store, StoreError, computers::valid_id,
    directory::database_now, models::authenticate, store::operation,
};
use chapchoo_crypto::SecretDigest;
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Bool, Nullable, Text},
};
use diesel_async::RunQueryDsl;

#[derive(Debug, diesel::QueryableByName)]
pub struct DeliveryComputer {
    #[diesel(sql_type=Text)]
    pub id: String,
    #[diesel(sql_type=Text)]
    pub owner_user_id: String,
    #[diesel(sql_type=Text)]
    pub provider_id: String,
    #[diesel(sql_type=Text)]
    pub current_request_id: String,
    #[diesel(sql_type=Text)]
    pub desired_state: String,
    #[diesel(sql_type=Text)]
    pub current_phase: String,
    #[diesel(sql_type=Text)]
    pub current_action: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub current_execution_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    pub expected_old_execution_id: Option<String>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub agent_removed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub cleanup_completed_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub initial_start_dispatch_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub prepare_admitted_at: Option<i64>,
}
#[derive(Debug, diesel::QueryableByName)]
pub struct DeliveryExecution {
    #[diesel(sql_type=Bool)]
    pub initial_allocation: bool,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub retired_at: Option<i64>,
}
#[derive(Debug)]
pub struct DeliverySnapshot {
    pub computer: DeliveryComputer,
    pub execution: Option<DeliveryExecution>,
    pub resources: Vec<ManagedResource>,
    pub substrate: chapchoo_domain::ProviderSubstrate,
}
impl Store {
    pub async fn verify_delivery_bootstrap(
        &self,
        scope: ComputerRequest<'_>,
        execution: &str,
        session: &str,
        key: &SecretDigest,
    ) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            #[derive(diesel::QueryableByName)]
            struct Found {
                #[diesel(sql_type=Text)]
                id: String,
            }
            let found=diesel::sql_query(dialect.query(
                "SELECT s.id FROM connector_sessions s JOIN computers c ON c.id=s.computer_id
                 JOIN computer_executions e ON e.id=s.execution_id
                 WHERE s.id=? AND s.computer_id=? AND s.execution_id=? AND s.initial_hash=?
                 AND s.revoked_at IS NULL AND c.current_request_id=? AND c.current_execution_id=s.execution_id
                 AND c.desired_state='RUNNING' AND c.current_phase<>'RETIRING' AND e.retired_at IS NULL"))
                .bind::<Text,_>(session).bind::<Text,_>(scope.computer_id)
                .bind::<Text,_>(execution).bind::<diesel::sql_types::Binary,_>(key.as_bytes().as_slice())
                .bind::<Text,_>(scope.request_id).get_result::<Found>(connection).await.optional()?;
            if found.is_none_or(|found| found.id != session) {
                return Err(StoreError::Forbidden);
            }
            Ok(())
        })
    }
    /// `None` is reserved for the trusted reconciler adapter, which must separately
    /// prove its exact live connection ownership. It is not anonymous access.
    pub async fn delivery_snapshot(
        &self,
        actor: Option<&SecretDigest>,
        scope: ComputerRequest<'_>,
        execution: Option<&str>,
    ) -> Result<DeliverySnapshot, StoreError> {
        if !valid_id(scope.computer_id)
            || !valid_id(scope.request_id)
            || execution.is_some_and(|id| !valid_id(id))
        {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let user = if let Some(actor) = actor {
                Some(authenticate!(connection, dialect, actor, now))
            } else {
                None
            };
            let query = match dialect {
                crate::store::Dialect::Postgres => {
                    "SELECT * FROM computers WHERE id=? FOR NO KEY UPDATE"
                }
                crate::store::Dialect::Sqlite => "SELECT * FROM computers WHERE id=?",
            };
            let computer = diesel::sql_query(dialect.query(query))
                .bind::<Text, _>(scope.computer_id)
                .get_result::<DeliveryComputer>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if let Some(user) = user {
                if computer.owner_user_id != user.id || computer.agent_removed_at.is_some() {
                    return Err(StoreError::NotFound);
                }
                #[derive(diesel::QueryableByName)]
                struct OwnedAgent {
                    #[diesel(sql_type = Text)]
                    id: String,
                }
                // Scope removal holds the Computer lock before removing access.
                // This read must not take a Team lock after the Computer lock.
                let agent = diesel::sql_query(dialect.query(
                    "SELECT a.id FROM agents a JOIN teams t ON t.id=a.team_id AND t.deletion_requested_at IS NULL JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.computer_id=? AND a.owner_user_id=?",
                ))
                    .bind::<Text, _>(scope.computer_id)
                    .bind::<Text, _>(&user.id)
                    .get_result::<OwnedAgent>(connection)
                    .await
                    .optional()?;
                if agent.is_none_or(|agent| agent.id.is_empty()) {
                    return Err(StoreError::NotFound);
                }
            }
            if computer.current_request_id != scope.request_id
                || computer.cleanup_completed_at.is_some()
            {
                return Err(StoreError::Conflict);
            }
            if execution.is_some_and(|id| {
                computer.current_execution_id.as_deref() != Some(id)
                    && computer.expected_old_execution_id.as_deref() != Some(id)
            }) {
                return Err(StoreError::Conflict);
            }
            let execution =
                if let Some(id) = execution {
                    Some(
                        diesel::sql_query(dialect.query(
                            "SELECT * FROM computer_executions WHERE id=? AND computer_id=?",
                        ))
                        .bind::<Text, _>(id)
                        .bind::<Text, _>(scope.computer_id)
                        .get_result::<DeliveryExecution>(connection)
                        .await
                        .optional()?
                        .ok_or(StoreError::NotFound)?,
                    )
                } else {
                    None
                };
            let provider =
                diesel::sql_query(dialect.query("SELECT * FROM computer_providers WHERE id=?"))
                    .bind::<Text, _>(&computer.provider_id)
                    .get_result::<crate::computers::ProviderRow>(connection)
                    .await?;
            let resources=diesel::sql_query(dialect.query("SELECT * FROM computer_resources WHERE computer_id=? ORDER BY resource_kind,native_scope,name")).bind::<Text,_>(scope.computer_id).load::<crate::native_resources::ResourceRow>(connection).await?
                .into_iter().map(ManagedResource::try_from).collect::<Result<Vec<_>,StoreError>>()?;
            crate::computer_executions::validate_native_inventory(&provider, &resources)?;
            Ok(DeliverySnapshot {
                computer,
                execution,
                resources,
                substrate: provider.substrate()?,
            })
        })
    }
}
