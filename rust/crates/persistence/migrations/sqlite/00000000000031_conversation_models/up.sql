CREATE TABLE conversation_model_scopes (
    conversation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    source_snapshot TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    harness_kind TEXT NOT NULL,
    authentication_id BLOB NOT NULL CHECK (length(authentication_id)=16),
    api_key_source BLOB CHECK (api_key_source IS NULL OR length(api_key_source)=24),
    launch_operation_id BLOB NOT NULL CHECK (length(launch_operation_id)=16),
    observed_current_id TEXT,
    observed_current_label TEXT,
    advertised_options TEXT,
    observation_order BIGINT NOT NULL CHECK (observation_order>=0),
    saved_model_id TEXT,
    saved_model_label TEXT,
    saved_request_id TEXT,
    recovery_operation_id BLOB,
    CHECK ((observed_current_id IS NULL)=(observed_current_label IS NULL)),
    CHECK ((saved_model_id IS NULL)=(saved_model_label IS NULL)),
    CHECK ((saved_model_id IS NULL)=(saved_request_id IS NULL))
);

CREATE TABLE conversation_model_save_receipts (
    agent_id BLOB NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    conversation_id BLOB NOT NULL CHECK (length(conversation_id)=16),
    source_snapshot TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    launch_operation_id BLOB NOT NULL CHECK (length(launch_operation_id)=16),
    expected_request_id TEXT,
    model_id TEXT NOT NULL,
    model_label TEXT NOT NULL,
    accepted_at BIGINT NOT NULL,
    PRIMARY KEY (agent_id, request_id)
);

CREATE TABLE conversation_model_starts (
    -- Force Removal hides the conversation before proving physical absence.
    conversation_id BLOB NOT NULL CHECK (length(conversation_id)=16),
    operation_id BLOB NOT NULL CHECK (length(operation_id)=16),
    source_snapshot TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    execution_id BLOB NOT NULL CHECK (length(execution_id)=16),
    launch_operation_id BLOB NOT NULL CHECK (length(launch_operation_id)=16),
    connection_id BLOB NOT NULL CHECK (length(connection_id)=16),
    model_id TEXT,
    settlement TEXT CHECK (settlement IN ('INSERTED','NOT_PUBLISHED','MODEL_UNAVAILABLE','RETIRED')),
    native_message_id TEXT,
    CHECK ((native_message_id IS NOT NULL)=(settlement IS NOT NULL AND settlement='INSERTED')),
    PRIMARY KEY (conversation_id, operation_id)
);
CREATE UNIQUE INDEX conversation_model_one_pending_start ON conversation_model_starts(conversation_id)
    WHERE settlement IS NULL;
CREATE INDEX conversation_model_pending_execution ON conversation_model_starts(execution_id)
    WHERE settlement IS NULL;
