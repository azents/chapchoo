-- Preserve Provider identity/native authority while adding canonical Team scope.
ALTER TABLE computer_providers
    DROP CONSTRAINT computer_providers_owner_kind_check,
    DROP CONSTRAINT computer_providers_check,
    ADD COLUMN owner_team_id TEXT REFERENCES teams(id),
    ADD CONSTRAINT provider_owner_kind CHECK(owner_kind IN ('SYSTEM','USER','TEAM')),
    ADD CONSTRAINT provider_exclusive_owner CHECK (
        (owner_kind='SYSTEM' AND system_owner_id IS NOT NULL AND user_owner_id IS NULL AND owner_team_id IS NULL)
        OR (owner_kind='USER' AND user_owner_id IS NOT NULL AND system_owner_id IS NULL AND owner_team_id IS NULL)
        OR (owner_kind='TEAM' AND owner_team_id IS NOT NULL AND system_owner_id IS NULL AND user_owner_id IS NULL)
    );
CREATE INDEX team_owned_providers ON computer_providers(owner_team_id) WHERE owner_team_id IS NOT NULL;

CREATE TABLE provider_registration_credentials (
    provider_id TEXT PRIMARY KEY NOT NULL REFERENCES computer_providers(id) ON DELETE CASCADE,
    credential_hash BYTEA NOT NULL UNIQUE CHECK(octet_length(credential_hash)=32),
    recovery_nonce BYTEA NOT NULL CHECK(octet_length(recovery_nonce)=24),
    recovery_ciphertext BYTEA NOT NULL CHECK(octet_length(recovery_ciphertext)=59),
    revoked_at BIGINT
);

-- Atomic upgrade boundary: old operational proof/recovery/route never survives.
-- Receipts, Provider identities, Computers and Connector authority are retained.
UPDATE provider_keys SET revoked_at=COALESCE(revoked_at,issued_at),previous_hash=NULL,previous_valid_until=NULL,recovery_nonce=NULL,recovery_ciphertext=NULL;
DELETE FROM provider_connections;
