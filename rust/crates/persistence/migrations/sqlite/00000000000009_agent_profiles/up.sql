-- Explicit preparation supplies private identity/profile staging; no aliases survive.

CREATE TEMP TABLE _profile_guard(valid INTEGER NOT NULL CHECK(valid=1));
INSERT INTO _profile_guard SELECT CASE WHEN (SELECT foreign_keys FROM pragma_foreign_keys)=1 AND (SELECT legacy_alter_table FROM pragma_legacy_alter_table)=0 THEN 1 ELSE 0 END;

CREATE TABLE profile_images (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    item_type TEXT NOT NULL CHECK(item_type='AGENT'),
    internal_name TEXT NOT NULL UNIQUE CHECK(length(internal_name)>0),
    source_kind TEXT NOT NULL CHECK(source_kind='BUILTIN'),
    asset_key TEXT NOT NULL UNIQUE CHECK(length(asset_key)>0)
);
INSERT INTO profile_images(id,item_type,internal_name,source_kind,asset_key)
SELECT id,'AGENT',internal_name,'BUILTIN',asset_key FROM _profile_images;

INSERT INTO _profile_guard SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM sqlite_schema s JOIN pragma_foreign_key_list(s.name) fk WHERE s.type='table' AND fk.[table] IN ('users','teams','memberships','personal_contexts','auth_sessions','grants','invitations','login_challenges','computer_providers','provider_keys','provider_key_requests','provider_registration_credentials','computers','agents','agent_creation_receipts','computer_executions','computer_resources','connector_sessions','connector_key_requests','control_instances','provider_connections','connector_connections') AND s.name NOT IN ('users','teams','memberships','personal_contexts','auth_sessions','grants','invitations','login_challenges','computer_providers','provider_keys','provider_key_requests','provider_registration_credentials','computers','agents','agent_creation_receipts','computer_executions','computer_resources','connector_sessions','connector_key_requests','control_instances','provider_connections','connector_connections')) THEN 1 ELSE 0 END;

-- Shipped tables have no triggers. Rebuilding must not silently drop a local
-- trigger (including constraint or backfill guards) attached to this graph.
INSERT INTO _profile_guard SELECT CASE WHEN NOT EXISTS(
    SELECT 1 FROM sqlite_schema WHERE type='trigger' AND tbl_name IN (
        'users','teams','memberships','personal_contexts','auth_sessions','grants',
        'invitations','login_challenges','computer_providers','provider_keys',
        'provider_key_requests','provider_registration_credentials','computers',
        'agents','agent_creation_receipts','computer_executions','computer_resources',
        'connector_sessions','connector_key_requests','control_instances',
        'provider_connections','connector_connections'
    )
) THEN 1 ELSE 0 END;

CREATE TABLE _profile_new_users (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    email_key TEXT NOT NULL UNIQUE,
    delivery_email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0 CHECK (is_admin IN (0, 1)),
    created_at BIGINT NOT NULL
, language TEXT NOT NULL DEFAULT '' CHECK (language IN ('','en','ko','ja','fr')), time_zone TEXT NOT NULL DEFAULT '', region TEXT NOT NULL DEFAULT '' CHECK (region IN ('','en-US','ko-KR','ja-JP','fr-FR')));

CREATE TABLE _profile_new_teams (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('PERSONAL','COLLABORATIVE')),
    name TEXT NOT NULL,
    handle TEXT,
    owner_user_id BLOB CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)) NOT NULL REFERENCES _profile_new_users(id),
    created_at BIGINT NOT NULL,
    deletion_requested_at BIGINT CHECK (deletion_requested_at IS NULL OR kind='COLLABORATIVE'),
    FOREIGN KEY (id,owner_user_id) REFERENCES _profile_new_memberships(team_id,user_id)
        DEFERRABLE INITIALLY DEFERRED
,
    CHECK((kind='PERSONAL' AND handle IS NULL) OR (kind='COLLABORATIVE' AND handle IS NOT NULL AND (instr(handle,char(0))=0 AND length(CAST(handle AS BLOB)) BETWEEN 3 AND 16 AND handle NOT GLOB '*[^a-z0-9-]*' AND substr(handle,1,1) GLOB '[a-z0-9]' AND substr(handle,-1,1) GLOB '[a-z0-9]')))
);

CREATE TABLE _profile_new_memberships (
    team_id BLOB CHECK(team_id IS NULL OR (typeof(team_id)='blob' AND length(team_id)=16)) NOT NULL REFERENCES _profile_new_teams(id) ON DELETE CASCADE,
    user_id BLOB CHECK(user_id IS NULL OR (typeof(user_id)='blob' AND length(user_id)=16)) NOT NULL REFERENCES _profile_new_users(id),
    role TEXT NOT NULL CHECK (role IN ('MANAGER','MEMBER')),
    PRIMARY KEY (team_id,user_id)
);

