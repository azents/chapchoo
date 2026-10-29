CREATE TABLE installation_key_binding (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    key_check BLOB NOT NULL CHECK (length(key_check) = 32)
);

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    email_key TEXT NOT NULL UNIQUE,
    delivery_email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0 CHECK (is_admin IN (0, 1)),
    created_at BIGINT NOT NULL
);

CREATE TABLE teams (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('PERSONAL', 'COLLABORATIVE')),
    name TEXT NOT NULL,
    handle TEXT NOT NULL UNIQUE,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    created_at BIGINT NOT NULL,
    FOREIGN KEY (id, owner_user_id) REFERENCES memberships(team_id, user_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX one_personal_team_per_owner ON teams(owner_user_id)
    WHERE kind = 'PERSONAL';

CREATE TABLE memberships (
    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    role TEXT NOT NULL CHECK (role IN ('MANAGER', 'MEMBER')),
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE personal_contexts (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id),
    team_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (team_id, user_id) REFERENCES memberships(team_id, user_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE auth_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
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

CREATE INDEX sessions_by_previous_hash ON auth_sessions(previous_hash);
CREATE INDEX sessions_by_user ON auth_sessions(user_id);

CREATE TABLE grants (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('SIGNUP', 'PASSWORD_RESET')),
    secret_hash BLOB NOT NULL UNIQUE CHECK (length(secret_hash) = 32),
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
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    recipient_email_key TEXT NOT NULL,
    recipient_delivery_email TEXT NOT NULL,
    invited_by TEXT NOT NULL REFERENCES users(id),
    status TEXT NOT NULL CHECK (status IN ('PENDING', 'ACCEPTED', 'DECLINED')),
    created_at BIGINT NOT NULL,
    processed_at BIGINT,
    CHECK ((status = 'PENDING') = (processed_at IS NULL))
);

CREATE INDEX invitations_by_recipient ON invitations(recipient_email_key, status);
CREATE INDEX invitations_by_team ON invitations(team_id, status);

CREATE TABLE login_challenges (
    id TEXT PRIMARY KEY NOT NULL,
    email_key TEXT NOT NULL,
    binding_hash BLOB NOT NULL UNIQUE CHECK (length(binding_hash) = 32),
    code_verifier BLOB NOT NULL CHECK (length(code_verifier) = 32),
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
