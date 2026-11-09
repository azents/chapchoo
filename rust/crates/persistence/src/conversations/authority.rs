//! Admission is revalidated inside the same completed operation as conversation changes.

use super::*;

#[derive(diesel::QueryableByName)]
pub(crate) struct AgentContext {
    #[diesel(sql_type=IdentitySql)]
    pub(crate) team_id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    pub(crate) computer_id: EntityId,
    #[diesel(sql_type=Text)]
    pub(crate) desired_state: String,
}
#[derive(diesel::QueryableByName)]
pub(crate) struct LockedTeam {
    #[diesel(sql_type=IdentitySql,column_name=id)]
    pub(crate) _id: EntityId,
}
pub(crate) const AGENT_QUERY: &str = "SELECT a.team_id,a.computer_id,c.desired_state FROM agents a JOIN computers c ON c.id=a.computer_id JOIN memberships m ON m.team_id=a.team_id AND m.user_id=a.owner_user_id WHERE a.id=? AND a.owner_user_id=?";
macro_rules! owned_agent {
    ($connection:expr,$dialect:expr,$actor:expr,$agent:expr,$now:expr) => {{
        use $crate::conversations::authority::{AgentContext,LockedTeam,AGENT_QUERY};
        let user = $crate::models::authenticate!($connection, $dialect, $actor, $now);
        let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
        let initial = diesel::sql_query($dialect.query(AGENT_QUERY))
            .bind::<IdentitySql,_>($agent).bind::<IdentitySql,_>(user_id)
            .get_result::<AgentContext>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
        let team_sql = match $dialect {
            Dialect::Postgres => "SELECT t.id FROM teams t WHERE t.id=? AND EXISTS (SELECT 1 FROM memberships m WHERE m.team_id=t.id AND m.user_id=?) FOR NO KEY UPDATE OF t",
            Dialect::Sqlite => "SELECT t.id FROM teams t WHERE t.id=? AND EXISTS (SELECT 1 FROM memberships m WHERE m.team_id=t.id AND m.user_id=?)",
        };
        diesel::sql_query($dialect.query(team_sql)).bind::<IdentitySql,_>(initial.team_id)
            .bind::<IdentitySql,_>(user_id).get_result::<LockedTeam>($connection)
            .await.optional()?.ok_or(StoreError::NotFound)?;
        let sql = match $dialect {
            Dialect::Postgres => format!("{AGENT_QUERY} FOR NO KEY UPDATE OF c"),
            Dialect::Sqlite => AGENT_QUERY.to_owned(),
        };
        let current = diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql,_>($agent).bind::<IdentitySql,_>(user_id)
            .get_result::<AgentContext>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
        if current.team_id != initial.team_id || current.computer_id != initial.computer_id {
            return Err(StoreError::Conflict);
        }
        current
    }};
}
pub(super) use owned_agent;

/// The single locked query binds current key, session, route, execution and Computer.
#[derive(diesel::QueryableByName)]
pub(crate) struct ConnectorContext {
    #[diesel(sql_type=IdentitySql)]
    pub(crate) computer_id: EntityId,
    #[diesel(sql_type=IdentitySql,column_name=execution_id)]
    pub(crate) _execution_id: EntityId,
    #[diesel(sql_type=Nullable<Binary>)]
    pub(crate) current_hash: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    pub(crate) previous_hash: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub(crate) previous_valid_until: Option<i64>,
    #[diesel(sql_type=Binary)]
    pub(crate) route_hash: Vec<u8>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub(crate) issued_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub(crate) expires_at: Option<i64>,
    #[diesel(sql_type=BigInt)]
    pub(crate) route_expires_at: i64,
    #[diesel(sql_type=BigInt)]
    pub(crate) control_expires_at: i64,
}
pub(crate) const CONNECTOR_QUERY: &str = "SELECT c.id AS computer_id,e.id AS execution_id,s.current_hash,s.previous_hash,s.previous_valid_until,r.credential_hash AS route_hash,s.issued_at,s.expires_at,r.expires_at AS route_expires_at,ct.expires_at AS control_expires_at FROM connector_sessions s JOIN computer_executions e ON e.id=s.execution_id JOIN computers c ON c.id=e.computer_id AND c.current_execution_id=e.id JOIN connector_connections r ON r.computer_id=c.id AND r.session_id=s.id JOIN control_instances ct ON ct.id=r.control_id WHERE s.id=? AND r.control_id=? AND r.connection_id=? AND s.revoked_at IS NULL AND e.retired_at IS NULL AND c.desired_state<>'DELETING' AND ct.draining=FALSE";
macro_rules! connector {
    ($connection:expr,$dialect:expr,$source:expr) => {{
        use $crate::conversations::authority::{CONNECTOR_QUERY, ConnectorContext};
        let source = $source;
        let sql = match $dialect {
            Dialect::Postgres => format!("{CONNECTOR_QUERY} FOR NO KEY UPDATE OF c"),
            Dialect::Sqlite => CONNECTOR_QUERY.to_owned(),
        };
        let _locked = diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ConnectorSession,
                source.session_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ControlInstance,
                source.control_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Connection,
                source.connection_id,
            )?)
            .get_result::<ConnectorContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::Unauthenticated)?;
        // A lock wait can outlive a session/route mutation under READ COMMITTED.
        let route_sql = match $dialect {
            Dialect::Postgres => format!("{CONNECTOR_QUERY} FOR NO KEY UPDATE OF r"),
            Dialect::Sqlite => CONNECTOR_QUERY.to_owned(),
        };
        let _route = diesel::sql_query($dialect.query(&route_sql))
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ConnectorSession,
                source.session_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ControlInstance,
                source.control_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Connection,
                source.connection_id,
            )?)
            .get_result::<ConnectorContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::Unauthenticated)?;
        // Keep Control withdrawal ordered after this completed source operation,
        // rather than merely observing its draining flag before the final write.
        let _ = $crate::directory::live_control!($connection, $dialect, source.control_id);
        let row = diesel::sql_query($dialect.query(CONNECTOR_QUERY))
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ConnectorSession,
                source.session_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::ControlInstance,
                source.control_id,
            )?)
            .bind::<IdentitySql, _>(EntityId::from_dto(
                EntityKind::Connection,
                source.connection_id,
            )?)
            .get_result::<ConnectorContext>($connection)
            .await
            .optional()?
            .ok_or(StoreError::Unauthenticated)?;
        let now = $crate::directory::database_now!($connection, $dialect);
        let matches =
            |hash: Option<&[u8]>| $crate::component_keys::key_matches(hash, source.credential);
        let valid_key = matches(row.current_hash.as_deref())?
            || (row.previous_valid_until.is_some_and(|expiry| expiry > now)
                && matches(row.previous_hash.as_deref())?);
        if !valid_key
            || !matches(Some(&row.route_hash))?
            || !row.issued_at.is_some_and(|issued| issued <= now)
            || !row.expires_at.is_some_and(|expiry| expiry > now)
            || row.route_expires_at <= now
            || row.control_expires_at <= now
        {
            return Err(StoreError::Unauthenticated);
        }
        row
    }};
}
pub(crate) use connector;