CREATE TABLE _profile_new_personal_contexts (
    user_id BLOB CHECK(user_id IS NULL OR (typeof(user_id)='blob' AND length(user_id)=16)) PRIMARY KEY NOT NULL REFERENCES _profile_new_users(id),
    team_id BLOB CHECK(team_id IS NULL OR (typeof(team_id)='blob' AND length(team_id)=16)) NOT NULL UNIQUE,
    FOREIGN KEY (team_id,user_id) REFERENCES _profile_new_memberships(team_id,user_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE _profile_new_auth_sessions (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    user_id BLOB CHECK(user_id IS NULL OR (typeof(user_id)='blob' AND length(user_id)=16)) NOT NULL REFERENCES _profile_new_users(id),
    current_hash BLOB NOT NULL UNIQUE CHECK (length(current_hash) = 32),
    previous_hash BLOB CHECK (previous_hash IS NULL OR length(previous_hash) = 32),
    recovery_nonce BLOB NOT NULL CHECK (length(recovery_nonce) = 24),
    recovery_ciphertext BLOB NOT NULL CHECK (length(recovery_ciphertext) = 59),
    created_at BIGINT NOT NULL,
    rotated_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    absolute_expires_at BIGINT,
    previous_valid_until BIGINT,
    revoked_at BIGINT,
    CHECK (expires_at > created_at),
    CHECK (rotated_at >= created_at),
    CHECK (absolute_expires_at IS NULL OR expires_at <= absolute_expires_at),
    CHECK ((previous_hash IS NULL) = (previous_valid_until IS NULL))
);

CREATE TABLE _profile_new_grants (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('SIGNUP', 'PASSWORD_RESET')),
    secret_hash BLOB NOT NULL UNIQUE CHECK (length(secret_hash) = 32),
    email_key TEXT NOT NULL,
    delivery_email TEXT NOT NULL,
    user_id BLOB CHECK(user_id IS NULL OR (typeof(user_id)='blob' AND length(user_id)=16)) REFERENCES _profile_new_users(id),
    issued_by BLOB CHECK(issued_by IS NULL OR (typeof(issued_by)='blob' AND length(issued_by)=16)) REFERENCES _profile_new_users(id),
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT,
    revoked_at BIGINT,
    CHECK (expires_at > issued_at),
    CHECK ((kind = 'SIGNUP' AND user_id IS NULL)
        OR (kind = 'PASSWORD_RESET' AND user_id IS NOT NULL AND issued_by IS NOT NULL))
);

CREATE TABLE _profile_new_invitations (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    team_id BLOB CHECK(team_id IS NULL OR (typeof(team_id)='blob' AND length(team_id)=16)) NOT NULL REFERENCES _profile_new_teams(id) ON DELETE CASCADE,
    recipient_email_key TEXT NOT NULL,
    recipient_delivery_email TEXT NOT NULL,
    invited_by BLOB CHECK(invited_by IS NULL OR (typeof(invited_by)='blob' AND length(invited_by)=16)) NOT NULL REFERENCES _profile_new_users(id),
    status TEXT NOT NULL CHECK (status IN ('PENDING','ACCEPTED','DECLINED')),
    created_at BIGINT NOT NULL,
    processed_at BIGINT,
    CHECK ((status='PENDING')=(processed_at IS NULL))
);

CREATE TABLE _profile_new_login_challenges (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    email_key TEXT NOT NULL,
    binding_hash BLOB NOT NULL UNIQUE CHECK (length(binding_hash) = 32),
    code_verifier BLOB NOT NULL CHECK (length(code_verifier) = 32),
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT,
    CHECK (expires_at > issued_at)
);

CREATE TABLE _profile_new_computer_providers (
    registration_key TEXT NOT NULL UNIQUE,
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    owner_kind TEXT NOT NULL CHECK (owner_kind IN ('SYSTEM','USER','TEAM')),
    system_owner_id TEXT,
    user_owner_id BLOB CHECK(user_owner_id IS NULL OR (typeof(user_owner_id)='blob' AND length(user_owner_id)=16)) REFERENCES _profile_new_users(id),
    owner_team_id BLOB CHECK(owner_team_id IS NULL OR (typeof(owner_team_id)='blob' AND length(owner_team_id)=16)) REFERENCES _profile_new_teams(id),
    name TEXT NOT NULL,
    provider_kind TEXT NOT NULL CHECK (provider_kind IN ('KUBERNETES','DOCKER')),
    docker_daemon_id TEXT,
    can_create BOOLEAN NOT NULL DEFAULT FALSE,
    can_start BOOLEAN NOT NULL DEFAULT FALSE,
    can_stop BOOLEAN NOT NULL DEFAULT FALSE,
    can_restart BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    CHECK ((owner_kind='SYSTEM' AND system_owner_id IS NOT NULL AND user_owner_id IS NULL AND owner_team_id IS NULL)
        OR (owner_kind='USER' AND user_owner_id IS NOT NULL AND system_owner_id IS NULL AND owner_team_id IS NULL)
        OR (owner_kind='TEAM' AND owner_team_id IS NOT NULL AND system_owner_id IS NULL AND user_owner_id IS NULL)),
    CHECK (provider_kind<>'DOCKER' OR owner_kind='SYSTEM'),
    CHECK (docker_daemon_id IS NULL OR (provider_kind='DOCKER'
        AND length(CAST(docker_daemon_id AS BLOB)) BETWEEN 1 AND 1024))
);

CREATE TABLE _profile_new_provider_keys (
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) PRIMARY KEY NOT NULL REFERENCES _profile_new_computer_providers(id) ON DELETE CASCADE,
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

CREATE TABLE _profile_new_provider_key_requests (
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) NOT NULL REFERENCES _profile_new_computer_providers(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    result_hash BLOB NOT NULL CHECK (length(result_hash)=32),
    retry_expires_at BIGINT NOT NULL CHECK (retry_expires_at>=0),
    PRIMARY KEY (provider_id,request_id)
);

CREATE TABLE _profile_new_provider_registration_credentials (
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) PRIMARY KEY NOT NULL REFERENCES _profile_new_computer_providers(id) ON DELETE CASCADE,
    credential_hash BLOB NOT NULL UNIQUE CHECK(length(credential_hash)=32),
    recovery_nonce BLOB NOT NULL CHECK(length(recovery_nonce)=24),
    recovery_ciphertext BLOB NOT NULL CHECK(length(recovery_ciphertext)=59),
    revoked_at BIGINT
);

