CREATE TEMP TABLE deleted_input_guard (n BIGINT CHECK(n=0));
INSERT INTO deleted_input_guard SELECT count(*) FROM conversation_inputs WHERE deleted_at IS NOT NULL;
DROP TABLE deleted_input_guard;
ALTER TABLE conversation_inputs DROP CONSTRAINT input_deleted_body;
ALTER TABLE conversation_inputs DROP COLUMN deleted_at;
ALTER TABLE conversation_inputs ALTER COLUMN submitted_text SET NOT NULL;
