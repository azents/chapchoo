-- Preserve the complete closed native authority graph with foreign keys ON.
-- Every retained value is compared before old rows are cleared and after rename.
-- Any failed assertion aborts the enclosing explicit prepare transaction.
CREATE TEMP TABLE _docker_migration_guard(valid INTEGER NOT NULL CHECK(valid=1));

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT foreign_keys FROM pragma_foreign_keys)=1 THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT legacy_alter_table FROM pragma_legacy_alter_table)=0 THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM sqlite_schema AS s JOIN pragma_foreign_key_list(s.name) AS fk
    WHERE s.type='table' AND fk.[table] IN ('computer_providers','provider_keys','provider_key_requests','computers','agents','computer_executions','computer_resources','connector_sessions','connector_key_requests','provider_connections','connector_connections')
        AND s.name NOT IN ('computer_providers','provider_keys','provider_key_requests','computers','agents','computer_executions','computer_resources','connector_sessions','connector_key_requests','provider_connections','connector_connections')
) THEN 1 ELSE 0 END;

CREATE TEMP TABLE _docker_original_computer_providers AS SELECT * FROM computer_providers;

CREATE TEMP TABLE _docker_original_provider_keys AS SELECT * FROM provider_keys;

CREATE TEMP TABLE _docker_original_provider_key_requests AS SELECT * FROM provider_key_requests;

CREATE TEMP TABLE _docker_original_computers AS SELECT * FROM computers;

CREATE TEMP TABLE _docker_original_agents AS SELECT * FROM agents;

CREATE TEMP TABLE _docker_original_computer_executions AS SELECT * FROM computer_executions;

CREATE TEMP TABLE _docker_original_computer_resources AS SELECT * FROM computer_resources;

CREATE TEMP TABLE _docker_original_connector_sessions AS SELECT * FROM connector_sessions;

CREATE TEMP TABLE _docker_original_connector_key_requests AS SELECT * FROM connector_key_requests;

CREATE TEMP TABLE _docker_original_provider_connections AS SELECT * FROM provider_connections;

CREATE TEMP TABLE _docker_original_connector_connections AS SELECT * FROM connector_connections;

CREATE TEMP TABLE _docker_original_users AS SELECT * FROM users;

CREATE TEMP TABLE _docker_original_teams AS SELECT * FROM teams;

CREATE TEMP TABLE _docker_original_control_instances AS SELECT * FROM control_instances;

CREATE TEMP TABLE _docker_original_agent_creation_receipts AS SELECT * FROM agent_creation_receipts;

CREATE TEMP TABLE _docker_original_installation_key_binding AS SELECT * FROM installation_key_binding;

CREATE TABLE _docker_new_computer_providers (
    id TEXT PRIMARY KEY NOT NULL,
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('SYSTEM','USER')),
    system_owner_id TEXT,
    user_owner_id TEXT REFERENCES users(id),
    name TEXT NOT NULL,
    provider_kind TEXT NOT NULL CHECK (provider_kind IN ('KUBERNETES','DOCKER')),
    docker_daemon_id TEXT,
    can_create BOOLEAN NOT NULL DEFAULT FALSE,
    can_start BOOLEAN NOT NULL DEFAULT FALSE,
    can_stop BOOLEAN NOT NULL DEFAULT FALSE,
    can_restart BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    CHECK ((owner_kind='SYSTEM' AND system_owner_id IS NOT NULL AND user_owner_id IS NULL)
        OR (owner_kind='USER' AND user_owner_id IS NOT NULL AND system_owner_id IS NULL)),
    CHECK (provider_kind<>'DOCKER' OR owner_kind='SYSTEM'),
    CHECK (docker_daemon_id IS NULL OR (provider_kind='DOCKER'
        AND length(CAST(docker_daemon_id AS BLOB)) BETWEEN 1 AND 1024))
);

CREATE TABLE _docker_new_provider_keys (
    provider_id TEXT PRIMARY KEY NOT NULL REFERENCES _docker_new_computer_providers(id) ON DELETE CASCADE,
    current_hash BLOB NOT NULL UNIQUE CHECK (length(current_hash)=32),
    previous_hash BLOB CHECK (previous_hash IS NULL OR length(previous_hash)=32),
    previous_valid_until BIGINT,
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at BIGINT,
    current_request_id TEXT NOT NULL,
    recovery_nonce BLOB CHECK (recovery_nonce IS NULL OR length(recovery_nonce)=24),
    recovery_ciphertext BLOB,
    CHECK (expires_at>issued_at),
    CHECK ((previous_hash IS NULL)=(previous_valid_until IS NULL)),
    CHECK ((recovery_nonce IS NULL)=(recovery_ciphertext IS NULL)),
    CHECK (revoked_at IS NOT NULL OR recovery_nonce IS NOT NULL)
);