CREATE TABLE _profile_new_computers (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    owner_user_id BLOB CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)) NOT NULL REFERENCES _profile_new_users(id),
    kind TEXT NOT NULL CHECK (kind IN ('CLOUD','LOCAL')),
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) REFERENCES _profile_new_computer_providers(id),
    desired_state TEXT NOT NULL CHECK (desired_state IN ('RUNNING','STOPPED','DELETING')),
    current_request_id TEXT NOT NULL,
    current_payload_hash BLOB NOT NULL CHECK (length(current_payload_hash)=32),
    current_action TEXT NOT NULL CHECK (current_action IN ('CREATE','START','STOP','RESTART','DELETE')),
    current_phase TEXT NOT NULL CHECK (current_phase IN ('ACCEPTED','RETIRING','STARTING','DELETING','COMPLETED','BLOCKED','UNCERTAIN')),
    initial_start_dispatch_at BIGINT,
    expected_old_execution_id BLOB CHECK(expected_old_execution_id IS NULL OR (typeof(expected_old_execution_id)='blob' AND length(expected_old_execution_id)=16)),
    current_execution_id BLOB CHECK(current_execution_id IS NULL OR (typeof(current_execution_id)='blob' AND length(current_execution_id)=16)),
    observed_power TEXT NOT NULL DEFAULT 'UNKNOWN' CHECK (observed_power IN ('RUNNING','STOPPED','UNKNOWN')),
    power_observed_at BIGINT,
    power_source_connection_id BLOB CHECK(power_source_connection_id IS NULL OR (typeof(power_source_connection_id)='blob' AND length(power_source_connection_id)=16)),
    power_source_execution_id BLOB CHECK(power_source_execution_id IS NULL OR (typeof(power_source_execution_id)='blob' AND length(power_source_execution_id)=16)),
    environment_source_connection_id BLOB CHECK(environment_source_connection_id IS NULL OR (typeof(environment_source_connection_id)='blob' AND length(environment_source_connection_id)=16)),
    environment_observed_at BIGINT,
    saved_environment TEXT NOT NULL DEFAULT 'UNKNOWN' CHECK (saved_environment IN ('VERIFIED','MISSING','UNKNOWN')),
    agent_removed_at BIGINT,
    removed_agent_id BLOB CHECK(removed_agent_id IS NULL OR (typeof(removed_agent_id)='blob' AND length(removed_agent_id)=16)) UNIQUE,
    cleanup_completed_at BIGINT,
    created_at BIGINT NOT NULL, prepare_admitted_at BIGINT,
    UNIQUE (id,owner_user_id,kind),
    UNIQUE (id,provider_id),
    CHECK (kind<>'CLOUD' OR provider_id IS NOT NULL),
    CHECK (agent_removed_at IS NULL OR desired_state='DELETING'),
    CHECK (removed_agent_id IS NULL OR (kind='CLOUD' AND desired_state='DELETING' AND agent_removed_at IS NOT NULL)),
    CHECK (cleanup_completed_at IS NULL OR desired_state='DELETING'),
    FOREIGN KEY (current_execution_id,id) REFERENCES _profile_new_computer_executions(id,computer_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE _profile_new_agents (
    handle TEXT NOT NULL CHECK (instr(handle,char(0))=0 AND length(CAST(handle AS BLOB)) BETWEEN 3 AND 16 AND handle NOT GLOB '*[^a-z0-9-]*' AND substr(handle,1,1) GLOB '[a-z0-9]' AND substr(handle,-1,1) GLOB '[a-z0-9]'),
    profile_image_id BLOB NOT NULL REFERENCES profile_images(id),
    palette_key TEXT NOT NULL CHECK(palette_key IN ('butter','sage','apricot','teal','lilac','rose')),
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    owner_user_id BLOB CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)) NOT NULL REFERENCES _profile_new_users(id),
    team_id BLOB CHECK(team_id IS NULL OR (typeof(team_id)='blob' AND length(team_id)=16)) NOT NULL REFERENCES _profile_new_teams(id),
    name TEXT NOT NULL,
    computer_id BLOB CHECK(computer_id IS NULL OR (typeof(computer_id)='blob' AND length(computer_id)=16)) NOT NULL,
    computer_kind TEXT NOT NULL CHECK (computer_kind IN ('CLOUD','LOCAL')),
    cloud_computer_id BLOB GENERATED ALWAYS AS
        (CASE WHEN computer_kind='CLOUD' THEN computer_id ELSE NULL END) STORED UNIQUE,
    created_at BIGINT NOT NULL,
    FOREIGN KEY (computer_id,owner_user_id,computer_kind)
        REFERENCES _profile_new_computers(id,owner_user_id,kind)
);

