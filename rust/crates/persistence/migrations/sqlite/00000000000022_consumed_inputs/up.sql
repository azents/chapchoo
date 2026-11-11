-- Install the terminal invariant before sweeping. Old connected writers may
-- still publish success while the preparation hook precedes backend rollout.
CREATE TRIGGER IF NOT EXISTS consume_conversation_input_on_insert
AFTER INSERT ON conversation_input_delivery
WHEN NEW.delivery_state IN ('INSERTED','STEERED')
BEGIN
    DELETE FROM conversation_inputs
    WHERE id=NEW.operation_id AND EXISTS (
        SELECT 1 FROM conversations c
        WHERE c.id=conversation_inputs.conversation_id
          AND c.source_snapshot=NEW.source_snapshot
          AND c.native_session_id=NEW.native_session_id
    );
END;
CREATE TRIGGER IF NOT EXISTS consume_conversation_input_on_update
AFTER UPDATE ON conversation_input_delivery
WHEN NEW.delivery_state IN ('INSERTED','STEERED')
BEGIN
    DELETE FROM conversation_inputs
    WHERE id=NEW.operation_id AND EXISTS (
        SELECT 1 FROM conversations c
        WHERE c.id=conversation_inputs.conversation_id
          AND c.source_snapshot=NEW.source_snapshot
          AND c.native_session_id=NEW.native_session_id
    );
END;

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