CREATE TABLE _docker_new_provider_key_requests (
    provider_id TEXT NOT NULL REFERENCES _docker_new_computer_providers(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    result_hash BLOB NOT NULL CHECK (length(result_hash)=32),
    PRIMARY KEY (provider_id,request_id)
);

CREATE TABLE _docker_new_computers (
    id TEXT PRIMARY KEY NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    kind TEXT NOT NULL CHECK (kind IN ('CLOUD','LOCAL')),
    provider_id TEXT REFERENCES _docker_new_computer_providers(id),
    desired_state TEXT NOT NULL CHECK (desired_state IN ('RUNNING','STOPPED','DELETING')),
    current_request_id TEXT NOT NULL,
    current_payload_hash BLOB NOT NULL CHECK (length(current_payload_hash)=32),
    current_action TEXT NOT NULL CHECK (current_action IN ('CREATE','START','STOP','RESTART','DELETE')),
    current_phase TEXT NOT NULL CHECK (current_phase IN ('ACCEPTED','RETIRING','STARTING','DELETING','COMPLETED','BLOCKED','UNCERTAIN')),
    initial_start_dispatch_at BIGINT,
    expected_old_execution_id TEXT,
    current_execution_id TEXT,
    observed_power TEXT NOT NULL DEFAULT 'UNKNOWN' CHECK (observed_power IN ('RUNNING','STOPPED','UNKNOWN')),
    power_observed_at BIGINT,
    power_source_connection_id TEXT,
    power_source_execution_id TEXT,
    environment_source_connection_id TEXT,
    environment_observed_at BIGINT,
    saved_environment TEXT NOT NULL DEFAULT 'UNKNOWN' CHECK (saved_environment IN ('VERIFIED','MISSING','UNKNOWN')),
    agent_removed_at BIGINT,
    removed_agent_id TEXT UNIQUE,
    cleanup_completed_at BIGINT,
    created_at BIGINT NOT NULL, prepare_admitted_at BIGINT,
    UNIQUE (id,owner_user_id,kind),
    UNIQUE (id,provider_id),
    CHECK (kind<>'CLOUD' OR provider_id IS NOT NULL),
    CHECK (agent_removed_at IS NULL OR desired_state='DELETING'),
    CHECK (removed_agent_id IS NULL OR (kind='CLOUD' AND desired_state='DELETING' AND agent_removed_at IS NOT NULL)),
    CHECK (cleanup_completed_at IS NULL OR desired_state='DELETING'),
    FOREIGN KEY (current_execution_id,id) REFERENCES _docker_new_computer_executions(id,computer_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE "_docker_new_agents" (
    id TEXT PRIMARY KEY NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    team_id TEXT NOT NULL REFERENCES teams(id),
    name TEXT NOT NULL,
    computer_id TEXT NOT NULL,
    computer_kind TEXT NOT NULL CHECK (computer_kind IN ('CLOUD','LOCAL')),
    cloud_computer_id TEXT GENERATED ALWAYS AS
        (CASE WHEN computer_kind='CLOUD' THEN computer_id ELSE NULL END) STORED UNIQUE,
    created_at BIGINT NOT NULL,
    FOREIGN KEY (computer_id,owner_user_id,computer_kind)
        REFERENCES _docker_new_computers(id,owner_user_id,kind)
);

CREATE TABLE _docker_new_computer_executions (
    id TEXT PRIMARY KEY NOT NULL,
    computer_id TEXT NOT NULL REFERENCES _docker_new_computers(id) ON DELETE CASCADE,
    initial_allocation BOOLEAN NOT NULL,
    running_confirmed_at BIGINT,
    retired_at BIGINT,
    created_at BIGINT NOT NULL,
    UNIQUE (id,computer_id)
);

CREATE TABLE _docker_new_computer_resources (
    computer_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET','DOCKER_CONTAINER','DOCKER_VOLUME')),
    native_scope_kind TEXT NOT NULL DEFAULT 'KUBERNETES_NAMESPACE',
    native_scope TEXT NOT NULL,
    name TEXT NOT NULL,
    resource_uid TEXT,
    docker_container_id TEXT,
    docker_volume_metadata TEXT,
    execution_id TEXT,
    presence TEXT NOT NULL CHECK (presence IN ('PRESENT','ABSENT','UNKNOWN')),
    observed_at BIGINT,
    absence_confirmed_at BIGINT,
    source_connection_id TEXT,
    PRIMARY KEY (computer_id,resource_kind,native_scope,name),
    UNIQUE (provider_id,resource_kind,native_scope,name),
    FOREIGN KEY (computer_id,provider_id) REFERENCES _docker_new_computers(id,provider_id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id,computer_id) REFERENCES _docker_new_computer_executions(id,computer_id),
    CHECK ((resource_kind IN ('PVC','DOCKER_VOLUME'))=(execution_id IS NULL)),
    CHECK ((resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET') AND native_scope_kind='KUBERNETES_NAMESPACE')
        OR (resource_kind IN ('DOCKER_CONTAINER','DOCKER_VOLUME') AND native_scope_kind='DOCKER_DAEMON'
            AND length(CAST(native_scope AS BLOB)) BETWEEN 1 AND 1024)),
    CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET') OR resource_uid IS NULL),
    CHECK (docker_container_id IS NULL OR (resource_kind='DOCKER_CONTAINER'
        AND length(docker_container_id)=64 AND docker_container_id NOT GLOB '*[^0-9a-f]*')),
    CHECK (docker_volume_metadata IS NULL OR (resource_kind='DOCKER_VOLUME'
        AND length(CAST(docker_volume_metadata AS BLOB))<=32768
        AND json_valid(docker_volume_metadata) AND json_type(docker_volume_metadata)='object')),
    CHECK (presence<>'PRESENT' OR resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET')
        OR (resource_kind='DOCKER_CONTAINER' AND docker_container_id IS NOT NULL)
        OR (resource_kind='DOCKER_VOLUME' AND docker_volume_metadata IS NOT NULL))
);

CREATE TABLE _docker_new_connector_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    computer_id TEXT NOT NULL,
    execution_id TEXT NOT NULL UNIQUE,
    initial_request_id TEXT NOT NULL,
    initial_payload_hash BLOB NOT NULL CHECK (length(initial_payload_hash)=32),
    initial_hash BLOB NOT NULL UNIQUE CHECK (length(initial_hash)=32),
    initial_nonce BLOB CHECK (initial_nonce IS NULL OR length(initial_nonce)=24),
    initial_ciphertext BLOB,
    current_hash BLOB UNIQUE CHECK (current_hash IS NULL OR length(current_hash)=32),
    previous_hash BLOB CHECK (previous_hash IS NULL OR length(previous_hash)=32),
    previous_valid_until BIGINT,
    issued_at BIGINT,
    expires_at BIGINT,
    current_request_id TEXT,
    recovery_nonce BLOB CHECK (recovery_nonce IS NULL OR length(recovery_nonce)=24),
    recovery_ciphertext BLOB,
    revoked_at BIGINT,
    created_at BIGINT NOT NULL,
    CHECK ((initial_nonce IS NULL)=(initial_ciphertext IS NULL)),
    CHECK ((previous_hash IS NULL)=(previous_valid_until IS NULL)),
    CHECK ((current_hash IS NULL)=(expires_at IS NULL)),
    CHECK ((current_hash IS NULL)=(issued_at IS NULL)),
    CHECK ((current_hash IS NULL)=(current_request_id IS NULL)),
    CHECK ((recovery_nonce IS NULL)=(recovery_ciphertext IS NULL)),
    CHECK (expires_at IS NULL OR expires_at>issued_at),
    CHECK (revoked_at IS NOT NULL OR initial_nonce IS NOT NULL),
    UNIQUE (id,computer_id),
    FOREIGN KEY (execution_id,computer_id) REFERENCES _docker_new_computer_executions(id,computer_id) ON DELETE CASCADE
);

CREATE TABLE _docker_new_connector_key_requests (
    session_id TEXT NOT NULL REFERENCES _docker_new_connector_sessions(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('EXCHANGE','REFRESH','REVOKE')),
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    result_hash BLOB NOT NULL CHECK (length(result_hash)=32),
    PRIMARY KEY (session_id,request_id)
);

CREATE TABLE _docker_new_provider_connections (
    provider_id TEXT PRIMARY KEY NOT NULL REFERENCES _docker_new_computer_providers(id) ON DELETE CASCADE,
    control_id TEXT NOT NULL REFERENCES control_instances(id),
    connection_id TEXT NOT NULL UNIQUE,
    credential_hash BLOB NOT NULL CHECK (length(credential_hash)=32),
    expires_at BIGINT NOT NULL
);

CREATE TABLE _docker_new_connector_connections (
    computer_id TEXT PRIMARY KEY NOT NULL REFERENCES _docker_new_computers(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL UNIQUE,
    control_id TEXT NOT NULL REFERENCES control_instances(id),
    connection_id TEXT NOT NULL UNIQUE,
    credential_hash BLOB NOT NULL CHECK (length(credential_hash)=32),
    expires_at BIGINT NOT NULL,
    FOREIGN KEY (session_id,computer_id) REFERENCES _docker_new_connector_sessions(id,computer_id) ON DELETE CASCADE
);

INSERT INTO _docker_new_computer_providers(id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at)
SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_original_computer_providers;

INSERT INTO _docker_new_provider_keys(provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext)
SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_original_provider_keys;

INSERT INTO _docker_new_provider_key_requests(provider_id,request_id,payload_hash,result_hash)
SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_original_provider_key_requests;

INSERT INTO _docker_new_computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at)
SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_original_computers;

INSERT INTO _docker_new_agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at)
SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,created_at FROM _docker_original_agents;

INSERT INTO _docker_new_computer_executions(id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at)
SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_original_computer_executions;

INSERT INTO _docker_new_computer_resources(computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id)
SELECT computer_id,provider_id,resource_kind,namespace,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_original_computer_resources;

INSERT INTO _docker_new_connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at)
SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_original_connector_sessions;

INSERT INTO _docker_new_connector_key_requests(session_id,request_id,purpose,payload_hash,result_hash)
SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_original_connector_key_requests;

INSERT INTO _docker_new_provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at)
SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_provider_connections;

