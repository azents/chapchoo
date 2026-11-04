//! Private catalogue rows and explicit preparation/readiness invariants.

use crate::{
    Store, StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    models::authenticate,
    store::operation,
};
use chapchoo_domain::{AgentPalette, Handle};
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel_async::RunQueryDsl;

/// A selected catalogue item, not a URL or a persisted row representation.
#[derive(Debug, Clone)]
pub struct AgentProfileImage {
    pub id: String,
    pub internal_name: String,
    pub asset_key: String,
}

impl Store {
    pub async fn agent_profile_images(
        &self,
        actor: &chapchoo_crypto::SecretDigest,
        after: Option<&str>,
        limit: i64,
        now: i64,
    ) -> Result<Vec<AgentProfileImage>, StoreError> {
        if !(1..=100).contains(&limit) {
            return Err(StoreError::InvalidInput);
        }
        operation!(self, |connection, dialect| {
            let _caller = authenticate!(connection, dialect, actor, now);
            let after = after
                .map(|id| EntityId::from_dto(EntityKind::ProfileImage, id))
                .transpose()?;
            let rows = diesel::sql_query(
                dialect.query("SELECT * FROM profile_images WHERE item_type='AGENT' AND (? IS NULL OR id>?) ORDER BY id LIMIT ?"),
            )
            .bind::<Nullable<IdentitySql>,_>(after).bind::<Nullable<IdentitySql>,_>(after)
            .bind::<BigInt,_>(limit)
            .load::<CatalogueRow>(connection)
            .await?;
            rows.into_iter()
                .map(|row| {
                    row.validate()?;
                    Ok(AgentProfileImage {
                        id: row.id.to_dto(EntityKind::ProfileImage),
                        internal_name: row.internal_name,
                        asset_key: row.asset_key,
                    })
                })
                .collect()
        })
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct CatalogueRow {
    #[diesel(sql_type=IdentitySql)]
    pub id: EntityId,
    #[diesel(sql_type=Text)]
    pub item_type: String,
    #[diesel(sql_type=Text)]
    pub internal_name: String,
    #[diesel(sql_type=Text)]
    pub source_kind: String,
    #[diesel(sql_type=Text)]
    pub asset_key: String,
}

impl CatalogueRow {
    pub(crate) fn validate(&self) -> Result<(), StoreError> {
        if self.item_type != "AGENT"
            || self.source_kind != "BUILTIN"
            || self.internal_name.is_empty()
            || self.asset_key.len() > 128
            || self
                .asset_key
                .as_bytes()
                .first()
                .is_none_or(|byte| !byte.is_ascii_lowercase() && !byte.is_ascii_digit())
            || !self
                .asset_key
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(StoreError::Integrity);
        }
        Ok(())
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct PreparedIdentity {
    #[diesel(sql_type=IdentitySql)]
    pub id: EntityId,
}

#[derive(diesel::QueryableByName)]
pub(crate) struct PreparedTeam {
    #[diesel(sql_type=Text)]
    pub kind: String,
    #[diesel(sql_type=Nullable<Text>)]
    pub handle: Option<String>,
}

impl PreparedTeam {
    pub(crate) fn validate(&self) -> Result<(), StoreError> {
        match (self.kind.as_str(), self.handle.as_deref()) {
            ("PERSONAL", None) => Ok(()),
            ("COLLABORATIVE", Some(handle)) => Handle::parse(handle)
                .map(|_| ())
                .map_err(|_| StoreError::Integrity),
            _ => Err(StoreError::Integrity),
        }
    }
}

#[derive(diesel::QueryableByName)]
pub(crate) struct PreparedAgent {
    #[diesel(sql_type=Text)]
    pub handle: String,
    #[diesel(sql_type=Text)]
    pub palette_key: String,
}

impl PreparedAgent {
    pub(crate) fn validate(&self) -> Result<(), StoreError> {
        Handle::parse(&self.handle).map_err(|_| StoreError::Integrity)?;
        AgentPalette::parse(&self.palette_key).map_err(|_| StoreError::Integrity)?;
        Ok(())
    }
}

macro_rules! seed_catalogue {
    ($connection:expr, $postgres:expr) => {{
        use diesel::{RunQueryDsl,sql_types::Text};
        use $crate::{entity_id::{EntityId,IdentitySql}, profile_catalogue::CatalogueRow, profile_initialization::INITIAL_IMAGES};
        let connection = &mut *$connection;
        let rows=diesel::sql_query("SELECT * FROM profile_images").load::<CatalogueRow>(connection)?;
        for image in INITIAL_IMAGES {
            if let Some(row)=rows.iter().find(|row|row.internal_name == image.name) {
                row.validate()?;
                if row.asset_key != image.asset_key {
                    return Err($crate::StoreError::Integrity.into());
                }
            } else {
                let now=diesel::sql_query(if $postgres { "SELECT floor(extract(epoch FROM clock_timestamp())*1000)::bigint AS n" }
                    else { "SELECT CAST(strftime('%s','now') AS INTEGER)*1000 AS n" }).get_result::<$crate::preparation::Count>(connection)?.n;
                let id=EntityId::generate(now)?;
                diesel::sql_query(if $postgres { "INSERT INTO profile_images(id,item_type,internal_name,source_kind,asset_key) VALUES($1,'AGENT',$2,'BUILTIN',$3)" }
                    else { "INSERT INTO profile_images(id,item_type,internal_name,source_kind,asset_key) VALUES(?,'AGENT',?,'BUILTIN',?)" })
                    .bind::<IdentitySql,_>(id).bind::<Text,_>(image.name).bind::<Text,_>(image.asset_key).execute(connection)?;
            }
        }
    }};
}
pub(crate) use seed_catalogue;

macro_rules! verify_profiles {
    ($connection:expr, $postgres:expr) => {{
        use diesel::RunQueryDsl;
        use $crate::profile_catalogue::{CatalogueRow,PreparedIdentity,PreparedTeam,PreparedAgent};
        let connection=&mut *$connection;
        let mut identities=Vec::new();
        for table in $crate::identity_transition::IDENTITY_COLUMNS {
            for column in *table {
                identities.push(format!("SELECT {} AS id FROM {} WHERE {} IS NOT NULL",column.column,column.table,column.column));
            }
        }
        identities.push("SELECT id FROM profile_images".to_owned());
        // Row decoding validates every UUID version/variant and compact SQLite
        // length, including identities retained only as historical witnesses.
        for row in diesel::sql_query(identities.join(" UNION ALL ")).load::<PreparedIdentity>(connection)? {
            let _validated_identity=row.id;
        }
        let rows=diesel::sql_query("SELECT * FROM profile_images").load::<CatalogueRow>(connection)?;
        for row in &rows { row.validate()?; let _validated_identity=row.id; }
        for image in &$crate::profile_initialization::INITIAL_IMAGES {
            if !rows.iter().any(|row|row.internal_name == image.name && row.asset_key == image.asset_key) {
                return Err($crate::PreparationError::Unprepared);
            }
        }
        for team in diesel::sql_query(if $postgres { "SELECT kind::text AS kind,handle FROM teams" }
            else { "SELECT kind,handle FROM teams" }).load::<PreparedTeam>(connection)? { team.validate()?; }
        for agent in diesel::sql_query("SELECT handle,palette_key FROM agents").load::<PreparedAgent>(connection)? { agent.validate()?; }
        if diesel::sql_query("SELECT count(*) AS n FROM agents a LEFT JOIN profile_images i ON i.id=a.profile_image_id WHERE i.id IS NULL")
            .get_result::<$crate::preparation::Count>(connection)?.n != 0 {
            return Err($crate::PreparationError::Unprepared);
        }
    }};
}
pub(crate) use verify_profiles;

#[cfg(test)]
mod tests;
