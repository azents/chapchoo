DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM conversation_native_details) THEN
        RAISE EXCEPTION 'Native detail downgrade requires empty new observations';
    END IF;
END $$;
DROP TABLE conversation_native_details;
