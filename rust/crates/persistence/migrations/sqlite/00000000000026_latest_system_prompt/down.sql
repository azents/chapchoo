-- Deleted historical content cannot be reconstructed.
DROP INDEX project_instruction_current_agent;
CREATE INDEX project_instruction_history ON project_instruction_evidence(conversation_id,created_at,operation_id);
