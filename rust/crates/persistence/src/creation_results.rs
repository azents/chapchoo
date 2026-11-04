//! Read-only accepted creation-result recovery across the coordinated cutover.

use crate::{
    Store, StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    models::authenticate,
    store::operation,
};
use diesel::{
    OptionalExtension,
    sql_types::{Nullable, Text},
};
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentCreationResult {
    Present {
        assigned_agent_id: String,
        team_id: String,
    },
    Removed {
        assigned_agent_id: String,
    },
}

#[derive(diesel::QueryableByName)]
struct AcceptedResultRow {
    #[diesel(sql_type=IdentitySql)]
    assigned_agent_id: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    current_team_id: Option<EntityId>,
}

impl Store {
    /// Authenticate the original owner and request key, without a mutation
    /// payload, fingerprint reinterpretation, allocation or lifecycle admission.
    pub async fn agent_creation_result(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        request_id: &str,
        now: i64,
    ) -> Result<Option<AgentCreationResult>, StoreError> {
        if !crate::computers::valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = EntityId::from_dto(EntityKind::User, &user.id)?;
            let row=diesel::sql_query(dialect.query(
                "SELECT r.assigned_agent_id,a.team_id AS current_team_id FROM agent_creation_receipts r
                 LEFT JOIN agents a ON a.id=r.assigned_agent_id AND a.owner_user_id=r.owner_user_id
                 WHERE r.owner_user_id=? AND r.request_id=?",
            )).bind::<IdentitySql,_>(owner).bind::<Text,_>(request_id)
                .get_result::<AcceptedResultRow>(connection).await.optional()?;
            Ok(row.map(|row| {
                let assigned_agent_id = row.assigned_agent_id.to_dto(EntityKind::Agent);
                match row.current_team_id {
                    Some(team) => AgentCreationResult::Present {
                        assigned_agent_id,
                        team_id: team.to_dto(EntityKind::Team),
                    },
                    None => AgentCreationResult::Removed { assigned_agent_id },
                }
            }))
        })
    }
}
