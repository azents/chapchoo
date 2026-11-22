-- A permission-only receipt has no model; restoring NOT NULL will refuse loss.
ALTER TABLE conversation_model_save_receipts DROP CONSTRAINT conversation_model_save_receipts_check;
ALTER TABLE conversation_model_save_receipts ALTER COLUMN model_id SET NOT NULL;
ALTER TABLE conversation_model_save_receipts ALTER COLUMN model_label SET NOT NULL;
ALTER TABLE conversation_model_scopes DROP CONSTRAINT conversation_model_scopes_check2;
ALTER TABLE conversation_model_scopes ADD CHECK ((saved_model_id IS NULL)=(saved_request_id IS NULL));
