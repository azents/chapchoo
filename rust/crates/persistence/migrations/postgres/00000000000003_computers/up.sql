CREATE TABLE computer_providers (
    id TEXT PRIMARY KEY NOT NULL,
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('SYSTEM','USER')),
    system_owner_id TEXT,
    user_owner_id TEXT REFERENCES users(id),
    name TEXT NOT NULL,
    provider_kind TEXT NOT NULL CHECK (provider_kind IN ('KUBERNETES')),
    can_create BOOLEAN NOT NULL DEFAULT FALSE,
    can_start BOOLEAN NOT NULL DEFAULT FALSE,
    can_stop BOOLEAN NOT NULL DEFAULT FALSE,
    can_restart BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    CHECK ((owner_kind='SYSTEM' AND system_owner_id IS NOT NULL AND user_owner_id IS NULL)
        OR (owner_kind='USER' AND user_owner_id IS NOT NULL AND system_owner_id IS NULL))
);

CREATE TABLE provider_keys (
    provider_id TEXT PRIMARY KEY NOT NULL REFERENCES computer_providers(id) ON DELETE CASCADE,
    current_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(current_hash)=32),
    previous_hash BYTEA CHECK (previous_hash IS NULL OR octet_length(previous_hash)=32),
    previous_valid_until BIGINT,
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at BIGINT,
    current_request_id TEXT NOT NULL,
    recovery_nonce BYTEA CHECK (recovery_nonce IS NULL OR octet_length(recovery_nonce)=24),
    recovery_ciphertext BYTEA,
    CHECK (expires_at>issued_at),
    CHECK ((previous_hash IS NULL)=(previous_valid_until IS NULL)),
    CHECK ((recovery_nonce IS NULL)=(recovery_ciphertext IS NULL)),
    CHECK (revoked_at IS NOT NULL OR recovery_nonce IS NOT NULL)
);

-- These receipts reject stale credential mutations; they contain no secret archive.
CREATE TABLE provider_key_requests (
    provider_id TEXT NOT NULL REFERENCES computer_providers(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    payload_hash BYTEA NOT NULL CHECK (octet_length(payload_hash)=32),
    result_hash BYTEA NOT NULL CHECK (octet_length(result_hash)=32),
    PRIMARY KEY (provider_id,request_id)
);

CREATE TABLE computers (
    id TEXT PRIMARY KEY NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    kind TEXT NOT NULL CHECK (kind IN ('CLOUD','LOCAL')),
    provider_id TEXT REFERENCES computer_providers(id),
    desired_state TEXT NOT NULL CHECK (desired_state IN ('RUNNING','STOPPED','DELETING')),
    current_request_id TEXT NOT NULL,
    current_payload_hash BYTEA NOT NULL CHECK (octet_length(current_payload_hash)=32),
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
    cleanup_completed_at BIGINT,
    created_at BIGINT NOT NULL,
    UNIQUE (id,owner_user_id,kind),
    UNIQUE (id,provider_id),
    CHECK (kind<>'CLOUD' OR provider_id IS NOT NULL),
    CHECK (agent_removed_at IS NULL OR desired_state='DELETING'),
    CHECK (cleanup_completed_at IS NULL OR desired_state='DELETING')
);
CREATE INDEX computers_by_provider ON computers(provider_id);
CREATE INDEX computers_pending_cleanup ON computers(desired_state,cleanup_completed_at);

CREATE TABLE agents (
    id TEXT PRIMARY KEY NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    computer_id TEXT NOT NULL,
    computer_kind TEXT NOT NULL CHECK (computer_kind IN ('CLOUD','LOCAL')),
    cloud_computer_id TEXT GENERATED ALWAYS AS
        (CASE WHEN computer_kind='CLOUD' THEN computer_id ELSE NULL END) STORED UNIQUE,
    created_at BIGINT NOT NULL,
    FOREIGN KEY (computer_id,owner_user_id,computer_kind)
        REFERENCES computers(id,owner_user_id,kind)
);
CREATE INDEX agents_by_owner ON agents(owner_user_id,id);
CREATE INDEX agents_by_computer ON agents(computer_id);

-- assigned_agent_id deliberately survives removal of the Agent row.
CREATE TABLE agent_creation_receipts (
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    payload_hash BYTEA NOT NULL CHECK (octet_length(payload_hash)=32),
    assigned_agent_id TEXT NOT NULL UNIQUE,
    PRIMARY KEY (owner_user_id,request_id)
);

CREATE TABLE computer_executions (
    id TEXT PRIMARY KEY NOT NULL,
    computer_id TEXT NOT NULL REFERENCES computers(id) ON DELETE CASCADE,
    initial_allocation BOOLEAN NOT NULL,
    running_confirmed_at BIGINT,
    retired_at BIGINT,
    created_at BIGINT NOT NULL,
    UNIQUE (id,computer_id)
);
CREATE INDEX executions_by_computer ON computer_executions(computer_id);

CREATE TABLE computer_resources (
    computer_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET')),
    namespace TEXT NOT NULL,
    name TEXT NOT NULL,
    resource_uid TEXT,
    execution_id TEXT,
    presence TEXT NOT NULL CHECK (presence IN ('PRESENT','ABSENT','UNKNOWN')),
    observed_at BIGINT,
    absence_confirmed_at BIGINT,
    source_connection_id TEXT,
    PRIMARY KEY (computer_id,resource_kind,namespace,name),
    UNIQUE (provider_id,resource_kind,namespace,name),
    FOREIGN KEY (computer_id,provider_id) REFERENCES computers(id,provider_id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id,computer_id) REFERENCES computer_executions(id,computer_id),
    CHECK ((resource_kind='PVC')=(execution_id IS NULL))
);
CREATE UNIQUE INDEX one_resource_kind_per_execution ON computer_resources(execution_id,resource_kind)
    WHERE execution_id IS NOT NULL;
CREATE UNIQUE INDEX one_saved_pvc_per_computer ON computer_resources(computer_id)
    WHERE resource_kind='PVC';

CREATE TABLE connector_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    execution_id TEXT NOT NULL UNIQUE REFERENCES computer_executions(id) ON DELETE CASCADE,
    initial_request_id TEXT NOT NULL,
    initial_payload_hash BYTEA NOT NULL CHECK (octet_length(initial_payload_hash)=32),
    initial_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(initial_hash)=32),
    initial_nonce BYTEA CHECK (initial_nonce IS NULL OR octet_length(initial_nonce)=24),
    initial_ciphertext BYTEA,
    current_hash BYTEA UNIQUE CHECK (current_hash IS NULL OR octet_length(current_hash)=32),
    previous_hash BYTEA CHECK (previous_hash IS NULL OR octet_length(previous_hash)=32),
    previous_valid_until BIGINT,
    issued_at BIGINT,
    expires_at BIGINT,
    current_request_id TEXT,
    recovery_nonce BYTEA CHECK (recovery_nonce IS NULL OR octet_length(recovery_nonce)=24),
    recovery_ciphertext BYTEA,
    revoked_at BIGINT,
    created_at BIGINT NOT NULL,
    CHECK ((initial_nonce IS NULL)=(initial_ciphertext IS NULL)),
    CHECK ((previous_hash IS NULL)=(previous_valid_until IS NULL)),
    CHECK ((current_hash IS NULL)=(expires_at IS NULL)),
    CHECK ((current_hash IS NULL)=(issued_at IS NULL)),
    CHECK ((current_hash IS NULL)=(current_request_id IS NULL)),
    CHECK ((recovery_nonce IS NULL)=(recovery_ciphertext IS NULL)),
    CHECK (expires_at IS NULL OR expires_at>issued_at),
    CHECK (revoked_at IS NOT NULL OR initial_nonce IS NOT NULL)
);

