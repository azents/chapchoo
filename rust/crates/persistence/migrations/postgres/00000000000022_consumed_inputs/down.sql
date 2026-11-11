-- Remove only the guard; consumed content cannot be reconstructed.
DROP TRIGGER IF EXISTS consume_conversation_input_on_delivery ON conversation_input_delivery;
DROP FUNCTION IF EXISTS consume_conversation_input_on_delivery();
