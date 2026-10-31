use chapchoo_crypto::{PasswordHash, PasswordHasher};
use chapchoo_domain::{DisplaySettings, EmailAddress};
use diesel::sql_types::{BigInt, Bool, Text};
use std::fmt;

use crate::StoreError;

#[derive(Debug)]
pub struct UserSummary {
    pub id: String,
    pub email: EmailAddress,
    pub is_admin: bool,
    pub created_at: i64,
    pub display: DisplaySettings,
}

#[derive(Debug)]
pub struct CredentialSnapshot {
    pub user: UserSummary,
    pub password: PasswordHash,
}

/// Constructed only by a successful CPU verification, before DB acceptance.
#[derive(Debug)]
pub struct VerifiedCredential {
    pub(crate) snapshot: CredentialSnapshot,
}

impl CredentialSnapshot {
    pub fn verify(
        self,
        input: &str,
        hasher: &PasswordHasher,
    ) -> Result<VerifiedCredential, StoreError> {
        if !hasher.verify(input, &self.password)? {
            return Err(StoreError::Unauthenticated);
        }
        Ok(VerifiedCredential { snapshot: self })
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct UserRow {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub email_key: String,
    #[diesel(sql_type = Text)]
    pub delivery_email: String,
    #[diesel(sql_type = Text)]
    pub password_hash: String,
    #[diesel(sql_type = Bool)]
    pub is_admin: bool,
    #[diesel(sql_type = BigInt)]
    pub created_at: i64,
    #[diesel(sql_type = Text)]
    pub language: String,
    #[diesel(sql_type = Text)]
    pub time_zone: String,
    #[diesel(sql_type = Text)]
    pub region: String,
}

impl fmt::Debug for UserRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UserRow([REDACTED])")
    }
}

impl UserRow {
    pub(crate) fn summary(&self) -> Result<UserSummary, StoreError> {
        let email = EmailAddress::parse(&self.delivery_email).map_err(|_| StoreError::Integrity)?;
        if email.lookup_key() != self.email_key {
            return Err(StoreError::Integrity);
        }
        Ok(UserSummary {
            id: self.id.clone(),
            email,
            is_admin: self.is_admin,
            created_at: self.created_at,
            display: DisplaySettings::parse(&self.language, &self.time_zone, &self.region)
                .map_err(|_| StoreError::Integrity)?,
        })
    }

    pub(crate) fn snapshot(self) -> Result<CredentialSnapshot, StoreError> {
        let user = self.summary()?;
        Ok(CredentialSnapshot {
            user,
            password: PasswordHash::parse(self.password_hash)?,
        })
    }
}

#[derive(Debug, diesel::QueryableByName)]
pub(crate) struct UserId {
    #[diesel(sql_type = Text)]
    pub user_id: String,
}

// Authentication is revalidated after taking the User lock. This keeps the same
// User-first lock order as reset/renew/logout rather than locking a Session first.
macro_rules! authenticate {
    ($connection:expr, $dialect:expr, $digest:expr, $now:expr) => {{
        use diesel_async::RunQueryDsl;
        let digest = $digest;
        let now = $now;
        let query = $dialect.query(
            "SELECT user_id FROM auth_sessions
             WHERE revoked_at IS NULL AND expires_at>? AND
             (current_hash=? OR (previous_hash=? AND previous_valid_until>?))",
        );
        let identities = diesel::sql_query(query)
            .bind::<diesel::sql_types::BigInt, _>(now)
            .bind::<diesel::sql_types::Binary, _>(digest.as_bytes().as_slice())
            .bind::<diesel::sql_types::Binary, _>(digest.as_bytes().as_slice())
            .bind::<diesel::sql_types::BigInt, _>(now)
            .load::<$crate::models::UserId>($connection)
            .await?;
        let [identity] = identities.as_slice() else {
            return Err($crate::StoreError::Unauthenticated);
        };
        let users = diesel::sql_query($dialect.query($dialect.user_lock()))
            .bind::<diesel::sql_types::Text, _>(&identity.user_id)
            .load::<$crate::models::UserRow>($connection)
            .await?;
        let [user] = users.as_slice() else {
            return Err($crate::StoreError::Unauthenticated);
        };
        // Taking the User lock may have waited for reset/revocation to commit.
        let current = diesel::sql_query($dialect.query(
            "SELECT user_id FROM auth_sessions
             WHERE user_id=? AND revoked_at IS NULL AND expires_at>? AND
             (current_hash=? OR (previous_hash=? AND previous_valid_until>?))",
        ))
        .bind::<diesel::sql_types::Text, _>(&user.id)
        .bind::<diesel::sql_types::BigInt, _>(now)
        .bind::<diesel::sql_types::Binary, _>(digest.as_bytes().as_slice())
        .bind::<diesel::sql_types::Binary, _>(digest.as_bytes().as_slice())
        .bind::<diesel::sql_types::BigInt, _>(now)
        .load::<$crate::models::UserId>($connection)
        .await?;
        if current.len() != 1 {
            return Err($crate::StoreError::Unauthenticated);
        }
        user.summary()?
    }};
}

pub(crate) use authenticate;
