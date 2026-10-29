use chapchoo_crypto::{BearerSecret, PasswordHash, SecretDigest};
use chapchoo_domain::EmailAddress;
use diesel::sql_types::{BigInt, Binary, Bool, Nullable, Text};
use diesel_async::RunQueryDsl;

use crate::{
    NewSession, SessionPolicy, SessionResult, Store, StoreError, UserSummary,
    models::{UserRow, authenticate},
    sessions::insert_session,
    store::{Dialect, operation},
};

#[derive(Debug)]
pub struct NewAccount {
    pub user_id: String,
    pub email: EmailAddress,
    pub password: PasswordHash,
    pub personal_team_id: String,
    pub personal_handle: String,
    pub session: NewSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantKind {
    Signup,
    PasswordReset,
}

impl GrantKind {
    fn sql(self) -> &'static str {
        match self {
            Self::Signup => "'SIGNUP'",
            Self::PasswordReset => "'PASSWORD_RESET'",
        }
    }
}

#[derive(Debug)]
pub struct GrantDraft {
    pub id: String,
    pub proof: SecretDigest,
    pub expires_at: i64,
}

#[derive(Debug)]
pub struct GrantSummary {
    pub id: String,
    pub kind: GrantKind,
    pub email: EmailAddress,
    pub issued_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
    pub revoked_at: Option<i64>,
}

#[derive(Debug)]
pub struct ProofPreview {
    pub email_hint: String,
    pub expires_at: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct Setup {
    #[diesel(sql_type = Nullable<BigInt>)]
    consumed_at: Option<i64>,
}

#[derive(diesel::QueryableByName)]
struct GrantRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    email_key: String,
    #[diesel(sql_type = Text)]
    delivery_email: String,
    #[diesel(sql_type = Nullable<Text>)]
    user_id: Option<String>,
    #[diesel(sql_type = BigInt)]
    issued_at: i64,
    #[diesel(sql_type = BigInt)]
    expires_at: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    consumed_at: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    revoked_at: Option<i64>,
}

impl std::fmt::Debug for GrantRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("GrantRow([REDACTED])")
    }
}

impl GrantRow {
    fn summary(&self) -> Result<GrantSummary, StoreError> {
        let kind = match self.kind.as_str() {
            "SIGNUP" => GrantKind::Signup,
            "PASSWORD_RESET" => GrantKind::PasswordReset,
            _ => return Err(StoreError::Integrity),
        };
        let email = EmailAddress::parse(&self.delivery_email).map_err(|_| StoreError::Integrity)?;
        if email.lookup_key() != self.email_key {
            return Err(StoreError::Integrity);
        }
        Ok(GrantSummary {
            id: self.id.clone(),
            kind,
            email,
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            consumed_at: self.consumed_at,
            revoked_at: self.revoked_at,
        })
    }

    fn usable(&self, now: i64) -> bool {
        now < self.expires_at && self.consumed_at.is_none() && self.revoked_at.is_none()
    }
}

macro_rules! new_account {
    ($connection:expr, $dialect:expr, $draft:expr, $admin:expr, $now:expr) => {{
        let draft = $draft;
        if draft.user_id.is_empty()
            || draft.personal_team_id.is_empty()
            || draft.personal_handle.is_empty()
        {
            return Err(StoreError::Integrity);
        }
        diesel::sql_query($dialect.query(
            "INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at)
             VALUES(?,?,?,?,?,?)",
        ))
        .bind::<Text, _>(&draft.user_id)
        .bind::<Text, _>(draft.email.lookup_key())
        .bind::<Text, _>(draft.email.delivery())
        .bind::<Text, _>(draft.password.encoded())
        .bind::<Bool, _>($admin)
        .bind::<BigInt, _>($now)
        .execute($connection)
        .await
        .map_err(|error| {
            StoreError::unique_field(
                error,
                "users_email_key_key",
                "users.email_key",
                StoreError::EmailRegistered,
            )
        })?;
        diesel::sql_query($dialect.query(
            "INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at)
             VALUES(?,'PERSONAL','Personal',?,?,?)",
        ))
        .bind::<Text, _>(&draft.personal_team_id)
        .bind::<Text, _>(&draft.personal_handle)
        .bind::<Text, _>(&draft.user_id)
        .bind::<BigInt, _>($now)
        .execute($connection)
        .await?;
        diesel::sql_query(
            $dialect.query("INSERT INTO memberships(team_id,user_id,role) VALUES(?,?,'MEMBER')"),
        )
        .bind::<Text, _>(&draft.personal_team_id)
        .bind::<Text, _>(&draft.user_id)
        .execute($connection)
        .await?;
        diesel::sql_query(
            $dialect.query("INSERT INTO personal_contexts(user_id,team_id) VALUES(?,?)"),
        )
        .bind::<Text, _>(&draft.user_id)
        .bind::<Text, _>(&draft.personal_team_id)
        .execute($connection)
        .await?;
        UserSummary {
            id: draft.user_id.clone(),
            email: draft.email.clone(),
            is_admin: $admin,
            created_at: $now,
        }
    }};
}