CREATE TABLE _profile_new_agent_creation_receipts (
    owner_user_id BLOB CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)) NOT NULL REFERENCES _profile_new_users(id) ON DELETE CASCADE,
    team_id BLOB CHECK(team_id IS NULL OR (typeof(team_id)='blob' AND length(team_id)=16)) NOT NULL,
    request_id TEXT NOT NULL,
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    assigned_agent_id BLOB CHECK(assigned_agent_id IS NULL OR (typeof(assigned_agent_id)='blob' AND length(assigned_agent_id)=16)) NOT NULL UNIQUE,
    PRIMARY KEY (owner_user_id,request_id)
);

CREATE TABLE _profile_new_computer_executions (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    computer_id BLOB CHECK(computer_id IS NULL OR (typeof(computer_id)='blob' AND length(computer_id)=16)) NOT NULL REFERENCES _profile_new_computers(id) ON DELETE CASCADE,
    initial_allocation BOOLEAN NOT NULL,
    running_confirmed_at BIGINT,
    retired_at BIGINT,
    created_at BIGINT NOT NULL,
    UNIQUE (id,computer_id)
);

CREATE TABLE _profile_new_computer_resources (
    allocation_computer_id TEXT NOT NULL CHECK(length(allocation_computer_id)>0),
    allocation_provider_id TEXT NOT NULL CHECK(length(allocation_provider_id)>0),
    allocation_execution_id TEXT,
    computer_id BLOB CHECK(computer_id IS NULL OR (typeof(computer_id)='blob' AND length(computer_id)=16)) NOT NULL,
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('POD','PVC','BOOTSTRAP_SECRET','DOCKER_CONTAINER','DOCKER_VOLUME')),
    native_scope_kind TEXT NOT NULL DEFAULT 'KUBERNETES_NAMESPACE',
    native_scope TEXT NOT NULL,
    name TEXT NOT NULL,
    resource_uid TEXT,
    docker_container_id TEXT,
    docker_volume_metadata TEXT,
    execution_id BLOB CHECK(execution_id IS NULL OR (typeof(execution_id)='blob' AND length(execution_id)=16)),
    presence TEXT NOT NULL CHECK (presence IN ('PRESENT','ABSENT','UNKNOWN')),
    observed_at BIGINT,
    absence_confirmed_at BIGINT,
    source_connection_id BLOB CHECK(source_connection_id IS NULL OR (typeof(source_connection_id)='blob' AND length(source_connection_id)=16)),
    PRIMARY KEY (computer_id,resource_kind,native_scope,name),
    UNIQUE (provider_id,resource_kind,native_scope,name),
    FOREIGN KEY (computer_id,provider_id) REFERENCES _profile_new_computers(id,provider_id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id,computer_id) REFERENCES _profile_new_computer_executions(id,computer_id),
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
,
    CHECK((execution_id IS NULL)=(allocation_execution_id IS NULL))
);

