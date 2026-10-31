use crate::{
    Store, StoreError, UserSummary,
    models::{UserRow, authenticate},
    store::operation,
};
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::DisplaySettings;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;

#[derive(Debug, diesel::QueryableByName)]
pub(crate) struct DisplayRow {
    #[diesel(sql_type = Text)]
    pub language: String,
    #[diesel(sql_type = Text)]
    pub time_zone: String,
    #[diesel(sql_type = Text)]
    pub region: String,
}
impl DisplayRow {
    pub(crate) fn value(&self) -> Result<DisplaySettings, StoreError> {
        DisplaySettings::parse(&self.language, &self.time_zone, &self.region)
            .map_err(|_| StoreError::Integrity)
    }
}
macro_rules! read_defaults {
    ($connection:expr, $dialect:expr) => {{
        let row = diesel::sql_query(
            "SELECT language,time_zone,region FROM display_defaults WHERE singleton=1",
        )
        .get_result::<$crate::display::DisplayRow>($connection)
        .await?;
        row.value()?
    }};
}
pub(crate) use read_defaults;

impl Store {
    pub async fn display_defaults(&self) -> Result<DisplaySettings, StoreError> {
        operation!(self, |connection, dialect| {
            let _ = dialect;
            Ok(read_defaults!(connection, dialect))
        })
    }
    pub async fn update_display(
        &self,
        actor: &SecretDigest,
        language: Option<&str>,
        time_zone: Option<&str>,
        region: Option<&str>,
        now: i64,
    ) -> Result<UserSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let display = user
                .display
                .patch(language, time_zone, region)
                .map_err(|_| StoreError::InvalidInput)?;
            let row =
                diesel::sql_query(dialect.query(
                    "UPDATE users SET language=?,time_zone=?,region=? WHERE id=? RETURNING *",
                ))
                .bind::<Text, _>(display.language())
                .bind::<Text, _>(display.time_zone())
                .bind::<Text, _>(display.region())
                .bind::<Text, _>(&user.id)
                .get_result::<UserRow>(connection)
                .await?;
            row.summary()
        })
    }
    pub async fn update_display_defaults(
        &self,
        actor: &SecretDigest,
        language: Option<&str>,
        time_zone: Option<&str>,
        region: Option<&str>,
        now: i64,
    ) -> Result<DisplaySettings, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            if !user.is_admin {
                return Err(StoreError::Forbidden);
            }
            let sql = match dialect {
                crate::store::Dialect::Postgres => {
                    "SELECT language,time_zone,region FROM display_defaults WHERE singleton=1 FOR UPDATE"
                }
                crate::store::Dialect::Sqlite => {
                    "SELECT language,time_zone,region FROM display_defaults WHERE singleton=1"
                }
            };
            let row = diesel::sql_query(sql)
                .get_result::<DisplayRow>(connection)
                .await?;
            let display = row
                .value()?
                .patch(language, time_zone, region)
                .map_err(|_| StoreError::InvalidInput)?;
            diesel::sql_query(dialect.query(
                "UPDATE display_defaults SET language=?,time_zone=?,region=? WHERE singleton=1",
            ))
            .bind::<Text, _>(display.language())
            .bind::<Text, _>(display.time_zone())
            .bind::<Text, _>(display.region())
            .execute(connection)
            .await?;
            Ok(display)
        })
    }
}
