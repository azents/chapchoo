use crate::{
    Store, StoreError,
    component_keys::{
        ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult, KeyRequestRow,
        authenticate_provider, key_matches, recover_key,
    },
    computer_executions::current_source,
    computers::valid_id,
    directory::{
        ComponentRoute, ConnectionOwner, ControlRegistration, DirectoryPolicy, ResolvedRoute,
        control_query, database_now, live_control, matches_owner, validate_owner,
    },
    store::{Dialect, operation},
};
use chapchoo_crypto::{BearerSecret, SecretDigest, payload_fingerprint};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

#[derive(Debug, Clone, Copy)]
enum SessionAuthority<'a> {
    Key {
        provider: &'a str,
        key: &'a SecretDigest,
    },
    Connection(&'a crate::ProviderSource<'a>),
}
impl<'a> SessionAuthority<'a> {
    fn provider(self) -> &'a str {
        match self {
            Self::Key { provider, .. } => provider,
            Self::Connection(source) => source.provider_id,
        }
    }
    fn key(self) -> &'a SecretDigest {
        match self {
            Self::Key { key, .. } => key,
            Self::Connection(source) => source.credential,
        }
    }
}

pub struct ConnectorSessionDraft {
    pub id: String,
    pub request_id: String,
    pub initial_key: BearerSecret,
}
impl fmt::Debug for ConnectorSessionDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectorSessionDraft([REDACTED])")
    }
}
pub struct ConnectorSessionResult {
    pub id: String,
    pub execution_id: String,
    pub accepted_request_id: String,
    pub initial_key: BearerSecret,
}
impl fmt::Debug for ConnectorSessionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectorSessionResult([REDACTED])")
    }
}
pub struct ConnectorIdentity {
    pub session_id: String,
    pub execution_id: String,
    pub computer_id: String,
    pub expires_at: i64,
}
pub struct ConnectorSource<'a> {
    pub session_id: &'a str,
    pub control_id: &'a str,
    pub connection_id: &'a str,
    pub credential: &'a SecretDigest,
}
impl fmt::Debug for ConnectorSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectorSource([REDACTED])")
    }
}
impl fmt::Debug for ConnectorIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ConnectorIdentity([REDACTED])")
    }
}

#[derive(diesel::QueryableByName)]
struct Locator {
    #[diesel(sql_type=Text)]
    computer_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    provider_id: Option<String>,
}
#[derive(diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type=Text)]
    id: String,
}
#[derive(diesel::QueryableByName)]
struct ExecutionBinding {
    #[diesel(sql_type=Text)]
    computer_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    current_execution_id: Option<String>,
    #[diesel(sql_type=Text)]
    desired_state: String,
    #[diesel(sql_type=Nullable<BigInt>)]
    retired_at: Option<i64>,
}
#[derive(diesel::QueryableByName)]
struct SessionRow {
    #[diesel(sql_type=Text)]
    id: String,
    #[diesel(sql_type=Text)]
    execution_id: String,
    #[diesel(sql_type=Text)]
    initial_request_id: String,
    #[diesel(sql_type=Binary)]
    initial_payload_hash: Vec<u8>,
    #[diesel(sql_type=Binary)]
    initial_hash: Vec<u8>,
    #[diesel(sql_type=Nullable<Binary>)]
    initial_nonce: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    initial_ciphertext: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    current_hash: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    previous_hash: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<BigInt>)]
    previous_valid_until: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    issued_at: Option<i64>,
    #[diesel(sql_type=Nullable<BigInt>)]
    expires_at: Option<i64>,
    #[diesel(sql_type=Nullable<Text>)]
    current_request_id: Option<String>,
    #[diesel(sql_type=Nullable<Binary>)]
    recovery_nonce: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    recovery_ciphertext: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<BigInt>)]
    revoked_at: Option<i64>,
    #[diesel(sql_type=Text)]
    computer_id: String,
    #[diesel(sql_type=Nullable<Text>)]
    provider_id: Option<String>,
    #[diesel(sql_type=Nullable<Text>)]
    current_execution_id: Option<String>,
    #[diesel(sql_type=Text)]
    desired_state: String,
    #[diesel(sql_type=Nullable<BigInt>)]
    retired_at: Option<i64>,
}
const SESSION_QUERY: &str = "SELECT s.*,c.id AS computer_id,c.provider_id,c.current_execution_id,c.desired_state,e.retired_at FROM connector_sessions s JOIN computer_executions e ON e.id=s.execution_id JOIN computers c ON c.id=e.computer_id";

