DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM conversation_record_fields WHERE field_name IN ('TOOL_INPUT','TOOL_OUTPUT')) THEN
        RAISE EXCEPTION 'Tool I/O downgrade requires empty new fields';
    END IF;
END $$;
ALTER TABLE conversation_record_fields DROP CONSTRAINT conversation_record_fields_field_name_check;
ALTER TABLE conversation_record_fields ADD CONSTRAINT conversation_record_fields_field_name_check CHECK(field_name IN ('CONTENT','TITLE','STATUS','KIND','LOCATIONS','DESCRIPTION','REQUEST','FOREGROUND'));
