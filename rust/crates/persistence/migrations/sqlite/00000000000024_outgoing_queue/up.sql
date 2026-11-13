-- Queue acceptance precedes delivery. Preserve all original receipts and inputs.
DROP TRIGGER require_chat_receipt_for_input;
-- Completed explicit deletions no longer retain bodyless operation records.
-- Ordinary Chat receipts have independent identities and remain unchanged.
DELETE FROM conversation_inputs WHERE deleted_at IS NOT NULL;
ALTER TABLE conversation_inputs ADD COLUMN replaces_input_id BLOB
CHECK(replaces_input_id IS NULL OR length(replaces_input_id)=16);
ALTER TABLE conversation_inputs ADD COLUMN queue_state TEXT NOT NULL DEFAULT 'ERROR'
CHECK(queue_state IN ('PENDING','ERROR'));
ALTER TABLE conversation_inputs ADD COLUMN consumer_expires_at BIGINT NOT NULL DEFAULT 0
CHECK(consumer_expires_at>=0);
CREATE INDEX conversation_input_consumers ON conversation_inputs(queue_state,consumer_expires_at);