macro_rules! get_grant {
    ($connection:expr, $dialect:expr, $proof:expr, $lock:expr) => {{
        let base = "SELECT id,CAST(kind AS TEXT) AS kind,email_key,delivery_email,user_id,
                    issued_at,expires_at,consumed_at,revoked_at FROM grants WHERE secret_hash=?";
        let sql = if $lock && matches!($dialect, Dialect::Postgres) {
            format!("{base} FOR UPDATE")
        } else {
            base.to_owned()
        };
        diesel::sql_query($dialect.query(&sql))
            .bind::<Binary, _>($proof.as_bytes().as_slice())
            .load::<GrantRow>($connection)
            .await?
    }};
}

impl Store {
    /// The application service must first establish configured mail delivery.
    /// Only its post-commit mail adapter receives the raw proof; the public RPC
    /// returns delivery status, not a usable signup credential.
    pub async fn issue_email_signup_grant(
        &self,
        email: &EmailAddress,
        draft: &GrantDraft,
        now: i64,
    ) -> Result<(), StoreError> {
        if draft.expires_at <= now {
            return Err(StoreError::InvalidProof);
        }
        operation!(self, |connection, dialect| {
            diesel::sql_query(dialect.query(
                "INSERT INTO grants(id,kind,secret_hash,email_key,delivery_email,issued_by,issued_at,expires_at)
                 VALUES(?,'SIGNUP',?,?,?,NULL,?,?)",
            ))
            .bind::<Text, _>(&draft.id)
            .bind::<Binary, _>(draft.proof.as_bytes().as_slice())
            .bind::<Text, _>(email.lookup_key())
            .bind::<Text, _>(email.delivery())
            .bind::<BigInt, _>(now)
            .bind::<BigInt, _>(draft.expires_at)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    pub async fn setup_available(&self) -> Result<bool, StoreError> {
        operation!(self, |connection, dialect| {
            let count = diesel::sql_query("SELECT count(*) AS n FROM users")
                .get_result::<Count>(connection)
                .await?
                .n;
            let setup = diesel::sql_query(
                dialect.query("SELECT consumed_at FROM initial_setup WHERE singleton=1"),
            )
            .load::<Setup>(connection)
            .await?;
            Ok(count == 0
                && setup
                    .as_slice()
                    .first()
                    .is_some_and(|row| row.consumed_at.is_none()))
        })
    }

    pub async fn initialize_admin(
        &self,
        configured_proof: &SecretDigest,
        presented: &BearerSecret,
        draft: &NewAccount,
        now: i64,
        policy: SessionPolicy,
    ) -> Result<SessionResult, StoreError> {
        if !configured_proof.matches(presented) {
            return Err(StoreError::InvalidProof);
        }
        operation!(self, |connection, dialect| {
            let sql = match dialect {
                Dialect::Postgres => {
                    "SELECT consumed_at FROM initial_setup WHERE singleton=1 FOR UPDATE"
                }
                Dialect::Sqlite => "SELECT consumed_at FROM initial_setup WHERE singleton=1",
            };
            let setup = diesel::sql_query(sql).load::<Setup>(connection).await?;
            if !setup
                .as_slice()
                .first()
                .is_some_and(|row| row.consumed_at.is_none())
            {
                return Err(StoreError::InvalidTransition);
            }
            let count = diesel::sql_query("SELECT count(*) AS n FROM users")
                .get_result::<Count>(connection)
                .await?
                .n;
            if count != 0 {
                return Err(StoreError::InvalidTransition);
            }
            let user = new_account!(connection, dialect, draft, true, now);
            let session = insert_session!(
                connection,
                dialect,
                self.cipher,
                user,
                &draft.session,
                now,
                policy
            );
            diesel::sql_query(
                dialect.query("UPDATE initial_setup SET consumed_at=? WHERE singleton=1"),
            )
            .bind::<BigInt, _>(now)
            .execute(connection)
            .await?;
            Ok(session)
        })
    }

    pub async fn issue_signup_grant(
        &self,
        actor: &SecretDigest,
        email: &EmailAddress,
        draft: &GrantDraft,
        now: i64,
    ) -> Result<GrantSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            if !user.is_admin {
                return Err(StoreError::Forbidden);
            }
            if draft.expires_at <= now {
                return Err(StoreError::InvalidProof);
            }
            diesel::sql_query(dialect.query(
                "INSERT INTO grants(id,kind,secret_hash,email_key,delivery_email,issued_by,issued_at,expires_at)
                 VALUES(?,'SIGNUP',?,?,?,?,?,?)",
            ))
            .bind::<Text,_>(&draft.id)
            .bind::<Binary,_>(draft.proof.as_bytes().as_slice())
            .bind::<Text,_>(email.lookup_key())
            .bind::<Text,_>(email.delivery())
            .bind::<Text,_>(&user.id)
            .bind::<BigInt,_>(now)
            .bind::<BigInt,_>(draft.expires_at)
            .execute(connection).await?;
            Ok(GrantSummary {
                id: draft.id.clone(),
                kind: GrantKind::Signup,
                email: email.clone(),
                issued_at: now,
                expires_at: draft.expires_at,
                consumed_at: None,
                revoked_at: None,
            })
        })
    }

