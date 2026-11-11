-- Connection startup is not an input. Existing input and preparation rows stay unchanged.
CREATE TABLE conversation_startups (
    operation_id BLOB PRIMARY KEY NOT NULL CHECK(typeof(operation_id)='blob' AND length(operation_id)=16),
    conversation_id BLOB NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    supply_kind TEXT NOT NULL CHECK(supply_kind IN ('MANAGED','EXISTING')),
    executable_path TEXT,
    selection_request_id TEXT CHECK(selection_request_id IS NULL OR length(selection_request_id) BETWEEN 1 AND 128),
    preparation_state TEXT NOT NULL CHECK(preparation_state IN ('POSSIBLE','READY','FAILED')),
    fresh_native_allowed BOOLEAN NOT NULL CHECK(fresh_native_allowed IN (TRUE,FALSE)),
    require_retained_source BOOLEAN NOT NULL CHECK(require_retained_source IN (TRUE,FALSE)),
    intended_snapshot TEXT,
    intended_native_session TEXT,
    ready_snapshot TEXT,
    ready_native_session TEXT,
    ready_connection_id BLOB CHECK(ready_connection_id IS NULL OR (typeof(ready_connection_id)='blob' AND length(ready_connection_id)=16)),
    failure_kind TEXT CHECK(failure_kind IS NULL OR failure_kind IN ('NOT_ADMITTED','BEFORE_NATIVE_LAUNCH','NATIVE_REJECTED','SOURCE_UNAVAILABLE','RETIREMENT_UNKNOWN')),
    admitted_at BIGINT NOT NULL,
    CHECK((supply_kind='MANAGED' AND executable_path IS NULL) OR (supply_kind='EXISTING' AND executable_path IS NOT NULL AND length(executable_path) BETWEEN 1 AND 4096)),
    CHECK((intended_snapshot IS NULL AND intended_native_session IS NULL) OR (intended_snapshot IS NOT NULL AND length(intended_snapshot) BETWEEN 1 AND 512 AND intended_native_session IS NOT NULL AND length(intended_native_session) BETWEEN 1 AND 512)),
    CHECK((fresh_native_allowed=TRUE AND intended_snapshot IS NULL) OR (fresh_native_allowed=FALSE AND intended_snapshot IS NOT NULL)),
    CHECK((preparation_state='READY' AND ready_connection_id IS NOT NULL) OR (preparation_state<>'READY' AND ready_connection_id IS NULL)),
    CHECK((preparation_state='POSSIBLE' AND ready_snapshot IS NULL AND ready_native_session IS NULL AND failure_kind IS NULL)
       OR (preparation_state='READY' AND ready_snapshot IS NOT NULL AND length(ready_snapshot) BETWEEN 1 AND 512 AND ready_native_session IS NOT NULL AND length(ready_native_session) BETWEEN 1 AND 512 AND failure_kind IS NULL)
       OR (preparation_state='FAILED' AND ready_snapshot IS NULL AND ready_native_session IS NULL AND failure_kind IS NOT NULL))
);
CREATE UNIQUE INDEX conversation_startups_one_possible ON conversation_startups(conversation_id) WHERE preparation_state='POSSIBLE';
CREATE INDEX conversation_startups_by_conversation ON conversation_startups(conversation_id,admitted_at);
