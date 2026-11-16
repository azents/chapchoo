-- New accepted work waits durably; historical attempts gain no replay authority.
ALTER TABLE conversation_inputs ADD COLUMN readiness_waiting BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE conversation_inputs ADD COLUMN submitting_session_id UUID REFERENCES auth_sessions(id) ON DELETE SET NULL;
ALTER TABLE conversation_inputs ADD COLUMN input_order BIGINT CHECK(input_order>0);
ALTER TABLE conversation_inputs ADD COLUMN wake_predecessor TEXT;
ALTER TABLE conversation_inputs ADD COLUMN consumer_claim_id UUID;
CREATE INDEX conversation_input_waiting ON conversation_inputs(readiness_waiting,id) WHERE queue_state='PENDING';
CREATE UNIQUE INDEX conversation_input_order ON conversation_inputs(conversation_id,input_order);