CREATE TABLE _profile_new_connector_sessions (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    computer_id BLOB CHECK(computer_id IS NULL OR (typeof(computer_id)='blob' AND length(computer_id)=16)) NOT NULL,
    execution_id BLOB CHECK(execution_id IS NULL OR (typeof(execution_id)='blob' AND length(execution_id)=16)) NOT NULL UNIQUE,
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
    FOREIGN KEY (execution_id,computer_id) REFERENCES _profile_new_computer_executions(id,computer_id) ON DELETE CASCADE
);

CREATE TABLE _profile_new_connector_key_requests (
    session_id BLOB CHECK(session_id IS NULL OR (typeof(session_id)='blob' AND length(session_id)=16)) NOT NULL REFERENCES _profile_new_connector_sessions(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('EXCHANGE','REFRESH','REVOKE')),
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    result_hash BLOB NOT NULL CHECK (length(result_hash)=32),
    PRIMARY KEY (session_id,request_id)
);

CREATE TABLE _profile_new_control_instances (
    id BLOB CHECK(id IS NULL OR (typeof(id)='blob' AND length(id)=16)) PRIMARY KEY NOT NULL,
    endpoint TEXT NOT NULL,
    draining BOOLEAN NOT NULL DEFAULT FALSE,
    renewed_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    CHECK (expires_at>renewed_at)
);

CREATE TABLE _profile_new_provider_connections (
    provider_id BLOB CHECK(provider_id IS NULL OR (typeof(provider_id)='blob' AND length(provider_id)=16)) PRIMARY KEY NOT NULL REFERENCES _profile_new_computer_providers(id) ON DELETE CASCADE,
    control_id BLOB CHECK(control_id IS NULL OR (typeof(control_id)='blob' AND length(control_id)=16)) NOT NULL REFERENCES _profile_new_control_instances(id),
    connection_id BLOB CHECK(connection_id IS NULL OR (typeof(connection_id)='blob' AND length(connection_id)=16)) NOT NULL UNIQUE,
    credential_hash BLOB NOT NULL CHECK (length(credential_hash)=32),
    expires_at BIGINT NOT NULL
);

CREATE TABLE _profile_new_connector_connections (
    computer_id BLOB CHECK(computer_id IS NULL OR (typeof(computer_id)='blob' AND length(computer_id)=16)) PRIMARY KEY NOT NULL REFERENCES _profile_new_computers(id) ON DELETE CASCADE,
    session_id BLOB CHECK(session_id IS NULL OR (typeof(session_id)='blob' AND length(session_id)=16)) NOT NULL UNIQUE,
    control_id BLOB CHECK(control_id IS NULL OR (typeof(control_id)='blob' AND length(control_id)=16)) NOT NULL REFERENCES _profile_new_control_instances(id),
    connection_id BLOB CHECK(connection_id IS NULL OR (typeof(connection_id)='blob' AND length(connection_id)=16)) NOT NULL UNIQUE,
    credential_hash BLOB NOT NULL CHECK (length(credential_hash)=32),
    expires_at BIGINT NOT NULL,
    FOREIGN KEY (session_id,computer_id) REFERENCES _profile_new_connector_sessions(id,computer_id) ON DELETE CASCADE
);

