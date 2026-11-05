-- Backend credential storage only; no Computer credential transport.
CREATE TABLE external_authentications (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    owner_user_id BLOB REFERENCES users(id) ON DELETE RESTRICT CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)),
    owner_team_id BLOB REFERENCES teams(id) ON DELETE RESTRICT CHECK(owner_team_id IS NULL OR (typeof(owner_team_id)='blob' AND length(owner_team_id)=16)),
    kind TEXT NOT NULL CHECK(kind IN ('CHATGPT','OPENAI_API_KEY')),
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 256),
    account_id TEXT,
    email TEXT,
    plan_type TEXT,
    nonce BLOB NOT NULL CHECK(typeof(nonce)='blob' AND length(nonce)=24),
    ciphertext BLOB NOT NULL CHECK(typeof(ciphertext)='blob' AND length(ciphertext) BETWEEN 17 AND 65552),
    expires_at BIGINT,
    current_request_id TEXT NOT NULL CHECK(length(current_request_id) BETWEEN 1 AND 128),
    active_attempt_id BLOB REFERENCES external_auth_attempts(id) DEFERRABLE INITIALLY DEFERRED CHECK(active_attempt_id IS NULL OR (typeof(active_attempt_id)='blob' AND length(active_attempt_id)=16)),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CHECK((owner_user_id IS NOT NULL AND owner_team_id IS NULL) OR
          (owner_user_id IS NULL AND owner_team_id IS NOT NULL)),
    CHECK(kind='CHATGPT' OR (account_id IS NULL AND email IS NULL AND plan_type IS NULL AND expires_at IS NULL))
);
CREATE INDEX external_auth_user_page ON external_authentications(owner_user_id,id);
CREATE INDEX external_auth_team_page ON external_authentications(owner_team_id,id);

CREATE TABLE external_auth_attempts (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    initiated_by BLOB NOT NULL REFERENCES users(id) ON DELETE RESTRICT CHECK(typeof(initiated_by)='blob' AND length(initiated_by)=16),
    owner_user_id BLOB REFERENCES users(id) ON DELETE RESTRICT CHECK(owner_user_id IS NULL OR (typeof(owner_user_id)='blob' AND length(owner_user_id)=16)),
    owner_team_id BLOB REFERENCES teams(id) ON DELETE RESTRICT CHECK(owner_team_id IS NULL OR (typeof(owner_team_id)='blob' AND length(owner_team_id)=16)),
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    target_id BLOB REFERENCES external_authentications(id) ON DELETE RESTRICT CHECK(target_id IS NULL OR (typeof(target_id)='blob' AND length(target_id)=16)),
    expected_target_request_id TEXT,
    phase TEXT NOT NULL CHECK(phase IN ('STARTING','PENDING','CHECKING','EXCHANGING','REGISTERED','CANCELLED','EXPIRED','FAILED','SUPERSEDED','UNCERTAIN')),
    step_id TEXT,
    pending_nonce BLOB CHECK(pending_nonce IS NULL OR typeof(pending_nonce)='blob'),
    pending_ciphertext BLOB CHECK(pending_ciphertext IS NULL OR typeof(pending_ciphertext)='blob'),
    interval_ms BIGINT CHECK(interval_ms BETWEEN 1000 AND 900000),
    next_check_at BIGINT,
    expires_at BIGINT NOT NULL,
    result_authentication_id BLOB CHECK(result_authentication_id IS NULL OR typeof(result_authentication_id)='blob'),
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(initiated_by,request_id),
    CHECK((owner_user_id IS NOT NULL AND owner_team_id IS NULL) OR
          (owner_user_id IS NULL AND owner_team_id IS NOT NULL)),
    CHECK((target_id IS NULL AND expected_target_request_id IS NULL) OR
          (target_id IS NOT NULL AND expected_target_request_id IS NOT NULL)),
    CHECK((pending_nonce IS NULL AND pending_ciphertext IS NULL) OR
          (length(pending_nonce)=24 AND length(pending_ciphertext) BETWEEN 17 AND 65552 AND pending_nonce IS NOT NULL AND pending_ciphertext IS NOT NULL)),
    CHECK(phase IN ('STARTING','PENDING','CHECKING','EXCHANGING') OR
          (pending_nonce IS NULL AND pending_ciphertext IS NULL)),
    CHECK(phase NOT IN ('PENDING','CHECKING','EXCHANGING') OR
          (pending_nonce IS NOT NULL AND interval_ms IS NOT NULL AND next_check_at IS NOT NULL)),
    CHECK((phase IN ('STARTING','CHECKING','EXCHANGING') AND step_id IS NOT NULL) OR
          (phase NOT IN ('STARTING','CHECKING','EXCHANGING') AND step_id IS NULL)),
    CHECK((phase='REGISTERED' AND result_authentication_id IS NOT NULL AND length(result_authentication_id)=16) OR
          (phase<>'REGISTERED' AND result_authentication_id IS NULL))
);
CREATE INDEX external_auth_attempt_scope ON external_auth_attempts(owner_team_id,initiated_by,id);
CREATE INDEX external_auth_attempt_target ON external_auth_attempts(target_id);

-- Results survive catalogue deletion. Assigned identities are historical, not live FKs.
CREATE TABLE external_auth_requests (
    actor_user_id BLOB NOT NULL REFERENCES users(id) ON DELETE RESTRICT CHECK(typeof(actor_user_id)='blob' AND length(actor_user_id)=16),
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    operation TEXT NOT NULL CHECK(operation IN ('REGISTER_KEY','REPLACE_KEY','RENAME','DELETE','SELECT','START_DEVICE','CANCEL_DEVICE')),
    fingerprint BLOB NOT NULL CHECK(typeof(fingerprint)='blob' AND length(fingerprint)=32),
    assigned_authentication_id BLOB CHECK(assigned_authentication_id IS NULL OR (typeof(assigned_authentication_id)='blob' AND length(assigned_authentication_id)=16)),
    agent_id BLOB CHECK(agent_id IS NULL OR (typeof(agent_id)='blob' AND length(agent_id)=16)),
    attempt_id BLOB CHECK(attempt_id IS NULL OR (typeof(attempt_id)='blob' AND length(attempt_id)=16)),
    accepted_at BIGINT NOT NULL,
    PRIMARY KEY(actor_user_id,request_id)
);
CREATE TABLE agent_authentication_selections (
    agent_id BLOB PRIMARY KEY NOT NULL REFERENCES agents(id) ON DELETE CASCADE CHECK(typeof(agent_id)='blob' AND length(agent_id)=16),
    authentication_id BLOB NOT NULL REFERENCES external_authentications(id) ON DELETE RESTRICT CHECK(typeof(authentication_id)='blob' AND length(authentication_id)=16),
    current_request_id TEXT NOT NULL CHECK(length(current_request_id) BETWEEN 1 AND 128),
    selected_at BIGINT NOT NULL
);
CREATE INDEX agent_authentication_references ON agent_authentication_selections(authentication_id,agent_id);
