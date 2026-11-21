ALTER TABLE conversation_model_scopes ADD COLUMN saved_tuning TEXT;
ALTER TABLE conversation_model_save_receipts ADD COLUMN tuning TEXT;
ALTER TABLE conversation_model_starts ADD COLUMN tuning TEXT;