CREATE TABLE connector_key_requests (
    session_id TEXT NOT NULL REFERENCES connector_sessions(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('EXCHANGE','REFRESH','REVOKE')),
    payload_hash BYTEA NOT NULL CHECK (octet_length(payload_hash)=32),
    result_hash BYTEA NOT NULL CHECK (octet_length(result_hash)=32),
    PRIMARY KEY (session_id,request_id)
);

CREATE TABLE control_instances (
    id TEXT PRIMARY KEY NOT NULL,
    endpoint TEXT NOT NULL,
    draining BOOLEAN NOT NULL DEFAULT FALSE,
    renewed_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    CHECK (expires_at>renewed_at)
);
CREATE INDEX control_expiry ON control_instances(expires_at);

CREATE TABLE provider_connections (
    provider_id TEXT PRIMARY KEY NOT NULL REFERENCES computer_providers(id) ON DELETE CASCADE,
    control_id TEXT NOT NULL REFERENCES control_instances(id),
    connection_id TEXT NOT NULL UNIQUE,
    credential_hash BYTEA NOT NULL CHECK (octet_length(credential_hash)=32),
    expires_at BIGINT NOT NULL
);
CREATE TABLE connector_connections (
    computer_id TEXT PRIMARY KEY NOT NULL REFERENCES computers(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES connector_sessions(id) ON DELETE CASCADE,
    control_id TEXT NOT NULL REFERENCES control_instances(id),
    connection_id TEXT NOT NULL UNIQUE,
    credential_hash BYTEA NOT NULL CHECK (octet_length(credential_hash)=32),
    expires_at BIGINT NOT NULL
);

ALTER TABLE computers ADD CONSTRAINT current_execution_belongs_to_computer
    FOREIGN KEY (current_execution_id,id) REFERENCES computer_executions(id,computer_id)
    DEFERRABLE INITIALLY DEFERRED;
