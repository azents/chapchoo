-- Backend credential storage only; no Computer credential transport.
CREATE TABLE external_authentications (
    id UUID PRIMARY KEY,
    owner_user_id UUID REFERENCES users(id) ON DELETE RESTRICT,
    owner_team_id UUID REFERENCES teams(id) ON DELETE RESTRICT,
    kind TEXT NOT NULL CHECK(kind IN ('CHATGPT','OPENAI_API_KEY')),
    name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 256),
    account_id TEXT,
    email TEXT,
    plan_type TEXT,
    nonce BYTEA NOT NULL CHECK(octet_length(nonce)=24),
    ciphertext BYTEA NOT NULL CHECK(octet_length(ciphertext) BETWEEN 17 AND 65552),
    expires_at BIGINT,
    current_request_id TEXT NOT NULL CHECK(length(current_request_id) BETWEEN 1 AND 128),
    active_attempt_id UUID,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CHECK((owner_user_id IS NOT NULL AND owner_team_id IS NULL) OR
          (owner_user_id IS NULL AND owner_team_id IS NOT NULL)),
    CHECK(kind='CHATGPT' OR (account_id IS NULL AND email IS NULL AND plan_type IS NULL AND expires_at IS NULL))
);
CREATE INDEX external_auth_user_page ON external_authentications(owner_user_id,id);
CREATE INDEX external_auth_team_page ON external_authentications(owner_team_id,id);

CREATE TABLE external_auth_attempts (
    id UUID PRIMARY KEY,
    initiated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    owner_user_id UUID REFERENCES users(id) ON DELETE RESTRICT,
    owner_team_id UUID REFERENCES teams(id) ON DELETE RESTRICT,
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    target_id UUID REFERENCES external_authentications(id) ON DELETE RESTRICT,
    expected_target_request_id TEXT,
    phase TEXT NOT NULL CHECK(phase IN ('STARTING','PENDING','CHECKING','EXCHANGING','REGISTERED','CANCELLED','EXPIRED','FAILED','SUPERSEDED','UNCERTAIN')),
    step_id TEXT,
    pending_nonce BYTEA,
    pending_ciphertext BYTEA,
    interval_ms BIGINT CHECK(interval_ms BETWEEN 1000 AND 900000),
    next_check_at BIGINT,
    expires_at BIGINT NOT NULL,
    result_authentication_id UUID,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(initiated_by,request_id),
    CHECK((owner_user_id IS NOT NULL AND owner_team_id IS NULL) OR
          (owner_user_id IS NULL AND owner_team_id IS NOT NULL)),
    CHECK((target_id IS NULL AND expected_target_request_id IS NULL) OR
          (target_id IS NOT NULL AND expected_target_request_id IS NOT NULL)),
    CHECK((pending_nonce IS NULL AND pending_ciphertext IS NULL) OR
          (octet_length(pending_nonce)=24 AND octet_length(pending_ciphertext) BETWEEN 17 AND 65552 AND pending_nonce IS NOT NULL AND pending_ciphertext IS NOT NULL)),
    CHECK(phase IN ('STARTING','PENDING','CHECKING','EXCHANGING') OR
          (pending_nonce IS NULL AND pending_ciphertext IS NULL)),
    CHECK(phase NOT IN ('PENDING','CHECKING','EXCHANGING') OR
          (pending_nonce IS NOT NULL AND interval_ms IS NOT NULL AND next_check_at IS NOT NULL)),
    CHECK((phase IN ('STARTING','CHECKING','EXCHANGING') AND step_id IS NOT NULL) OR
          (phase NOT IN ('STARTING','CHECKING','EXCHANGING') AND step_id IS NULL)),
    CHECK((phase='REGISTERED' AND result_authentication_id IS NOT NULL) OR
          (phase<>'REGISTERED' AND result_authentication_id IS NULL))
);
ALTER TABLE external_authentications ADD CONSTRAINT external_auth_active_attempt
    FOREIGN KEY(active_attempt_id) REFERENCES external_auth_attempts(id) DEFERRABLE INITIALLY DEFERRED;
CREATE INDEX external_auth_attempt_scope ON external_auth_attempts(owner_team_id,initiated_by,id);
CREATE INDEX external_auth_attempt_target ON external_auth_attempts(target_id);

-- Results survive catalogue deletion. Assigned identities are historical, not live FKs.
CREATE TABLE external_auth_requests (
    actor_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    operation TEXT NOT NULL CHECK(operation IN ('REGISTER_KEY','REPLACE_KEY','RENAME','DELETE','SELECT','START_DEVICE','CANCEL_DEVICE')),
    fingerprint BYTEA NOT NULL CHECK(octet_length(fingerprint)=32),
    assigned_authentication_id UUID,
    agent_id UUID,
    attempt_id UUID,
    accepted_at BIGINT NOT NULL,
    owner_user_id UUID REFERENCES users(id) ON DELETE RESTRICT,
    owner_team_id UUID REFERENCES teams(id) ON DELETE RESTRICT,
    CHECK((owner_user_id IS NOT NULL AND owner_team_id IS NULL) OR
          (owner_user_id IS NULL AND owner_team_id IS NOT NULL)),
    PRIMARY KEY(actor_user_id,request_id)
);
CREATE TABLE agent_authentication_selections (
    agent_id UUID PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    authentication_id UUID NOT NULL REFERENCES external_authentications(id) ON DELETE RESTRICT,
    current_request_id TEXT NOT NULL CHECK(length(current_request_id) BETWEEN 1 AND 128),
    selected_at BIGINT NOT NULL
);
CREATE INDEX agent_authentication_references ON agent_authentication_selections(authentication_id,agent_id);
