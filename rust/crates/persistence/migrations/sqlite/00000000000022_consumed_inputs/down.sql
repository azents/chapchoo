-- Remove only the guard; consumed content cannot be reconstructed.
DROP TRIGGER IF EXISTS consume_conversation_input_on_insert;
DROP TRIGGER IF EXISTS consume_conversation_input_on_update;
