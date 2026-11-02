use crate::{
    Store, StoreError,
    computers::{ProviderChoice, ProviderRow, provider_lock, valid_id},
    models::authenticate,
    store::operation,
};
use chapchoo_crypto::{
    BearerSecret, CredentialCipher, RecoveryEnvelope, SecretDigest, VerifiedSystemOwner,
    payload_fingerprint,
};
use diesel::{
    OptionalExtension,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;
use std::fmt;

#[derive(Debug)]
pub enum ProviderOwner<'a> {
    System(&'a VerifiedSystemOwner),
    User(&'a SecretDigest),
}

pub struct ComponentKeyDraft {
    pub request_id: String,
    pub token: BearerSecret,
}
impl fmt::Debug for ComponentKeyDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComponentKeyDraft([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentKeyPolicy {
    pub validity_ms: i64,
    pub proactive_after_ms: i64,
    pub overlap_ms: i64,
}
impl Default for ComponentKeyPolicy {
    fn default() -> Self {
        Self {
            validity_ms: 86_400_000,
            proactive_after_ms: 43_200_000,
            overlap_ms: 300_000,
        }
    }
}
impl ComponentKeyPolicy {
    pub(crate) fn times(self, now: i64) -> Result<(i64, i64), StoreError> {
        if self.validity_ms <= 0
            || self.proactive_after_ms <= 0
            || self.proactive_after_ms >= self.validity_ms
            || self.overlap_ms < 0
            || self.overlap_ms > 300_000
        {
            return Err(StoreError::InvalidInput);
        }
        Ok((
            now.checked_add(self.validity_ms)
                .ok_or(StoreError::InvalidInput)?,
            now.checked_add(self.proactive_after_ms)
                .ok_or(StoreError::InvalidInput)?,
        ))
    }
}

pub struct ComponentKeyResult {
    pub token: BearerSecret,
    pub issued_at: i64,
    pub expires_at: i64,
    pub proactive_after: i64,
}
impl fmt::Debug for ComponentKeyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComponentKeyResult([REDACTED])")
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct ProviderKeyRow {
    #[diesel(sql_type=Binary)]
    pub current_hash: Vec<u8>,
    #[diesel(sql_type=Nullable<Binary>)]
    pub previous_hash: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub previous_valid_until: Option<i64>,
    #[diesel(sql_type=BigInt)]
    pub issued_at: i64,
    #[diesel(sql_type=BigInt)]
    pub expires_at: i64,
    #[diesel(sql_type=Nullable<BigInt>)]
    pub revoked_at: Option<i64>,
    #[diesel(sql_type=Text)]
    pub current_request_id: String,
    #[diesel(sql_type=Nullable<Binary>)]
    pub recovery_nonce: Option<Vec<u8>>,
    #[diesel(sql_type=Nullable<Binary>)]
    pub recovery_ciphertext: Option<Vec<u8>>,
}
#[derive(diesel::QueryableByName)]
pub(crate) struct KeyRequestRow {
    #[diesel(sql_type=Binary)]
    pub payload_hash: Vec<u8>,
    #[diesel(sql_type=Binary)]
    pub result_hash: Vec<u8>,
}

pub(crate) fn recover_key(
    cipher: &CredentialCipher,
    context: &str,
    hash: &[u8],
    nonce: Option<&[u8]>,
    ciphertext: Option<&[u8]>,
) -> Result<BearerSecret, StoreError> {
    let digest = SecretDigest::from_bytes(hash.try_into().map_err(|_| StoreError::Integrity)?);
    let nonce = nonce
        .ok_or(StoreError::Integrity)?
        .try_into()
        .map_err(|_| StoreError::Integrity)?;
    let envelope =
        RecoveryEnvelope::from_parts(nonce, ciphertext.ok_or(StoreError::Integrity)?.to_vec())?;
    Ok(cipher.recover_latest(context, &digest, &envelope)?)
}

pub(crate) fn key_matches(
    stored: Option<&[u8]>,
    presented: &SecretDigest,
) -> Result<bool, StoreError> {
    let Some(stored) = stored else {
        return Ok(false);
    };
    let digest = SecretDigest::from_bytes(stored.try_into().map_err(|_| StoreError::Integrity)?);
    Ok(digest.matches_digest(presented))
}

macro_rules! provider_owner {
    ($connection:expr,$dialect:expr,$authority:expr,$provider_id:expr,$now:expr) => {{
        let actor = match $authority {
            ProviderOwner::User(digest) => Some(authenticate!($connection, $dialect, digest, $now)),
            ProviderOwner::System(proof) => {
                if proof.provider_id() != $provider_id {
                    return Err(StoreError::Forbidden);
                }
                None
            }
        };
        let provider = diesel::sql_query($dialect.query(provider_lock($dialect)))
            .bind::<Text, _>($provider_id)
            .get_result::<ProviderRow>($connection)
            .await
            .optional()?
            .ok_or(StoreError::NotFound)?;
        let permitted = match $authority {
            ProviderOwner::System(proof) => {
                provider.owner_kind == "SYSTEM"
                    && provider.system_owner_id.as_deref() == Some(proof.owner_id())
            }
            ProviderOwner::User(_) => {
                provider.owner_kind == "USER"
                    && actor.as_ref().is_some_and(|user| {
                        provider.user_owner_id.as_deref() == Some(user.id.as_str())
                    })
            }
        };
        if !permitted {
            return Err(StoreError::Forbidden);
        }
        provider
    }};
}

macro_rules! authenticate_provider {
    ($connection:expr,$dialect:expr,$provider_id:expr,$digest:expr,$now:expr) => {{
        let provider =
            diesel::sql_query($dialect.query($crate::computers::provider_lock($dialect)))
                .bind::<diesel::sql_types::Text, _>($provider_id)
                .get_result::<$crate::computers::ProviderRow>($connection)
                .await
                .optional()?
                .ok_or($crate::StoreError::Unauthenticated)?;
        let key =
            diesel::sql_query($dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                .bind::<diesel::sql_types::Text, _>($provider_id)
                .get_result::<$crate::component_keys::ProviderKeyRow>($connection)
                .await
                .optional()?
                .ok_or($crate::StoreError::Unauthenticated)?;
        let current =
            $crate::component_keys::key_matches(Some(key.current_hash.as_slice()), $digest)?;
        let previous = $crate::component_keys::key_matches(key.previous_hash.as_deref(), $digest)?
            && key.previous_valid_until.is_some_and(|expiry| expiry > $now);
        if key.revoked_at.is_some() || key.expires_at <= $now || (!current && !previous) {
            return Err($crate::StoreError::Unauthenticated);
        }
        provider
    }};
}

pub(crate) use authenticate_provider;

impl Store {
    pub async fn rotate_provider_key(
        &self,
        authority: ProviderOwner<'_>,
        provider_id: &str,
        draft: &ComponentKeyDraft,
        policy: ComponentKeyPolicy,
        now: i64,
    ) -> Result<ComponentKeyResult, StoreError> {
        if !valid_id(provider_id) || !valid_id(&draft.request_id) {
            return Err(StoreError::InvalidInput);
        }
        let (expires_at, proactive_after) = policy.times(now)?;
        let fingerprint = payload_fingerprint("provider-key-rotation", &[provider_id.as_bytes()]);
        let context = format!("computer-provider:{provider_id}");
        let envelope = self.cipher.protect_latest(&context, &draft.token)?;
        operation!(self, |connection, dialect| {
            let _provider = provider_owner!(connection, dialect, &authority, provider_id, now);
            let existing =
                diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                    .bind::<Text, _>(provider_id)
                    .get_result::<ProviderKeyRow>(connection)
                    .await
                    .optional()?;
            let receipt=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM provider_key_requests WHERE provider_id=? AND request_id=?"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(&draft.request_id).get_result::<KeyRequestRow>(connection).await.optional()?;
            if let Some(receipt) = receipt {
                if receipt.payload_hash != fingerprint {
                    return Err(StoreError::Conflict);
                }
                let current = existing.ok_or(StoreError::InvalidTransition)?;
                if current.current_request_id != draft.request_id
                    || current.current_hash != receipt.result_hash
                    || current.revoked_at.is_some()
                    || current.expires_at <= now
                {
                    return Err(StoreError::InvalidTransition);
                }
                let token = recover_key(
                    &self.cipher,
                    &context,
                    &current.current_hash,
                    current.recovery_nonce.as_deref(),
                    current.recovery_ciphertext.as_deref(),
                )?;
                let refresh = current
                    .issued_at
                    .checked_add(policy.proactive_after_ms)
                    .ok_or(StoreError::Integrity)?
                    .min(current.expires_at);
                return Ok(ComponentKeyResult {
                    token,
                    issued_at: current.issued_at,
                    expires_at: current.expires_at,
                    proactive_after: refresh,
                });
            }
            let (previous, overlap) = match &existing {
                Some(old) if old.revoked_at.is_none() && old.expires_at > now => {
                    if now < old.issued_at {
                        return Err(StoreError::InvalidInput);
                    }
                    let overlap = now
                        .checked_add(policy.overlap_ms)
                        .ok_or(StoreError::InvalidInput)?
                        .min(old.expires_at);
                    (Some(old.current_hash.as_slice()), Some(overlap))
                }
                _ => (None, None),
            };
            let digest = draft.token.digest();
            if existing.as_ref().is_some_and(|old| {
                old.current_hash.as_slice() == digest.as_bytes().as_slice()
                    || old.previous_hash.as_deref() == Some(digest.as_bytes().as_slice())
            }) {
                return Err(StoreError::InvalidInput);
            }
            diesel::sql_query(dialect.query("INSERT INTO provider_keys(provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES(?,?,?,?,?,?,?,?,?) ON CONFLICT(provider_id) DO UPDATE SET current_hash=excluded.current_hash,previous_hash=excluded.previous_hash,previous_valid_until=excluded.previous_valid_until,issued_at=excluded.issued_at,expires_at=excluded.expires_at,revoked_at=NULL,current_request_id=excluded.current_request_id,recovery_nonce=excluded.recovery_nonce,recovery_ciphertext=excluded.recovery_ciphertext"))
                .bind::<Text,_>(provider_id).bind::<Binary,_>(digest.as_bytes().as_slice()).bind::<Nullable<Binary>,_>(previous).bind::<Nullable<BigInt>,_>(overlap)
                .bind::<BigInt,_>(now).bind::<BigInt,_>(expires_at).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(envelope.nonce().as_slice()).bind::<Binary,_>(envelope.ciphertext()).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO provider_key_requests(provider_id,request_id,payload_hash,result_hash) VALUES(?,?,?,?)"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(digest.as_bytes().as_slice()).execute(connection).await?;
            Ok(ComponentKeyResult {
                token: BearerSecret::parse(draft.token.expose().to_owned())?,
                issued_at: now,
                expires_at,
                proactive_after,
            })
        })
    }

    pub async fn revoke_provider_key(
        &self,
        authority: ProviderOwner<'_>,
        provider_id: &str,
        request_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        if !valid_id(provider_id) || !valid_id(request_id) {
            return Err(StoreError::InvalidInput);
        }
        let fingerprint = payload_fingerprint("provider-key-revocation", &[provider_id.as_bytes()]);
        operation!(self, |connection, dialect| {
            let _provider = provider_owner!(connection, dialect, &authority, provider_id, now);
            let current =
                diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                    .bind::<Text, _>(provider_id)
                    .get_result::<ProviderKeyRow>(connection)
                    .await
                    .optional()?
                    .ok_or(StoreError::NotFound)?;
            if let Some(prior)=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM provider_key_requests WHERE provider_id=? AND request_id=?"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(request_id).get_result::<KeyRequestRow>(connection).await.optional()? {
                if prior.payload_hash!=fingerprint { return Err(StoreError::Conflict); }
                if prior.result_hash!=current.current_hash || current.revoked_at.is_none() { return Err(StoreError::InvalidTransition); }
                return Ok(());
            }
            diesel::sql_query(dialect.query("UPDATE provider_keys SET revoked_at=COALESCE(revoked_at,?),previous_hash=NULL,previous_valid_until=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE provider_id=?"))
                .bind::<BigInt,_>(now).bind::<Text,_>(provider_id).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO provider_key_requests(provider_id,request_id,payload_hash,result_hash) VALUES(?,?,?,?)"))
                .bind::<Text,_>(provider_id).bind::<Text,_>(request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(&current.current_hash).execute(connection).await?;
            diesel::sql_query(
                dialect.query("DELETE FROM provider_connections WHERE provider_id=?"),
            )
            .bind::<Text, _>(provider_id)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    /// Admission snapshot only; every accepted component operation rechecks authority.
    pub async fn provider_key_identity(
        &self,
        provider_id: &str,
        digest: &SecretDigest,
        now: i64,
    ) -> Result<ProviderChoice, StoreError> {
        operation!(self, |connection, dialect| {
            let provider = authenticate_provider!(connection, dialect, provider_id, digest, now);
            Ok(ProviderChoice {
                id: provider.id,
                name: provider.name,
            })
        })
    }
}
