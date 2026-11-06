//! Agent-owned references, independent of Computer connectivity and secret material.

use super::*;
use crate::{Store, models::authenticate, store::operation};

#[derive(Clone)]
pub struct AgentAuthenticationSelection {
    pub authentication_id: String,
    pub name: String,
    pub kind: AuthenticationKind,
    pub request_id: String,
    pub selected_at: i64,
}
impl fmt::Debug for AgentAuthenticationSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AgentAuthenticationSelection([REDACTED])")
    }
}

#[derive(Debug)]
pub struct ExpectedAgentAuthentication {
    pub authentication_id: String,
    pub request_id: String,
}
pub struct SelectAgentAuthentication {
    pub request_id: String,
    pub agent_id: String,
    pub authentication_id: String,
    pub expected: Option<ExpectedAgentAuthentication>,
}
impl fmt::Debug for SelectAgentAuthentication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SelectAgentAuthentication([REDACTED])")
    }
}

#[derive(diesel::QueryableByName)]
struct AgentContext {
    #[diesel(sql_type=IdentitySql)]
    team_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    computer_id: EntityId,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    provider_id: Option<EntityId>,
    #[diesel(sql_type=Text)]
    desired_state: String,
}
#[derive(diesel::QueryableByName)]
struct SelectionRow {
    #[diesel(sql_type=IdentitySql)]
    authentication_id: EntityId,
    #[diesel(sql_type=Text)]
    current_request_id: String,
}
#[derive(diesel::QueryableByName)]
struct RowLock {
    #[diesel(sql_type=IdentitySql,column_name=id)]
    _id: EntityId,
}
const AGENT_CONTEXT: &str = "SELECT a.team_id,a.computer_id,c.provider_id,c.desired_state FROM agents a JOIN computers c ON c.id=a.computer_id WHERE a.id=? AND a.owner_user_id=?";
macro_rules! owned_agent {
    ($connection:expr,$dialect:expr,$agent:expr,$user:expr,$lock:expr) => {{
        let sql = if $lock && matches!($dialect, Dialect::Postgres) {
            format!("{AGENT_CONTEXT} FOR NO KEY UPDATE OF c")
        } else {
            AGENT_CONTEXT.to_owned()
        };
        diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql, _>($agent)
            .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::User, $user)?)
            .get_result::<AgentContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?
    }};
}

impl Store {
    pub async fn select_agent_authentication(
        &self,
        actor: &SecretDigest,
        draft: &SelectAgentAuthentication,
        now: i64,
    ) -> Result<AuthenticationAcceptedResult, StoreError> {
        request_id(&draft.request_id)?;
        let agent = EntityId::from_dto(EntityKind::Agent, &draft.agent_id)?;
        let selected = EntityId::from_dto(EntityKind::Authentication, &draft.authentication_id)?;
        let expected = draft
            .expected
            .as_ref()
            .map(|expected| {
                request_id(&expected.request_id)?;
                Ok::<_, StoreError>((
                    EntityId::from_dto(EntityKind::Authentication, &expected.authentication_id)?,
                    expected.request_id.as_str(),
                ))
            })
            .transpose()?;
        let (user, team) = operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let initial = owned_agent!(connection, dialect, agent, &user.id, false);
            let scope = AuthenticationScope::Team {
                team_id: initial.team_id.to_dto(EntityKind::Team),
            };
            admit_scope!(connection, dialect, &scope, &user.id, false);
            Ok((user.id, initial.team_id))
        })?;
        // Retain the Agent's Team as receipt authority even when the selected
        // credential is personal. Membership loss must also fence result recovery.
        let owner = Owner::team(team);
        let owner_key = owner.key()?;
        let operation = AuthenticationOperation::Select;
        let fingerprint = self.cipher.external_request_fingerprint(
            &context(&owner_key, &user, ExternalSecretKind::OpenAiApiKey)?,
            operation.stored(),
            &[
                draft.request_id.as_bytes(),
                draft.agent_id.as_bytes(),
                draft.authentication_id.as_bytes(),
                draft
                    .expected
                    .as_ref()
                    .map_or("", |value| value.authentication_id.as_str())
                    .as_bytes(),
                draft
                    .expected
                    .as_ref()
                    .map_or("", |value| value.request_id.as_str())
                    .as_bytes(),
            ],
        )?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let initial = owned_agent!(connection, dialect, agent, &user.id, false);
            if initial.team_id != team {
                return Err(StoreError::Conflict);
            }
            admit_scope!(connection, dialect, &owner.scope()?, &user.id, false);
            if let Some(receipt) = receipt!(connection, dialect, &user.id, &draft.request_id) {
                return receipt.replay(owner, operation, &fingerprint);
            }
            let previous=diesel::sql_query(dialect.query("SELECT authentication_id,current_request_id FROM agent_authentication_selections WHERE agent_id=?"))
                .bind::<IdentitySql,_>(agent).get_result::<SelectionRow>(connection).await.optional()?;
            if previous
                .as_ref()
                .map(|row| (row.authentication_id, row.current_request_id.as_str()))
                != expected
            {
                return Err(StoreError::Conflict);
            }
            let mut records = vec![selected];
            records.extend(previous.as_ref().map(|row| row.authentication_id));
            records.sort_unstable();
            records.dedup();
            for id in records {
                let sql = match dialect {
                    Dialect::Postgres => format!("{AUTH_COLUMNS} WHERE id=? FOR UPDATE"),
                    Dialect::Sqlite => format!("{AUTH_COLUMNS} WHERE id=?"),
                };
                let row = diesel::sql_query(dialect.query(&sql))
                    .bind::<IdentitySql, _>(id)
                    .get_result::<AuthenticationRow>(connection)
                    .await
                    .optional()?
                    .ok_or(StoreError::NotFound)?;
                // Record scope grants use, not authority over another member's Agent.
                let eligible = row.owner()
                    == Owner::personal(EntityId::from_dto(EntityKind::User, &user.id)?)
                    || row.owner() == owner;
                if !eligible {
                    return Err(StoreError::NotFound);
                }
            }
            if let Some(provider) = initial.provider_id {
                diesel::sql_query(dialect.query(crate::computers::provider_lock(dialect)))
                    .bind::<IdentitySql, _>(provider)
                    .get_result::<RowLock>(connection)
                    .await
                    .optional()?
                    .ok_or(StoreError::Integrity)?;
            }
            let current = owned_agent!(connection, dialect, agent, &user.id, true);
            if current.team_id != team
                || current.computer_id != initial.computer_id
                || current.provider_id != initial.provider_id
            {
                return Err(StoreError::Conflict);
            }
            if current.desired_state == "DELETING" {
                return Err(StoreError::InvalidTransition);
            }
            diesel::sql_query(dialect.query("INSERT INTO agent_authentication_selections(agent_id,authentication_id,current_request_id,selected_at) VALUES(?,?,?,?) ON CONFLICT(agent_id) DO UPDATE SET authentication_id=excluded.authentication_id,current_request_id=excluded.current_request_id,selected_at=excluded.selected_at"))
                .bind::<IdentitySql,_>(agent).bind::<IdentitySql,_>(selected).bind::<Text,_>(&draft.request_id).bind::<BigInt,_>(now).execute(connection).await?;
            save_receipt!(
                connection,
                dialect,
                &user.id,
                owner,
                &draft.request_id,
                operation,
                &fingerprint,
                Some(selected),
                Some(agent),
                None::<EntityId>,
                now
            );
            Ok(AuthenticationAcceptedResult::Selection {
                agent_id: draft.agent_id.clone(),
                authentication_id: draft.authentication_id.clone(),
            })
        })
    }
}
