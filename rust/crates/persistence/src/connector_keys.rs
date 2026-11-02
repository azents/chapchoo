use crate::{
    Store, StoreError,
    component_keys::{
        ComponentKeyDraft, ComponentKeyPolicy, ComponentKeyResult, KeyRequestRow,
        authenticate_provider, key_matches, recover_key,
    },
    computers::valid_id,
    store::{Dialect, operation},
};
use chapchoo_crypto::{BearerSecret, SecretDigest, payload_fingerprint};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

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

impl Store {
    pub async fn issue_connector_session(
        &self,
        provider_id: &str,
        provider_key: &SecretDigest,
        execution_id: &str,
        draft: &ConnectorSessionDraft,
        now: i64,
    ) -> Result<ConnectorSessionResult, StoreError> {
        if !valid_id(execution_id) || !valid_id(&draft.id) || !valid_id(&draft.request_id) {
            return Err(StoreError::InvalidInput);
        }
        let context = format!("computer-connector-bootstrap:{}", draft.id);
        let protected = self.cipher.protect_latest(&context, &draft.initial_key)?;
        operation!(self, |connection, dialect| {
            let _provider =
                authenticate_provider!(connection, dialect, provider_id, provider_key, now);
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
            diesel::sql_query(dialect.query("INSERT INTO connector_sessions(id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,created_at) VALUES(?,?,?,?,?,?,?,?)"))
                .bind::<Text,_>(&draft.id).bind::<Text,_>(execution_id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(digest.as_bytes().as_slice())
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
        if !valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        let fingerprint =
            payload_fingerprint("connector-session-revocation", &[session_id.as_bytes()]);
        operation!(self, |connection, dialect| {
            let _provider =
                authenticate_provider!(connection, dialect, provider_id, provider_key, now);
            let current = lock_session!(connection, dialect, session_id, Some(provider_id));
            if current.provider_id.as_deref() != Some(provider_id) {
                return Err(StoreError::NotFound);
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