INSERT INTO _docker_new_connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at)
SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_connector_connections;

-- Verify all retained fields, including generated values and credential BLOBs.

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_providers)=(SELECT count(*) FROM _docker_new_computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_original_computer_providers EXCEPT SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_new_computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_new_computer_providers EXCEPT SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_original_computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_keys)=(SELECT count(*) FROM _docker_new_provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_original_provider_keys EXCEPT SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_new_provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_new_provider_keys EXCEPT SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_original_provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_key_requests)=(SELECT count(*) FROM _docker_new_provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_original_provider_key_requests EXCEPT SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_new_provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_new_provider_key_requests EXCEPT SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_original_provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computers)=(SELECT count(*) FROM _docker_new_computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_original_computers EXCEPT SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_new_computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_new_computers EXCEPT SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_original_computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_agents)=(SELECT count(*) FROM _docker_new_agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_original_agents EXCEPT SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_new_agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_new_agents EXCEPT SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_original_agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_executions)=(SELECT count(*) FROM _docker_new_computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_original_computer_executions EXCEPT SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_new_computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_new_computer_executions EXCEPT SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_original_computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_resources)=(SELECT count(*) FROM _docker_new_computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,provider_id,resource_kind,namespace,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_original_computer_resources EXCEPT SELECT computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_new_computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_new_computer_resources EXCEPT SELECT computer_id,provider_id,resource_kind,namespace,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_original_computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_sessions)=(SELECT count(*) FROM _docker_new_connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_original_connector_sessions EXCEPT SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_new_connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_new_connector_sessions EXCEPT SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_original_connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_key_requests)=(SELECT count(*) FROM _docker_new_connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_original_connector_key_requests EXCEPT SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_new_connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_new_connector_key_requests EXCEPT SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_original_connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_connections)=(SELECT count(*) FROM _docker_new_provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_provider_connections EXCEPT SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_new_provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_new_provider_connections EXCEPT SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_connections)=(SELECT count(*) FROM _docker_new_connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_connector_connections EXCEPT SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_new_connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_new_connector_connections EXCEPT SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM pragma_foreign_key_check) THEN 1 ELSE 0 END;

