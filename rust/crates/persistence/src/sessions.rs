use chapchoo_crypto::{BearerSecret, RecoveryEnvelope, SecretDigest};
use diesel::sql_types::{BigInt, Binary, Nullable, Text};
use diesel_async::RunQueryDsl;

use crate::{
    Store, StoreError, UserSummary, VerifiedCredential,
    models::{UserRow, authenticate},
    store::{Dialect, operation},
};

#[derive(Debug, Clone, Copy)]
pub struct SessionPolicy {
    pub validity_ms: i64,
    pub rotation_ms: i64,
    pub overlap_ms: i64,
    pub absolute_lifetime_ms: Option<i64>,
}

impl Default for SessionPolicy {
    fn default() -> Self {
        Self {
            validity_ms: 180 * 24 * 60 * 60 * 1000,
            rotation_ms: 10 * 60 * 1000,
            overlap_ms: 5 * 60 * 1000,
            absolute_lifetime_ms: None,
        }
    }
}

impl SessionPolicy {
    pub(crate) fn expiry(
        self,
        created_at: i64,
        now: i64,
        existing_cap: Option<i64>,
    ) -> Result<(i64, Option<i64>), StoreError> {
        if self.validity_ms <= 0
            || self.rotation_ms <= 0
            || self.overlap_ms <= 0
            || self.absolute_lifetime_ms.is_some_and(|value| value <= 0)
            || now < created_at
        {
            return Err(StoreError::Integrity);
        }
        let configured_cap = self
            .absolute_lifetime_ms
            .map(|duration| {
                created_at
                    .checked_add(duration)
                    .ok_or(StoreError::Integrity)
            })
            .transpose()?;
        let cap = match (existing_cap, configured_cap) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let expires = now
            .checked_add(self.validity_ms)
            .ok_or(StoreError::Integrity)?;
        let expires = cap.map_or(expires, |value| value.min(expires));
        if expires <= now {
            return Err(StoreError::Unauthenticated);
        }
        Ok((expires, cap))
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn expiry_is_checked_clamped_and_never_revives_absolute_expiry() -> Result<(), StoreError> {
        let policy = SessionPolicy {
            validity_ms: 100,
            rotation_ms: 10,
            overlap_ms: 5,
            absolute_lifetime_ms: Some(150),
        };
        assert_eq!(policy.expiry(0, 0, None)?, (100, Some(150)));
        assert_eq!(policy.expiry(0, 80, Some(150))?, (150, Some(150)));
        assert!(matches!(
            policy.expiry(0, 150, Some(150)),
            Err(StoreError::Unauthenticated)
        ));
        assert_eq!(
            SessionPolicy {
                absolute_lifetime_ms: None,
                ..policy
            }
            .expiry(0, 80, None)?,
            (180, None)
        );
        assert!(policy.expiry(10, 0, None).is_err());
        assert!(
            SessionPolicy {
                validity_ms: 0,
                ..policy
            }
            .expiry(0, 0, None)
            .is_err()
        );
        assert!(
            SessionPolicy {
                absolute_lifetime_ms: None,
                ..policy
            }
            .expiry(i64::MAX - 1, i64::MAX - 1, None)
            .is_err()
        );
        Ok(())
    }
}

#[derive(Debug)]
pub struct NewSession {
    pub id: String,
    pub token: BearerSecret,
}

#[derive(Debug)]
pub struct SessionResult {
    pub user: UserSummary,
    pub token: BearerSecret,
    pub expires_at: i64,
    pub created_at: i64,
    pub absolute_expires_at: Option<i64>,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct SessionRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Binary)]
    pub current_hash: Vec<u8>,
    #[diesel(sql_type = Nullable<Binary>)]
    pub previous_hash: Option<Vec<u8>>,
    #[diesel(sql_type = Binary)]
    pub recovery_nonce: Vec<u8>,
    #[diesel(sql_type = Binary)]
    pub recovery_ciphertext: Vec<u8>,
    #[diesel(sql_type = BigInt)]
    pub created_at: i64,
    #[diesel(sql_type = BigInt)]
    pub rotated_at: i64,
    #[diesel(sql_type = BigInt)]
    pub expires_at: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub absolute_expires_at: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub previous_valid_until: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    pub revoked_at: Option<i64>,
}

impl std::fmt::Debug for SessionRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SessionRow([REDACTED])")
    }
}

impl SessionRow {
    fn current_digest(&self) -> Result<SecretDigest, StoreError> {
        Ok(SecretDigest::from_bytes(
            self.current_hash
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?,
        ))
    }

    fn is_previous(&self, token: &BearerSecret, now: i64) -> Result<bool, StoreError> {
        let Some(bytes) = &self.previous_hash else {
            return Ok(false);
        };
        let digest = SecretDigest::from_bytes(
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?,
        );
        Ok(digest.matches(token) && self.previous_valid_until.is_some_and(|end| now < end))
    }
}

