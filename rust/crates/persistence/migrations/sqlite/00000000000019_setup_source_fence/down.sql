-- Removing a sealed recovery constraint could permit replacement of saved context.
CREATE TEMP TABLE setup_source_fence_guard (n BIGINT CHECK(n=0));
INSERT INTO setup_source_fence_guard SELECT count(*) FROM conversation_input_preparation WHERE require_retained_source=TRUE;
DROP TABLE setup_source_fence_guard;
ALTER TABLE conversation_input_preparation DROP COLUMN require_retained_source;
