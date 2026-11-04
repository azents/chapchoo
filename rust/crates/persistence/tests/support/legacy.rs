//! Historical native fixtures are populated before explicit identity preparation.
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use std::error::Error;
type TestError = Box<dyn Error + Send + Sync>;
const SQLITE: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations/sqlite");
const POSTGRES: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations/postgres");

pub fn sqlite(
    connection: &mut diesel::SqliteConnection,
    cipher: &chapchoo_crypto::CredentialCipher,
) -> Result<(), TestError> {
    let pending = connection.pending_migrations(SQLITE)?;
    connection.run_migrations(&pending[..8])?;
    let check = cipher.installation_check()?;
    diesel::sql_query("INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,?)")
        .bind::<diesel::sql_types::Binary, _>(check.as_bytes().as_slice())
        .execute(connection)?;
    Ok(())
}

pub fn postgres(
    connection: &mut diesel_async::async_connection_wrapper::AsyncConnectionWrapper<
        diesel_async::AsyncPgConnection,
    >,
    cipher: &chapchoo_crypto::CredentialCipher,
) -> Result<(), TestError> {
    let pending = connection.pending_migrations(POSTGRES)?;
    connection.run_migrations(&pending[..8])?;
    let check = cipher.installation_check()?;
    diesel::sql_query("INSERT INTO installation_key_binding(singleton,key_check) VALUES(1,$1)")
        .bind::<diesel::sql_types::Binary, _>(check.as_bytes().as_slice())
        .execute(connection)?;
    Ok(())
}
