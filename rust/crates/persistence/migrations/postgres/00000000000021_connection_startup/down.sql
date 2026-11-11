DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM conversation_startups) THEN
        RAISE EXCEPTION 'Connection startup downgrade requires an empty startup history';
    END IF;
END $$;
DROP TABLE conversation_startups;
