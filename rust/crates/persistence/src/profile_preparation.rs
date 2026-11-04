//! Atomic preparation input for the preserving identity/profile migration.
//!
//! Staging is transaction-local. Runtime Store operations never consult it.

use crate::{
    StoreError,
    entity_id::{EntityId, EntityKind},
    identity_transition::{IdentityTransition, rewrap_recovery},
    profile_initialization::{HandleInitialization, HandleResource, INITIAL_IMAGES},
};
use chapchoo_crypto::CredentialCipher;
use chapchoo_domain::Handle;
use diesel::{
    QueryableByName,
    sql_types::{Binary, Nullable, Text},
};
use std::collections::BTreeMap;

#[derive(diesel::QueryableByName)]
pub(crate) struct NamedLegacyRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct LegacyTeamRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub kind: String,
    #[diesel(sql_type = Text)]
    pub handle: String,
    #[diesel(sql_type = Nullable<diesel::sql_types::BigInt>)]
    pub deletion_requested_at: Option<i64>,
}

pub(crate) struct TeamAssignment {
    pub original: String,
    pub handle: Option<Handle>,
}

/// Reserve every already-canonical live Team handle first. Deleted Teams are
/// initialized independently because their namespace has already been released.
pub(crate) fn team_assignments(
    rows: Vec<LegacyTeamRow>,
) -> Result<Vec<TeamAssignment>, StoreError> {
    let mut live = HandleInitialization::new(rows.iter().filter_map(|row| {
        (row.kind == "COLLABORATIVE" && row.deletion_requested_at.is_none())
            .then(|| Handle::parse(&row.handle).ok())
            .flatten()
    }));
    rows.into_iter()
        .map(|row| {
            let handle = match row.kind.as_str() {
                "PERSONAL" => None,
                "COLLABORATIVE" => Some(match Handle::parse(&row.handle) {
                    Ok(handle) => handle,
                    Err(_) if row.deletion_requested_at.is_some() => {
                        HandleInitialization::new([]).assign(HandleResource::Team, &row.handle)?
                    }
                    Err(_) => live.assign(HandleResource::Team, &row.handle)?,
                }),
                _ => return Err(StoreError::Integrity),
            };
            Ok(TeamAssignment {
                original: row.id,
                handle,
            })
        })
        .collect()
}

#[derive(diesel::QueryableByName)]
pub(crate) struct EnvelopeRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Binary)]
    pub digest: Vec<u8>,
    #[diesel(sql_type = Nullable<Binary>)]
    pub nonce: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Binary>)]
    pub ciphertext: Option<Vec<u8>>,
}

