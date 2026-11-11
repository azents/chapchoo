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
