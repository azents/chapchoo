//! DB directory authority, independent of socket lifetime and dispatch queues.
use crate::{
    Store, StoreError,
    component_keys::{authenticate_provider, key_matches},
    computers::valid_id,
    store::{Dialect, operation},
};
use chapchoo_crypto::SecretDigest;
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Bool, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct DirectoryPolicy {
    pub control_validity_ms: i64,
    pub component_validity_ms: i64,
}
impl Default for DirectoryPolicy {
    fn default() -> Self {
        Self {
            control_validity_ms: 60_000,
            component_validity_ms: 60_000,
        }
    }
}
impl DirectoryPolicy {
    pub(crate) fn expiry(self, now: i64, component: bool) -> Result<i64, StoreError> {
        if self.control_validity_ms <= 0 || self.component_validity_ms <= 0 {
            return Err(StoreError::InvalidInput);
        }
        now.checked_add(if component {
            self.component_validity_ms
        } else {
            self.control_validity_ms
        })
        .ok_or(StoreError::InvalidInput)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionOwner {
    pub control_id: String,
    pub connection_id: String,
}
#[derive(Clone, diesel::QueryableByName)]
pub struct ControlRegistration {
    #[diesel(sql_type=Text)]
    pub id: String,
    #[diesel(sql_type=Text)]
    pub endpoint: String,
    #[diesel(sql_type=Bool)]
    pub draining: bool,
    #[diesel(sql_type=BigInt)]
    pub expires_at: i64,
}
impl fmt::Debug for ControlRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ControlRegistration([REDACTED])")
    }
}
#[derive(Clone, diesel::QueryableByName)]
pub struct ComponentRoute {
    #[diesel(sql_type=Text)]
    pub control_id: String,
    #[diesel(sql_type=Text)]
    pub connection_id: String,
    #[diesel(sql_type=Binary)]
    pub(crate) credential_hash: Vec<u8>,
    #[diesel(sql_type=BigInt)]
    pub expires_at: i64,
}
impl ComponentRoute {
    pub fn owner(&self) -> ConnectionOwner {
        ConnectionOwner {
            control_id: self.control_id.clone(),
            connection_id: self.connection_id.clone(),
        }
    }
}
impl fmt::Debug for ComponentRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComponentRoute([REDACTED])")
    }
}
#[derive(Debug)]
pub struct ResolvedRoute {
    pub component: ComponentRoute,
    pub control: ControlRegistration,
}
#[derive(diesel::QueryableByName)]
pub(crate) struct DatabaseClock {
    #[diesel(sql_type=BigInt)]
    pub now: i64,
}

macro_rules! database_now {
    ($connection:expr,$dialect:expr) => {{
        let query = match $dialect {
            $crate::store::Dialect::Postgres => {
                "SELECT floor(extract(epoch FROM clock_timestamp())*1000)::bigint AS now"
            }
            $crate::store::Dialect::Sqlite => {
                "SELECT CAST((julianday('now')-2440587.5)*86400000 AS BIGINT) AS now"
            }
        };
        diesel::sql_query(query)
            .get_result::<$crate::directory::DatabaseClock>($connection)
            .await?
            .now
    }};
}
pub(crate) use database_now;
pub(crate) fn control_query(dialect: Dialect, exclusive: bool) -> &'static str {
    match (dialect, exclusive) {
        (Dialect::Postgres, true) => "SELECT * FROM control_instances WHERE id=? FOR UPDATE",
        (Dialect::Postgres, false) => "SELECT * FROM control_instances WHERE id=? FOR SHARE",
        (Dialect::Sqlite, _) => "SELECT * FROM control_instances WHERE id=?",
    }
}
fn provider_route_query(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Postgres => {
            "SELECT * FROM provider_connections WHERE provider_id=? FOR NO KEY UPDATE"
        }
        Dialect::Sqlite => "SELECT * FROM provider_connections WHERE provider_id=?",
    }
}
pub(crate) fn matches_owner(
    route: Option<&ComponentRoute>,
    expected: Option<&ConnectionOwner>,
) -> bool {
    match (route, expected) {
        (None, None) => true,
        (Some(route), Some(expected)) => {
            route.control_id == expected.control_id && route.connection_id == expected.connection_id
        }
        _ => false,
    }
}
pub(crate) fn validate_owner(owner: &ConnectionOwner) -> Result<(), StoreError> {
    if valid_id(&owner.control_id) && valid_id(&owner.connection_id) {
        Ok(())
    } else {
        Err(StoreError::InvalidInput)
    }
}
macro_rules! live_control {
    ($connection:expr,$dialect:expr,$id:expr) => {{
        let row =
            diesel::sql_query($dialect.query($crate::directory::control_query($dialect, false)))
                .bind::<diesel::sql_types::Text, _>($id)
                .get_result::<$crate::directory::ControlRegistration>($connection)
                .await
                .optional()?
                .ok_or(StoreError::InvalidTransition)?;
        if row.draining || row.expires_at <= $crate::directory::database_now!($connection, $dialect)
        {
            return Err(StoreError::InvalidTransition);
        }
        row
    }};
}