INSERT INTO _profile_new_users(id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.id),old.email_key,old.delivery_email,old.password_hash,old.is_admin,old.created_at,old.language,old.time_zone,old.region FROM users old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM users)=(SELECT count(*) FROM _profile_new_users) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_teams(id,kind,name,handle,owner_user_id,created_at,deletion_requested_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.id),old.kind,old.name,(SELECT handle FROM _profile_teams WHERE old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.owner_user_id),old.created_at,old.deletion_requested_at FROM teams old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM teams)=(SELECT count(*) FROM _profile_new_teams) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_memberships(team_id,user_id,role) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.team_id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.user_id),old.role FROM memberships old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM memberships)=(SELECT count(*) FROM _profile_new_memberships) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_personal_contexts(user_id,team_id) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.user_id),(SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.team_id) FROM personal_contexts old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM personal_contexts)=(SELECT count(*) FROM _profile_new_personal_contexts) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_auth_sessions(id,user_id,current_hash,previous_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at,absolute_expires_at,previous_valid_until,revoked_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='S' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.user_id),old.current_hash,old.previous_hash,old.recovery_nonce,old.recovery_ciphertext,old.created_at,old.rotated_at,old.expires_at,old.absolute_expires_at,old.previous_valid_until,old.revoked_at FROM auth_sessions old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM auth_sessions)=(SELECT count(*) FROM _profile_new_auth_sessions) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_grants(id,kind,secret_hash,email_key,delivery_email,user_id,issued_by,issued_at,expires_at,consumed_at,revoked_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='G' AND old_id=old.id),old.kind,old.secret_hash,old.email_key,old.delivery_email,(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.user_id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.issued_by),old.issued_at,old.expires_at,old.consumed_at,old.revoked_at FROM grants old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM grants)=(SELECT count(*) FROM _profile_new_grants) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_invitations(id,team_id,recipient_email_key,recipient_delivery_email,invited_by,status,created_at,processed_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='I' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.team_id),old.recipient_email_key,old.recipient_delivery_email,(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.invited_by),old.status,old.created_at,old.processed_at FROM invitations old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM invitations)=(SELECT count(*) FROM _profile_new_invitations) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_login_challenges(id,email_key,binding_hash,code_verifier,issued_at,expires_at,consumed_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='L' AND old_id=old.id),old.email_key,old.binding_hash,old.code_verifier,old.issued_at,old.expires_at,old.consumed_at FROM login_challenges old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM login_challenges)=(SELECT count(*) FROM _profile_new_login_challenges) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_computer_providers(id,owner_kind,system_owner_id,user_owner_id,owner_team_id,name,provider_kind,docker_daemon_id,can_create,can_start,can_stop,can_restart,created_at,registration_key) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.id),old.owner_kind,old.system_owner_id,(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.user_owner_id),(SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.owner_team_id),old.name,old.provider_kind,old.docker_daemon_id,old.can_create,old.can_start,old.can_stop,old.can_restart,old.created_at,old.id FROM computer_providers old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM computer_providers)=(SELECT count(*) FROM _profile_new_computer_providers) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_provider_keys(provider_id,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,revoked_at,current_request_id,recovery_nonce,recovery_ciphertext) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),old.current_hash,old.previous_hash,old.previous_valid_until,old.issued_at,old.expires_at,old.revoked_at,old.current_request_id,old.recovery_nonce,old.recovery_ciphertext FROM provider_keys old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM provider_keys)=(SELECT count(*) FROM _profile_new_provider_keys) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_provider_key_requests(provider_id,request_id,payload_hash,result_hash,retry_expires_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),old.request_id,old.payload_hash,old.result_hash,old.retry_expires_at FROM provider_key_requests old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM provider_key_requests)=(SELECT count(*) FROM _profile_new_provider_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_provider_registration_credentials(provider_id,credential_hash,recovery_nonce,recovery_ciphertext,revoked_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),old.credential_hash,old.recovery_nonce,old.recovery_ciphertext,old.revoked_at FROM provider_registration_credentials old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM provider_registration_credentials)=(SELECT count(*) FROM _profile_new_provider_registration_credentials) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,initial_start_dispatch_at,expected_old_execution_id,current_execution_id,observed_power,power_observed_at,power_source_connection_id,power_source_execution_id,environment_source_connection_id,environment_observed_at,saved_environment,agent_removed_at,removed_agent_id,cleanup_completed_at,created_at,prepare_admitted_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.owner_user_id),old.kind,(SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),old.desired_state,old.current_request_id,old.current_payload_hash,old.current_action,old.current_phase,old.initial_start_dispatch_at,(SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.expected_old_execution_id),(SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.current_execution_id),old.observed_power,old.power_observed_at,(SELECT new_id FROM _profile_id_map WHERE kind='CN' AND old_id=old.power_source_connection_id),(SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.power_source_execution_id),(SELECT new_id FROM _profile_id_map WHERE kind='CN' AND old_id=old.environment_source_connection_id),old.environment_observed_at,old.saved_environment,old.agent_removed_at,(SELECT new_id FROM _profile_id_map WHERE kind='A' AND old_id=old.removed_agent_id),old.cleanup_completed_at,old.created_at,old.prepare_admitted_at FROM computers old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM computers)=(SELECT count(*) FROM _profile_new_computers) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at,handle,profile_image_id,palette_key) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='A' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.owner_user_id),(SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.team_id),old.name,(SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.computer_id),old.computer_kind,old.created_at,(SELECT handle FROM _profile_agents WHERE old_id=old.id),(SELECT profile_image_id FROM _profile_agents WHERE old_id=old.id),(SELECT palette_key FROM _profile_agents WHERE old_id=old.id) FROM agents old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM agents)=(SELECT count(*) FROM _profile_new_agents) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_agent_creation_receipts(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='U' AND old_id=old.owner_user_id),(SELECT new_id FROM _profile_id_map WHERE kind='T' AND old_id=old.team_id),old.request_id,old.payload_hash,(SELECT new_id FROM _profile_id_map WHERE kind='A' AND old_id=old.assigned_agent_id) FROM agent_creation_receipts old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM agent_creation_receipts)=(SELECT count(*) FROM _profile_new_agent_creation_receipts) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_computer_executions(id,computer_id,initial_allocation,running_confirmed_at,retired_at,created_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.computer_id),old.initial_allocation,old.running_confirmed_at,old.retired_at,old.created_at FROM computer_executions old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM computer_executions)=(SELECT count(*) FROM _profile_new_computer_executions) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_computer_resources(computer_id,provider_id,resource_kind,native_scope_kind,native_scope,name,resource_uid,docker_container_id,docker_volume_metadata,execution_id,presence,observed_at,absence_confirmed_at,source_connection_id,allocation_computer_id,allocation_provider_id,allocation_execution_id) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.computer_id),(SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),old.resource_kind,old.native_scope_kind,old.native_scope,old.name,old.resource_uid,old.docker_container_id,old.docker_volume_metadata,(SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.execution_id),old.presence,old.observed_at,old.absence_confirmed_at,(SELECT new_id FROM _profile_id_map WHERE kind='CN' AND old_id=old.source_connection_id),old.computer_id,old.provider_id,old.execution_id FROM computer_resources old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM computer_resources)=(SELECT count(*) FROM _profile_new_computer_resources) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_connector_sessions(id,computer_id,execution_id,initial_request_id,initial_payload_hash,initial_hash,initial_nonce,initial_ciphertext,current_hash,previous_hash,previous_valid_until,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext,revoked_at,created_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='CS' AND old_id=old.id),(SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.computer_id),(SELECT new_id FROM _profile_id_map WHERE kind='E' AND old_id=old.execution_id),old.initial_request_id,old.initial_payload_hash,old.initial_hash,old.initial_nonce,old.initial_ciphertext,old.current_hash,old.previous_hash,old.previous_valid_until,old.issued_at,old.expires_at,old.current_request_id,old.recovery_nonce,old.recovery_ciphertext,old.revoked_at,old.created_at FROM connector_sessions old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM connector_sessions)=(SELECT count(*) FROM _profile_new_connector_sessions) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_connector_key_requests(session_id,request_id,purpose,payload_hash,result_hash) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='CS' AND old_id=old.session_id),old.request_id,old.purpose,old.payload_hash,old.result_hash FROM connector_key_requests old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM connector_key_requests)=(SELECT count(*) FROM _profile_new_connector_key_requests) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_control_instances(id,endpoint,draining,renewed_at,expires_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='CT' AND old_id=old.id),old.endpoint,old.draining,old.renewed_at,old.expires_at FROM control_instances old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM control_instances)=(SELECT count(*) FROM _profile_new_control_instances) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='P' AND old_id=old.provider_id),(SELECT new_id FROM _profile_id_map WHERE kind='CT' AND old_id=old.control_id),(SELECT new_id FROM _profile_id_map WHERE kind='CN' AND old_id=old.connection_id),old.credential_hash,old.expires_at FROM provider_connections old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM provider_connections)=(SELECT count(*) FROM _profile_new_provider_connections) THEN 1 ELSE 0 END;

