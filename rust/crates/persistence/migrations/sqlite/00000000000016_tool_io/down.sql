CREATE TEMP TABLE tool_io_down_guard (empty INTEGER NOT NULL CHECK(empty=0));
INSERT INTO tool_io_down_guard SELECT count(*) FROM conversation_record_fields WHERE field_name IN ('TOOL_INPUT','TOOL_OUTPUT');
DROP TABLE tool_io_down_guard;
CREATE TABLE conversation_record_fields_previous (
    conversation_id BLOB NOT NULL CHECK(typeof(conversation_id)='blob' AND length(conversation_id)=16),
    record_kind TEXT NOT NULL,
    native_id TEXT NOT NULL,
    field_name TEXT NOT NULL CHECK(field_name IN ('CONTENT','TITLE','STATUS','KIND','LOCATIONS','DESCRIPTION','REQUEST','FOREGROUND')),
    content BLOB NOT NULL CHECK(typeof(content)='blob'),
    updated_ordinal BIGINT NOT NULL CHECK(updated_ordinal>0),
    PRIMARY KEY(conversation_id,record_kind,native_id,field_name),
    FOREIGN KEY(conversation_id,record_kind,native_id) REFERENCES conversation_records(conversation_id,record_kind,native_id) ON DELETE CASCADE
);
INSERT INTO conversation_record_fields_previous SELECT conversation_id,record_kind,native_id,field_name,content,updated_ordinal FROM conversation_record_fields;
DROP TABLE conversation_record_fields;
ALTER TABLE conversation_record_fields_previous RENAME TO conversation_record_fields;
