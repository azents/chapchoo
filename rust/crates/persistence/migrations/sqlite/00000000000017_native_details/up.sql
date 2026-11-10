-- Only explicitly reported actual-start facts are retained. Existing rows stay unknown.
CREATE TABLE conversation_native_details (
    conversation_id BLOB PRIMARY KEY NOT NULL REFERENCES conversation_native_state(conversation_id) ON DELETE CASCADE CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    preparation_payload BLOB CHECK(preparation_payload IS NULL OR (typeof(preparation_payload)='blob' AND length(preparation_payload) BETWEEN 1 AND 32768)),
    authentication_id BLOB CHECK(authentication_id IS NULL OR (typeof(authentication_id)='blob' AND length(authentication_id)=16)),
    CHECK(preparation_payload IS NOT NULL OR authentication_id IS NOT NULL)
);
