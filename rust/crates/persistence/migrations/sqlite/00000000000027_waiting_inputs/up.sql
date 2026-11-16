-- New accepted work waits durably; historical attempts gain no replay authority.
ALTER TABLE conversation_inputs ADD COLUMN readiness_waiting BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE conversation_inputs ADD COLUMN submitting_session_id BLOB REFERENCES auth_sessions(id) ON DELETE SET NULL CHECK(submitting_session_id IS NULL OR length(submitting_session_id)=16);
ALTER TABLE conversation_inputs ADD COLUMN input_order BIGINT CHECK(input_order>0);
ALTER TABLE conversation_inputs ADD COLUMN wake_predecessor TEXT;
ALTER TABLE conversation_inputs ADD COLUMN consumer_claim_id BLOB CHECK(consumer_claim_id IS NULL OR length(consumer_claim_id)=16);
CREATE INDEX conversation_input_waiting ON conversation_inputs(readiness_waiting,id) WHERE queue_state='PENDING';
CREATE UNIQUE INDEX conversation_input_order ON conversation_inputs(conversation_id,input_order);
