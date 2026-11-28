CREATE TABLE agent_mcp_keys (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    integration_id UUID NOT NULL,
    slot BIGINT NOT NULL CHECK (slot BETWEEN 1 AND 9999999999),
    PRIMARY KEY (agent_id, integration_id),
    UNIQUE (agent_id, slot)
);
