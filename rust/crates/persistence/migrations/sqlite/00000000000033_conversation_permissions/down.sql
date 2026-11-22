-- Copies reject permission-only receipts rather than discarding them.
CREATE TABLE conversation_model_scopes_prior (
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
    saved_tuning TEXT,
    CHECK ((observed_current_id IS NULL)=(observed_current_label IS NULL)),
    CHECK ((saved_model_id IS NULL)=(saved_model_label IS NULL)),
    CHECK ((saved_model_id IS NULL)=(saved_request_id IS NULL))
);
INSERT INTO conversation_model_scopes_prior SELECT * FROM conversation_model_scopes;
DROP TABLE conversation_model_scopes;
ALTER TABLE conversation_model_scopes_prior RENAME TO conversation_model_scopes;

CREATE TABLE conversation_model_save_receipts_prior (
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
    tuning TEXT,
    PRIMARY KEY (agent_id, request_id)
);
INSERT INTO conversation_model_save_receipts_prior SELECT * FROM conversation_model_save_receipts;
DROP TABLE conversation_model_save_receipts;
ALTER TABLE conversation_model_save_receipts_prior RENAME TO conversation_model_save_receipts;