    pub async fn redeem_signup(
        &self,
        proof: &SecretDigest,
        draft: &NewAccount,
        now: i64,
        policy: SessionPolicy,
    ) -> Result<SessionResult, StoreError> {
        operation!(self, |connection, dialect| {
            let rows = get_grant!(connection, dialect, proof, true);
            let [grant] = rows.as_slice() else {
                return Err(StoreError::InvalidProof);
            };
            if grant.kind != "SIGNUP" || !grant.usable(now) {
                return Err(StoreError::InvalidProof);
            }
            if grant.email_key != draft.email.lookup_key() {
                return Err(StoreError::EmailMismatch);
            }
            let user = new_account!(connection, dialect, draft, false, now);
            let changed = diesel::sql_query(dialect.query(
                "UPDATE grants SET consumed_at=? WHERE id=? AND consumed_at IS NULL AND revoked_at IS NULL AND expires_at>?",
            )).bind::<BigInt,_>(now).bind::<Text,_>(&grant.id).bind::<BigInt,_>(now).execute(connection).await?;
            if changed != 1 {
                return Err(StoreError::InvalidProof);
            }
            Ok(insert_session!(
                connection,
                dialect,
                self.cipher,
                user,
                &draft.session,
                now,
                policy
            ))
        })
    }

    pub async fn preview_grant(
        &self,
        proof: &SecretDigest,
        kind: GrantKind,
        now: i64,
    ) -> Result<Option<ProofPreview>, StoreError> {
        operation!(self, |connection, dialect| {
            let rows = get_grant!(connection, dialect, proof, false);
            let [grant] = rows.as_slice() else {
                return Ok(None);
            };
            if !grant.usable(now) || grant.summary()?.kind != kind {
                return Ok(None);
            }
            let Some((local, domain)) = grant.delivery_email.rsplit_once('@') else {
                return Err(StoreError::Integrity);
            };
            let initial = local.chars().next().ok_or(StoreError::Integrity)?;
            Ok(Some(ProofPreview {
                email_hint: format!("{initial}***@{domain}"),
                expires_at: grant.expires_at,
            }))
        })
    }

