CREATE TABLE agent_mcp_keys (
    agent_id BLOB NOT NULL REFERENCES agents(id) ON DELETE CASCADE
        CHECK (typeof(agent_id) = 'blob' AND length(agent_id) = 16),
    integration_id BLOB NOT NULL CHECK (typeof(integration_id) = 'blob' AND length(integration_id) = 16),
    slot INTEGER NOT NULL CHECK (typeof(slot) = 'integer' AND slot BETWEEN 1 AND 9999999999),
    PRIMARY KEY (agent_id, integration_id),
    UNIQUE (agent_id, slot)
);