impl SessionRow {
    fn active(&self) -> bool {
        self.revoked_at.is_none()
            && self.retired_at.is_none()
            && self.desired_state != "DELETING"
            && self.current_execution_id.as_deref() == Some(self.execution_id.as_str())
    }
    fn operational(&self, digest: &SecretDigest, now: i64) -> Result<bool, StoreError> {
        if !self.active()
            || !self.expires_at.is_some_and(|expiry| expiry > now)
            || !self.issued_at.is_some_and(|issued| issued <= now)
        {
            return Err(StoreError::Unauthenticated);
        }
        if key_matches(self.current_hash.as_deref(), digest)? {
            return Ok(true);
        }
        if key_matches(self.previous_hash.as_deref(), digest)?
            && self.previous_valid_until.is_some_and(|expiry| expiry > now)
        {
            return Ok(false);
        }
        Err(StoreError::Unauthenticated)
    }
}

macro_rules! lock_session {
    ($connection:expr,$dialect:expr,$session_id:expr,$expected_provider:expr)=>{{
        let locator=diesel::sql_query($dialect.query("SELECT c.id AS computer_id,c.provider_id FROM connector_sessions s JOIN computer_executions e ON e.id=s.execution_id JOIN computers c ON c.id=e.computer_id WHERE s.id=?"))
            .bind::<Text,_>($session_id).get_result::<Locator>($connection).await.optional()?.ok_or(StoreError::Unauthenticated)?;
        let expected:Option<&str>=$expected_provider;
        if expected.is_some_and(|id|locator.provider_id.as_deref()!=Some(id)) { return Err(StoreError::NotFound); }
        let q=match $dialect { Dialect::Postgres=>"SELECT id FROM computers WHERE id=? FOR NO KEY UPDATE",Dialect::Sqlite=>"SELECT id FROM computers WHERE id=?" };
        let current=diesel::sql_query($dialect.query(q)).bind::<Text,_>(&locator.computer_id).get_result::<IdRow>($connection).await.optional()?.ok_or(StoreError::Unauthenticated)?;
        if current.id!=locator.computer_id { return Err(StoreError::Integrity); }
        let q=format!("{SESSION_QUERY} WHERE s.id=?");
        diesel::sql_query($dialect.query(&q)).bind::<Text,_>($session_id).get_result::<SessionRow>($connection).await.optional()?.ok_or(StoreError::Unauthenticated)?
    }};
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyMutation {
    Exchange,
    Refresh,
}
impl KeyMutation {
    fn label(self) -> &'static str {
        match self {
            Self::Exchange => "EXCHANGE",
            Self::Refresh => "REFRESH",
        }
    }
}

fn connector_route_query(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Postgres => {
            "SELECT * FROM connector_connections WHERE computer_id=? FOR NO KEY UPDATE"
        }
        Dialect::Sqlite => "SELECT * FROM connector_connections WHERE computer_id=?",
    }
}
#[derive(diesel::QueryableByName)]
struct ConnectorRouteRow {
    #[diesel(sql_type=Text)]
    session_id: String,
    #[diesel(sql_type=Text)]
    control_id: String,
    #[diesel(sql_type=Text)]
    connection_id: String,
    #[diesel(sql_type=Binary)]
    credential_hash: Vec<u8>,
    #[diesel(sql_type=BigInt)]
    expires_at: i64,
}
impl ConnectorRouteRow {
    fn route(self) -> ComponentRoute {
        ComponentRoute {
            control_id: self.control_id,
            connection_id: self.connection_id,
            credential_hash: self.credential_hash,
            expires_at: self.expires_at,
        }
    }
}