    pub async fn grant_administrator(
        &self,
        existing_email: &EmailAddress,
    ) -> Result<UserSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let users = diesel::sql_query(
                dialect.query("UPDATE users SET is_admin=TRUE WHERE email_key=? RETURNING *"),
            )
            .bind::<Text, _>(existing_email.lookup_key())
            .load::<UserRow>(connection)
            .await?;
            let [user] = users.as_slice() else {
                return Err(StoreError::NotFound);
            };
            user.summary()
        })
    }

    pub async fn issue_password_reset(
        &self,
        actor: &SecretDigest,
        existing_email: &EmailAddress,
        draft: &GrantDraft,
        now: i64,
    ) -> Result<GrantSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let actor = authenticate!(connection, dialect, actor, now);
            if !actor.is_admin {
                return Err(StoreError::Forbidden);
            }
            if draft.expires_at <= now {
                return Err(StoreError::InvalidProof);
            }
            let users = diesel::sql_query(dialect.query("SELECT * FROM users WHERE email_key=?"))
                .bind::<Text, _>(existing_email.lookup_key())
                .load::<UserRow>(connection)
                .await?;
            let [user] = users.as_slice() else {
                return Err(StoreError::NotFound);
            };
            diesel::sql_query(dialect.query(
                "INSERT INTO grants(id,kind,secret_hash,email_key,delivery_email,user_id,issued_by,issued_at,expires_at)
                 VALUES(?,'PASSWORD_RESET',?,?,?,?,?,?,?)",
            ))
            .bind::<Text,_>(&draft.id).bind::<Binary,_>(draft.proof.as_bytes().as_slice())
            .bind::<Text,_>(&user.email_key).bind::<Text,_>(&user.delivery_email)
            .bind::<Text,_>(&user.id).bind::<Text,_>(&actor.id).bind::<BigInt,_>(now)
            .bind::<BigInt,_>(draft.expires_at).execute(connection).await?;
            Ok(GrantSummary {
                id: draft.id.clone(),
                kind: GrantKind::PasswordReset,
                email: user.summary()?.email,
                issued_at: now,
                expires_at: draft.expires_at,
                consumed_at: None,
                revoked_at: None,
            })
        })
    }

    pub async fn redeem_password_reset(
        &self,
        proof: &SecretDigest,
        password: &PasswordHash,
        now: i64,
    ) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            // Discover the target without locking Grant first; all credential
            // transitions acquire User before any existing Session/Grant row.
            let rows = get_grant!(connection, dialect, proof, false);
            let [grant] = rows.as_slice() else {
                return Err(StoreError::InvalidProof);
            };
            let target = grant.user_id.as_ref().ok_or(StoreError::InvalidProof)?;
            let users = diesel::sql_query(dialect.query(dialect.user_lock()))
                .bind::<Text, _>(target)
                .load::<UserRow>(connection)
                .await?;
            if users.len() != 1 {
                return Err(StoreError::InvalidProof);
            }
            let rows = get_grant!(connection, dialect, proof, true);
            let [grant] = rows.as_slice() else {
                return Err(StoreError::InvalidProof);
            };
            if grant.kind != "PASSWORD_RESET"
                || !grant.usable(now)
                || grant.user_id.as_ref() != Some(target)
            {
                return Err(StoreError::InvalidProof);
            }
            let count = diesel::sql_query(dialect.query(
                "UPDATE grants SET consumed_at=? WHERE id=? AND consumed_at IS NULL AND revoked_at IS NULL AND expires_at>?",
            )).bind::<BigInt,_>(now).bind::<Text,_>(&grant.id).bind::<BigInt,_>(now).execute(connection).await?;
            if count != 1 {
                return Err(StoreError::InvalidProof);
            }
            diesel::sql_query(dialect.query("UPDATE users SET password_hash=? WHERE id=?"))
                .bind::<Text, _>(password.encoded())
                .bind::<Text, _>(target)
                .execute(connection)
                .await?;
            diesel::sql_query(dialect.query(
                "UPDATE auth_sessions SET revoked_at=? WHERE user_id=? AND revoked_at IS NULL",
            ))
            .bind::<BigInt, _>(now)
            .bind::<Text, _>(target)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    pub async fn list_grants(
        &self,
        actor: &SecretDigest,
        kind: GrantKind,
        offset: i64,
        limit: i64,
        now: i64,
    ) -> Result<(Vec<GrantSummary>, i64), StoreError> {
        if offset < 0 || !(1..=1000).contains(&limit) {
            return Err(StoreError::InvalidTransition);
        }
        operation!(self, |connection, dialect| {
            let actor = authenticate!(connection, dialect, actor, now);
            if !actor.is_admin {
                return Err(StoreError::Forbidden);
            }
            let count = diesel::sql_query(format!(
                "SELECT count(*) AS n FROM grants WHERE kind={}",
                kind.sql()
            ))
            .get_result::<Count>(connection)
            .await?
            .n;
            let sql = format!(
                "SELECT id,CAST(kind AS TEXT) AS kind,email_key,delivery_email,user_id,issued_at,expires_at,consumed_at,revoked_at FROM grants WHERE kind={} ORDER BY issued_at DESC,id DESC LIMIT ? OFFSET ?",
                kind.sql()
            );
            let rows = diesel::sql_query(dialect.query(&sql))
                .bind::<BigInt, _>(limit)
                .bind::<BigInt, _>(offset)
                .load::<GrantRow>(connection)
                .await?;
            Ok((
                rows.iter()
                    .map(GrantRow::summary)
                    .collect::<Result<_, _>>()?,
                count,
            ))
        })
    }

    pub async fn revoke_grant(
        &self,
        actor: &SecretDigest,
        id: &str,
        kind: GrantKind,
        now: i64,
    ) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            let actor = authenticate!(connection, dialect, actor, now);
            if !actor.is_admin {
                return Err(StoreError::Forbidden);
            }
            let count = diesel::sql_query(dialect.query(&format!(
                "UPDATE grants SET revoked_at=? WHERE id=? AND kind={} AND revoked_at IS NULL",
                kind.sql()
            )))
            .bind::<BigInt, _>(now)
            .bind::<Text, _>(id)
            .execute(connection)
            .await?;
            if count == 0 {
                return Err(StoreError::NotFound);
            }
            Ok(())
        })
    }
}
