CREATE TEMP TABLE external_mcp_reuse_downgrade_guard (active_count INTEGER CHECK (active_count = 0));
INSERT INTO external_mcp_reuse_downgrade_guard
SELECT count(*) FROM conversation_startups s JOIN conversations c ON c.current_preparation_operation_id=s.operation_id
JOIN agents a ON a.id=c.agent_id JOIN computers co ON co.id=a.computer_id
WHERE s.external_selection_reused=TRUE AND s.execution_id=co.current_execution_id;
DROP TABLE external_mcp_reuse_downgrade_guard;
ALTER TABLE conversation_startups DROP COLUMN external_selection_reused;
