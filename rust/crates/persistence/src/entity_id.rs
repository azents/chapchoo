//! The sole owner of product entity ID representation.
//!
//! Private database values never cross Store DTO boundaries. Native identifiers,
//! credentials and operation request IDs are intentionally not this type.

use crate::StoreError;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Binary,
    sqlite::{Sqlite, SqliteValue},
};
use uuid::{Builder, Uuid, Variant, Version};

/// One shared SQL identity contract, with backend-owned compact representations.
#[derive(Debug, Clone, Copy, diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
#[diesel(postgres_type(name = "uuid"))]
#[diesel(sqlite_type(name = "Binary"))]
pub(crate) struct IdentitySql;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EntityId(Uuid);

/// Only kinds actually crossing Repository DTO boundaries have prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EntityKind {
    User,
    Team,
    Agent,
    Computer,
    Provider,
    AuthSession,
    Grant,
    Invitation,
    LoginChallenge,
    Execution,
    ConnectorSession,
    ControlInstance,
    Connection,
    ProfileImage,
    Authentication,
    AuthenticationAttempt,
    Conversation,
    ConversationOperation,
}

impl EntityKind {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::User => "U",
            Self::Team => "T",
            Self::Agent => "A",
            Self::Computer => "C",
            Self::Provider => "P",
            Self::AuthSession => "S",
            Self::Grant => "G",
            Self::Invitation => "I",
            Self::LoginChallenge => "L",
            Self::Execution => "E",
            Self::ConnectorSession => "CS",
            Self::ControlInstance => "CT",
            Self::Connection => "CN",
            Self::ProfileImage => "PI",
            Self::Authentication => "HA",
            Self::AuthenticationAttempt => "HD",
            Self::Conversation => "CV",
            Self::ConversationOperation => "CO",
        }
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("EntityId([REDACTED])")
    }
}

impl EntityId {
    /// Accept time from the operation's existing acceptance boundary.
    /// UUID Builder owns the RFC layout; entropy failure remains a typed error.
    pub(crate) fn generate(now_ms: i64) -> Result<Self, StoreError> {
        let millis = u64::try_from(now_ms).map_err(|_| StoreError::InvalidInput)?;
        if millis >= (1_u64 << 48) {
            return Err(StoreError::InvalidInput);
        }
        let mut random = [0_u8; 10];
        getrandom::fill(&mut random).map_err(chapchoo_crypto::CryptoError::Random)?;
        Ok(Self(
            Builder::from_unix_timestamp_millis(millis, &random).into_uuid(),
        ))
    }

    pub(crate) fn from_uuid(value: Uuid) -> Result<Self, StoreError> {
        if value.get_version() != Some(Version::SortRand) || value.get_variant() != Variant::RFC4122
        {
            return Err(StoreError::Integrity);
        }
        Ok(Self(value))
    }

    pub(crate) fn to_dto(self, kind: EntityKind) -> String {
        format!(
            "{}{}",
            kind.prefix(),
            URL_SAFE_NO_PAD.encode(self.0.as_bytes())
        )
    }

    pub(crate) fn from_dto(kind: EntityKind, value: &str) -> Result<Self, StoreError> {
        let prefix = kind.prefix();
        if value.len() != prefix.len() + 22 || !value.starts_with(prefix) {
            return Err(StoreError::InvalidInput);
        }
        let body = value.get(prefix.len()..).ok_or(StoreError::InvalidInput)?;
        let bytes = URL_SAFE_NO_PAD
            .decode(body)
            .map_err(|_| StoreError::InvalidInput)?;
        let uuid = Uuid::from_slice(&bytes).map_err(|_| StoreError::InvalidInput)?;
        let id = Self::from_uuid(uuid).map_err(|_| StoreError::InvalidInput)?;
        if id.to_dto(kind) != value {
            return Err(StoreError::InvalidInput);
        }
        Ok(id)
    }
}

impl ToSql<IdentitySql, Pg> for EntityId {
    fn to_sql<'b>(&'b self, output: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <Uuid as ToSql<diesel::sql_types::Uuid, Pg>>::to_sql(&self.0, output)
    }
}

impl FromSql<IdentitySql, Pg> for EntityId {
    fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
        let uuid = <Uuid as FromSql<diesel::sql_types::Uuid, Pg>>::from_sql(value)?;
        Self::from_uuid(uuid).map_err(Into::into)
    }
}

impl ToSql<IdentitySql, Sqlite> for EntityId {
    fn to_sql<'b>(&'b self, output: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        output.set_value(self.0.as_bytes().to_vec());
        Ok(IsNull::No)
    }
}

impl FromSql<IdentitySql, Sqlite> for EntityId {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(value)?;
        let uuid = Uuid::from_slice(&bytes)?;
        Self::from_uuid(uuid).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{Connection, RunQueryDsl};

    #[derive(Debug, diesel::QueryableByName)]
    struct IdentityRow {
        #[diesel(sql_type = IdentitySql)]
        id: EntityId,
    }

