ALTER TABLE conversation_input_preparation ADD COLUMN require_retained_source BOOLEAN NOT NULL DEFAULT FALSE CHECK(require_retained_source IN (TRUE,FALSE));
