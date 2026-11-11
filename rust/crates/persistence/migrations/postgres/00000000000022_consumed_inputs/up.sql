-- Install the terminal invariant before sweeping. Old connected writers may
-- still publish success while the preparation hook precedes backend rollout.
CREATE OR REPLACE FUNCTION consume_conversation_input_on_delivery()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.delivery_state IN ('INSERTED','STEERED') THEN
        DELETE FROM conversation_inputs i
        USING conversations c
        WHERE i.id=NEW.operation_id AND c.id=i.conversation_id
          AND c.source_snapshot=NEW.source_snapshot
          AND c.native_session_id=NEW.native_session_id;
    END IF;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS consume_conversation_input_on_delivery ON conversation_input_delivery;
CREATE TRIGGER consume_conversation_input_on_delivery
AFTER INSERT OR UPDATE ON conversation_input_delivery
FOR EACH ROW EXECUTE FUNCTION consume_conversation_input_on_delivery();

-- Remove completed delivery records only when the independent native source
-- binding still carries the session-preservation authority. Failed or unresolved
-- preparation/delivery remains pending and is not inferred to have been consumed.
DELETE FROM conversation_inputs
WHERE id IN (
    SELECT i.id
    FROM conversation_inputs i
    JOIN conversation_input_delivery d ON d.operation_id=i.id
    JOIN conversations c ON c.id=i.conversation_id
    WHERE d.delivery_state IN ('INSERTED','STEERED')
      AND c.source_snapshot=d.source_snapshot
      AND c.native_session_id=d.native_session_id
);