pub(crate) fn envelope(
    cipher: &CredentialCipher,
    mapping: &IdentityTransition,
    row: &EnvelopeRow,
    kind: EntityKind,
    namespace: &str,
) -> Result<Option<chapchoo_crypto::RecoveryEnvelope>, StoreError> {
    match (&row.nonce, &row.ciphertext) {
        (None, None) => Ok(None),
        (Some(nonce), Some(ciphertext)) => {
            let id = mapping.required(kind, &row.id)?.to_dto(kind);
            Ok(Some(rewrap_recovery(
                cipher,
                &format!("{namespace}{}", row.id),
                &format!("{namespace}{id}"),
                &row.digest,
                nonce,
                ciphertext,
            )?))
        }
        _ => Err(StoreError::Integrity),
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct RegistrationEnvelopeRow {
    #[diesel(embed)]
    pub envelope: EnvelopeRow,
    #[diesel(sql_type = Text)]
    pub owner_kind: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub system_owner_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub user_owner_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub owner_team_id: Option<String>,
}

pub(crate) fn registration_envelope(
    cipher: &CredentialCipher,
    mapping: &IdentityTransition,
    row: &RegistrationEnvelopeRow,
) -> Result<chapchoo_crypto::RecoveryEnvelope, StoreError> {
    use crate::{ProviderRegistrationOwner as Owner, provider_registration_context};
    let (original, canonical, system) = match (
        row.owner_kind.as_str(),
        &row.system_owner_id,
        &row.user_owner_id,
        &row.owner_team_id,
    ) {
        ("SYSTEM", Some(id), None, None) => (id.clone(), id.clone(), true),
        ("USER", None, Some(id), None) => (
            id.clone(),
            mapping
                .required(EntityKind::User, id)?
                .to_dto(EntityKind::User),
            false,
        ),
        ("TEAM", None, None, Some(id)) => (
            id.clone(),
            mapping
                .required(EntityKind::Team, id)?
                .to_dto(EntityKind::Team),
            false,
        ),
        _ => return Err(StoreError::Integrity),
    };
    let owner = |id| {
        if system {
            Owner::System(id)
        } else if row.owner_kind == "USER" {
            Owner::User(id)
        } else {
            Owner::Team(id)
        }
    };
    let id = mapping
        .required(EntityKind::Provider, &row.envelope.id)?
        .to_dto(EntityKind::Provider);
    let old = provider_registration_context(&row.envelope.id, owner(&original))?;
    let new = provider_registration_context(&id, owner(&canonical))?;
    let nonce = row.envelope.nonce.as_deref().ok_or(StoreError::Integrity)?;
    let ciphertext = row
        .envelope
        .ciphertext
        .as_deref()
        .ok_or(StoreError::Integrity)?;
    rewrap_recovery(cipher, &old, &new, &row.envelope.digest, nonce, ciphertext)
}

pub(crate) fn seed_images(now: i64) -> Result<BTreeMap<&'static str, EntityId>, StoreError> {
    INITIAL_IMAGES
        .iter()
        .map(|image| Ok((image.name, EntityId::generate(now)?)))
        .collect()
}

macro_rules! stage_transition {
    ($connection:expr, $postgres:expr, $cipher:expr) => {{
        use diesel::{RunQueryDsl, connection::SimpleConnection, sql_types::{Text, Nullable, Binary}};
        use $crate::{entity_id::{EntityKind, IdentitySql}, profile_preparation::*, profile_initialization::*};
        let connection = &mut *$connection;
        let postgres = $postgres;
        let now = diesel::sql_query(if postgres {
            "SELECT floor(extract(epoch FROM clock_timestamp())*1000)::bigint AS n"
        } else { "SELECT CAST(strftime('%s','now') AS INTEGER)*1000 AS n" })
            .get_result::<$crate::preparation::Count>(connection)?.n;
        let mapping = $crate::identity_transition::inventory_identities!(connection, now);
        connection.batch_execute(if postgres {
            "CREATE TEMP TABLE _profile_id_map(kind TEXT NOT NULL,old_id TEXT NOT NULL,new_id UUID NOT NULL UNIQUE,PRIMARY KEY(kind,old_id));
             CREATE TEMP TABLE _profile_images(id UUID PRIMARY KEY,internal_name TEXT NOT NULL,asset_key TEXT NOT NULL);
             CREATE TEMP TABLE _profile_agents(old_id TEXT PRIMARY KEY,handle TEXT NOT NULL,profile_image_id UUID NOT NULL,palette_key TEXT NOT NULL);
             CREATE TEMP TABLE _profile_teams(old_id TEXT PRIMARY KEY,handle TEXT)"
        } else {
            "CREATE TEMP TABLE _profile_id_map(kind TEXT NOT NULL,old_id TEXT NOT NULL,new_id BLOB NOT NULL UNIQUE,PRIMARY KEY(kind,old_id));
             CREATE TEMP TABLE _profile_images(id BLOB PRIMARY KEY,internal_name TEXT NOT NULL,asset_key TEXT NOT NULL);
             CREATE TEMP TABLE _profile_agents(old_id TEXT PRIMARY KEY,handle TEXT NOT NULL,profile_image_id BLOB NOT NULL,palette_key TEXT NOT NULL);
             CREATE TEMP TABLE _profile_teams(old_id TEXT PRIMARY KEY,handle TEXT)"
        })?;
        for ((kind, old), id) in mapping.entries() {
            diesel::sql_query(if postgres { "INSERT INTO _profile_id_map VALUES($1,$2,$3)" } else { "INSERT INTO _profile_id_map VALUES(?,?,?)" })
                .bind::<Text,_>(kind.prefix()).bind::<Text,_>(old).bind::<IdentitySql,_>(*id).execute(connection)?;
        }
        let images = seed_images(now)?;
        for image in &INITIAL_IMAGES {
            let id = images.get(image.name).ok_or($crate::StoreError::Integrity)?;
            diesel::sql_query(if postgres { "INSERT INTO _profile_images VALUES($1,$2,$3)" } else { "INSERT INTO _profile_images VALUES(?,?,?)" })
                .bind::<IdentitySql,_>(*id).bind::<Text,_>(image.name).bind::<Text,_>(image.asset_key).execute(connection)?;
        }
        let teams = diesel::sql_query(if postgres {
            "SELECT id,kind::text AS kind,handle,deletion_requested_at FROM teams ORDER BY id"
        } else { "SELECT id,kind,handle,deletion_requested_at FROM teams ORDER BY id" })
            .load::<LegacyTeamRow>(connection)?;
        for team in team_assignments(teams)? {
            diesel::sql_query(if postgres { "INSERT INTO _profile_teams VALUES($1,$2)" } else { "INSERT INTO _profile_teams VALUES(?,?)" })
                .bind::<Text,_>(team.original).bind::<Nullable<Text>,_>(team.handle.map(|value| value.as_str().to_owned())).execute(connection)?;
        }
        let agents = diesel::sql_query("SELECT id,name FROM agents ORDER BY id").load::<NamedLegacyRow>(connection)?;
        let mut handles = HandleInitialization::new([]);
        for agent in agents {
            let handle = handles.assign(HandleResource::Agent, &agent.name)?;
            let image = images.get(initial_image(&agent.id).name).ok_or($crate::StoreError::Integrity)?;
            diesel::sql_query(if postgres { "INSERT INTO _profile_agents VALUES($1,$2,$3,$4)" } else { "INSERT INTO _profile_agents VALUES(?,?,?,?)" })
                .bind::<Text,_>(&agent.id).bind::<Text,_>(handle.as_str()).bind::<IdentitySql,_>(*image)
                .bind::<Text,_>(initial_palette(&agent.id).as_str()).execute(connection)?;
        }
        // Only ciphertext/nonce change. Digests, bearer values, validity, grace,
        // revocation and all accepted payload hashes remain byte-for-byte intact.
        for (table, id_column, kind, namespace, hash, nonce, ciphertext) in [
            ("auth_sessions", "id", EntityKind::AuthSession, "", "current_hash", "recovery_nonce", "recovery_ciphertext"),
            ("provider_keys", "provider_id", EntityKind::Provider, "computer-provider:", "current_hash", "recovery_nonce", "recovery_ciphertext"),
            ("connector_sessions", "id", EntityKind::ConnectorSession, "computer-connector-bootstrap:", "initial_hash", "initial_nonce", "initial_ciphertext"),
            ("connector_sessions", "id", EntityKind::ConnectorSession, "computer-connector-operational:", "current_hash", "recovery_nonce", "recovery_ciphertext"),
        ] {
            let query = format!("SELECT {id_column} AS id,{hash} AS digest,{nonce} AS nonce,{ciphertext} AS ciphertext FROM {table} WHERE {nonce} IS NOT NULL OR {ciphertext} IS NOT NULL");
            for row in diesel::sql_query(query).load::<EnvelopeRow>(connection)? {
                let protected = envelope($cipher, &mapping, &row, kind, namespace)?.ok_or($crate::StoreError::Integrity)?;
                let query = if postgres { format!("UPDATE {table} SET {nonce}=$1,{ciphertext}=$2 WHERE {id_column}=$3") }
                    else { format!("UPDATE {table} SET {nonce}=?,{ciphertext}=? WHERE {id_column}=?") };
                diesel::sql_query(query).bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext())
                    .bind::<Text,_>(&row.id).execute(connection)?;
            }
        }
        let rows = diesel::sql_query("SELECT p.id,p.owner_kind,p.system_owner_id,p.user_owner_id,p.owner_team_id,r.credential_hash AS digest,r.recovery_nonce AS nonce,r.recovery_ciphertext AS ciphertext FROM provider_registration_credentials r JOIN computer_providers p ON p.id=r.provider_id")
            .load::<RegistrationEnvelopeRow>(connection)?;
        for row in rows {
            let protected = registration_envelope($cipher, &mapping, &row)?;
            diesel::sql_query(if postgres { "UPDATE provider_registration_credentials SET recovery_nonce=$1,recovery_ciphertext=$2 WHERE provider_id=$3" }
                else { "UPDATE provider_registration_credentials SET recovery_nonce=?,recovery_ciphertext=? WHERE provider_id=?" })
                .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext()).bind::<Text,_>(&row.envelope.id).execute(connection)?;
        }
    }};
}
pub(crate) use stage_transition;

pub(crate) const DROP_STAGING: &str = "DROP TABLE _profile_id_map; DROP TABLE _profile_images; DROP TABLE _profile_agents; DROP TABLE _profile_teams";

#[cfg(test)]
mod tests;
