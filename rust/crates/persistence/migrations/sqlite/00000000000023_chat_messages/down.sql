CREATE TEMP TABLE chat_message_downgrade_guard(n BIGINT CHECK(n=0));
INSERT INTO chat_message_downgrade_guard SELECT count(*) FROM conversation_messages;
DROP TABLE chat_message_downgrade_guard;
DROP TRIGGER require_chat_receipt_for_input;
DROP TABLE conversation_messages;
DROP TABLE conversation_message_heads;
