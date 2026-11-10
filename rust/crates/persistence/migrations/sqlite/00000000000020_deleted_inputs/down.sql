CREATE TEMP TABLE deleted_input_guard (n BIGINT CHECK(n=0));
INSERT INTO deleted_input_guard SELECT count(*) FROM conversation_inputs WHERE deleted_at IS NOT NULL;
DROP TABLE deleted_input_guard;
ALTER TABLE conversation_inputs DROP COLUMN deleted_at;
CREATE TEMP TABLE input_body_copy AS SELECT id,submitted_text FROM conversation_inputs;
ALTER TABLE conversation_inputs DROP COLUMN submitted_text;
ALTER TABLE conversation_inputs ADD COLUMN submitted_text TEXT NOT NULL DEFAULT ' ' CHECK(length(submitted_text)>0);
UPDATE conversation_inputs SET submitted_text=(SELECT submitted_text FROM input_body_copy WHERE input_body_copy.id=conversation_inputs.id);
DROP TABLE input_body_copy;
