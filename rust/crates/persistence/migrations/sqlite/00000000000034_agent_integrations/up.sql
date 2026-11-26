CREATE TABLE agent_integrations (
    id BLOB PRIMARY KEY NOT NULL CHECK (typeof(id) = 'blob' AND length(id) = 16),
    agent_id BLOB NOT NULL REFERENCES agents(id) ON DELETE CASCADE
        CHECK (typeof(agent_id) = 'blob' AND length(agent_id) = 16),
    display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 256),
    kind TEXT NOT NULL CHECK (kind <> '')
);
CREATE INDEX agent_integrations_by_agent ON agent_integrations(agent_id, id);

CREATE TABLE agent_mcp_settings (
    integration_id BLOB PRIMARY KEY NOT NULL REFERENCES agent_integrations(id) ON DELETE CASCADE
        CHECK (typeof(integration_id) = 'blob' AND length(integration_id) = 16),
    endpoint TEXT NOT NULL CHECK (length(CAST(endpoint AS BLOB)) BETWEEN 1 AND 4096),
    token_nonce BLOB CHECK (token_nonce IS NULL OR (typeof(token_nonce) = 'blob' AND length(token_nonce) = 24)),
    token_ciphertext BLOB CHECK (token_ciphertext IS NULL OR (typeof(token_ciphertext) = 'blob' AND length(token_ciphertext) BETWEEN 17 AND 8208)),
    CHECK ((token_nonce IS NULL) = (token_ciphertext IS NULL))
);

CREATE TABLE agent_integration_receipts (
    agent_id BLOB NOT NULL REFERENCES agents(id) ON DELETE CASCADE
        CHECK (typeof(agent_id) = 'blob' AND length(agent_id) = 16),
    request_id TEXT NOT NULL CHECK (length(request_id) BETWEEN 1 AND 128),
    payload_fingerprint BLOB NOT NULL CHECK (typeof(payload_fingerprint) = 'blob' AND length(payload_fingerprint) = 32),
    integration_id BLOB NOT NULL CHECK (typeof(integration_id) = 'blob' AND length(integration_id) = 16),
    action TEXT NOT NULL CHECK (action IN ('CREATE', 'UPDATE', 'DELETE')),
    accepted_at BIGINT NOT NULL CHECK (accepted_at >= 0),
    PRIMARY KEY (agent_id, request_id)
);
