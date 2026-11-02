ALTER TABLE teams ADD COLUMN deletion_requested_at BIGINT
    CHECK (deletion_requested_at IS NULL OR kind='COLLABORATIVE');
ALTER TABLE teams DROP CONSTRAINT teams_handle_key;
CREATE UNIQUE INDEX teams_handle_key ON teams(handle) WHERE deletion_requested_at IS NULL;
ALTER TABLE agents ADD COLUMN team_id TEXT REFERENCES teams(id);
UPDATE agents a SET team_id=(SELECT pc.team_id FROM personal_contexts pc WHERE pc.user_id=a.owner_user_id);
ALTER TABLE agents ALTER COLUMN team_id SET NOT NULL;
CREATE INDEX agents_by_team_owner ON agents(team_id,owner_user_id,id);

-- Original request context survives Agent/Team removal; it is part of receipt
-- identity, not an access grant or a cascading Team relation.
ALTER TABLE agent_creation_receipts ADD COLUMN team_id TEXT;
UPDATE agent_creation_receipts r SET team_id=(SELECT pc.team_id FROM personal_contexts pc WHERE pc.user_id=r.owner_user_id);
ALTER TABLE agent_creation_receipts ALTER COLUMN team_id SET NOT NULL;
