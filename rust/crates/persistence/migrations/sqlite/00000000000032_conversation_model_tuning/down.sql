CREATE TEMP TABLE conversation_model_tuning_downgrade_guard(n BIGINT CHECK (n=0));
INSERT INTO conversation_model_tuning_downgrade_guard SELECT count(*) FROM conversation_model_save_receipts WHERE tuning IS NOT NULL;
INSERT INTO conversation_model_tuning_downgrade_guard SELECT count(*) FROM conversation_model_scopes WHERE saved_tuning IS NOT NULL;
INSERT INTO conversation_model_tuning_downgrade_guard SELECT count(*) FROM conversation_model_starts WHERE tuning IS NOT NULL;
DROP TABLE conversation_model_tuning_downgrade_guard;
ALTER TABLE conversation_model_starts DROP COLUMN tuning;
ALTER TABLE conversation_model_save_receipts DROP COLUMN tuning;
ALTER TABLE conversation_model_scopes DROP COLUMN saved_tuning;
