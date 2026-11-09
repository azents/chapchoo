-- Current-body byte ends do not introduce a body archive or a new revision counter.
CREATE TABLE conversation_field_extents (
    conversation_id UUID NOT NULL,
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
