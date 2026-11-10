-- Conversation state is additive; existing identity and creation receipts stay unchanged.
CREATE TABLE agent_harness_supplies (
    agent_id UUID PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    supply_kind TEXT NOT NULL CHECK(supply_kind IN ('MANAGED','EXISTING')),
    executable_path TEXT,
    current_request_id TEXT NOT NULL CHECK(length(current_request_id) BETWEEN 1 AND 128),
    selected_at BIGINT NOT NULL,
    CHECK((supply_kind='MANAGED' AND executable_path IS NULL) OR
          (supply_kind='EXISTING' AND executable_path IS NOT NULL AND length(executable_path) BETWEEN 1 AND 4096))
);
CREATE TABLE agent_harness_supply_requests (
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    expected_request_id TEXT,
    supply_kind TEXT NOT NULL CHECK(supply_kind IN ('MANAGED','EXISTING')),
    executable_path TEXT,
    accepted_at BIGINT NOT NULL,
    PRIMARY KEY(agent_id,request_id),
    CHECK((supply_kind='MANAGED' AND executable_path IS NULL) OR
          (supply_kind='EXISTING' AND executable_path IS NOT NULL AND length(executable_path) BETWEEN 1 AND 4096))
);
CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    native_session_id TEXT,
    source_snapshot TEXT,
    source_state TEXT NOT NULL CHECK(source_state IN ('NOT_STARTED','LOADING','READY')),
    source_ordinal BIGINT NOT NULL DEFAULT 0 CHECK(source_ordinal>=0),
    source_event_id TEXT,
    created_at BIGINT NOT NULL,
    CHECK((source_state='NOT_STARTED' AND source_snapshot IS NULL AND native_session_id IS NULL AND source_ordinal=0 AND source_event_id IS NULL) OR
          (source_state IN ('LOADING','READY') AND source_snapshot IS NOT NULL AND length(source_snapshot) BETWEEN 1 AND 512 AND native_session_id IS NOT NULL AND length(native_session_id) BETWEEN 1 AND 512)),
    CHECK((source_ordinal=0 AND source_event_id IS NULL) OR
          (source_ordinal>0 AND source_event_id IS NOT NULL AND length(source_event_id) BETWEEN 1 AND 512)),
    UNIQUE(agent_id,id)
);
-- Only the initial selected conversation is unique, not every conversation per Agent.
CREATE TABLE agent_conversations (
    agent_id UUID PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL UNIQUE,
    FOREIGN KEY(agent_id,conversation_id) REFERENCES conversations(agent_id,id) ON DELETE CASCADE
);
CREATE TABLE conversation_inputs (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    target_snapshot TEXT,
    submitted_text TEXT NOT NULL CHECK(length(submitted_text)>0),
    accepted_at BIGINT NOT NULL,
    UNIQUE(conversation_id,request_id)
);
CREATE TABLE conversation_event_pairs (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    source_snapshot TEXT NOT NULL CHECK(length(source_snapshot) BETWEEN 1 AND 512),
    ordinal BIGINT NOT NULL CHECK(ordinal>0),
    occurrence_id TEXT NOT NULL CHECK(length(occurrence_id) BETWEEN 1 AND 512),
    raw_event BYTEA NOT NULL CHECK(octet_length(raw_event)>0),
    normalized_event BYTEA NOT NULL CHECK(octet_length(normalized_event)>0),
    PRIMARY KEY(conversation_id,occurrence_id),
    UNIQUE(conversation_id,ordinal)
);