-- Break only the old current-execution cycle; the staging copy keeps it.

UPDATE computers SET current_execution_id=NULL;

DELETE FROM connector_connections;

DELETE FROM provider_connections;

DELETE FROM connector_key_requests;

DELETE FROM connector_sessions;

DELETE FROM computer_resources;

DELETE FROM agents;

DELETE FROM computer_executions;

DELETE FROM computers;

DELETE FROM provider_key_requests;

DELETE FROM provider_keys;

DELETE FROM computer_providers;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM connector_connections) THEN 1 ELSE 0 END;

DROP TABLE connector_connections;

DROP TABLE provider_connections;

DROP TABLE connector_key_requests;

DROP TABLE connector_sessions;

DROP TABLE computer_resources;

DROP TABLE agents;

DROP TABLE computer_executions;

DROP TABLE computers;

DROP TABLE provider_key_requests;

DROP TABLE provider_keys;

DROP TABLE computer_providers;

ALTER TABLE _docker_new_computer_providers RENAME TO computer_providers;

ALTER TABLE _docker_new_provider_keys RENAME TO provider_keys;

ALTER TABLE _docker_new_provider_key_requests RENAME TO provider_key_requests;

ALTER TABLE _docker_new_computers RENAME TO computers;

ALTER TABLE _docker_new_agents RENAME TO agents;

