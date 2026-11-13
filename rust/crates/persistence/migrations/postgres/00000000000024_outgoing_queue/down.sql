CREATE TEMP TABLE outgoing_queue_downgrade_guard(n BIGINT CHECK(n=0));
INSERT INTO outgoing_queue_downgrade_guard
SELECT count(*) FROM conversation_inputs i
WHERE i.deleted_at IS NULL AND NOT EXISTS (
    SELECT 1 FROM conversation_messages m
    WHERE m.conversation_id=i.conversation_id AND m.source_input_id=i.id
      AND m.source_input_request_id=i.request_id AND m.role='USER'
      AND m.body=i.submitted_text AND m.original_accepted_at=i.accepted_at
);
DROP TABLE outgoing_queue_downgrade_guard;
DROP INDEX conversation_input_consumers;
ALTER TABLE conversation_inputs DROP COLUMN consumer_expires_at;
ALTER TABLE conversation_inputs DROP COLUMN queue_state;
ALTER TABLE conversation_inputs DROP COLUMN replaces_input_id;
CREATE FUNCTION require_chat_receipt_for_input() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM conversation_messages m
        WHERE m.conversation_id=NEW.conversation_id AND m.source_input_id=NEW.id
          AND m.source_input_request_id=NEW.request_id
          AND m.role='USER' AND m.body=NEW.submitted_text
          AND m.original_accepted_at=NEW.accepted_at
    ) THEN
        RAISE EXCEPTION 'Conversation input requires its chat receipt' USING ERRCODE='23514';
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER require_chat_receipt_for_input
BEFORE INSERT ON conversation_inputs
FOR EACH ROW EXECUTE FUNCTION require_chat_receipt_for_input();
