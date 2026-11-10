CREATE TEMP TABLE input_body_copy AS SELECT id,submitted_text FROM conversation_inputs;
ALTER TABLE conversation_inputs DROP COLUMN submitted_text;
ALTER TABLE conversation_inputs ADD COLUMN submitted_text TEXT CHECK(submitted_text IS NULL OR length(submitted_text)>0);
UPDATE conversation_inputs SET submitted_text=(SELECT submitted_text FROM input_body_copy WHERE input_body_copy.id=conversation_inputs.id);
DROP TABLE input_body_copy;
ALTER TABLE conversation_inputs ADD COLUMN deleted_at BIGINT CHECK((submitted_text IS NULL)=(deleted_at IS NOT NULL));
