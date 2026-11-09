-- Record deletion must not erase an already observed application or prevent deletion.
CREATE TABLE conversation_native_details (
    conversation_id UUID PRIMARY KEY REFERENCES conversation_native_state(conversation_id) ON DELETE CASCADE,
    preparation_payload BYTEA CHECK(preparation_payload IS NULL OR length(preparation_payload) BETWEEN 1 AND 32768),
    authentication_id UUID,
    CHECK(preparation_payload IS NOT NULL OR authentication_id IS NOT NULL)
);