pub(crate) use live_control;

impl Store {
    /// Insert a fresh process identity only after its listener is ready. Endpoint
    /// reuse cannot update or revive an earlier process registration.
    pub async fn register_control(
        &self,
        id: &str,
        endpoint: &str,
        policy: DirectoryPolicy,
    ) -> Result<ControlRegistration, StoreError> {
        if !valid_id(id)
            || endpoint.is_empty()
            || endpoint.len() > 2048
            || endpoint.chars().any(char::is_whitespace)
        {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let expiry = policy.expiry(now, false)?;
            diesel::sql_query(dialect.query(
                "INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES(?,?,?,?)",
            ))
            .bind::<Text, _>(id)
            .bind::<Text, _>(endpoint)
            .bind::<BigInt, _>(now)
            .bind::<BigInt, _>(expiry)
            .execute(connection)
            .await?;
            Ok(ControlRegistration {
                id: id.into(),
                endpoint: endpoint.into(),
                draining: false,
                expires_at: expiry,
            })
        })
    }
    pub async fn renew_control(
        &self,
        id: &str,
        policy: DirectoryPolicy,
    ) -> Result<ControlRegistration, StoreError> {
        if !valid_id(id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let mut row = diesel::sql_query(dialect.query(control_query(dialect, true)))
                .bind::<Text, _>(id)
                .get_result::<ControlRegistration>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            let now = database_now!(connection, dialect);
            if row.draining || row.expires_at <= now {
                return Err(StoreError::InvalidTransition);
            }
            row.expires_at = policy.expiry(now, false)?;
            diesel::sql_query(
                dialect.query("UPDATE control_instances SET renewed_at=?,expires_at=? WHERE id=?"),
            )
            .bind::<BigInt, _>(now)
            .bind::<BigInt, _>(row.expires_at)
            .bind::<Text, _>(id)
            .execute(connection)
            .await?;
            Ok(row)
        })
    }
    /// Drain only the registration here. Exact component closes happen in
    /// separate operations, avoiding Control->component lock-order inversion.
    pub async fn drain_control(&self, id: &str) -> Result<(), StoreError> {
        if !valid_id(id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let changed = diesel::sql_query(
                dialect.query("UPDATE control_instances SET draining=TRUE WHERE id=?"),
            )
            .bind::<Text, _>(id)
            .execute(connection)
            .await?;
            if changed != 1 {
                return Err(StoreError::NotFound);
            }
            Ok(())
        })
    }
    /// Drain is monotonic and must have committed first. This separate operation
    /// removes only that exact process's rows without a Control->route lock cycle.
    /// It does not require still-valid component keys during owned shutdown.
    pub async fn withdraw_drained_control_connections(
        &self,
        id: &str,
    ) -> Result<usize, StoreError> {
        if !valid_id(id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let control =
                diesel::sql_query(dialect.query("SELECT * FROM control_instances WHERE id=?"))
                    .bind::<Text, _>(id)
                    .get_result::<ControlRegistration>(connection)
                    .await
                    .optional()?
                    .ok_or(StoreError::NotFound)?;
            if !control.draining {
                return Err(StoreError::InvalidTransition);
            }
            let providers = diesel::sql_query(
                dialect.query("DELETE FROM provider_connections WHERE control_id=?"),
            )
            .bind::<Text, _>(id)
            .execute(connection)
            .await?;
            let connectors = diesel::sql_query(
                dialect.query("DELETE FROM connector_connections WHERE control_id=?"),
            )
            .bind::<Text, _>(id)
            .execute(connection)
            .await?;
            Ok(providers + connectors)
        })
    }
    /// Capture even an expired tuple. A later admission compares exactly this
    /// snapshot, never loops internally to steal a concurrent winning stream.
    pub async fn provider_connection_owner(
        &self,
        provider_id: &str,
        key: &SecretDigest,
    ) -> Result<Option<ConnectionOwner>, StoreError> {
        if !valid_id(provider_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            let route = diesel::sql_query(dialect.query(provider_route_query(dialect)))
                .bind::<Text, _>(provider_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            Ok(route.map(|route| route.owner()))
        })
    }
    pub async fn admit_provider_connection(
        &self,
        provider_id: &str,
        key: &SecretDigest,
        expected: Option<&ConnectionOwner>,
        owner: &ConnectionOwner,
        policy: DirectoryPolicy,
    ) -> Result<ComponentRoute, StoreError> {
        if !valid_id(provider_id) {
            return Err(StoreError::InvalidInput);
        }
        validate_owner(owner)?;
        if let Some(expected) = expected {
            validate_owner(expected)?;
            if expected == owner {
                return Err(StoreError::InvalidInput);
            }
        }
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            let route = diesel::sql_query(dialect.query(provider_route_query(dialect)))
                .bind::<Text, _>(provider_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            if !matches_owner(route.as_ref(), expected) {
                return Err(StoreError::Conflict);
            }
            let _ = live_control!(connection, dialect, &owner.control_id);
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            let expiry = policy.expiry(now, true)?;
            diesel::sql_query(dialect.query("INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES(?,?,?,?,?) ON CONFLICT(provider_id) DO UPDATE SET control_id=excluded.control_id,connection_id=excluded.connection_id,credential_hash=excluded.credential_hash,expires_at=excluded.expires_at"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).bind::<Binary,_>(key.as_bytes().as_slice()).bind::<BigInt,_>(expiry).execute(connection).await?;
            Ok(ComponentRoute {
                control_id: owner.control_id.clone(),
                connection_id: owner.connection_id.clone(),
                credential_hash: key.as_bytes().to_vec(),
                expires_at: expiry,
            })
        })
    }
    pub async fn renew_provider_connection(
        &self,
        provider_id: &str,
        key: &SecretDigest,
        owner: &ConnectionOwner,
        policy: DirectoryPolicy,
    ) -> Result<ComponentRoute, StoreError> {
        if !valid_id(provider_id) {
            return Err(StoreError::InvalidInput);
        }
        validate_owner(owner)?;
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            let mut route = diesel::sql_query(dialect.query(provider_route_query(dialect)))
                .bind::<Text, _>(provider_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?
                .ok_or(StoreError::Conflict)?;
            if !matches_owner(Some(&route), Some(owner))
                || !key_matches(Some(&route.credential_hash), key)?
            {
                return Err(StoreError::Conflict);
            }
            let _ = live_control!(connection, dialect, &owner.control_id);
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            if route.expires_at <= now {
                return Err(StoreError::InvalidTransition);
            }
            route.expires_at = policy.expiry(now, true)?;
            diesel::sql_query(dialect.query("UPDATE provider_connections SET expires_at=? WHERE provider_id=? AND control_id=? AND connection_id=?"))
                .bind::<BigInt,_>(route.expires_at).bind::<Text,_>(provider_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).execute(connection).await?;
            Ok(route)
        })
    }
    pub async fn close_provider_connection(
        &self,
        provider_id: &str,
        key: &SecretDigest,
        owner: &ConnectionOwner,
    ) -> Result<bool, StoreError> {
        if !valid_id(provider_id) {
            return Err(StoreError::InvalidInput);
        }
        validate_owner(owner)?;
        operation!(self, |connection, dialect| {
            let now = database_now!(connection, dialect);
            let _ = authenticate_provider!(connection, dialect, provider_id, key, now);
            let route = diesel::sql_query(dialect.query(provider_route_query(dialect)))
                .bind::<Text, _>(provider_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            let Some(route) = route else {
                return Ok(false);
            };
            if !matches_owner(Some(&route), Some(owner))
                || !key_matches(Some(&route.credential_hash), key)?
            {
                return Ok(false);
            }
            Ok(diesel::sql_query(dialect.query("DELETE FROM provider_connections WHERE provider_id=? AND control_id=? AND connection_id=?"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).execute(connection).await?==1)
        })
    }
    /// Internal trusted caller only. Returned route remains a snapshot; receivers
    /// must revalidate exact identities before forwarding or accepting a frame.
    pub async fn resolve_provider_route(
        &self,
        provider_id: &str,
    ) -> Result<Option<ResolvedRoute>, StoreError> {
        if !valid_id(provider_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            // Lock the Provider before reading its route/key, matching revocation.
            let provider =
                diesel::sql_query(dialect.query(crate::computers::provider_lock(dialect)))
                    .bind::<Text, _>(provider_id)
                    .get_result::<crate::computers::ProviderRow>(connection)
                    .await
                    .optional()?;
            if provider.is_none() {
                return Ok(None);
            }
            let route = diesel::sql_query(dialect.query(provider_route_query(dialect)))
                .bind::<Text, _>(provider_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            let Some(route) = route else {
                return Ok(None);
            };
            let control = diesel::sql_query(dialect.query(control_query(dialect, false)))
                .bind::<Text, _>(&route.control_id)
                .get_result::<ControlRegistration>(connection)
                .await
                .optional()?;
            let Some(control) = control else {
                return Ok(None);
            };
            let now = database_now!(connection, dialect);
            if control.draining || control.expires_at <= now || route.expires_at <= now {
                return Ok(None);
            }
            let digest = SecretDigest::from_bytes(
                route
                    .credential_hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Integrity)?,
            );
            // This macro's authentication failure is intentionally an unavailable
            // route, not an unauthenticated user request or stale-address retry.
            let key =
                diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                    .bind::<Text, _>(provider_id)
                    .get_result::<crate::component_keys::ProviderKeyRow>(connection)
                    .await
                    .optional()?;
            let Some(key) = key else {
                return Ok(None);
            };
            let valid = key_matches(Some(&key.current_hash), &digest)?
                || (key_matches(key.previous_hash.as_deref(), &digest)?
                    && key.previous_valid_until.is_some_and(|expiry| expiry > now));
            if key.revoked_at.is_some() || key.expires_at <= now || !valid {
                return Ok(None);
            }
            Ok(Some(ResolvedRoute {
                component: route,
                control,
            }))
        })
    }
}
