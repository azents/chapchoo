-- Current-body byte ends are derived from the same captured ordinal as the field.
-- Replaced bodies are not archived; their extent metadata is deleted by ingestion.
CREATE TABLE conversation_field_extents (
    conversation_id BLOB NOT NULL CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    record_kind TEXT NOT NULL,
    native_id TEXT NOT NULL,
    field_name TEXT NOT NULL,
    ordinal BIGINT NOT NULL CHECK(ordinal>0),
    total_bytes BIGINT NOT NULL CHECK(total_bytes>=0),
    PRIMARY KEY(conversation_id,record_kind,native_id,field_name,ordinal),
    FOREIGN KEY(conversation_id,record_kind,native_id,field_name)
        REFERENCES conversation_record_fields(conversation_id,record_kind,native_id,field_name) ON DELETE CASCADE
);
INSERT INTO conversation_field_extents
SELECT conversation_id,record_kind,native_id,field_name,updated_ordinal,length(content)
FROM conversation_record_fields;
