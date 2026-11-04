//! Preparation-local identity mapping and credential evidence preservation.
//!
//! There is no runtime alternate-ID lookup: the map exists only in the explicit
//! preparation transaction, and disappears with it after reference conversion.

use crate::{
    StoreError,
    entity_id::{EntityId, EntityKind},
};
use chapchoo_crypto::{CredentialCipher, RecoveryEnvelope, SecretDigest};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub(crate) struct IdentityColumn {
    pub table: &'static str,
    pub column: &'static str,
    pub kind: EntityKind,
    /// Primary keys and deliberately retained historical witnesses introduce an
    /// identity. Ordinary foreign references must resolve an existing assignment.
    pub defines_identity: bool,
}

macro_rules! columns {
    ($table:literal => $(($column:literal, $kind:ident, $witness:literal)),+ $(,)?) => {
        &[$(IdentityColumn {
            table: $table,
            column: $column,
            kind: EntityKind::$kind,
            defines_identity: $witness,
        }),+]
    };
}

/// Explicit closed conversion graph. Operation correlation keys, secret hashes,
/// native UIDs and external system owner keys are deliberately not entity IDs.
pub(crate) const IDENTITY_COLUMNS: &[&[IdentityColumn]] = &[
    columns!("users" =>
        ("id", User, true)),
    columns!("teams" =>
        ("id", Team, true),
        ("owner_user_id", User, false)),
    columns!("memberships" =>
        ("team_id", Team, false),
        ("user_id", User, false)),
    columns!("personal_contexts" =>
        ("user_id", User, false),
        ("team_id", Team, false)),
    columns!("auth_sessions" =>
        ("id", AuthSession, true),
        ("user_id", User, false)),
    columns!("grants" =>
        ("id", Grant, true),
        ("user_id", User, false),
        ("issued_by", User, false)),
    columns!("invitations" =>
        ("id", Invitation, true),
        ("team_id", Team, false),
        ("invited_by", User, false)),
    columns!("login_challenges" =>
        ("id", LoginChallenge, true)),
    columns!("computer_providers" =>
        ("id", Provider, true),
        ("user_owner_id", User, false),
        ("owner_team_id", Team, false)),
    columns!("provider_keys" =>
        ("provider_id", Provider, false)),
    columns!("provider_key_requests" =>
        ("provider_id", Provider, false)),
    columns!("provider_registration_credentials" =>
        ("provider_id", Provider, false)),
    columns!("computers" =>
        ("id", Computer, true),
        ("owner_user_id", User, false),
        ("provider_id", Provider, false),
        ("current_execution_id", Execution, false),
        ("expected_old_execution_id", Execution, true),
        ("removed_agent_id", Agent, true),
        ("power_source_connection_id", Connection, true),
        ("power_source_execution_id", Execution, true),
        ("environment_source_connection_id", Connection, true)),
    columns!("agents" =>
        ("id", Agent, true),
        ("owner_user_id", User, false),
        ("team_id", Team, false),
        ("computer_id", Computer, false)),
    columns!("agent_creation_receipts" =>
        ("owner_user_id", User, false),
        ("team_id", Team, true),
        ("assigned_agent_id", Agent, true)),
    columns!("computer_executions" =>
        ("id", Execution, true),
        ("computer_id", Computer, false)),
    columns!("computer_resources" =>
        ("computer_id", Computer, false),
        ("provider_id", Provider, false),
        ("execution_id", Execution, false),
        ("source_connection_id", Connection, true)),
    columns!("connector_sessions" =>
        ("id", ConnectorSession, true),
        ("computer_id", Computer, false),
        ("execution_id", Execution, false)),
    columns!("connector_key_requests" =>
        ("session_id", ConnectorSession, false)),
    columns!("control_instances" =>
        ("id", ControlInstance, true)),
    columns!("provider_connections" =>
        ("provider_id", Provider, false),
        ("control_id", ControlInstance, false),
        ("connection_id", Connection, true)),
    columns!("connector_connections" =>
        ("computer_id", Computer, false),
        ("session_id", ConnectorSession, false),
        ("control_id", ControlInstance, false),
        ("connection_id", Connection, true)),
];

#[derive(diesel::QueryableByName)]
pub(crate) struct LegacyIdentityRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
}

/// All interpolated identifiers come from the closed table above, never an
/// operator parameter or persisted payload. Keep dialect value binding separate.
pub(crate) fn inventory_query(column: IdentityColumn) -> String {
    format!(
        "SELECT {} AS id FROM {} WHERE {} IS NOT NULL ORDER BY {}",
        column.column, column.table, column.column, column.column
    )
}

