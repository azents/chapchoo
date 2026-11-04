use crate::{
    Store, StoreError,
    computers::{ProviderCapabilities, ProviderIdentity, ProviderRow, provider_lock, valid_id},
    entity_id::{EntityId, EntityKind, IdentitySql},
    models::authenticate,
    store::operation,
};
use chapchoo_crypto::{
    BearerSecret, CredentialCipher, RecoveryEnvelope, SecretDigest, VerifiedSystemOwner,
    payload_fingerprint,
};
use chapchoo_domain::{InvalidProviderKeyRequest, ProviderKeyRequestId, ProviderSubstrate};
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
    Bootstrap {
        proof: &'a VerifiedSystemOwner,
        name: &'a str,
        capabilities: ProviderCapabilities,
        substrate: &'a ProviderSubstrate,
    },
    Registration {
        credential: &'a SecretDigest,
        substrate: &'a ProviderSubstrate,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ProviderRegistrationOwner<'a> {
    System(&'a str),
    User(&'a str),
    Team(&'a str),
}
/// Encryption context, not an ownership grant or credential issuer.
pub fn provider_registration_context(
    provider_id: &str,
    owner: ProviderRegistrationOwner<'_>,
) -> Result<String, StoreError> {
    let (kind, id) = match owner {
        ProviderRegistrationOwner::System(id) => ("SYSTEM", id),
        ProviderRegistrationOwner::User(id) => ("USER", id),
        ProviderRegistrationOwner::Team(id) => ("TEAM", id),
    };
    if !valid_id(provider_id) || !valid_id(id) {
        return Err(StoreError::InvalidInput);
    }
    let fingerprint = payload_fingerprint(
        "provider-registration-storage",
        &[provider_id.as_bytes(), kind.as_bytes(), id.as_bytes()],
    );
    Ok(format!(
        "provider-registration:{}",
        fingerprint
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

#[derive(diesel::QueryableByName)]
struct RegistrationCredentialRow {
    #[diesel(sql_type=Binary)]
    credential_hash: Vec<u8>,
    #[diesel(sql_type=Nullable<BigInt>)]
    revoked_at: Option<i64>,
    #[diesel(sql_type=Binary)]
    recovery_nonce: Vec<u8>,
    #[diesel(sql_type=Binary)]
    recovery_ciphertext: Vec<u8>,
}

#[derive(diesel::QueryableByName)]
struct RegistrationOwnerRow {
    #[diesel(sql_type=Text)]
    owner_kind: String,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    user_owner_id: Option<EntityId>,
    #[diesel(sql_type=Nullable<IdentitySql>)]
    owner_team_id: Option<EntityId>,
}
#[derive(diesel::QueryableByName)]
struct RegistrationTeamRow {
    #[diesel(sql_type=Nullable<BigInt>)]
    deletion_requested_at: Option<i64>,
}
#[derive(diesel::QueryableByName)]
struct OwnerIdRow {
    #[diesel(sql_type=IdentitySql)]
    id: EntityId,
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
/// A registration key remains an operator input; only Repository assigns the
/// canonical Provider identity returned alongside its operational credential.
#[derive(Debug)]
pub struct ProviderCredentialResult {
    pub provider_id: String,
    pub key: ComponentKeyResult,
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

// Configured registration keys are accepted only in this operator admission
// lane. Ordinary owner/operational methods use complete canonical Provider IDs.
macro_rules! provider_target {
    ($connection:expr,$dialect:expr,$authority:expr,$input:expr,$now:expr) => {{
        match $authority {
            ProviderOwner::User(_) => EntityId::from_dto(EntityKind::Provider, $input)?,
            authority => {
                match authority {
                    ProviderOwner::System(proof) | ProviderOwner::Bootstrap { proof, .. }
                        if proof.provider_id() != $input => return Err(StoreError::Forbidden),
                    _ => {}
                }
                let existing_id = diesel::sql_query($dialect.query("SELECT id FROM computer_providers WHERE registration_key=?"))
                    .bind::<Text,_>($input).get_result::<OwnerIdRow>($connection).await.optional()?;
                if existing_id.is_none()
                    && let ProviderOwner::Bootstrap { proof, name, capabilities, substrate } = authority {
                    if !crate::computers::valid_name(name) {
                        return Err(StoreError::InvalidInput);
                    }
                    let assigned = EntityId::generate($now)?;
                    diesel::sql_query($dialect.query("INSERT INTO computer_providers(id,registration_key,owner_kind,system_owner_id,name,provider_kind,docker_daemon_id,can_create,can_start,can_stop,can_restart,created_at) VALUES(?,?,'SYSTEM',?,?,?,?,?,?,?,?,?) ON CONFLICT(registration_key) DO NOTHING"))
                        .bind::<IdentitySql,_>(assigned).bind::<Text,_>($input)
                        .bind::<Text,_>(proof.owner_id()).bind::<Text,_>(*name)
                        .bind::<Text,_>(substrate.kind().as_str()).bind::<Nullable<Text>,_>(substrate.docker_daemon_id())
                        .bind::<diesel::sql_types::Bool,_>(capabilities.create)
                        .bind::<diesel::sql_types::Bool,_>(capabilities.start)
                        .bind::<diesel::sql_types::Bool,_>(capabilities.stop)
                        .bind::<diesel::sql_types::Bool,_>(capabilities.restart)
                        .bind::<BigInt,_>($now).execute($connection).await?;
                }
                diesel::sql_query($dialect.query("SELECT id FROM computer_providers WHERE registration_key=?"))
                    .bind::<Text,_>($input).get_result::<OwnerIdRow>($connection).await.optional()?
                    .ok_or(if matches!(authority, ProviderOwner::Registration { .. }) {
                        StoreError::Unauthenticated
                    } else { StoreError::NotFound })?.id
            }
        }
    }};
}

macro_rules! provider_owner {
    ($connection:expr,$dialect:expr,$authority:expr,$provider_id:expr,$now:expr,$cipher:expr) => {{
        let target_id = provider_target!($connection, $dialect, $authority, $provider_id, $now);
        if let ProviderOwner::Registration { .. } = $authority {
            let owner = diesel::sql_query($dialect.query("SELECT owner_kind,user_owner_id,owner_team_id FROM computer_providers WHERE id=?"))
                .bind::<IdentitySql,_>(target_id).get_result::<RegistrationOwnerRow>($connection)
                .await.optional()?.ok_or(StoreError::Unauthenticated)?;
            if let Some(team) = owner.owner_team_id {
                let sql = match $dialect {
                    crate::store::Dialect::Postgres => "SELECT deletion_requested_at FROM teams WHERE id=? FOR UPDATE",
                    crate::store::Dialect::Sqlite => "SELECT deletion_requested_at FROM teams WHERE id=?",
                };
                let team = diesel::sql_query($dialect.query(sql)).bind::<IdentitySql,_>(team)
                    .get_result::<RegistrationTeamRow>($connection).await.optional()?
                    .ok_or(StoreError::Unauthenticated)?;
                if owner.owner_kind != "TEAM" || team.deletion_requested_at.is_some() {
                    return Err(StoreError::Unauthenticated);
                }
            } else if let Some(user) = owner.user_owner_id {
                let sql = match $dialect {
                    crate::store::Dialect::Postgres => "SELECT id FROM users WHERE id=? FOR UPDATE",
                    crate::store::Dialect::Sqlite => "SELECT id FROM users WHERE id=?",
                };
                let locked = diesel::sql_query($dialect.query(sql)).bind::<IdentitySql,_>(user)
                    .get_result::<OwnerIdRow>($connection).await.optional()?
                    .ok_or(StoreError::Unauthenticated)?;
                if owner.owner_kind != "USER" || locked.id != user {
                    return Err(StoreError::Unauthenticated);
                }
            } else if owner.owner_kind != "SYSTEM" {
                return Err(StoreError::Unauthenticated);
            }
        }
        let actor = match $authority {
            ProviderOwner::User(digest) => Some(authenticate!($connection, $dialect, digest, $now)),
            ProviderOwner::System(proof) => {
                if proof.provider_id() != $provider_id {
                    return Err(StoreError::Forbidden);
                }
                None
            }
            ProviderOwner::Bootstrap { proof, .. } => {
                if proof.provider_id() != $provider_id {
                    return Err(StoreError::Forbidden);
                }
                None
            }
            ProviderOwner::Registration { .. } => None,
        };
        let provider = diesel::sql_query($dialect.query(provider_lock($dialect)))
            .bind::<IdentitySql, _>(target_id)
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
                        provider.user_owner_id.map(|id|id.to_dto(EntityKind::User)).as_ref() == Some(&user.id)
                    })
            }
            ProviderOwner::Bootstrap { proof, substrate, .. } => {
                if provider.owner_kind != "SYSTEM" || provider.provider_kind != substrate.kind().as_str()
                    || provider.system_owner_id.as_deref() != Some(proof.owner_id())
                    || provider.docker_daemon_id.as_deref().is_some_and(|bound|Some(bound)!=substrate.docker_daemon_id()) {
                    return Err(StoreError::Forbidden);
                }
                if provider.docker_daemon_id.is_none() && substrate.docker_daemon_id().is_some() {
                    diesel::sql_query($dialect.query("UPDATE computer_providers SET docker_daemon_id=? WHERE id=?"))
                        .bind::<Nullable<Text>,_>(substrate.docker_daemon_id())
                        .bind::<IdentitySql,_>(target_id).execute($connection).await?;
                }
                true
            }
            ProviderOwner::Registration { credential: digest, substrate } => {
                if provider.substrate()? != **substrate
                    || (provider.owner_kind == "TEAM" && provider.owner_team_id.is_none()) {
                    return Err(StoreError::Forbidden);
                }
                let stored = diesel::sql_query($dialect.query("SELECT credential_hash,revoked_at,recovery_nonce,recovery_ciphertext FROM provider_registration_credentials WHERE provider_id=?"))
                    .bind::<IdentitySql,_>(target_id)
                    .get_result::<RegistrationCredentialRow>($connection).await.optional()?;
                match stored {
                    Some(row) if row.revoked_at.is_none() && key_matches(Some(&row.credential_hash), digest)? => {
                        let user = provider.user_owner_id.map(|id|id.to_dto(EntityKind::User));
                        let team = provider.owner_team_id.map(|id|id.to_dto(EntityKind::Team));
                        let owner = match provider.owner_kind.as_str() {
                            "SYSTEM" => ProviderRegistrationOwner::System(provider.system_owner_id.as_deref().ok_or(StoreError::Integrity)?),
                            "USER" => ProviderRegistrationOwner::User(user.as_deref().ok_or(StoreError::Integrity)?),
                            "TEAM" => ProviderRegistrationOwner::Team(team.as_deref().ok_or(StoreError::Integrity)?),
                            _ => return Err(StoreError::Integrity),
                        };
                        let context = provider_registration_context(&target_id.to_dto(EntityKind::Provider), owner)?;
                        let _verified = recover_key($cipher, &context, &row.credential_hash,
                            Some(&row.recovery_nonce),Some(&row.recovery_ciphertext))?;
                        true
                    }
                    _ => false,
                }
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
        let target_id = $crate::entity_id::EntityId::from_dto(
            $crate::entity_id::EntityKind::Provider,
            $provider_id,
        )?;
        let provider =
            diesel::sql_query($dialect.query($crate::computers::provider_lock($dialect)))
                .bind::<$crate::entity_id::IdentitySql, _>(target_id)
                .get_result::<$crate::computers::ProviderRow>($connection)
                .await
                .optional()?
                .ok_or($crate::StoreError::Unauthenticated)?;
        let key =
            diesel::sql_query($dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                .bind::<$crate::entity_id::IdentitySql, _>(target_id)
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
            .substrate()
            .map_err(|_| $crate::StoreError::Unauthenticated)?;
        provider
    }};
}

pub(crate) use authenticate_provider;

fn provider_request(value: &str, now: i64) -> Result<ProviderKeyRequestId, StoreError> {
    let request = ProviderKeyRequestId::parse(value).map_err(|_| StoreError::InvalidInput)?;
    request.admit_at(now).map_err(|error| match error {
        InvalidProviderKeyRequest::Expired => StoreError::InvalidTransition,
        _ => StoreError::InvalidInput,
    })?;
    Ok(request)
}

impl Store {
    pub async fn rotate_provider_key(
        &self,
        authority: ProviderOwner<'_>,
        provider_id: &str,
        draft: &ComponentKeyDraft,
        policy: ComponentKeyPolicy,
        now: i64,
    ) -> Result<ProviderCredentialResult, StoreError> {
        if !valid_id(provider_id) || !valid_id(&draft.request_id) {
            return Err(StoreError::InvalidInput);
        }
        let request = provider_request(&draft.request_id, now)?;
        let (expires_at, proactive_after) = policy.times(now)?;
        let fingerprint = match &authority {
            ProviderOwner::Bootstrap {
                proof,
                name,
                capabilities,
                substrate,
            } => {
                let flags = [
                    u8::from(capabilities.create),
                    u8::from(capabilities.start),
                    u8::from(capabilities.stop),
                    u8::from(capabilities.restart),
                ];
                payload_fingerprint(
                    "provider-bootstrap-exchange",
                    &[
                        provider_id.as_bytes(),
                        name.as_bytes(),
                        substrate.kind().as_str().as_bytes(),
                        substrate.docker_daemon_id().unwrap_or("").as_bytes(),
                        &flags,
                        &proof.authentication_fingerprint(),
                    ],
                )
            }
            ProviderOwner::Registration { substrate, .. } => payload_fingerprint(
                "provider-registration-exchange",
                &[
                    provider_id.as_bytes(),
                    substrate.kind().as_str().as_bytes(),
                    substrate.docker_daemon_id().unwrap_or("").as_bytes(),
                ],
            ),
            _ => payload_fingerprint("provider-key-rotation", &[provider_id.as_bytes()]),
        };
        operation!(self, |connection, dialect| {
            let provider = provider_owner!(
                connection,
                dialect,
                &authority,
                provider_id,
                now,
                &self.cipher
            );
            let provider_id = provider.id;
            let canonical_id = provider_id.to_dto(EntityKind::Provider);
            let context = format!("computer-provider:{canonical_id}");
            let envelope = self.cipher.protect_latest(&context, &draft.token)?;
            diesel::sql_query(dialect.query(
                "DELETE FROM provider_key_requests WHERE provider_id=? AND retry_expires_at<=?",
            ))
            .bind::<IdentitySql, _>(provider_id)
            .bind::<BigInt, _>(now)
            .execute(connection)
            .await?;
            let existing =
                diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                    .bind::<IdentitySql, _>(provider_id)
                    .get_result::<ProviderKeyRow>(connection)
                    .await
                    .optional()?;
            let receipt=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM provider_key_requests WHERE provider_id=? AND request_id=?"))
                .bind::<IdentitySql,_>(provider_id).bind::<Text,_>(&draft.request_id).get_result::<KeyRequestRow>(connection).await.optional()?;
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
                return Ok(ProviderCredentialResult {
                    provider_id: canonical_id,
                    key: ComponentKeyResult {
                        token,
                        issued_at: current.issued_at,
                        expires_at: current.expires_at,
                        proactive_after: refresh,
                    },
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
                .bind::<IdentitySql,_>(provider_id).bind::<Binary,_>(digest.as_bytes().as_slice()).bind::<Nullable<Binary>,_>(previous).bind::<Nullable<BigInt>,_>(overlap)
                .bind::<BigInt,_>(now).bind::<BigInt,_>(expires_at).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(envelope.nonce().as_slice()).bind::<Binary,_>(envelope.ciphertext()).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO provider_key_requests(provider_id,request_id,payload_hash,result_hash,retry_expires_at) VALUES(?,?,?,?,?)"))
                .bind::<IdentitySql,_>(provider_id).bind::<Text,_>(&draft.request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(digest.as_bytes().as_slice()).bind::<BigInt,_>(request.retry_expires_at()).execute(connection).await?;
            Ok(ProviderCredentialResult {
                provider_id: canonical_id,
                key: ComponentKeyResult {
                    token: BearerSecret::parse(draft.token.expose().to_owned())?,
                    issued_at: now,
                    expires_at,
                    proactive_after,
                },
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
        let request = provider_request(request_id, now)?;
        let fingerprint = payload_fingerprint("provider-key-revocation", &[provider_id.as_bytes()]);
        operation!(self, |connection, dialect| {
            let provider = provider_owner!(
                connection,
                dialect,
                &authority,
                provider_id,
                now,
                &self.cipher
            );
            let provider_id = provider.id;
            diesel::sql_query(dialect.query(
                "DELETE FROM provider_key_requests WHERE provider_id=? AND retry_expires_at<=?",
            ))
            .bind::<IdentitySql, _>(provider_id)
            .bind::<BigInt, _>(now)
            .execute(connection)
            .await?;
            let current =
                diesel::sql_query(dialect.query("SELECT * FROM provider_keys WHERE provider_id=?"))
                    .bind::<IdentitySql, _>(provider_id)
                    .get_result::<ProviderKeyRow>(connection)
                    .await
                    .optional()?
                    .ok_or(StoreError::NotFound)?;
            if let Some(prior)=diesel::sql_query(dialect.query("SELECT payload_hash,result_hash FROM provider_key_requests WHERE provider_id=? AND request_id=?"))
                .bind::<IdentitySql,_>(provider_id).bind::<Text,_>(request_id).get_result::<KeyRequestRow>(connection).await.optional()? {
                if prior.payload_hash!=fingerprint { return Err(StoreError::Conflict); }
                if prior.result_hash!=current.current_hash || current.revoked_at.is_none() { return Err(StoreError::InvalidTransition); }
                return Ok(());
            }
            diesel::sql_query(dialect.query("UPDATE provider_keys SET revoked_at=COALESCE(revoked_at,?),previous_hash=NULL,previous_valid_until=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL WHERE provider_id=?"))
                .bind::<BigInt,_>(now).bind::<IdentitySql,_>(provider_id).execute(connection).await?;
            diesel::sql_query(dialect.query("INSERT INTO provider_key_requests(provider_id,request_id,payload_hash,result_hash,retry_expires_at) VALUES(?,?,?,?,?)"))
                .bind::<IdentitySql,_>(provider_id).bind::<Text,_>(request_id).bind::<Binary,_>(fingerprint.as_slice()).bind::<Binary,_>(&current.current_hash).bind::<BigInt,_>(request.retry_expires_at()).execute(connection).await?;
            diesel::sql_query(
                dialect.query("DELETE FROM provider_connections WHERE provider_id=?"),
            )
            .bind::<IdentitySql, _>(provider_id)
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
    ) -> Result<ProviderIdentity, StoreError> {
        operation!(self, |connection, dialect| {
            let provider = authenticate_provider!(connection, dialect, provider_id, digest, now);
            let substrate = provider.substrate()?;
            Ok(ProviderIdentity {
                id: provider.id.to_dto(EntityKind::Provider),
                name: provider.name,
                substrate,
            })
        })
    }
}

#[cfg(test)]
mod tests;