macro_rules! insert_session {
    ($connection:expr, $dialect:expr, $cipher:expr, $user:expr, $new:expr, $now:expr, $policy:expr) => {{
        let new = $new;
        let user = $user;
        let now = $now;
        let (expires_at, cap) = $policy.expiry(now, now, None)?;
        let envelope = $cipher.protect_latest(&new.id, &new.token)?;
        diesel::sql_query($dialect.query(
            "INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,
             created_at,rotated_at,expires_at,absolute_expires_at)
             VALUES(?,?,?,?,?,?,?,?,?)",
        ))
        .bind::<Text, _>(&new.id)
        .bind::<Text, _>(&user.id)
        .bind::<Binary, _>(new.token.digest().as_bytes().as_slice())
        .bind::<Binary, _>(envelope.nonce().as_slice())
        .bind::<Binary, _>(envelope.ciphertext())
        .bind::<BigInt, _>(now)
        .bind::<BigInt, _>(now)
        .bind::<BigInt, _>(expires_at)
        .bind::<Nullable<BigInt>, _>(cap)
        .execute($connection)
        .await?;
        SessionResult {
            user,
            token: BearerSecret::parse(new.token.expose().to_owned())?,
            expires_at,
            created_at: now,
            absolute_expires_at: cap,
        }
    }};
}

pub(crate) use insert_session;

impl Store {
    /// Called only after verifying this exact detached credential snapshot outside DB locks.
    pub async fn accept_password_login(
        &self,
        verified: &VerifiedCredential,
        new: &NewSession,
        now: i64,
        policy: SessionPolicy,
    ) -> Result<SessionResult, StoreError> {
        operation!(self, |connection, dialect| {
            let verified = &verified.snapshot;
            let rows = diesel::sql_query(dialect.query(dialect.user_lock()))
                .bind::<Text, _>(&verified.user.id)
                .load::<UserRow>(connection)
                .await?;
            let [current] = rows.as_slice() else {
                return Err(StoreError::Unauthenticated);
            };
            if current.password_hash != verified.password.encoded()
                || current.email_key != verified.user.email.lookup_key()
            {
                return Err(StoreError::Unauthenticated);
            }
            let user = current.summary()?;
            Ok(insert_session!(
                connection,
                dialect,
                self.cipher,
                user,
                new,
                now,
                policy
            ))
        })
    }

    pub async fn renew_session(
        &self,
        presented: &BearerSecret,
        now: i64,
        policy: SessionPolicy,
    ) -> Result<SessionResult, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, &presented.digest(), now);
            let sql = match dialect {
                Dialect::Postgres => {
                    "SELECT * FROM auth_sessions WHERE user_id=? AND (current_hash=? OR previous_hash=?) FOR UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT * FROM auth_sessions WHERE user_id=? AND (current_hash=? OR previous_hash=?)"
                }
            };
            let rows = diesel::sql_query(dialect.query(sql))
                .bind::<Text, _>(&user.id)
                .bind::<Binary, _>(presented.digest().as_bytes().as_slice())
                .bind::<Binary, _>(presented.digest().as_bytes().as_slice())
                .load::<SessionRow>(connection)
                .await?;
            let [session] = rows.as_slice() else {
                return Err(StoreError::Unauthenticated);
            };
            if session.revoked_at.is_some() || now >= session.expires_at {
                return Err(StoreError::Unauthenticated);
            }
            let current = session.current_digest()?;
            if current.matches(presented) {
                let elapsed = now
                    .checked_sub(session.rotated_at)
                    .ok_or(StoreError::Integrity)?;
                let (expires_at, cap) =
                    policy.expiry(session.created_at, now, session.absolute_expires_at)?;
                if elapsed >= policy.rotation_ms {
                    let latest = BearerSecret::generate()?;
                    let envelope = self.cipher.protect_latest(&session.id, &latest)?;
                    let overlap_end = now
                        .checked_add(policy.overlap_ms)
                        .ok_or(StoreError::Integrity)?;
                    diesel::sql_query(dialect.query(
                        "UPDATE auth_sessions SET previous_hash=current_hash,previous_valid_until=?,
                         current_hash=?,recovery_nonce=?,recovery_ciphertext=?,rotated_at=?,
                         expires_at=?,absolute_expires_at=? WHERE id=?",
                    ))
                    .bind::<BigInt, _>(overlap_end)
                    .bind::<Binary, _>(latest.digest().as_bytes().as_slice())
                    .bind::<Binary, _>(envelope.nonce().as_slice())
                    .bind::<Binary, _>(envelope.ciphertext())
                    .bind::<BigInt, _>(now)
                    .bind::<BigInt, _>(expires_at)
                    .bind::<Nullable<BigInt>, _>(cap)
                    .bind::<Text, _>(&session.id)
                    .execute(connection)
                    .await?;
                    return Ok(SessionResult {
                        user,
                        token: latest,
                        expires_at,
                        created_at: session.created_at,
                        absolute_expires_at: cap,
                    });
                }
            } else if !session.is_previous(presented, now)? {
                return Err(StoreError::Unauthenticated);
            }
            let nonce = session
                .recovery_nonce
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?;
            let envelope =
                RecoveryEnvelope::from_parts(nonce, session.recovery_ciphertext.clone())?;
            let token = self
                .cipher
                .recover_latest(&session.id, &current, &envelope)?;
            Ok(SessionResult {
                user,
                token,
                expires_at: session.expires_at,
                created_at: session.created_at,
                absolute_expires_at: session.absolute_expires_at,
            })
        })
    }

    pub async fn logout_session(&self, token: &SecretDigest, now: i64) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, token, now);
            diesel::sql_query(dialect.query(
                "UPDATE auth_sessions SET revoked_at=? WHERE user_id=? AND (current_hash=? OR previous_hash=?) AND revoked_at IS NULL",
            ))
            .bind::<BigInt, _>(now)
            .bind::<Text, _>(&user.id)
            .bind::<Binary, _>(token.as_bytes().as_slice())
            .bind::<Binary, _>(token.as_bytes().as_slice())
            .execute(connection)
            .await?;
            Ok(())
        })
    }
}