    const KINDS: [EntityKind; 18] = [
        EntityKind::User,
        EntityKind::Team,
        EntityKind::Agent,
        EntityKind::Computer,
        EntityKind::Provider,
        EntityKind::AuthSession,
        EntityKind::Grant,
        EntityKind::Invitation,
        EntityKind::LoginChallenge,
        EntityKind::Execution,
        EntityKind::ConnectorSession,
        EntityKind::ControlInstance,
        EntityKind::Connection,
        EntityKind::ProfileImage,
        EntityKind::Authentication,
        EntityKind::AuthenticationAttempt,
        EntityKind::Conversation,
        EntityKind::ConversationOperation,
    ];

    #[test]
    fn complete_dto_roundtrips_without_exposing_uuid_format() -> Result<(), StoreError> {
        let id = EntityId::generate(1_793_750_000_000)?;
        for kind in KINDS {
            let dto = id.to_dto(kind);
            assert_eq!(dto.len(), kind.prefix().len() + 22);
            assert!(!dto.contains('='));
            assert_eq!(EntityId::from_dto(kind, &dto)?, id);
            for other in KINDS {
                if other != kind {
                    assert!(EntityId::from_dto(other, &dto).is_err());
                }
            }
        }
        assert_eq!(format!("{id:?}"), "EntityId([REDACTED])");
        Ok(())
    }

    #[test]
    fn malformed_values_and_noncanonical_trailing_bits_reject() -> Result<(), StoreError> {
        let id = EntityId::generate(1_793_750_000_000)?;
        let canonical = id.to_dto(EntityKind::Agent);
        for value in [
            String::new(),
            canonical.to_lowercase(),
            format!("{canonical}="),
            format!(" {canonical}"),
            format!("{canonical}\n"),
            "A00000000-0000-4000-8000-000000000000".to_owned(),
            "A00000000000000000000+Q".to_owned(),
            "A00000000000000000000/Q".to_owned(),
        ] {
            assert!(EntityId::from_dto(EntityKind::Agent, &value).is_err());
        }
        let v4 = Uuid::parse_str("00000000-0000-4000-8000-000000000000")
            .map_err(|_| StoreError::Integrity)?;
        let v4_dto = format!("A{}", URL_SAFE_NO_PAD.encode(v4.as_bytes()));
        assert!(EntityId::from_dto(EntityKind::Agent, &v4_dto).is_err());
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let last = canonical.as_bytes().last().ok_or(StoreError::Integrity)?;
        let index = alphabet
            .iter()
            .position(|byte| byte == last)
            .ok_or(StoreError::Integrity)?;
        let mut alias = canonical.clone();
        alias.pop();
        alias.push(char::from(alphabet[index + 1]));
        assert!(EntityId::from_dto(EntityKind::Agent, &alias).is_err());
        Ok(())
    }

    #[test]
    fn time_bounds_and_compact_sqlite_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        assert!(EntityId::generate(-1).is_err());
        assert!(EntityId::generate(1_i64 << 48).is_err());
        let id = EntityId::generate(1_793_750_000_000)?;
        let mut connection = diesel::SqliteConnection::establish(":memory:")?;
        let row = diesel::sql_query("SELECT ? AS id")
            .bind::<IdentitySql, _>(id)
            .get_result::<IdentityRow>(&mut connection)?;
        assert_eq!(row.id, id);
        assert_eq!(row.id.0.as_bytes().len(), 16);
        let malformed = diesel::sql_query("SELECT zeroblob(15) AS id")
            .get_result::<IdentityRow>(&mut connection);
        assert!(malformed.is_err());
        let nil = diesel::sql_query("SELECT zeroblob(16) AS id")
            .get_result::<IdentityRow>(&mut connection);
        assert!(nil.is_err());
        Ok(())
    }

    #[test]
    fn compact_postgres_roundtrip_and_type() -> Result<(), Box<dyn std::error::Error>> {
        use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};

        #[derive(Debug, diesel::QueryableByName)]
        struct Stored {
            #[diesel(sql_type = IdentitySql)]
            id: EntityId,
            #[diesel(sql_type = diesel::sql_types::Text)]
            type_name: String,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            bytes: i32,
        }

        // The repository test runner supplies an isolated PostgreSQL instance.
        // Missing prerequisites must fail, not masquerade as executed coverage.
        let url = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        let id = EntityId::generate(1_793_750_000_000)?;
        let row = diesel::sql_query(
            "SELECT $1 AS id,pg_typeof($1)::text AS type_name,octet_length(uuid_send($1)) AS bytes",
        )
        .bind::<IdentitySql, _>(id)
        .get_result::<Stored>(&mut connection)?;
        assert_eq!(row.id, id);
        assert_eq!(row.type_name, "uuid");
        assert_eq!(row.bytes, 16);
        let malformed =
            diesel::sql_query("SELECT '00000000-0000-4000-8000-000000000000'::uuid AS id")
                .get_result::<IdentityRow>(&mut connection);
        assert!(malformed.is_err());
        Ok(())
    }
}
