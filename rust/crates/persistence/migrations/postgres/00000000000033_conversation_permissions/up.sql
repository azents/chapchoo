ALTER TABLE conversation_model_scopes DROP CONSTRAINT conversation_model_scopes_check2;
ALTER TABLE conversation_model_scopes ADD CHECK (saved_model_id IS NULL OR saved_request_id IS NOT NULL);
ALTER TABLE conversation_model_save_receipts ALTER COLUMN model_id DROP NOT NULL;
ALTER TABLE conversation_model_save_receipts ALTER COLUMN model_label DROP NOT NULL;
ALTER TABLE conversation_model_save_receipts ADD CHECK ((model_id IS NULL)=(model_label IS NULL));
