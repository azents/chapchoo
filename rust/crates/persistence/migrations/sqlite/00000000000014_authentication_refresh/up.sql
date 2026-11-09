CREATE TABLE authentication_refreshes (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    authentication_id BLOB NOT NULL REFERENCES external_authentications(id) ON DELETE CASCADE CHECK(typeof(authentication_id)='blob' AND length(authentication_id)=16),
    source_nonce BLOB NOT NULL CHECK(typeof(source_nonce)='blob' AND length(source_nonce)=24),
    account_id TEXT NOT NULL CHECK(length(account_id) BETWEEN 1 AND 4096),
    state TEXT NOT NULL CHECK(state IN ('POSSIBLE','NOT_DISPATCHED','REJECTED','UNCERTAIN','UPDATED')),
    created_at BIGINT NOT NULL CHECK(created_at>=0),
    updated_at BIGINT NOT NULL CHECK(updated_at>=created_at),
    UNIQUE(authentication_id,source_nonce)
);
-- Applied-only provenance deliberately does not reference the catalogue:
-- deleting an unsaved applied record must not create a runtime InUse blocker.
CREATE TABLE execution_authentication_applications (
    operation_id BLOB PRIMARY KEY NOT NULL CHECK(typeof(operation_id)='blob' AND length(operation_id)=16),
    execution_id BLOB NOT NULL REFERENCES computer_executions(id) ON DELETE CASCADE CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    agent_id BLOB NOT NULL REFERENCES agents(id) ON DELETE CASCADE CHECK(typeof(agent_id)='blob' AND length(agent_id)=16),
    conversation_id BLOB NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    authentication_id BLOB NOT NULL CHECK(typeof(authentication_id)='blob' AND length(authentication_id)=16),
    selection_request_id TEXT NOT NULL CHECK(length(selection_request_id) BETWEEN 1 AND 128),
    is_current BOOLEAN NOT NULL CHECK(is_current IN (TRUE,FALSE)),
    admitted_at BIGINT NOT NULL CHECK(admitted_at>=0)
);
CREATE UNIQUE INDEX one_current_execution_authentication
    ON execution_authentication_applications(execution_id) WHERE is_current=TRUE;