ALTER TABLE _docker_new_computer_executions RENAME TO computer_executions;

ALTER TABLE _docker_new_computer_resources RENAME TO computer_resources;

ALTER TABLE _docker_new_connector_sessions RENAME TO connector_sessions;

ALTER TABLE _docker_new_connector_key_requests RENAME TO connector_key_requests;

ALTER TABLE _docker_new_provider_connections RENAME TO provider_connections;

ALTER TABLE _docker_new_connector_connections RENAME TO connector_connections;

CREATE INDEX agents_by_computer ON agents(computer_id);

CREATE INDEX agents_by_owner ON agents(owner_user_id,id);

CREATE INDEX agents_by_team_owner ON agents(team_id,owner_user_id,id);

CREATE INDEX computers_by_provider ON computers(provider_id);

CREATE INDEX computers_pending_cleanup ON computers(desired_state,cleanup_completed_at);

CREATE INDEX executions_by_computer ON computer_executions(computer_id);

CREATE UNIQUE INDEX one_resource_kind_per_execution ON computer_resources(execution_id,resource_kind)
    WHERE execution_id IS NOT NULL;

CREATE UNIQUE INDEX one_saved_environment_per_computer ON computer_resources(computer_id) WHERE resource_kind IN ('PVC','DOCKER_VOLUME');

