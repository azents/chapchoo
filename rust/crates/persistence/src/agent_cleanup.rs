//! Shared terminal effects for private Agent and context-driven deletion.

use crate::{Store, StoreError, store::operation};
use chapchoo_crypto::payload_fingerprint;
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel_async::RunQueryDsl;

#[derive(diesel::QueryableByName)]
pub(crate) struct ScopedAgent {
    #[diesel(sql_type = Text)]
    pub agent_id: String,
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub provider_id: Option<String>,
    #[diesel(sql_type = Text)]
    pub current_request_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub current_execution_id: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub prepare_admitted_at: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub initial_start_dispatch_at: Option<i64>,
    #[diesel(sql_type = Text)]
    pub current_phase: String,
    #[diesel(sql_type = Text)]
    pub desired_state: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub provider_kind: Option<String>,
}

pub(crate) const SCOPED_AGENTS: &str = "SELECT a.id AS agent_id,c.id,c.kind,c.provider_id,c.current_request_id,c.current_execution_id,c.prepare_admitted_at,c.initial_start_dispatch_at,c.current_phase,c.desired_state,p.provider_kind FROM agents a JOIN computers c ON c.id=a.computer_id LEFT JOIN computer_providers p ON p.id=c.provider_id";

pub(crate) fn deletion_phase(
    provider: Option<&str>,
    phase: &str,
    prepare_admitted: bool,
) -> &'static str {
    if provider == Some("DOCKER")
        && (phase == "UNCERTAIN" || (phase == "STARTING" && prepare_admitted))
    {
        "UNCERTAIN"
    } else {
        "DELETING"
    }
}

pub(crate) fn deletion_identity(team: &str, agent: &str, previous: &str) -> (String, [u8; 32]) {
    let identity = payload_fingerprint(
        "team-agent-delete",
        &[team.as_bytes(), agent.as_bytes(), previous.as_bytes()],
    );
    let request = identity.iter().map(|byte| format!("{byte:02x}")).collect();
    let payload = payload_fingerprint(
        "computer-lifecycle",
        &[agent.as_bytes(), b"DELETE", previous.as_bytes()],
    );
    (request, payload)
}

macro_rules! revoke_deleted_computer {
    ($connection:expr, $dialect:expr, $computer:expr, $execution:expr, $never_prepared:expr, $now:expr) => {{
        if $never_prepared {
            // Only the same locked, never-admitted preparation can establish
            // absence without an external observation. Never erase allocated UIDs.
            diesel::sql_query($dialect.query("UPDATE computer_resources SET presence='ABSENT',observed_at=?,source_connection_id=NULL,absence_confirmed_at=COALESCE(absence_confirmed_at,?) WHERE computer_id=? AND resource_uid IS NULL AND docker_container_id IS NULL AND docker_volume_metadata IS NULL AND (resource_kind IN ('PVC','DOCKER_VOLUME') OR execution_id=?)"))
                .bind::<diesel::sql_types::BigInt,_>($now)
                .bind::<diesel::sql_types::BigInt,_>($now)
                .bind::<diesel::sql_types::Text,_>($computer)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>,_>($execution)
                .execute($connection).await?;
        }
        diesel::sql_query($dialect.query("UPDATE connector_sessions SET revoked_at=COALESCE(revoked_at,?),initial_nonce=NULL,initial_ciphertext=NULL,current_hash=NULL,previous_hash=NULL,previous_valid_until=NULL,issued_at=NULL,expires_at=NULL,current_request_id=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE execution_id IN (SELECT id FROM computer_executions WHERE computer_id=?)"))
            .bind::<diesel::sql_types::BigInt,_>($now)
            .bind::<diesel::sql_types::Text,_>($computer)
            .execute($connection).await?;
        diesel::sql_query($dialect.query("DELETE FROM connector_connections WHERE computer_id=?"))
            .bind::<diesel::sql_types::Text,_>($computer)
            .execute($connection).await?;
    }};
}
pub(crate) use revoke_deleted_computer;

