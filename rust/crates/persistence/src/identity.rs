use chapchoo_crypto::SecretDigest;
use chapchoo_domain::EmailAddress;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;

use crate::{
    CredentialSnapshot, Store, StoreError, UserSummary,
    models::{UserRow, authenticate},
    store::operation,
};

impl Store {
    /// Detached snapshot for CPU password verification outside the DB transaction.
    pub async fn password_snapshot(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<CredentialSnapshot>, StoreError> {
        operation!(self, |connection, dialect| {
            let rows = diesel::sql_query(dialect.query("SELECT * FROM users WHERE email_key=?"))
                .bind::<Text, _>(email.lookup_key())
                .load::<UserRow>(connection)
                .await?;
            match rows.into_iter().next() {
                Some(user) => Ok(Some(user.snapshot()?)),
                None => Ok(None),
            }
        })
    }

    pub async fn current_identity(
        &self,
        digest: &SecretDigest,
        now: i64,
    ) -> Result<UserSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, digest, now);
            Ok(user)
        })
    }
}