INSERT INTO _profile_new_connector_connections(computer_id,session_id,control_id,connection_id,credential_hash,expires_at) SELECT (SELECT new_id FROM _profile_id_map WHERE kind='C' AND old_id=old.computer_id),(SELECT new_id FROM _profile_id_map WHERE kind='CS' AND old_id=old.session_id),(SELECT new_id FROM _profile_id_map WHERE kind='CT' AND old_id=old.control_id),(SELECT new_id FROM _profile_id_map WHERE kind='CN' AND old_id=old.connection_id),old.credential_hash,old.expires_at FROM connector_connections old;

INSERT INTO _profile_guard SELECT CASE WHEN (SELECT count(*) FROM connector_connections)=(SELECT count(*) FROM _profile_new_connector_connections) THEN 1 ELSE 0 END;

DROP TABLE connector_connections;

DROP TABLE provider_connections;

DROP TABLE control_instances;

DROP TABLE connector_key_requests;

DROP TABLE connector_sessions;

DROP TABLE computer_resources;

DROP TABLE computer_executions;

DROP TABLE agent_creation_receipts;

DROP TABLE agents;

DROP TABLE computers;

DROP TABLE provider_registration_credentials;

DROP TABLE provider_key_requests;

DROP TABLE provider_keys;

DROP TABLE computer_providers;

