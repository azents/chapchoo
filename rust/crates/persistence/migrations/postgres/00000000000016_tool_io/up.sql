ALTER TABLE conversation_record_fields DROP CONSTRAINT conversation_record_fields_field_name_check;
ALTER TABLE conversation_record_fields ADD CONSTRAINT conversation_record_fields_field_name_check CHECK(field_name IN ('CONTENT','TITLE','STATUS','KIND','LOCATIONS','DESCRIPTION','REQUEST','FOREGROUND','TOOL_INPUT','TOOL_OUTPUT'));
