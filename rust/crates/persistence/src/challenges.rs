use chapchoo_crypto::{BearerSecret, CodeVerifier, EmailCode};
use chapchoo_domain::EmailAddress;
use diesel::sql_types::{BigInt, Binary, Nullable, Text};
use diesel_async::RunQueryDsl;

use crate::{
    NewSession, SessionPolicy, SessionResult, Store, StoreError,
    models::UserRow,
    sessions::insert_session,
    store::{Dialect, operation},
};

#[derive(Debug)]
pub struct NewChallenge {
    pub id: String,
    pub email: EmailAddress,
    pub binding: BearerSecret,
    pub code: EmailCode,
    pub expires_at: i64,
}

#[derive(Debug)]
pub enum EmailLoginOutcome {
    Authenticated(Box<SessionResult>),
    RegistrationRequired,
}

#[derive(diesel::QueryableByName)]
struct ChallengeRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Binary)]
    code_verifier: Vec<u8>,
    #[diesel(sql_type = BigInt)]
    expires_at: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    consumed_at: Option<i64>,
}

impl std::fmt::Debug for ChallengeRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ChallengeRow([REDACTED])")
    }
}

impl Store {
    pub async fn create_login_challenge(
        &self,
        draft: &NewChallenge,
        now: i64,
    ) -> Result<(), StoreError> {
        if draft.expires_at <= now {
            return Err(StoreError::InvalidProof);
        }
        let verifier = draft
            .code
            .verifier(&self.cipher, &draft.email, &draft.binding)?;
        operation!(self, |connection, dialect| {
            diesel::sql_query(dialect.query(
                "DELETE FROM login_challenges WHERE email_key=? AND (expires_at<=? OR consumed_at IS NOT NULL)",
            ))
            .bind::<Text,_>(draft.email.lookup_key()).bind::<BigInt,_>(now)
            .execute(connection).await?;
            diesel::sql_query(dialect.query(
                "INSERT INTO login_challenges(id,email_key,binding_hash,code_verifier,issued_at,expires_at)
                 VALUES(?,?,?,?,?,?)",
            ))
            .bind::<Text,_>(&draft.id).bind::<Text,_>(draft.email.lookup_key())
            .bind::<Binary,_>(draft.binding.digest().as_bytes().as_slice())
            .bind::<Binary,_>(verifier.as_bytes().as_slice())
            .bind::<BigInt,_>(now).bind::<BigInt,_>(draft.expires_at)
            .execute(connection).await?;
            Ok(())
        })
    }

    pub async fn verify_login_challenge(
        &self,
        email: &EmailAddress,
        binding: &BearerSecret,
        code: &EmailCode,
        new: &NewSession,
        now: i64,
        policy: SessionPolicy,
    ) -> Result<EmailLoginOutcome, StoreError> {
        operation!(self, |connection, dialect| {
            // Existing accounts use the same User-first lock order as reset.
            let sql = match dialect {
                Dialect::Postgres => "SELECT * FROM users WHERE email_key=? FOR NO KEY UPDATE",
                Dialect::Sqlite => "SELECT * FROM users WHERE email_key=?",
            };
            let users = diesel::sql_query(dialect.query(sql))
                .bind::<Text, _>(email.lookup_key())
                .load::<UserRow>(connection)
                .await?;
            let sql = match dialect {
                Dialect::Postgres => {
                    "SELECT id,code_verifier,expires_at,consumed_at FROM login_challenges WHERE email_key=? AND binding_hash=? FOR UPDATE"
                }
                Dialect::Sqlite => {
                    "SELECT id,code_verifier,expires_at,consumed_at FROM login_challenges WHERE email_key=? AND binding_hash=?"
                }
            };
            let challenges = diesel::sql_query(dialect.query(sql))
                .bind::<Text, _>(email.lookup_key())
                .bind::<Binary, _>(binding.digest().as_bytes().as_slice())
                .load::<ChallengeRow>(connection)
                .await?;
            let [challenge] = challenges.as_slice() else {
                return Err(StoreError::InvalidProof);
            };
            if now >= challenge.expires_at || challenge.consumed_at.is_some() {
                return Err(StoreError::InvalidProof);
            }
            let bytes = challenge
                .code_verifier
                .as_slice()
                .try_into()
                .map_err(|_| StoreError::Integrity)?;
            if !CodeVerifier::from_bytes(bytes).matches(&self.cipher, email, binding, code)? {
                return Err(StoreError::InvalidProof);
            }
            let changed = diesel::sql_query(dialect.query(
                "UPDATE login_challenges SET consumed_at=? WHERE id=? AND consumed_at IS NULL AND expires_at>?",
            )).bind::<BigInt,_>(now).bind::<Text,_>(&challenge.id).bind::<BigInt,_>(now)
                .execute(connection).await?;
            if changed != 1 {
                return Err(StoreError::InvalidProof);
            }
            let Some(user) = users.as_slice().first() else {
                // Committed email verification without unrestricted account creation.
                return Ok(EmailLoginOutcome::RegistrationRequired);
            };
            let user = user.summary()?;
            Ok(EmailLoginOutcome::Authenticated(Box::new(insert_session!(
                connection,
                dialect,
                self.cipher,
                user,
                new,
                now,
                policy
            ))))
        })
    }
}
