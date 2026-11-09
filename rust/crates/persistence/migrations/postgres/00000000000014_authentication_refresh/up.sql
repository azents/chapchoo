CREATE TABLE authentication_refreshes (
    id UUID PRIMARY KEY NOT NULL,
    authentication_id UUID NOT NULL REFERENCES external_authentications(id) ON DELETE CASCADE,
    source_nonce BYTEA NOT NULL CHECK(length(source_nonce)=24),
    account_id TEXT NOT NULL CHECK(length(account_id) BETWEEN 1 AND 4096),
    state TEXT NOT NULL CHECK(state IN ('POSSIBLE','NOT_DISPATCHED','REJECTED','UNCERTAIN','UPDATED')),
    created_at BIGINT NOT NULL CHECK(created_at>=0),
    updated_at BIGINT NOT NULL CHECK(updated_at>=created_at),
    UNIQUE(authentication_id,source_nonce)
);
-- Retain applied provenance after catalogue deletion without blocking deletion.
CREATE TABLE execution_authentication_applications (
    operation_id UUID PRIMARY KEY NOT NULL,
    execution_id UUID NOT NULL REFERENCES computer_executions(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    authentication_id UUID NOT NULL,
    selection_request_id TEXT NOT NULL CHECK(length(selection_request_id) BETWEEN 1 AND 128),
    is_current BOOLEAN NOT NULL,
    admitted_at BIGINT NOT NULL CHECK(admitted_at>=0)
);
CREATE UNIQUE INDEX one_current_execution_authentication
    ON execution_authentication_applications(execution_id) WHERE is_current=TRUE;