impl Store {
    /// Check an operational frame/dequeue without extending heartbeat expiry.
    pub async fn verify_connector_connection(
        &self,
        source: &ConnectorSource<'_>,
    ) -> Result<ConnectorIdentity, StoreError> {
        if ![source.session_id, source.control_id, source.connection_id]
            .into_iter()
            .all(valid_id)
        {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let session = lock_session!(connection, dialect, source.session_id, None);
            session.operational(source.credential, database_now!(connection, dialect))?;
            let row = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(&session.computer_id)
                .get_result::<ConnectorRouteRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::Conflict)?;
            if row.session_id != source.session_id
                || row.control_id != source.control_id
                || row.connection_id != source.connection_id
                || !key_matches(Some(&row.credential_hash), source.credential)?
            {
                return Err(StoreError::Conflict);
            }
            let _ = live_control!(connection, dialect, source.control_id);
            let now = database_now!(connection, dialect);
            session.operational(source.credential, now)?;
            if row.expires_at <= now {
                return Err(StoreError::InvalidTransition);
            }
            Ok(ConnectorIdentity {
                session_id: session.id,
                execution_id: session.execution_id,
                computer_id: session.computer_id,
                expires_at: session.expires_at.ok_or(StoreError::Integrity)?,
            })
        })
    }
    /// Bootstrap keys cannot open operational streams. This snapshot is captured
    /// before local stream preparation and compared exactly during admission.
    pub async fn connector_connection_owner(
        &self,
        session_id: &str,
        key: &SecretDigest,
    ) -> Result<Option<ConnectionOwner>, StoreError> {
        if !valid_id(session_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let session = lock_session!(connection, dialect, session_id, None);
            session.operational(key, database_now!(connection, dialect))?;
            let route = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(&session.computer_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            session.operational(key, database_now!(connection, dialect))?;
            Ok(route.map(|row| row.owner()))
        })
    }
    pub async fn admit_connector_connection(
        &self,
        session_id: &str,
        key: &SecretDigest,
        expected: Option<&ConnectionOwner>,
        owner: &ConnectionOwner,
        policy: DirectoryPolicy,
    ) -> Result<ComponentRoute, StoreError> {
        if !valid_id(session_id) {
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
            let session = lock_session!(connection, dialect, session_id, None);
            session.operational(key, database_now!(connection, dialect))?;
            let route = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(&session.computer_id)
                .get_result::<ComponentRoute>(connection)
                .await
                .optional()?;
            if !matches_owner(route.as_ref(), expected) {
                return Err(StoreError::Conflict);
            }
            let _ = live_control!(connection, dialect, &owner.control_id);
            let now = database_now!(connection, dialect);
            session.operational(key, now)?;
            let expiry = policy.expiry(now, true)?;
            diesel::sql_query(dialect.query("INSERT INTO connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at) VALUES(?,?,?,?,?,?) ON CONFLICT(computer_id) DO UPDATE SET session_id=excluded.session_id,control_id=excluded.control_id,connection_id=excluded.connection_id,credential_hash=excluded.credential_hash,expires_at=excluded.expires_at"))
                .bind::<Text,_>(&session.computer_id).bind::<Text,_>(session_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).bind::<Binary,_>(key.as_bytes().as_slice()).bind::<BigInt,_>(expiry).execute(connection).await?;
            Ok(ComponentRoute {
                control_id: owner.control_id.clone(),
                connection_id: owner.connection_id.clone(),
                credential_hash: key.as_bytes().to_vec(),
                expires_at: expiry,
            })
        })
    }
    pub async fn renew_connector_connection(
        &self,
        session_id: &str,
        key: &SecretDigest,
        owner: &ConnectionOwner,
        policy: DirectoryPolicy,
    ) -> Result<ComponentRoute, StoreError> {
        if !valid_id(session_id) {
            return Err(StoreError::InvalidInput);
        }
        validate_owner(owner)?;
        operation!(self, |connection, dialect| {
            let session = lock_session!(connection, dialect, session_id, None);
            session.operational(key, database_now!(connection, dialect))?;
            let row = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(&session.computer_id)
                .get_result::<ConnectorRouteRow>(connection)
                .await
                .optional()?
                .ok_or(StoreError::Conflict)?;
            if row.session_id != session_id {
                return Err(StoreError::Conflict);
            }
            let mut route = row.route();
            if !matches_owner(Some(&route), Some(owner))
                || !key_matches(Some(&route.credential_hash), key)?
            {
                return Err(StoreError::Conflict);
            }
            let _ = live_control!(connection, dialect, &owner.control_id);
            let now = database_now!(connection, dialect);
            session.operational(key, now)?;
            if route.expires_at <= now {
                return Err(StoreError::InvalidTransition);
            }
            route.expires_at = policy.expiry(now, true)?;
            diesel::sql_query(dialect.query("UPDATE connector_connections SET expires_at=? WHERE computer_id=? AND session_id=? AND control_id=? AND connection_id=?"))
                .bind::<BigInt,_>(route.expires_at).bind::<Text,_>(&session.computer_id).bind::<Text,_>(session_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).execute(connection).await?;
            Ok(route)
        })
    }
    pub async fn close_connector_connection(
        &self,
        session_id: &str,
        key: &SecretDigest,
        owner: &ConnectionOwner,
    ) -> Result<bool, StoreError> {
        if !valid_id(session_id) {
            return Err(StoreError::InvalidInput);
        }
        validate_owner(owner)?;
        operation!(self, |connection, dialect| {
            let session = lock_session!(connection, dialect, session_id, None);
            session.operational(key, database_now!(connection, dialect))?;
            let row = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(&session.computer_id)
                .get_result::<ConnectorRouteRow>(connection)
                .await
                .optional()?;
            let Some(row) = row else {
                return Ok(false);
            };
            if row.session_id != session_id {
                return Ok(false);
            }
            let route = row.route();
            if !matches_owner(Some(&route), Some(owner))
                || !key_matches(Some(&route.credential_hash), key)?
            {
                return Ok(false);
            }
            Ok(diesel::sql_query(dialect.query("DELETE FROM connector_connections WHERE computer_id=? AND session_id=? AND control_id=? AND connection_id=?"))
                .bind::<Text,_>(&session.computer_id).bind::<Text,_>(session_id).bind::<Text,_>(&owner.control_id).bind::<Text,_>(&owner.connection_id).execute(connection).await?==1)
        })
    }
    pub async fn resolve_connector_route(
        &self,
        computer_id: &str,
    ) -> Result<Option<ResolvedRoute>, StoreError> {
        if !valid_id(computer_id) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let lock = match dialect {
                Dialect::Postgres => "SELECT id FROM computers WHERE id=? FOR NO KEY UPDATE",
                Dialect::Sqlite => "SELECT id FROM computers WHERE id=?",
            };
            let computer = diesel::sql_query(dialect.query(lock))
                .bind::<Text, _>(computer_id)
                .get_result::<IdRow>(connection)
                .await
                .optional()?;
            if computer.is_none() {
                return Ok(None);
            }
            let route = diesel::sql_query(dialect.query(connector_route_query(dialect)))
                .bind::<Text, _>(computer_id)
                .get_result::<ConnectorRouteRow>(connection)
                .await
                .optional()?;
            let Some(route) = route else {
                return Ok(None);
            };
            let session =
                diesel::sql_query(dialect.query(&format!("{SESSION_QUERY} WHERE s.id=?")))
                    .bind::<Text, _>(&route.session_id)
                    .get_result::<SessionRow>(connection)
                    .await
                    .optional()?;
            let Some(session) = session else {
                return Ok(None);
            };
            let route = route.route();
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
            let key = SecretDigest::from_bytes(
                route
                    .credential_hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| StoreError::Integrity)?,
            );
            match session.operational(&key, now) {
                Ok(_) => {}
                Err(StoreError::Unauthenticated) => return Ok(None),
                Err(error) => return Err(error),
            }
            Ok(Some(ResolvedRoute {
                component: route,
                control,
            }))
        })
    }
    pub async fn issue_connector_session(
        &self,
        provider_id: &str,
        provider_key: &SecretDigest,
        execution_id: &str,
        draft: &ConnectorSessionDraft,
        now: i64,
    ) -> Result<ConnectorSessionResult, StoreError> {
        self.issue_connector_session_authorized(
            SessionAuthority::Key {
                provider: provider_id,
                key: provider_key,
            },
            execution_id,
            draft,
            now,
        )
        .await
    }
    pub async fn issue_connector_session_from_source(
        &self,
        source: &crate::ProviderSource<'_>,
        execution_id: &str,
        draft: &ConnectorSessionDraft,
        now: i64,
    ) -> Result<ConnectorSessionResult, StoreError> {
        self.issue_connector_session_authorized(
            SessionAuthority::Connection(source),
            execution_id,
            draft,
            now,
        )
        .await
    }
    async fn issue_connector_session_authorized(
        &self,
        authority: SessionAuthority<'_>,
        execution_id: &str,
        draft: &ConnectorSessionDraft,
        now: i64,
    ) -> Result<ConnectorSessionResult, StoreError> {
        let provider_id = authority.provider();
        let provider_key = authority.key();
        if !valid_id(execution_id) || !valid_id(&draft.id) || !valid_id(&draft.request_id) {
            return Err(StoreError::InvalidInput);
        }
        let context = format!("computer-connector-bootstrap:{}", draft.id);
        let protected = self.cipher.protect_latest(&context, &draft.initial_key)?;
        operation!(self, |connection, dialect| {
            let now = if let SessionAuthority::Connection(source) = authority {
                current_source!(connection, dialect, source).1
            } else {
                let _ = authenticate_provider!(connection, dialect, provider_id, provider_key, now);
                now
            };
            let q = "SELECT c.id AS computer_id,c.current_execution_id,c.desired_state,e.retired_at FROM computer_executions e JOIN computers c ON c.id=e.computer_id WHERE e.id=? AND c.provider_id=?";
            let q = match dialect {
                Dialect::Postgres => format!("{q} FOR NO KEY UPDATE OF c"),
                Dialect::Sqlite => q.into(),
            };
            let binding = diesel::sql_query(dialect.query(&q))
                .bind::<Text, _>(execution_id)
                .bind::<Text, _>(provider_id)
                .get_result::<ExecutionBinding>(connection)
                .await
                .optional()?
                .ok_or(StoreError::NotFound)?;
            if binding.current_execution_id.as_deref() != Some(execution_id)
                || binding.retired_at.is_some()
                || binding.desired_state == "DELETING"
            {
                return Err(StoreError::InvalidTransition);
            }
            if let SessionAuthority::Connection(source) = authority {
                let _ = current_source!(connection, dialect, source);
            }
            let fingerprint = payload_fingerprint(
                "connector-session",
                &[binding.computer_id.as_bytes(), execution_id.as_bytes()],
            );
            let q = format!("{SESSION_QUERY} WHERE s.execution_id=?");
            if let Some(existing) = diesel::sql_query(dialect.query(&q))
                .bind::<Text, _>(execution_id)
                .get_result::<SessionRow>(connection)
                .await
                .optional()?
            {
                if !existing.active() {
                    return Err(StoreError::InvalidTransition);
                }
                if existing.initial_payload_hash != fingerprint {
                    return Err(StoreError::Integrity);
                }
                let key = recover_key(
                    &self.cipher,
                    &format!("computer-connector-bootstrap:{}", existing.id),
                    &existing.initial_hash,
                    existing.initial_nonce.as_deref(),
                    existing.initial_ciphertext.as_deref(),
                )?;
                return Ok(ConnectorSessionResult {
                    id: existing.id,
                    execution_id: existing.execution_id,
                    accepted_request_id: existing.initial_request_id,
                    initial_key: key,
                });
            }
            let digest = draft.initial_key.digest();
            diesel::sql_query(dialect.query("INSERT INTO connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,created_at) VALUES(?,?,?,?,?,?,?,?,?)"))
                .bind::<Text,_>(&draft.id).bind::<Text,_>(&binding.computer_id).bind::<Text,_>(execution_id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(digest.as_bytes().as_slice())
                .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext()).bind::<BigInt,_>(now).execute(connection).await?;
            Ok(ConnectorSessionResult {
                id: draft.id.clone(),
                execution_id: execution_id.into(),
                accepted_request_id: draft.request_id.clone(),
                initial_key: BearerSecret::parse(draft.initial_key.expose().to_owned())?,
            })
        })
    }

    pub async fn exchange_connector_key(
        &self,
        session_id: &str,
        initial_key: &SecretDigest,
        draft: &ComponentKeyDraft,
        policy: ComponentKeyPolicy,
        now: i64,
    ) -> Result<ComponentKeyResult, StoreError> {
        self.mutate_connector_key(
            session_id,
            initial_key,
            draft,
            policy,
            now,
            KeyMutation::Exchange,
        )
        .await
    }
    pub async fn refresh_connector_key(
        &self,
        session_id: &str,
        current_key: &SecretDigest,
        draft: &ComponentKeyDraft,
        policy: ComponentKeyPolicy,
        now: i64,
    ) -> Result<ComponentKeyResult, StoreError> {
        self.mutate_connector_key(
            session_id,
            current_key,
            draft,
            policy,
            now,
            KeyMutation::Refresh,
        )
        .await
    }
    async fn mutate_connector_key(
        &self,
        session_id: &str,
        presented: &SecretDigest,
        draft: &ComponentKeyDraft,
        policy: ComponentKeyPolicy,
        now: i64,
        kind: KeyMutation,
    ) -> Result<ComponentKeyResult, StoreError> {
        if !valid_id(session_id) || !valid_id(&draft.request_id) {
            return Err(StoreError::InvalidInput);
        }
        let (expires_at, proactive_after) = policy.times(now)?;
        let fingerprint = payload_fingerprint(
            "connector-key-mutation",
            &[session_id.as_bytes(), kind.label().as_bytes()],
        );
        let context = format!("computer-connector-operational:{session_id}");
        let envelope = self.cipher.protect_latest(&context, &draft.token)?;
        operation!(self, |connection, dialect| {
            let current = lock_session!(connection, dialect, session_id, None);
            if !current.active() {
                return Err(StoreError::Unauthenticated);
            }
            let may_rotate = match kind {
                KeyMutation::Exchange => {
                    if !key_matches(Some(current.initial_hash.as_slice()), presented)? {
                        return Err(StoreError::Unauthenticated);
                    }
                    true
                }
                KeyMutation::Refresh => current.operational(presented, now)?,
            };
            let receipt=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM connector_key_requests WHERE session_id=? AND request_id=?"))
                .bind::<Text,_>(session_id).bind::<Text,_>(&draft.request_id).get_result::<KeyRequestRow>(connection).await.optional()?;
            if let Some(receipt) = receipt {
                if receipt.payload_hash != fingerprint {
                    return Err(StoreError::Conflict);
                }
                if current.current_request_id.as_deref() != Some(draft.request_id.as_str())
                    || current.current_hash.as_deref() != Some(receipt.result_hash.as_slice())
                    || !current.expires_at.is_some_and(|expiry| expiry > now)
                {
                    return Err(StoreError::InvalidTransition);
                }
                let token = recover_key(
                    &self.cipher,
                    &context,
                    &receipt.result_hash,
                    current.recovery_nonce.as_deref(),
                    current.recovery_ciphertext.as_deref(),
                )?;
                let issued = current.issued_at.ok_or(StoreError::Integrity)?;
                let expires = current.expires_at.ok_or(StoreError::Integrity)?;
                return Ok(ComponentKeyResult {
                    token,
                    issued_at: issued,
                    expires_at: expires,
                    proactive_after: issued
                        .checked_add(policy.proactive_after_ms)
                        .ok_or(StoreError::Integrity)?
                        .min(expires),
                });
            }
            if !may_rotate {
                return Err(StoreError::InvalidTransition);
            }
            let digest = draft.token.digest();
            if current.current_hash.as_deref() == Some(digest.as_bytes().as_slice())
                || current.previous_hash.as_deref() == Some(digest.as_bytes().as_slice())
                || current.initial_hash.as_slice() == digest.as_bytes().as_slice()
            {
                return Err(StoreError::InvalidInput);
            }
            let (previous, overlap) = if kind == KeyMutation::Refresh {
                let expiry = current.expires_at.ok_or(StoreError::Integrity)?;
                (
                    current.current_hash.as_deref(),
                    Some(
                        now.checked_add(policy.overlap_ms)
                            .ok_or(StoreError::InvalidInput)?
                            .min(expiry),
                    ),
                )
            } else {
                (None, None)
            };
            diesel::sql_query(dialect.query("UPDATE connector_sessions SET current_hash=?,previous_hash=?,previous_valid_until=?,issued_at=?,expires_at=?,current_request_id=?,recovery_nonce=?,recovery_ciphertext=? WHERE id=?"))
                .bind::<Binary,_>(digest.as_bytes().as_slice()).bind::<Nullable<Binary>,_>(previous).bind::<Nullable<BigInt>,_>(overlap).bind::<BigInt,_>(now).bind::<BigInt,_>(expires_at)
                .bind::<Text,_>(&draft.request_id).bind::<Binary,_>(envelope.nonce().as_slice()).bind::<Binary,_>(envelope.ciphertext()).bind::<Text,_>(session_id).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO connector_key_requests(session_id,request_id,purpose,payload_hash,result_hash) VALUES(?,?,?,?,?)"))
                .bind::<Text,_>(session_id).bind::<Text,_>(&draft.request_id).bind::<Text,_>(kind.label()).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(digest.as_bytes().as_slice()).execute(connection).await?;
            if kind == KeyMutation::Exchange {
                diesel::sql_query(
                    dialect.query("DELETE FROM connector_connections WHERE session_id=?"),
                )
                .bind::<Text, _>(session_id)
                .execute(connection)
                .await?;
            }
            Ok(ComponentKeyResult {
                token: BearerSecret::parse(draft.token.expose().to_owned())?,
                issued_at: now,
                expires_at,
                proactive_after,
            })
        })
    }

    pub async fn connector_key_identity(
        &self,
        session_id: &str,
        key: &SecretDigest,
        now: i64,
    ) -> Result<ConnectorIdentity, StoreError> {
        operation!(self, |connection, dialect| {
            let current = lock_session!(connection, dialect, session_id, None);
            current.operational(key, now)?;
            Ok(ConnectorIdentity {
                session_id: current.id,
                execution_id: current.execution_id,
                computer_id: current.computer_id,
                expires_at: current.expires_at.ok_or(StoreError::Integrity)?,
            })
        })
    }

    pub async fn revoke_connector_session(
        &self,
        provider_id: &str,
        provider_key: &SecretDigest,
        session_id: &str,
        request_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        self.revoke_connector_session_authorized(
            SessionAuthority::Key {
                provider: provider_id,
                key: provider_key,
            },
            session_id,
            request_id,
            now,
        )
        .await
    }
    pub async fn revoke_connector_session_from_source(
        &self,
        source: &crate::ProviderSource<'_>,
        session_id: &str,
        request_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        self.revoke_connector_session_authorized(
            SessionAuthority::Connection(source),
            session_id,
            request_id,
            now,
        )
        .await
    }
    async fn revoke_connector_session_authorized(
        &self,
        authority: SessionAuthority<'_>,
        session_id: &str,
        request_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        let provider_id = authority.provider();
        let provider_key = authority.key();
        if !valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        let fingerprint =
            payload_fingerprint("connector-session-revocation", &[session_id.as_bytes()]);
        operation!(self, |connection, dialect| {
            let now = if let SessionAuthority::Connection(source) = authority {
                current_source!(connection, dialect, source).1
            } else {
                let _ = authenticate_provider!(connection, dialect, provider_id, provider_key, now);
                now
            };
            let current = lock_session!(connection, dialect, session_id, Some(provider_id));
            if current.provider_id.as_deref() != Some(provider_id) {
                return Err(StoreError::NotFound);
            }
            if let SessionAuthority::Connection(source) = authority {
                let _ = current_source!(connection, dialect, source);
            }
            if let Some(prior)=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM connector_key_requests WHERE session_id=? AND request_id=?"))
                .bind::<Text,_>(session_id).bind::<Text,_>(request_id).get_result::<KeyRequestRow>(connection).await.optional()? {
                if prior.payload_hash!=fingerprint { return Err(StoreError::Conflict); }
                if current.revoked_at.is_none() || prior.result_hash!=current.initial_hash { return Err(StoreError::InvalidTransition); }
                return Ok(());
            }
            diesel::sql_query(dialect.query("UPDATE connector_sessions SET revoked_at=COALESCE(revoked_at,?),initial_nonce=NULL,initial_ciphertext=NULL,current_hash=NULL,previous_hash=NULL,previous_valid_until=NULL,issued_at=NULL,expires_at=NULL,current_request_id=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(session_id).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO connector_key_requests(session_id,request_id,purpose,payload_hash,result_hash) VALUES(?,?,'REVOKE',?,?)"))
                .bind::<Text,_>(session_id).bind::<Text,_>(request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(&current.initial_hash).execute(connection).await?;
            diesel::sql_query(
                dialect.query("DELETE FROM connector_connections WHERE session_id=?"),
            )
            .bind::<Text, _>(session_id)
            .execute(connection)
            .await?;
            Ok(())
        })
    }
}
