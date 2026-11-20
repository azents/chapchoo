CREATE TEMP TABLE conversation_models_downgrade_guard(n BIGINT CHECK (n=0));
INSERT INTO conversation_models_downgrade_guard SELECT count(*) FROM conversation_model_scopes
    WHERE saved_model_id IS NOT NULL;
INSERT INTO conversation_models_downgrade_guard SELECT count(*) FROM conversation_model_save_receipts;
INSERT INTO conversation_models_downgrade_guard SELECT count(*) FROM conversation_model_starts
    WHERE settlement IS NULL;
DROP TABLE conversation_models_downgrade_guard;
DROP TABLE conversation_model_starts;
DROP TABLE conversation_model_save_receipts;
DROP TABLE conversation_model_scopes;
