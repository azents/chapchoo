-- Keep only the current Agent conversation's causally selected payload.
DELETE FROM project_instruction_evidence
WHERE NOT EXISTS (
 SELECT 1 FROM agent_conversations a
 JOIN conversations c ON c.id=a.conversation_id AND c.agent_id=a.agent_id
 WHERE a.agent_id=project_instruction_evidence.agent_id
 AND c.id=project_instruction_evidence.conversation_id
 AND c.current_preparation_operation_id=project_instruction_evidence.operation_id
);
DROP INDEX project_instruction_history;
CREATE UNIQUE INDEX project_instruction_current_agent ON project_instruction_evidence(agent_id);
