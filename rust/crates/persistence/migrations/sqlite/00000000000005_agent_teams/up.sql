-- Preserve the existing immediate release of a deleted Team's handle. Cleanup
-- bookkeeping is not an independently visible reservation. Rebuild the closed
-- FK group together with foreign keys enabled and without dropping any rows.
CREATE TABLE teams_with_cleanup (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('PERSONAL','COLLABORATIVE')),
    name TEXT NOT NULL,
    handle TEXT NOT NULL,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    created_at BIGINT NOT NULL,
    deletion_requested_at BIGINT CHECK (deletion_requested_at IS NULL OR kind='COLLABORATIVE'),
    FOREIGN KEY (id,owner_user_id) REFERENCES memberships_with_cleanup(team_id,user_id)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE memberships_with_cleanup (
    team_id TEXT NOT NULL REFERENCES teams_with_cleanup(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    role TEXT NOT NULL CHECK (role IN ('MANAGER','MEMBER')),
    PRIMARY KEY (team_id,user_id)
);
CREATE TABLE personal_contexts_with_cleanup (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id),
    team_id TEXT NOT NULL UNIQUE,
    FOREIGN KEY (team_id,user_id) REFERENCES memberships_with_cleanup(team_id,user_id)
        DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE invitations_with_cleanup (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams_with_cleanup(id) ON DELETE CASCADE,
    recipient_email_key TEXT NOT NULL,
    recipient_delivery_email TEXT NOT NULL,
    invited_by TEXT NOT NULL REFERENCES users(id),
    status TEXT NOT NULL CHECK (status IN ('PENDING','ACCEPTED','DECLINED')),
    created_at BIGINT NOT NULL,
    processed_at BIGINT,
    CHECK ((status='PENDING')=(processed_at IS NULL))
);
INSERT INTO teams_with_cleanup(id,kind,name,handle,owner_user_id,created_at)
SELECT id,kind,name,handle,owner_user_id,created_at FROM teams;
INSERT INTO memberships_with_cleanup SELECT team_id,user_id,role FROM memberships;
INSERT INTO personal_contexts_with_cleanup SELECT user_id,team_id FROM personal_contexts;
INSERT INTO invitations_with_cleanup
SELECT id,team_id,recipient_email_key,recipient_delivery_email,invited_by,status,created_at,processed_at FROM invitations;
DROP TABLE invitations;
DROP TABLE personal_contexts;
DROP TABLE memberships;
DROP TABLE teams;
ALTER TABLE teams_with_cleanup RENAME TO teams;
ALTER TABLE memberships_with_cleanup RENAME TO memberships;
ALTER TABLE personal_contexts_with_cleanup RENAME TO personal_contexts;
ALTER TABLE invitations_with_cleanup RENAME TO invitations;
CREATE UNIQUE INDEX teams_handle_key ON teams(handle) WHERE deletion_requested_at IS NULL;
CREATE UNIQUE INDEX one_personal_team_per_owner ON teams(owner_user_id) WHERE kind='PERSONAL';
CREATE INDEX invitations_by_recipient ON invitations(recipient_email_key,status);
CREATE INDEX invitations_by_team ON invitations(team_id,status);

-- Preserve every Agent and its Computer; a missing personal mapping fails the
-- transaction rather than dropping or silently assigning the Agent elsewhere.
CREATE TABLE agents_with_teams (
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
        REFERENCES computers(id,owner_user_id,kind)
);
INSERT INTO agents_with_teams(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at)
SELECT a.id,a.owner_user_id,
    (SELECT pc.team_id FROM personal_contexts pc WHERE pc.user_id=a.owner_user_id),
    a.name,a.computer_id,a.computer_kind,a.created_at
FROM agents a;
DROP TABLE agents;
ALTER TABLE agents_with_teams RENAME TO agents;
CREATE INDEX agents_by_owner ON agents(owner_user_id,id);
CREATE INDEX agents_by_team_owner ON agents(team_id,owner_user_id,id);
CREATE INDEX agents_by_computer ON agents(computer_id);

-- A receipt records the original creation context even after Agent removal or
-- context disposition. Its historical context is not a cascading Team relation.
CREATE TABLE agent_creation_receipts_with_teams (
    owner_user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    team_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    payload_hash BLOB NOT NULL CHECK (length(payload_hash)=32),
    assigned_agent_id TEXT NOT NULL UNIQUE,
    PRIMARY KEY (owner_user_id,request_id)
);
INSERT INTO agent_creation_receipts_with_teams(owner_user_id,team_id,request_id,payload_hash,assigned_agent_id)
SELECT r.owner_user_id,
    (SELECT pc.team_id FROM personal_contexts pc WHERE pc.user_id=r.owner_user_id),
    r.request_id,r.payload_hash,r.assigned_agent_id
FROM agent_creation_receipts r;
DROP TABLE agent_creation_receipts;
ALTER TABLE agent_creation_receipts_with_teams RENAME TO agent_creation_receipts;
