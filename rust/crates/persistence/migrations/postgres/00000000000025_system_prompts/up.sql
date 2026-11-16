-- Settings and evidence have independent lifetimes from consumable inputs.
CREATE TABLE agent_system_prompts (
 agent_id UUID PRIMARY KEY NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
 content TEXT NOT NULL CHECK(octet_length(content)<=16384)
);
CREATE TABLE project_instruction_evidence (
 operation_id UUID PRIMARY KEY NOT NULL,
 agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
 conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
 execution_id UUID NOT NULL,
 sources_json TEXT NOT NULL,
 rendered_document TEXT NOT NULL,
 owner_content TEXT NOT NULL,
 tool_catalog BYTEA NOT NULL,
 applied_snapshot TEXT,
 applied_native_session TEXT,
 applied_at BIGINT,
 created_at BIGINT NOT NULL CHECK(created_at>=0),
 CHECK((applied_snapshot IS NULL AND applied_native_session IS NULL AND applied_at IS NULL) OR (applied_snapshot IS NOT NULL AND applied_native_session IS NOT NULL AND applied_at IS NOT NULL))
);
CREATE INDEX project_instruction_history ON project_instruction_evidence(conversation_id,created_at,operation_id);
-- Operational selection, reserved before evidence exists and independent of input lifetime.
ALTER TABLE conversations ADD COLUMN current_preparation_operation_id UUID;
