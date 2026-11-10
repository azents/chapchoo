//! Exact encrypted source admission; alias changes do not create another refresh grant.
use super::*;
use crate::{Store, models::authenticate, store::operation};
use chapchoo_crypto::{ExternalSecret, ExternalSecretEnvelope};

#[derive(Clone, PartialEq, Eq)]
pub struct AuthenticationCredentialSource([u8; 24]);
impl AuthenticationCredentialSource {
    /// An opaque equality identity, not a revision counter or authorization.
    pub fn from_bytes(bytes: [u8; 24]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; 24] {
        &self.0
    }
}
pub struct AuthenticationCredentialMaterial {
    pub authentication_id: String,
    pub source: AuthenticationCredentialSource,
    pub account_id: String,
    pub expires_at: Option<i64>,
    pub secret: ExternalSecret,
}
#[derive(Clone)]
pub struct AuthenticationRefreshStep {
    pub(super) id: EntityId,
    pub(super) authentication: EntityId,
    pub(super) owner: Owner,
    pub(super) source: AuthenticationCredentialSource,
    pub(super) account_id: String,
}
impl AuthenticationRefreshStep {
    pub fn id(&self) -> String {
        self.id.to_dto(EntityKind::AuthenticationAttempt)
    }
    pub fn authentication_id(&self) -> String {
        self.authentication.to_dto(EntityKind::Authentication)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticationRefreshState {
    Possible,
    NotDispatched,
    Rejected,
    Uncertain,
    Updated,
}
impl AuthenticationRefreshState {
    pub(super) fn parse(value: &str) -> Result<Self, StoreError> {
        Ok(match value {
            "POSSIBLE" => Self::Possible,
            "NOT_DISPATCHED" => Self::NotDispatched,
            "REJECTED" => Self::Rejected,
            "UNCERTAIN" => Self::Uncertain,
            "UPDATED" => Self::Updated,
            _ => return Err(StoreError::Integrity),
        })
    }
    fn stored(self) -> &'static str {
        match self {
            Self::Possible => "POSSIBLE",
            Self::NotDispatched => "NOT_DISPATCHED",
            Self::Rejected => "REJECTED",
            Self::Uncertain => "UNCERTAIN",
            Self::Updated => "UPDATED",
        }
    }
}
pub enum AuthenticationRefreshAdmission {
    Current(AuthenticationCredentialMaterial),
    Granted {
        step: AuthenticationRefreshStep,
        credentials: AuthenticationCredentialMaterial,
    },
    Existing {
        attempt_id: String,
        state: AuthenticationRefreshState,
    },
}
macro_rules! safe_debug {($($name:ty),+)=>{$(impl fmt::Debug for $name {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($name),"([REDACTED])"))}
})+};}
safe_debug!(
    AuthenticationCredentialSource,
    AuthenticationCredentialMaterial,
    AuthenticationRefreshStep,
    AuthenticationRefreshAdmission
);
#[derive(diesel::QueryableByName)]
pub(super) struct SecretRow {
    #[diesel(sql_type=Binary)]
    pub(super) nonce: Vec<u8>,
    #[diesel(sql_type=Binary)]
    pub(super) ciphertext: Vec<u8>,
}
#[derive(diesel::QueryableByName)]
pub(super) struct RefreshRow {
    #[diesel(sql_type=IdentitySql)]
    pub(super) id: EntityId,
    #[diesel(sql_type=IdentitySql)]
    pub(super) authentication_id: EntityId,
    #[diesel(sql_type=Binary)]
    pub(super) source_nonce: Vec<u8>,
    #[diesel(sql_type=Text)]
    pub(super) account_id: String,
    #[diesel(sql_type=Text)]
    pub(super) state: String,
}
macro_rules! secret {
    ($connection:expr,$dialect:expr,$id:expr) => {{
        diesel::sql_query(
            $dialect.query("SELECT nonce,ciphertext FROM external_authentications WHERE id=?"),
        )
        .bind::<IdentitySql, _>($id)
        .get_result::<SecretRow>($connection)
        .await?
    }};
}
pub(super) use secret as refresh_secret;
pub(super) fn refresh_source(
    secret: &SecretRow,
) -> Result<AuthenticationCredentialSource, StoreError> {
    Ok(AuthenticationCredentialSource(
        secret
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| StoreError::Integrity)?,
    ))
}
// Callers supply an already locked record under their own current authority.
// These database-only operations share source/attempt semantics across consumers.
macro_rules! begin_record_refresh {
    ($store:expr,$connection:expr,$dialect:expr,$row:expr,$now:expr) => {{
        use $crate::external_auth::refresh::{SecretRow, RefreshRow, refresh_secret, refresh_source};
        let row = $row;
        let now = $now;
        let renewal_after = now.checked_add(60_000).ok_or(StoreError::InvalidInput)?;
        if row.kind != "CHATGPT" { return Err(StoreError::InvalidTransition); }
        let account = row.account_id.as_ref().filter(|v| !v.is_empty() && v.len() <= 4096).ok_or(StoreError::InvalidTransition)?;
        let id = row.id;
        let authentication = id.to_dto(EntityKind::Authentication);
        let secret = refresh_secret!($connection,$dialect,id);
        let source = refresh_source(&secret)?;
        let previous = diesel::sql_query($dialect.query("SELECT * FROM authentication_refreshes WHERE authentication_id=? AND source_nonce=?"))
            .bind::<IdentitySql,_>(id).bind::<Binary,_>(source.as_bytes().as_slice())
            .get_result::<RefreshRow>($connection).await.optional()?;
        if let Some(previous) = previous {
            return Ok(AuthenticationRefreshAdmission::Existing { attempt_id: previous.id.to_dto(EntityKind::AuthenticationAttempt), state: AuthenticationRefreshState::parse(&previous.state)? });
        }
        let owner = row.owner();
        let material = AuthenticationCredentialMaterial {
            authentication_id: authentication.clone(), source: source.clone(), account_id: account.clone(), expires_at: row.expires_at,
            secret: $store.cipher.recover_external(
                &super::context(&owner.key()?, &authentication, ExternalSecretKind::ChatGpt)?,
                &ExternalSecretEnvelope::from_parts(*source.as_bytes(), secret.ciphertext)?,
            )?,
        };
        // Missing expiry is not a guessed expiration or permission to rotate.
        if row.expires_at.is_none_or(|expiry| expiry > renewal_after) {
            return Ok(AuthenticationRefreshAdmission::Current(material));
        }
        let attempt = EntityId::generate(now)?;
        diesel::sql_query($dialect.query("INSERT INTO authentication_refreshes(id,authentication_id,source_nonce,account_id,state,created_at,updated_at) VALUES(?,?,?,?,'POSSIBLE',?,?)"))
            .bind::<IdentitySql,_>(attempt).bind::<IdentitySql,_>(id).bind::<Binary,_>(source.as_bytes().as_slice())
            .bind::<Text,_>(account).bind::<BigInt,_>(now).bind::<BigInt,_>(now).execute($connection).await?;
        Ok(AuthenticationRefreshAdmission::Granted {
            step: AuthenticationRefreshStep { id: attempt, authentication: id, owner, source, account_id: account.clone() },
            credentials: material,
        })
    }};
}
pub(super) use begin_record_refresh;
macro_rules! complete_record_refresh {
    ($connection:expr,$dialect:expr,$row:expr,$step:expr,$protected:expr,$tokens:expr,$now:expr) => {{
        use $crate::external_auth::refresh::{SecretRow, RefreshRow, refresh_secret, refresh_source};
        let row = $row;
        let step = $step;
        let protected = $protected;
        let tokens = $tokens;
        let now = $now;
        if row.id != step.authentication || row.owner() != step.owner { return Err(StoreError::NotFound); }
        let secret = refresh_secret!($connection,$dialect,step.authentication);
        if refresh_source(&secret)? != step.source || row.account_id.as_deref() != Some(step.account_id.as_str()) {
            return Err(StoreError::Conflict);
        }
        let attempt = diesel::sql_query($dialect.query("SELECT * FROM authentication_refreshes WHERE id=?"))
            .bind::<IdentitySql,_>(step.id).get_result::<RefreshRow>($connection).await.optional()?.ok_or(StoreError::NotFound)?;
        if attempt.authentication_id != step.authentication || attempt.source_nonce.as_slice() != step.source.as_bytes()
            || attempt.account_id != step.account_id || attempt.state != "POSSIBLE" {
            return Err(StoreError::Conflict);
        }
        // Preserve the owner's mutation identity and alias.
        diesel::sql_query($dialect.query("UPDATE external_authentications SET nonce=?,ciphertext=?,email=?,plan_type=?,expires_at=?,updated_at=? WHERE id=?"))
            .bind::<Binary,_>(protected.nonce().as_slice()).bind::<Binary,_>(protected.ciphertext())
            .bind::<Nullable<Text>,_>(tokens.email.as_deref()).bind::<Nullable<Text>,_>(tokens.plan_type.as_deref())
            .bind::<Nullable<BigInt>,_>(tokens.expires_at).bind::<BigInt,_>(now).bind::<IdentitySql,_>(step.authentication).execute($connection).await?;
        diesel::sql_query($dialect.query("UPDATE authentication_refreshes SET state='UPDATED',updated_at=? WHERE id=?"))
            .bind::<BigInt,_>(now).bind::<IdentitySql,_>(step.id).execute($connection).await?;
        Ok(AuthenticationCredentialSource::from_bytes(*protected.nonce()))
    }};
}
pub(super) use complete_record_refresh;
impl Store {
    /// Coalesce by the exact protected credential source, not mutable alias/request metadata.
    /// Existing pending/terminal source facts never grant another external exchange.
    pub async fn begin_external_authentication_refresh(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        authentication: &str,
        now: i64,
    ) -> Result<AuthenticationRefreshAdmission, StoreError> {
        let id = EntityId::from_dto(EntityKind::Authentication, authentication)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, scope, &user.id, false);
            let row = lock_record!(connection, dialect, id, owner);
            begin_record_refresh!(self, connection, dialect, row, now)
        })
    }
    pub(super) fn protect_refresh_result(
        &self,
        step: &AuthenticationRefreshStep,
        tokens: &DeviceAuthenticationTokens,
    ) -> Result<ExternalSecretEnvelope, StoreError> {
        if tokens.account_id.as_deref() != Some(step.account_id.as_str())
            || tokens
                .expires_at
                .is_some_and(|expiry| !(0..=MAX_AUTHENTICATION_EXPIRY_MILLIS).contains(&expiry))
        {
            return Err(StoreError::InvalidInput);
        }
        for value in [&tokens.account_id, &tokens.email, &tokens.plan_type]
            .into_iter()
            .flatten()
        {
            if value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control) {
                return Err(StoreError::InvalidInput);
            }
        }
        Ok(self.cipher.protect_external(
            &context(
                &step.owner.key()?,
                &step.authentication_id(),
                ExternalSecretKind::ChatGpt,
            )?,
            &tokens.secret,
        )?)
    }
    pub async fn complete_external_authentication_refresh(
        &self,
        actor: &SecretDigest,
        scope: &AuthenticationScope,
        step: &AuthenticationRefreshStep,
        tokens: &DeviceAuthenticationTokens,
        now: i64,
    ) -> Result<AuthenticationCredentialSource, StoreError> {
        let protected = self.protect_refresh_result(step, tokens)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let owner = admit_scope!(connection, dialect, scope, &user.id, false);
            if owner != step.owner {
                return Err(StoreError::NotFound);
            }
            let row = lock_record!(connection, dialect, step.authentication, owner);
            complete_record_refresh!(connection, dialect, row, step, &protected, tokens, now)
        })
    }
    /// Narrow cleanup cannot publish material or restore authority. Only the
    /// admitted attempt can move from possible to its final non-success fact.
    pub async fn fail_external_authentication_refresh(
        &self,
        step: &AuthenticationRefreshStep,
        failure: AuthenticationRefreshState,
        now: i64,
    ) -> Result<(), StoreError> {
        if !matches!(
            failure,
            AuthenticationRefreshState::NotDispatched
                | AuthenticationRefreshState::Rejected
                | AuthenticationRefreshState::Uncertain
        ) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let query = match dialect {
                Dialect::Postgres => format!("{AUTH_COLUMNS} WHERE id=? FOR UPDATE"),
                Dialect::Sqlite => format!("{AUTH_COLUMNS} WHERE id=?"),
            };
            let Some(record) = diesel::sql_query(dialect.query(&query))
                .bind::<IdentitySql, _>(step.authentication)
                .get_result::<AuthenticationRow>(connection)
                .await
                .optional()?
            else {
                return Ok(());
            };
            if record.owner() != step.owner {
                return Err(StoreError::Integrity);
            }
            diesel::sql_query(dialect.query("UPDATE authentication_refreshes SET state=?,updated_at=? WHERE id=? AND authentication_id=? AND source_nonce=? AND account_id=? AND state='POSSIBLE'"))
                .bind::<Text,_>(failure.stored()).bind::<BigInt,_>(now).bind::<IdentitySql,_>(step.id)
                .bind::<IdentitySql,_>(step.authentication).bind::<Binary,_>(step.source.as_bytes().as_slice()).bind::<Text,_>(&step.account_id).execute(connection).await?;
            Ok(())
        })
    }
}
