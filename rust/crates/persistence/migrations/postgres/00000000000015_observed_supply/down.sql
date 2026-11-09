DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM conversation_applied_supply) THEN
        RAISE EXCEPTION 'Observed supply downgrade requires empty new state';
    END IF;
END $$;
DROP TABLE conversation_applied_supply;
