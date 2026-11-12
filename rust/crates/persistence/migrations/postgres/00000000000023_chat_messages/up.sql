CREATE TABLE conversation_message_heads (
    conversation_id UUID PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    last_order BIGINT NOT NULL CHECK(last_order>=0)
);
CREATE TABLE conversation_messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('USER','AGENT')),
    body TEXT NOT NULL CHECK(octet_length(body)>0),
    receipt_order BIGINT NOT NULL CHECK(receipt_order>0),
    received_at BIGINT NOT NULL CHECK(received_at>=0),
    source_input_id UUID UNIQUE,
    source_input_request_id TEXT,
    original_accepted_at BIGINT,
    UNIQUE(conversation_id,receipt_order),
    CHECK((role='USER' AND source_input_id IS NOT NULL AND source_input_request_id IS NOT NULL AND length(source_input_request_id) BETWEEN 1 AND 128 AND original_accepted_at IS NOT NULL)
       OR (role='AGENT' AND source_input_id IS NULL AND source_input_request_id IS NULL AND original_accepted_at IS NULL))
);

-- Original input UUIDs are reused only as new typed message identities during
-- transfer; source_input_id has no owning foreign key to temporary delivery.
INSERT INTO conversation_messages(id,conversation_id,role,body,receipt_order,received_at,source_input_id,source_input_request_id,original_accepted_at)
SELECT id,conversation_id,'USER',submitted_text,
       row_number() OVER (PARTITION BY conversation_id ORDER BY accepted_at,id),
       (extract(epoch FROM statement_timestamp())*1000)::BIGINT,id,request_id,accepted_at
FROM conversation_inputs WHERE submitted_text IS NOT NULL AND deleted_at IS NULL;
INSERT INTO conversation_message_heads(conversation_id,last_order)
SELECT conversation_id,max(receipt_order) FROM conversation_messages GROUP BY conversation_id;

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
