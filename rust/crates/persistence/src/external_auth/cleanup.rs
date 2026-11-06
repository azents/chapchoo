//! Authentication cleanup at the already-locked Team authority boundary.

#[derive(diesel::QueryableByName)]
pub(crate) struct AuthenticationLock {
    #[diesel(sql_type=crate::entity_id::IdentitySql, column_name=id)]
    _id: crate::entity_id::EntityId,
}

macro_rules! detach_team_authentications {
    ($connection:expr,$dialect:expr,$team:expr,$owner:expr,$now:expr) => {{
        use $crate::entity_id::{EntityId,IdentitySql};
        use diesel::sql_types::{BigInt,Bool,Nullable};
        let team: EntityId = $team;
        let departing: Option<EntityId> = $owner;
        // Lock all Team-owned records plus personal records selected by affected
        // Agents before native Provider/Computer admission. No second User lock.
        let sql = "SELECT ea.id FROM external_authentications ea WHERE ea.owner_team_id=? OR ea.id IN (SELECT s.authentication_id FROM agent_authentication_selections s JOIN agents a ON a.id=s.agent_id WHERE a.team_id=? AND (a.owner_user_id=? OR ?=TRUE)) ORDER BY ea.id";
        let sql = match $dialect {
            $crate::store::Dialect::Postgres => format!("{sql} FOR UPDATE"),
            $crate::store::Dialect::Sqlite => sql.to_owned(),
        };
        diesel::sql_query($dialect.query(&sql)).bind::<IdentitySql,_>(team).bind::<IdentitySql,_>(team)
            .bind::<Nullable<IdentitySql>,_>(departing).bind::<Bool,_>(departing.is_none())
            .load::<$crate::external_auth::AuthenticationLock>($connection).await?;
        diesel::sql_query($dialect.query("DELETE FROM agent_authentication_selections WHERE agent_id IN (SELECT id FROM agents WHERE team_id=? AND (owner_user_id=? OR ?=TRUE))"))
            .bind::<IdentitySql,_>(team).bind::<Nullable<IdentitySql>,_>(departing).bind::<Bool,_>(departing.is_none()).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE external_authentications SET active_attempt_id=NULL WHERE active_attempt_id IN (SELECT id FROM external_auth_attempts WHERE owner_team_id=? AND (initiated_by=? OR ?=TRUE))"))
            .bind::<IdentitySql,_>(team).bind::<Nullable<IdentitySql>,_>(departing).bind::<Bool,_>(departing.is_none()).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE external_auth_attempts SET phase='CANCELLED',step_id=NULL,pending_nonce=NULL,pending_ciphertext=NULL,updated_at=? WHERE owner_team_id=? AND (initiated_by=? OR ?=TRUE) AND phase IN ('STARTING','PENDING','CHECKING','EXCHANGING','UNCERTAIN')"))
            .bind::<BigInt,_>($now).bind::<IdentitySql,_>(team).bind::<Nullable<IdentitySql>,_>(departing).bind::<Bool,_>(departing.is_none()).execute($connection).await?;
        if departing.is_none() {
            // Remove both ends of the deferrable active-attempt relationship
            // before deleting secret-bearing catalogue and attempt rows.
            diesel::sql_query($dialect.query("DELETE FROM external_auth_requests WHERE owner_team_id=?"))
                .bind::<IdentitySql,_>(team).execute($connection).await?;
            diesel::sql_query($dialect.query("DELETE FROM external_auth_attempts WHERE owner_team_id=?"))
                .bind::<IdentitySql,_>(team).execute($connection).await?;
            diesel::sql_query($dialect.query("DELETE FROM external_authentications WHERE owner_team_id=?"))
                .bind::<IdentitySql,_>(team).execute($connection).await?;
        }
    }};
}
pub(crate) use detach_team_authentications;
