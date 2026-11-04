-- Opaque IDs are no longer admissible; live keys and canonical records are untouched.
DROP TABLE provider_key_requests;
CREATE TABLE provider_key_requests (
    provider_id TEXT NOT NULL REFERENCES computer_providers(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    payload_hash BYTEA NOT NULL CHECK (octet_length(payload_hash)=32),
    result_hash BYTEA NOT NULL CHECK (octet_length(result_hash)=32),
    retry_expires_at BIGINT NOT NULL CHECK (retry_expires_at>=0),
    PRIMARY KEY (provider_id,request_id)
);
CREATE INDEX provider_key_requests_expiry ON provider_key_requests(provider_id,retry_expires_at);