-- Verify the renamed graph and the untouched external authority records.

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_providers)=(SELECT count(*) FROM computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_original_computer_providers EXCEPT SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM computer_providers EXCEPT SELECT id,owner_kind,system_owner_id,user_owner_id,name,provider_kind,can_create,can_start,can_stop,can_restart,created_at FROM _docker_original_computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_keys)=(SELECT count(*) FROM provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_original_provider_keys EXCEPT SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM provider_keys EXCEPT SELECT provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext FROM _docker_original_provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_key_requests)=(SELECT count(*) FROM provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_original_provider_key_requests EXCEPT SELECT provider_id,request_id,payload_hash,result_hash FROM provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,request_id,payload_hash,result_hash FROM provider_key_requests EXCEPT SELECT provider_id,request_id,payload_hash,result_hash FROM _docker_original_provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computers)=(SELECT count(*) FROM computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_original_computers EXCEPT SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM computers EXCEPT SELECT id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at FROM _docker_original_computers) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_agents)=(SELECT count(*) FROM agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_original_agents EXCEPT SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM agents EXCEPT SELECT id,owner_user_id,team_id,name,computer_id,computer_kind,cloud_computer_id,created_at FROM _docker_original_agents) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_executions)=(SELECT count(*) FROM computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_original_computer_executions EXCEPT SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM computer_executions EXCEPT SELECT id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at FROM _docker_original_computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_computer_resources)=(SELECT count(*) FROM computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,provider_id,resource_kind,namespace,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_original_computer_resources EXCEPT SELECT computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,provider_id,resource_kind,native_scope,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM computer_resources EXCEPT SELECT computer_id,provider_id,resource_kind,namespace,name,resource_uid,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id FROM _docker_original_computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_sessions)=(SELECT count(*) FROM connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_original_connector_sessions EXCEPT SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM connector_sessions EXCEPT SELECT id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at FROM _docker_original_connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_key_requests)=(SELECT count(*) FROM connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_original_connector_key_requests EXCEPT SELECT session_id,request_id,purpose,payload_hash,result_hash FROM connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT session_id,request_id,purpose,payload_hash,result_hash FROM connector_key_requests EXCEPT SELECT session_id,request_id,purpose,payload_hash,result_hash FROM _docker_original_connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_provider_connections)=(SELECT count(*) FROM provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_provider_connections EXCEPT SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM provider_connections EXCEPT SELECT provider_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_connector_connections)=(SELECT count(*) FROM connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_connector_connections EXCEPT SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM connector_connections EXCEPT SELECT computer_id,session_id,control_id,connection_id,credential_hash,expires_at FROM _docker_original_connector_connections) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_users)=(SELECT count(*) FROM users) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region FROM _docker_original_users EXCEPT SELECT id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region FROM users) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region FROM users EXCEPT SELECT id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region FROM _docker_original_users) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_teams)=(SELECT count(*) FROM teams) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,kind,name,handle,owner_user_id,created_at,deletion_requested_at FROM _docker_original_teams EXCEPT SELECT id,kind,name,handle,owner_user_id,created_at,deletion_requested_at FROM teams) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,kind,name,handle,owner_user_id,created_at,deletion_requested_at FROM teams EXCEPT SELECT id,kind,name,handle,owner_user_id,created_at,deletion_requested_at FROM _docker_original_teams) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_control_instances)=(SELECT count(*) FROM control_instances) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,endpoint,draining,renewed_at,expires_at FROM _docker_original_control_instances EXCEPT SELECT id,endpoint,draining,renewed_at,expires_at FROM control_instances) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT id,endpoint,draining,renewed_at,expires_at FROM control_instances EXCEPT SELECT id,endpoint,draining,renewed_at,expires_at FROM _docker_original_control_instances) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_agent_creation_receipts)=(SELECT count(*) FROM agent_creation_receipts) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT owner_user_id,team_id,request_id,payload_hash,assigned_agent_id FROM _docker_original_agent_creation_receipts EXCEPT SELECT owner_user_id,team_id,request_id,payload_hash,assigned_agent_id FROM agent_creation_receipts) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT owner_user_id,team_id,request_id,payload_hash,assigned_agent_id FROM agent_creation_receipts EXCEPT SELECT owner_user_id,team_id,request_id,payload_hash,assigned_agent_id FROM _docker_original_agent_creation_receipts) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN (SELECT count(*) FROM _docker_original_installation_key_binding)=(SELECT count(*) FROM installation_key_binding) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT singleton,key_check FROM _docker_original_installation_key_binding EXCEPT SELECT singleton,key_check FROM installation_key_binding) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT singleton,key_check FROM installation_key_binding EXCEPT SELECT singleton,key_check FROM _docker_original_installation_key_binding) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM pragma_foreign_key_check) THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM sqlite_schema WHERE name GLOB '_docker_new_*') THEN 1 ELSE 0 END;

INSERT INTO _docker_migration_guard SELECT CASE WHEN NOT EXISTS (SELECT 1 FROM sqlite_schema AS s JOIN pragma_foreign_key_list(s.name) AS fk WHERE s.type='table' AND fk.[table] GLOB '_docker_new_*') THEN 1 ELSE 0 END;

DROP TABLE _docker_original_computer_providers;

DROP TABLE _docker_original_provider_keys;

DROP TABLE _docker_original_provider_key_requests;

DROP TABLE _docker_original_computers;

DROP TABLE _docker_original_agents;

DROP TABLE _docker_original_computer_executions;

DROP TABLE _docker_original_computer_resources;

DROP TABLE _docker_original_connector_sessions;

DROP TABLE _docker_original_connector_key_requests;

DROP TABLE _docker_original_provider_connections;

DROP TABLE _docker_original_connector_connections;

DROP TABLE _docker_original_users;

DROP TABLE _docker_original_teams;

DROP TABLE _docker_original_control_instances;

DROP TABLE _docker_original_agent_creation_receipts;

DROP TABLE _docker_original_installation_key_binding;

DROP TABLE _docker_migration_guard;
