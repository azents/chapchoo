-- Downgrade must not discard saved settings or supplied originals.
CREATE TEMP TABLE instruction_downgrade_guard(n BIGINT CHECK(n=0));
INSERT INTO instruction_downgrade_guard SELECT (SELECT count(*) FROM agent_system_prompts)+(SELECT count(*) FROM project_instruction_evidence)+(SELECT count(*) FROM conversations WHERE current_preparation_operation_id IS NOT NULL);
DROP TABLE instruction_downgrade_guard;
ALTER TABLE conversations DROP COLUMN current_preparation_operation_id;
DROP TABLE project_instruction_evidence;
DROP TABLE agent_system_prompts;
