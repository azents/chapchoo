//! Forward native schema preservation, independent of external Provider IO.

use chapchoo_crypto::BearerSecret;
use chapchoo_persistence::{POSTGRES_MIGRATIONS, SQLITE_MIGRATIONS};
use diesel::{
    Connection, RunQueryDsl,
    connection::SimpleConnection,
    sql_types::{BigInt, Text},
};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use diesel_migrations::MigrationHarness;

type Error = Box<dyn std::error::Error + Send + Sync>;

const TABLES: [&str; 16] = [
    "computer_providers",
    "provider_keys",
    "provider_key_requests",
    "computers",
    "agents",
    "computer_executions",
    "computer_resources",
    "connector_sessions",
    "connector_key_requests",
    "provider_connections",
    "connector_connections",
    "users",
    "teams",
    "control_instances",
    "agent_creation_receipts",
    "installation_key_binding",
];

#[derive(diesel::QueryableByName)]
struct Value {
    #[diesel(sql_type = Text)]
    value: String,
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

fn blob(postgres: bool, byte: u8, size: usize) -> String {
    let hex = format!("{byte:02x}").repeat(size);
    if postgres {
        format!("decode('{hex}','hex')")
    } else {
        format!("X'{hex}'")
    }
}

fn fixture(postgres: bool) -> String {
    let mut sql = include_str!("fixtures/native_migration.sql").to_owned();
    for i in 1..=7 {
        sql = sql.replace(&format!("{{hash{i}}}"), &blob(postgres, i, 32));
    }
    for i in 1..=2 {
        sql = sql.replace(&format!("{{nonce{i}}}"), &blob(postgres, i, 24));
        sql = sql.replace(&format!("{{cipher{i}}}"), &blob(postgres, i, 41));
    }
    format!("BEGIN;\n{sql}\nCOMMIT;")
}

fn snapshot_query(table: &str, columns: &[String], postgres: bool, migrated: bool) -> String {
    let source_column = |column: &str| {
        if migrated && table == "computer_resources" && column == "namespace" {
            "native_scope".to_owned()
        } else {
            column.to_owned()
        }
    };
    if postgres {
        let selected = columns
            .iter()
            .map(|c| format!("{} AS {c}", source_column(c)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "SELECT row_to_json(snapshot)::text AS value FROM (SELECT {selected} FROM {table}) snapshot"
        )
    } else {
        let fields = columns.iter().map(|c| {
            let source = source_column(c);
            format!("'{c}',json_array(typeof({source}),CASE WHEN typeof({source})='blob' THEN hex({source}) ELSE {source} END)")
        }).collect::<Vec<_>>().join(",");
        format!("SELECT json_object({fields}) AS value FROM {table}")
    }
}

macro_rules! snapshot {
    ($connection:expr, $table:expr, $columns:expr, $postgres:expr, $migrated:expr) => {{
        let query = snapshot_query($table, $columns, $postgres, $migrated);
        let mut values = diesel::sql_query(query)
            .load::<Value>($connection)?
            .into_iter()
            .map(|r| r.value)
            .collect::<Vec<_>>();
        values.sort();
        values
    }};
}

macro_rules! exercise {
    ($connection:expr, $source:expr, $postgres:expr) => {{
        let connection = $connection;
        let postgres = $postgres;
        let pending = connection.pending_migrations($source)?;
        assert_eq!(pending.len(), 6);
        connection.run_migrations(&pending[..5])?;
        connection.batch_execute(&fixture(postgres))?;
        let mut originals = Vec::new();
        for table in TABLES {
            let query = if postgres {
                format!("SELECT column_name AS value FROM information_schema.columns WHERE table_schema=current_schema() AND table_name='{table}' ORDER BY ordinal_position")
            } else {
                format!("SELECT name AS value FROM pragma_table_xinfo('{table}') ORDER BY cid")
            };
            let columns = diesel::sql_query(query).load::<Value>(connection)?
                .into_iter().map(|r| r.value).collect::<Vec<_>>();
            assert!(!columns.is_empty());
            assert!(columns.iter().all(|c| c.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')));
            let values = snapshot!(connection, table, &columns, postgres, false);
            assert!(!values.is_empty(), "Fixture must populate {table}");
            originals.push((table, columns, values));
        }
        if !postgres {
            // Unknown incoming dependencies are not silently lost during rebuild.
            connection.batch_execute("CREATE TABLE unexpected_native_dependency(provider_id TEXT REFERENCES computer_providers(id))")?;
            assert!(connection.run_migration(&*pending[5]).is_err());
            assert_eq!(connection.applied_migrations()?.len(), 5);
            for (table, columns, values) in &originals {
                assert_eq!(&snapshot!(connection, table, columns, postgres, false), values, "{table}");
            }
            connection.batch_execute("DROP TABLE unexpected_native_dependency")?;
        }
        connection.run_migration(&*pending[5])?;
        assert_eq!(connection.applied_migrations()?.len(), 6);
        for (table, columns, values) in &originals {
            assert_eq!(&snapshot!(connection, table, columns, postgres, true), values, "{table}");
        }
        if postgres {
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pg_constraint WHERE connamespace=current_schema()::regnamespace AND contype='f' AND NOT convalidated").get_result::<Count>(connection)?.n, 0);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pg_constraint WHERE conrelid='computers'::regclass AND contype='f' AND condeferrable AND condeferred").get_result::<Count>(connection)?.n, 1);
        } else {
            assert_eq!(diesel::sql_query("SELECT foreign_keys AS n FROM pragma_foreign_keys").get_result::<Count>(connection)?.n, 1);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM pragma_foreign_key_check").get_result::<Count>(connection)?.n, 0);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM sqlite_schema WHERE name GLOB '_docker_new_*'").get_result::<Count>(connection)?.n, 0);
            assert_eq!(diesel::sql_query("SELECT count(*) AS n FROM sqlite_temp_schema WHERE name GLOB '_docker_*'").get_result::<Count>(connection)?.n, 0);
        }
        assert!(connection.revert_last_migration($source).is_err());
        assert_eq!(connection.applied_migrations()?.len(), 6);
        for (table, columns, values) in &originals {
            assert_eq!(&snapshot!(connection, table, columns, postgres, true), values, "{table}");
        }
        connection.batch_execute(&format!(
            "INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,docker_daemon_id,created_at) VALUES('docker','SYSTEM','native-system','Docker','DOCKER','daemon:original',1);
            INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('docker-computer','native-owner','CLOUD','docker','RUNNING','docker-create',{},'CREATE','ACCEPTED',1);
            INSERT INTO computer_executions(id,computer_id,initial_allocation,created_at) VALUES('docker-execution','docker-computer',TRUE,1);
            INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,presence) VALUES('docker-computer','docker','DOCKER_VOLUME','DOCKER_DAEMON','daemon:original','saved','UNKNOWN');
            INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,execution_id,presence) VALUES('docker-computer','docker','DOCKER_CONTAINER','DOCKER_DAEMON','daemon:original','execution','docker-execution','UNKNOWN');",
            blob(postgres, 8, 32)
        ))?;
        for invalid in [
            "UPDATE computer_providers SET docker_daemon_id='daemon' WHERE id='native-provider'",
            "UPDATE computer_providers SET docker_daemon_id='' WHERE id='docker'",
            "UPDATE computer_providers SET owner_kind='USER',system_owner_id=NULL,user_owner_id='native-owner' WHERE id='docker'",
            "UPDATE computer_resources SET execution_id='docker-execution' WHERE resource_kind='DOCKER_VOLUME'",
            "UPDATE computer_resources SET execution_id=NULL WHERE resource_kind='DOCKER_CONTAINER'",
            "UPDATE computer_resources SET native_scope_kind='KUBERNETES_NAMESPACE' WHERE resource_kind='DOCKER_CONTAINER'",
            "UPDATE computer_resources SET resource_uid='fake-uid' WHERE resource_kind='DOCKER_VOLUME'",
            "UPDATE computer_resources SET docker_container_id='short-id' WHERE resource_kind='DOCKER_CONTAINER'",
            "UPDATE computer_resources SET docker_volume_metadata='invalid-json' WHERE resource_kind='DOCKER_VOLUME'",
            "UPDATE computer_resources SET presence='PRESENT' WHERE resource_kind='DOCKER_VOLUME'",
            "INSERT INTO computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,presence) VALUES('docker-computer','docker','PVC','KUBERNETES_NAMESPACE','namespace','second-saved','UNKNOWN')",
        ] {
            assert!(connection.batch_execute(invalid).is_err(), "Must reject: {invalid}");
        }
        connection.batch_execute("UPDATE computer_resources SET docker_volume_metadata='{\"driver\":\"local\",\"scope\":\"local\",\"options\":{},\"created_at\":null,\"labels\":{}}',presence='PRESENT' WHERE resource_kind='DOCKER_VOLUME'")?;
    }};
}

#[test]
fn sqlite_native_migration_preserves_closed_graph_and_rejects_unknown_incoming_fk()
-> Result<(), Error> {
    let mut connection = diesel::SqliteConnection::establish(":memory:")?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    exercise!(&mut connection, SQLITE_MIGRATIONS, false);
    Ok(())
}

#[test]
fn postgres_native_migration_preserves_closed_graph_and_constraints() -> Result<(), Error> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("native_migration_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        exercise!(&mut connection, POSTGRES_MIGRATIONS, true);
        Ok::<_, Error>(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}
