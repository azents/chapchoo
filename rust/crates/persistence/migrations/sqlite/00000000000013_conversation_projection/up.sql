CREATE TABLE conversation_records (
    conversation_id BLOB NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    record_kind TEXT NOT NULL CHECK(record_kind IN ('USER','ASSISTANT','THOUGHT','TOOL','PLAN','TERMINAL','REQUEST','STATE','NOTICE')),
    native_id TEXT NOT NULL CHECK(length(native_id) BETWEEN 1 AND 512),
    first_ordinal BIGINT NOT NULL CHECK(first_ordinal>0),
    first_index BIGINT NOT NULL CHECK(first_index>=0),
    updated_ordinal BIGINT NOT NULL CHECK(updated_ordinal>=first_ordinal),
    PRIMARY KEY(conversation_id,record_kind,native_id),
    UNIQUE(conversation_id,first_ordinal,first_index)
);
CREATE TABLE conversation_record_fields (
    conversation_id BLOB NOT NULL CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    record_kind TEXT NOT NULL,
    native_id TEXT NOT NULL,
    field_name TEXT NOT NULL CHECK(field_name IN ('CONTENT','TITLE','STATUS','KIND','LOCATIONS','DESCRIPTION','REQUEST','FOREGROUND')),
    content BLOB NOT NULL CHECK(typeof(content)='blob'),
    updated_ordinal BIGINT NOT NULL CHECK(updated_ordinal>0),
    PRIMARY KEY(conversation_id,record_kind,native_id,field_name),
    FOREIGN KEY(conversation_id,record_kind,native_id) REFERENCES conversation_records(conversation_id,record_kind,native_id) ON DELETE CASCADE
);
CREATE TABLE conversation_input_delivery (
    operation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversation_inputs(id) ON DELETE CASCADE CHECK(typeof(operation_id)='blob' AND length(operation_id)=16),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    native_session_id TEXT NOT NULL CHECK(length(native_session_id) BETWEEN 1 AND 512),
    delivery_state TEXT NOT NULL CHECK(delivery_state IN ('POSSIBLE','INSERTED','STEERED','FAILED')),
    native_message_id TEXT,
    CHECK((delivery_state='INSERTED' AND native_message_id IS NOT NULL AND length(native_message_id) BETWEEN 1 AND 512) OR (delivery_state<>'INSERTED' AND native_message_id IS NULL))
);
CREATE TABLE conversation_requests (
    conversation_id BLOB NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    responder_token TEXT NOT NULL CHECK(length(responder_token) BETWEEN 1 AND 128),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    native_session_id TEXT NOT NULL CHECK(length(native_session_id) BETWEEN 1 AND 512),
    ordinal BIGINT NOT NULL CHECK(ordinal>0),
    occurrence_id TEXT NOT NULL CHECK(length(occurrence_id) BETWEEN 1 AND 512),
    request_kind TEXT NOT NULL CHECK(request_kind IN ('PERMISSION','FORM')),
    is_open BOOLEAN NOT NULL CHECK(is_open IN (TRUE,FALSE)),
    PRIMARY KEY(conversation_id,responder_token),
    UNIQUE(conversation_id,occurrence_id),
    UNIQUE(conversation_id,ordinal)
);
CREATE TABLE conversation_answers (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    conversation_id BLOB NOT NULL CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    responder_token TEXT NOT NULL,
    ordinal BIGINT NOT NULL CHECK(ordinal>0),
    occurrence_id TEXT NOT NULL CHECK(length(occurrence_id) BETWEEN 1 AND 512),
    answer BLOB NOT NULL CHECK(typeof(answer)='blob' AND length(answer)>0),
    accepted_at BIGINT NOT NULL,
    delivery_state TEXT NOT NULL CHECK(delivery_state IN ('ACCEPTED','POSSIBLE','RESPONDER_ACCEPTED','REJECTED')),
    UNIQUE(conversation_id,request_id),
    UNIQUE(conversation_id,responder_token),
    FOREIGN KEY(conversation_id,responder_token) REFERENCES conversation_requests(conversation_id,responder_token) ON DELETE CASCADE
);
CREATE TABLE conversation_input_preparation (
    operation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversation_inputs(id) ON DELETE CASCADE CHECK(typeof(operation_id)='blob' AND length(operation_id)=16),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    supply_kind TEXT NOT NULL CHECK(supply_kind IN ('MANAGED','EXISTING')),
    executable_path TEXT,
    selection_request_id TEXT CHECK(selection_request_id IS NULL OR length(selection_request_id) BETWEEN 1 AND 128),
    preparation_state TEXT NOT NULL CHECK(preparation_state IN ('POSSIBLE','READY','FAILED')),
    fresh_native_allowed BOOLEAN NOT NULL CHECK(fresh_native_allowed IN (TRUE,FALSE)),
    ready_snapshot TEXT,
    ready_native_session TEXT,
    failure_kind TEXT CHECK(failure_kind IS NULL OR failure_kind IN ('NOT_ADMITTED','BEFORE_NATIVE_LAUNCH','NATIVE_REJECTED','SOURCE_UNAVAILABLE','RETIREMENT_UNKNOWN')),
    CHECK((supply_kind='MANAGED' AND executable_path IS NULL) OR (supply_kind='EXISTING' AND executable_path IS NOT NULL AND length(executable_path) BETWEEN 1 AND 4096)),
    CHECK((preparation_state='POSSIBLE' AND ready_snapshot IS NULL AND ready_native_session IS NULL AND failure_kind IS NULL)
       OR (preparation_state='READY' AND ready_snapshot IS NOT NULL AND length(ready_snapshot) BETWEEN 1 AND 512 AND ready_native_session IS NOT NULL AND length(ready_native_session) BETWEEN 1 AND 512 AND failure_kind IS NULL)
       OR (preparation_state='FAILED' AND ready_snapshot IS NULL AND ready_native_session IS NULL AND failure_kind IS NOT NULL))
);
CREATE TABLE conversation_native_state (
    conversation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    native_session_id TEXT NOT NULL CHECK(length(native_session_id) BETWEEN 1 AND 512),
    ordinal BIGINT NOT NULL CHECK(ordinal>0),
    occurrence_id TEXT NOT NULL CHECK(length(occurrence_id) BETWEEN 1 AND 512),
    foreground TEXT NOT NULL CHECK(foreground IN ('IDLE','RUNNING','REQUIRES_ACTION','OTHER_ACTIVE')),
    work_id TEXT,
    CHECK((foreground='IDLE' AND work_id IS NULL) OR (foreground<>'IDLE' AND work_id IS NOT NULL AND length(work_id) BETWEEN 1 AND 128))
);
CREATE TABLE conversation_stops (
    id BLOB PRIMARY KEY NOT NULL CHECK(typeof(id)='blob' AND length(id)=16),
    conversation_id BLOB NOT NULL REFERENCES conversations(id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    execution_id BLOB NOT NULL CHECK(typeof(execution_id)='blob' AND length(execution_id)=16),
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    native_session_id TEXT NOT NULL CHECK(length(native_session_id) BETWEEN 1 AND 512),
    work_id TEXT NOT NULL CHECK(length(work_id) BETWEEN 1 AND 128),
    stop_state TEXT NOT NULL CHECK(stop_state IN ('POSSIBLE','CANCEL_PUBLISHED','REJECTED')),
    UNIQUE(conversation_id,request_id)
);
