CREATE TYPE team_kind AS ENUM ('PERSONAL', 'COLLABORATIVE');
CREATE TYPE membership_role AS ENUM ('MANAGER', 'MEMBER');
CREATE TYPE grant_kind AS ENUM ('SIGNUP', 'PASSWORD_RESET');
CREATE TYPE invitation_status AS ENUM ('PENDING', 'ACCEPTED', 'DECLINED');

CREATE TABLE installation_key_binding (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    key_check BYTEA NOT NULL CHECK (octet_length(key_check) = 32)
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email_key TEXT NOT NULL UNIQUE,
    delivery_email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL
);

CREATE TABLE teams (
    id TEXT PRIMARY KEY,
    kind team_kind NOT NULL,
    name TEXT NOT NULL,
    handle TEXT NOT NULL UNIQUE,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    created_at BIGINT NOT NULL
);

CREATE UNIQUE INDEX one_personal_team_per_owner ON teams(owner_user_id)
    WHERE kind = 'PERSONAL';

CREATE TABLE memberships (
    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    role membership_role NOT NULL,
    PRIMARY KEY (team_id, user_id)
);

ALTER TABLE teams ADD CONSTRAINT team_owner_membership
    FOREIGN KEY (id, owner_user_id) REFERENCES memberships(team_id, user_id)
    DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE personal_contexts (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    team_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (team_id, user_id) REFERENCES memberships(team_id, user_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE auth_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    current_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(current_hash) = 32),
    previous_hash BYTEA CHECK (previous_hash IS NULL OR octet_length(previous_hash) = 32),
    recovery_nonce BYTEA NOT NULL CHECK (octet_length(recovery_nonce) = 24),
    recovery_ciphertext BYTEA NOT NULL CHECK (octet_length(recovery_ciphertext) = 59),
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

CREATE INDEX sessions_by_previous_hash ON auth_sessions(previous_hash);
CREATE INDEX sessions_by_user ON auth_sessions(user_id);

CREATE TABLE grants (
    id TEXT PRIMARY KEY,
    kind grant_kind NOT NULL,
    secret_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(secret_hash) = 32),
    email_key TEXT NOT NULL,
    delivery_email TEXT NOT NULL,
    user_id TEXT REFERENCES users(id),
    issued_by TEXT REFERENCES users(id),
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT,
    revoked_at BIGINT,
    CHECK (expires_at > issued_at),
    CHECK ((kind = 'SIGNUP' AND user_id IS NULL)
        OR (kind = 'PASSWORD_RESET' AND user_id IS NOT NULL AND issued_by IS NOT NULL))
);

CREATE TABLE invitations (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    recipient_email_key TEXT NOT NULL,
    recipient_delivery_email TEXT NOT NULL,
    invited_by TEXT NOT NULL REFERENCES users(id),
    status invitation_status NOT NULL,
    created_at BIGINT NOT NULL,
    processed_at BIGINT,
    CHECK ((status = 'PENDING') = (processed_at IS NULL))
);

CREATE INDEX invitations_by_recipient ON invitations(recipient_email_key, status);
CREATE INDEX invitations_by_team ON invitations(team_id, status);

CREATE TABLE login_challenges (
    id TEXT PRIMARY KEY,
    email_key TEXT NOT NULL,
    binding_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(binding_hash) = 32),
    code_verifier BYTEA NOT NULL CHECK (octet_length(code_verifier) = 32),
    issued_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    consumed_at BIGINT,
    CHECK (expires_at > issued_at)
);

CREATE INDEX challenges_by_email ON login_challenges(email_key);

CREATE TABLE initial_setup (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    consumed_at BIGINT
);

INSERT INTO initial_setup(singleton, consumed_at) VALUES (1, NULL);