DROP TABLE login_challenges;

DROP TABLE invitations;

DROP TABLE grants;

DROP TABLE auth_sessions;

DROP TABLE personal_contexts;

DROP TABLE memberships;

DROP TABLE teams;

DROP TABLE users;

ALTER TABLE _profile_new_users RENAME TO users;

ALTER TABLE _profile_new_teams RENAME TO teams;

ALTER TABLE _profile_new_memberships RENAME TO memberships;

ALTER TABLE _profile_new_personal_contexts RENAME TO personal_contexts;

ALTER TABLE _profile_new_auth_sessions RENAME TO auth_sessions;

ALTER TABLE _profile_new_grants RENAME TO grants;

ALTER TABLE _profile_new_invitations RENAME TO invitations;

ALTER TABLE _profile_new_login_challenges RENAME TO login_challenges;

ALTER TABLE _profile_new_computer_providers RENAME TO computer_providers;

ALTER TABLE _profile_new_provider_keys RENAME TO provider_keys;

ALTER TABLE _profile_new_provider_key_requests RENAME TO provider_key_requests;

ALTER TABLE _profile_new_provider_registration_credentials RENAME TO provider_registration_credentials;

ALTER TABLE _profile_new_computers RENAME TO computers;

ALTER TABLE _profile_new_agents RENAME TO agents;

ALTER TABLE _profile_new_agent_creation_receipts RENAME TO agent_creation_receipts;

ALTER TABLE _profile_new_computer_executions RENAME TO computer_executions;

ALTER TABLE _profile_new_computer_resources RENAME TO computer_resources;

ALTER TABLE _profile_new_connector_sessions RENAME TO connector_sessions;

ALTER TABLE _profile_new_connector_key_requests RENAME TO connector_key_requests;

ALTER TABLE _profile_new_control_instances RENAME TO control_instances;

ALTER TABLE _profile_new_provider_connections RENAME TO provider_connections;

ALTER TABLE _profile_new_connector_connections RENAME TO connector_connections;

CREATE INDEX agents_by_computer ON agents(computer_id);

CREATE INDEX agents_by_owner ON agents(owner_user_id,id);

CREATE INDEX agents_by_team_owner ON agents(team_id,owner_user_id,id);

CREATE INDEX challenges_by_email ON login_challenges(email_key);

CREATE INDEX computers_by_provider ON computers(provider_id);

CREATE INDEX computers_pending_cleanup ON computers(desired_state,cleanup_completed_at);

CREATE INDEX control_expiry ON control_instances(expires_at);

CREATE INDEX executions_by_computer ON computer_executions(computer_id);

CREATE INDEX invitations_by_recipient ON invitations(recipient_email_key,status);

CREATE INDEX invitations_by_team ON invitations(team_id,status);

CREATE UNIQUE INDEX one_personal_team_per_owner ON teams(owner_user_id) WHERE kind='PERSONAL';

CREATE UNIQUE INDEX one_resource_kind_per_execution ON computer_resources(execution_id,resource_kind)
    WHERE execution_id IS NOT NULL;

CREATE UNIQUE INDEX one_saved_environment_per_computer ON computer_resources(computer_id) WHERE resource_kind IN ('PVC','DOCKER_VOLUME');

CREATE INDEX provider_key_requests_expiry ON provider_key_requests(provider_id,retry_expires_at);

CREATE INDEX sessions_by_previous_hash ON auth_sessions(previous_hash);

CREATE INDEX sessions_by_user ON auth_sessions(user_id);

CREATE INDEX team_owned_providers ON computer_providers(owner_team_id) WHERE owner_team_id IS NOT NULL;

CREATE UNIQUE INDEX teams_handle_key ON teams(handle) WHERE deletion_requested_at IS NULL;

CREATE UNIQUE INDEX agents_handle_key ON agents(handle);

INSERT INTO _profile_guard SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM pragma_foreign_key_check) THEN 1 ELSE 0 END;
DROP TABLE _profile_guard;
