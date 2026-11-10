ALTER TABLE conversation_inputs ALTER COLUMN submitted_text DROP NOT NULL;
ALTER TABLE conversation_inputs ADD COLUMN deleted_at BIGINT;
ALTER TABLE conversation_inputs ADD CONSTRAINT input_deleted_body CHECK((submitted_text IS NULL)=(deleted_at IS NOT NULL));
