CREATE TEMP TABLE mcp_key_downgrade_guard (occupied INTEGER CHECK (occupied = 0));
INSERT INTO mcp_key_downgrade_guard SELECT EXISTS (SELECT 1 FROM agent_mcp_keys);
DROP TABLE mcp_key_downgrade_guard;
DROP TABLE agent_mcp_keys;
