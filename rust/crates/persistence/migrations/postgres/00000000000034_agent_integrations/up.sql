CREATE TABLE agent_integrations (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 256),
    kind TEXT NOT NULL CHECK (kind <> '')
);
CREATE INDEX agent_integrations_by_agent ON agent_integrations(agent_id, id);

CREATE TABLE agent_mcp_settings (
    integration_id UUID PRIMARY KEY REFERENCES agent_integrations(id) ON DELETE CASCADE,
    endpoint TEXT NOT NULL CHECK (octet_length(endpoint) BETWEEN 1 AND 4096),
    token_nonce BYTEA,
    token_ciphertext BYTEA,
    CHECK ((token_nonce IS NULL) = (token_ciphertext IS NULL)),
    CHECK (token_nonce IS NULL OR octet_length(token_nonce) = 24),
    CHECK (token_ciphertext IS NULL OR octet_length(token_ciphertext) BETWEEN 17 AND 8208)
);

CREATE TABLE agent_integration_receipts (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL CHECK (char_length(request_id) BETWEEN 1 AND 128),
    payload_fingerprint BYTEA NOT NULL CHECK (octet_length(payload_fingerprint) = 32),
    integration_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('CREATE', 'UPDATE', 'DELETE')),
    accepted_at BIGINT NOT NULL CHECK (accepted_at >= 0),
    PRIMARY KEY (agent_id, request_id)
);
