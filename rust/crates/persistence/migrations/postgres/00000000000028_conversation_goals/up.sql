CREATE TABLE conversation_goals (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    goal_key TEXT NOT NULL,
    original_objective TEXT NOT NULL,
    objective TEXT NOT NULL,
    disposition TEXT NOT NULL CHECK (disposition IN ('active', 'waiting_for_input', 'completed')),
    PRIMARY KEY (conversation_id, goal_key)
);
CREATE TABLE conversation_continuation_counts (
    conversation_id UUID PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    admissions BIGINT NOT NULL CHECK (admissions BETWEEN 0 AND 3)
);
CREATE TABLE conversation_ending_decisions (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    execution_id UUID NOT NULL REFERENCES computer_executions(id) ON DELETE CASCADE,
    launch_operation_id UUID NOT NULL,
    source_snapshot TEXT NOT NULL,
    native_session_id TEXT NOT NULL,
    ordinal BIGINT NOT NULL CHECK (ordinal>0),
    occurrence_id TEXT NOT NULL,
    work_id TEXT NOT NULL,
    PRIMARY KEY (conversation_id, execution_id, source_snapshot, occurrence_id),
    UNIQUE (conversation_id, execution_id, source_snapshot, work_id)
);
CREATE TABLE conversation_launch_captures (
    operation_id UUID PRIMARY KEY REFERENCES execution_authentication_applications(operation_id) ON DELETE CASCADE,
    source_snapshot TEXT NOT NULL,
    retained_ordinal BIGINT NOT NULL,
    retained_occurrence_id TEXT,
    CHECK ((retained_ordinal=0 AND retained_occurrence_id IS NULL)
        OR (retained_ordinal>0 AND retained_occurrence_id IS NOT NULL))
);
CREATE TABLE conversation_continuation_inputs (
    conversation_id UUID NOT NULL,
    execution_id UUID NOT NULL,
    source_snapshot TEXT NOT NULL,
    occurrence_id TEXT NOT NULL,
    id UUID NOT NULL UNIQUE,
    delivery_state TEXT NOT NULL DEFAULT 'ADMITTED' CHECK (delivery_state IN ('ADMITTED', 'POSSIBLE', 'CLOSED')),
    PRIMARY KEY (conversation_id, execution_id, source_snapshot, occurrence_id),
    FOREIGN KEY (conversation_id, execution_id, source_snapshot, occurrence_id)
        REFERENCES conversation_ending_decisions(conversation_id, execution_id, source_snapshot, occurrence_id)
        ON DELETE CASCADE
);