macro_rules! inventory_identities {
    ($connection:expr, $now:expr) => {{
        use diesel::RunQueryDsl;
        let mut result = $crate::identity_transition::IdentityTransition::new();
        for definitions in $crate::identity_transition::IDENTITY_COLUMNS {
            for column in *definitions {
                if column.defines_identity {
                    let values =
                        diesel::sql_query($crate::identity_transition::inventory_query(*column))
                            .load::<$crate::identity_transition::LegacyIdentityRow>(
                            &mut *$connection,
                        )?;
                    for value in values {
                        result.register(column.kind, &value.id, $now)?;
                    }
                }
            }
        }
        for definitions in $crate::identity_transition::IDENTITY_COLUMNS {
            for column in *definitions {
                if !column.defines_identity {
                    let values =
                        diesel::sql_query($crate::identity_transition::inventory_query(*column))
                            .load::<$crate::identity_transition::LegacyIdentityRow>(
                            &mut *$connection,
                        )?;
                    for value in values {
                        result.required(column.kind, &value.id)?;
                    }
                }
            }
        }
        result
    }};
}
pub(crate) use inventory_identities;

pub(crate) struct IdentityTransition {
    identities: BTreeMap<(EntityKind, String), EntityId>,
}

impl std::fmt::Debug for IdentityTransition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("IdentityTransition([REDACTED])")
    }
}

impl IdentityTransition {
    pub(crate) fn entries(&self) -> impl Iterator<Item = (&(EntityKind, String), &EntityId)> {
        self.identities.iter()
    }

    pub(crate) fn new() -> Self {
        Self {
            identities: BTreeMap::new(),
        }
    }

    /// Register canonical rows and identities surviving only in retained evidence.
    /// Duplicate observations of the same kind/key receive the same assignment.
    pub(crate) fn register(
        &mut self,
        kind: EntityKind,
        old: &str,
        now: i64,
    ) -> Result<EntityId, StoreError> {
        if old.is_empty() {
            return Err(StoreError::Integrity);
        }
        let key = (kind, old.to_owned());
        if let Some(id) = self.identities.get(&key) {
            return Ok(*id);
        }
        let id = EntityId::generate(now)?;
        self.identities.insert(key, id);
        Ok(id)
    }

    /// References must resolve to a deliberately inventoried identity. Inventing
    /// a target on lookup could silently adopt an orphaned resource.
    pub(crate) fn required(&self, kind: EntityKind, old: &str) -> Result<EntityId, StoreError> {
        self.identities
            .get(&(kind, old.to_owned()))
            .copied()
            .ok_or(StoreError::Integrity)
    }
}