macro_rules! delete_team_agents {
    ($connection:expr, $dialect:expr, $team:expr, $owner:expr, $now:expr) => {{
        let owner: Option<&str> = $owner;
        let team: &str = $team;
        let query = format!("{} WHERE a.team_id=? AND (a.owner_user_id=? OR ?=TRUE) ORDER BY c.provider_id,c.id", $crate::agent_cleanup::SCOPED_AGENTS);
        let initial = diesel::sql_query($dialect.query(&query))
            .bind::<diesel::sql_types::Text,_>(team)
            .bind::<diesel::sql_types::Text,_>(owner.unwrap_or(""))
            .bind::<diesel::sql_types::Bool,_>(owner.is_none())
            .load::<$crate::agent_cleanup::ScopedAgent>($connection).await?;
        // Keep the same Team -> Provider -> Computer ordering as direct Agent
        // operations; reconciliation can never observe a half-admitted deletion.
        let mut providers: std::collections::BTreeSet<String> = initial.iter()
            .filter_map(|row| row.provider_id.clone()).collect();
        let owned_providers = if owner.is_none() {
            diesel::sql_query($dialect.query("SELECT id,owner_kind,system_owner_id,user_owner_id,owner_team_id,name,provider_kind,docker_daemon_id,can_create,can_start,can_stop,can_restart FROM computer_providers WHERE owner_kind='TEAM' AND owner_team_id=? ORDER BY id"))
                .bind::<diesel::sql_types::Text,_>(team)
                .load::<$crate::computers::ProviderRow>($connection).await?
        } else {
            Vec::new()
        };
        providers.extend(owned_providers.iter().map(|provider| provider.id.clone()));
        for provider in &providers {
            diesel::sql_query($dialect.query($crate::computers::provider_lock($dialect)))
                .bind::<diesel::sql_types::Text,_>(provider)
                .get_result::<$crate::computers::ProviderRow>($connection).await?;
        }
        // Team deletion withdraws its credentials without transferring native
        // resource ownership or pretending external cleanup has completed.
        for provider in owned_providers {
            diesel::sql_query($dialect.query("UPDATE provider_registration_credentials SET revoked_at=COALESCE(revoked_at,?) WHERE provider_id=?"))
                .bind::<diesel::sql_types::BigInt,_>($now)
                .bind::<diesel::sql_types::Text,_>(&provider.id)
                .execute($connection).await?;
            diesel::sql_query($dialect.query("UPDATE provider_keys SET revoked_at=COALESCE(revoked_at,?),previous_hash=NULL,previous_valid_until=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE provider_id=?"))
                .bind::<diesel::sql_types::BigInt,_>($now)
                .bind::<diesel::sql_types::Text,_>(&provider.id)
                .execute($connection).await?;
            diesel::sql_query($dialect.query("DELETE FROM provider_connections WHERE provider_id=?"))
                .bind::<diesel::sql_types::Text,_>(&provider.id)
                .execute($connection).await?;
        }
        for initial in initial {
            let mut query = format!("{} WHERE a.id=? AND a.team_id=?", $crate::agent_cleanup::SCOPED_AGENTS);
            if matches!($dialect, $crate::store::Dialect::Postgres) {
                query.push_str(" FOR NO KEY UPDATE OF c");
            }
            let current = diesel::OptionalExtension::optional(
                diesel::sql_query($dialect.query(&query))
                    .bind::<diesel::sql_types::Text,_>(&initial.agent_id)
                    .bind::<diesel::sql_types::Text,_>(team)
                    .get_result::<$crate::agent_cleanup::ScopedAgent>($connection).await
            )?;
            let Some(current) = current else { continue; };
            if current.desired_state == "DELETING" { continue; }
            if current.kind != "CLOUD" { return Err($crate::StoreError::InvalidTransition); }
            let (request, payload) = $crate::agent_cleanup::deletion_identity(team, &current.agent_id, &current.current_request_id);
            let phase = $crate::agent_cleanup::deletion_phase(current.provider_kind.as_deref(), &current.current_phase, current.prepare_admitted_at.is_some());
            let changed = diesel::sql_query($dialect.query("UPDATE computers SET desired_state='DELETING',current_request_id=?,current_payload_hash=?,current_action='DELETE',current_phase=?,initial_start_dispatch_at=NULL,prepare_admitted_at=NULL,expected_old_execution_id=current_execution_id WHERE id=? AND current_request_id=?"))
                .bind::<diesel::sql_types::Text,_>(&request)
                .bind::<diesel::sql_types::Binary,_>(payload.as_slice())
                .bind::<diesel::sql_types::Text,_>(phase)
                .bind::<diesel::sql_types::Text,_>(&current.id)
                .bind::<diesel::sql_types::Text,_>(&current.current_request_id)
                .execute($connection).await?;
            if changed != 1 { return Err($crate::StoreError::Conflict); }
            let never_prepared = current.current_execution_id.is_some()
                && current.prepare_admitted_at.is_none()
                && (current.initial_start_dispatch_at.is_none() || current.current_phase == "BLOCKED");
            $crate::agent_cleanup::revoke_deleted_computer!(
                $connection, $dialect, &current.id, current.current_execution_id.as_deref(), never_prepared, $now
            );
        }
    }};
}
pub(crate) use delete_team_agents;

macro_rules! purge_deleted_teams {
    ($connection:expr, $dialect:expr) => {{
        // Retain exact Team/Provider identities while native resources or
        // Computers reference them. Empty Providers have no cleanup authority.
        diesel::sql_query($dialect.query("DELETE FROM computer_providers WHERE owner_kind='TEAM' AND owner_team_id IN (SELECT id FROM teams WHERE deletion_requested_at IS NOT NULL) AND NOT EXISTS (SELECT 1 FROM computers WHERE computers.provider_id=computer_providers.id) AND NOT EXISTS (SELECT 1 FROM computer_resources WHERE computer_resources.provider_id=computer_providers.id)"))
            .execute($connection).await?;
        diesel::sql_query($dialect.query("DELETE FROM teams WHERE deletion_requested_at IS NOT NULL AND NOT EXISTS (SELECT 1 FROM agents WHERE agents.team_id=teams.id) AND NOT EXISTS (SELECT 1 FROM computer_providers WHERE computer_providers.owner_team_id=teams.id)"))
            .execute($connection).await?;
    }};
}

impl Store {
    pub(crate) async fn collect_deleted_teams(&self) -> Result<(), StoreError> {
        // Run only after the caller's Provider/Computer transaction commits.
        // Taking Team locks while holding those locks would invert admission.
        operation!(self, |connection, dialect| {
            purge_deleted_teams!(connection, dialect);
            Ok(())
        })
    }
}