/// Change only authenticated storage context, never bearer, digest or validity.
/// Callers retain all expiry/revocation/grace fields verbatim in the same
/// transaction. Decryption failure aborts preparation instead of resetting keys.
pub(crate) fn rewrap_recovery(
    cipher: &CredentialCipher,
    old_context: &str,
    new_context: &str,
    current_hash: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<RecoveryEnvelope, StoreError> {
    let digest =
        SecretDigest::from_bytes(current_hash.try_into().map_err(|_| StoreError::Integrity)?);
    let envelope = RecoveryEnvelope::from_parts(
        nonce.try_into().map_err(|_| StoreError::Integrity)?,
        ciphertext.to_vec(),
    )?;
    let token = cipher.recover_latest(old_context, &digest, &envelope)?;
    Ok(cipher.protect_latest(new_context, &token)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::URL_SAFE};
    use chapchoo_crypto::{BearerSecret, RootKey};

    fn fixture_cipher(value: u8) -> Result<CredentialCipher, StoreError> {
        Ok(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([value; 32]),
        )?)?)
    }

    #[test]
    fn mapping_is_kind_scoped_and_references_never_create_targets() -> Result<(), StoreError> {
        let mut mapping = IdentityTransition::new();
        let user = mapping.register(EntityKind::User, "same-old-key", 1000)?;
        let team = mapping.register(EntityKind::Team, "same-old-key", 1000)?;
        assert_ne!(user, team);
        assert_eq!(
            mapping.register(EntityKind::User, "same-old-key", 2000)?,
            user
        );
        assert_eq!(mapping.required(EntityKind::User, "same-old-key")?, user);
        assert!(mapping.required(EntityKind::Agent, "orphan").is_err());
        assert!(mapping.register(EntityKind::Agent, "", 1000).is_err());
        assert_eq!(format!("{mapping:?}"), "IdentityTransition([REDACTED])");
        Ok(())
    }

    #[test]
    fn retained_receipt_identity_can_exist_without_an_agent_row() -> Result<(), StoreError> {
        let mut mapping = IdentityTransition::new();
        let assigned = mapping.register(EntityKind::Agent, "removed-agent-from-receipt", 1000)?;
        assert_eq!(
            mapping.required(EntityKind::Agent, "removed-agent-from-receipt")?,
            assigned
        );
        Ok(())
    }

    #[test]
    fn rewrap_preserves_token_and_digest_but_changes_authenticated_context()
    -> Result<(), StoreError> {
        let cipher = fixture_cipher(7)?;
        let token = BearerSecret::generate()?;
        let digest = token.digest();
        let old = cipher.protect_latest("old-session-id", &token)?;
        let new_id = EntityId::generate(1000)?.to_dto(EntityKind::AuthSession);
        let rewrapped = rewrap_recovery(
            &cipher,
            "old-session-id",
            &new_id,
            digest.as_bytes(),
            old.nonce(),
            old.ciphertext(),
        )?;
        let recovered = cipher.recover_latest(&new_id, &digest, &rewrapped)?;
        assert!(digest.matches(&recovered));
        assert_eq!(recovered.expose(), token.expose());
        assert!(
            cipher
                .recover_latest("old-session-id", &digest, &rewrapped)
                .is_err()
        );
        assert!(cipher.recover_latest(&new_id, &digest, &old).is_err());
        Ok(())
    }

    #[test]
    fn wrong_root_context_digest_or_storage_fails_without_resetting() -> Result<(), StoreError> {
        let cipher = fixture_cipher(7)?;
        let other_root = fixture_cipher(8)?;
        let token = BearerSecret::generate()?;
        let old = cipher.protect_latest("old", &token)?;
        let digest = token.digest();
        for (root, context, hash, nonce, ciphertext) in [
            (
                &other_root,
                "old",
                digest.as_bytes().as_slice(),
                old.nonce().as_slice(),
                old.ciphertext(),
            ),
            (
                &cipher,
                "wrong",
                digest.as_bytes().as_slice(),
                old.nonce().as_slice(),
                old.ciphertext(),
            ),
            (
                &cipher,
                "old",
                [0_u8; 32].as_slice(),
                old.nonce().as_slice(),
                old.ciphertext(),
            ),
            (
                &cipher,
                "old",
                digest.as_bytes().as_slice(),
                [0_u8; 23].as_slice(),
                old.ciphertext(),
            ),
            (
                &cipher,
                "old",
                digest.as_bytes().as_slice(),
                old.nonce().as_slice(),
                [0_u8; 58].as_slice(),
            ),
        ] {
            assert!(rewrap_recovery(root, context, "new", hash, nonce, ciphertext).is_err());
        }
        assert!(cipher.recover_latest("old", &digest, &old).is_ok());
        Ok(())
    }

    #[test]
    fn inventories_populated_legacy_graph_including_removed_receipts()
    -> Result<(), Box<dyn std::error::Error>> {
        use diesel::{Connection, connection::SimpleConnection};

        let mut connection = diesel::SqliteConnection::establish(":memory:")?;
        connection.batch_execute("PRAGMA foreign_keys=ON; BEGIN IMMEDIATE")?;
        // Fixed pre-transition history is the fixture; future migrations must
        // not change what was previously shipped or hide a missing reference.
        for sql in [
            include_str!("../migrations/sqlite/00000000000001_accounts/up.sql"),
            include_str!("../migrations/sqlite/00000000000002_display/up.sql"),
            include_str!("../migrations/sqlite/00000000000003_computers/up.sql"),
            include_str!("../migrations/sqlite/00000000000004_prepare_admission/up.sql"),
            include_str!("../migrations/sqlite/00000000000005_agent_teams/up.sql"),
            include_str!("../migrations/sqlite/00000000000006_native_computers/up.sql"),
            include_str!("../migrations/sqlite/00000000000007_provider_authentication/up.sql"),
            include_str!("../migrations/sqlite/00000000000008_provider_request_retention/up.sql"),
        ] {
            connection.batch_execute(sql)?;
        }
        connection.batch_execute(
            "INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region)
             VALUES('user-old','fixture@example.invalid','fixture@example.invalid','fixture',0,0,'en','UTC','en-US');
             INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at)
             VALUES('team-old','PERSONAL','Personal','personal-old','user-old',0);
             INSERT INTO memberships(team_id,user_id,role)
             VALUES('team-old','user-old','MANAGER');
             INSERT INTO personal_contexts(user_id,team_id) VALUES('user-old','team-old');
             INSERT INTO agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id)
             VALUES('user-old','deleted-team-from-receipt','correlation-id',zeroblob(32),'removed-agent-from-receipt')",
        )?;
        let mapping = (|| -> Result<IdentityTransition, StoreError> {
            Ok(inventory_identities!(&mut connection, 1000))
        })()?;
        assert!(mapping.required(EntityKind::User, "user-old").is_ok());
        assert!(mapping.required(EntityKind::Team, "team-old").is_ok());
        assert!(
            mapping
                .required(EntityKind::Team, "deleted-team-from-receipt")
                .is_ok()
        );
        assert!(
            mapping
                .required(EntityKind::Agent, "removed-agent-from-receipt")
                .is_ok()
        );
        // Request correlation is not itself a product entity identity.
        assert!(
            mapping
                .required(EntityKind::Agent, "correlation-id")
                .is_err()
        );
        connection.batch_execute("COMMIT")?;
        Ok(())
    }
}
