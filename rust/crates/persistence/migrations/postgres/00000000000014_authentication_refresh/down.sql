DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM authentication_refreshes)
       OR EXISTS (SELECT 1 FROM execution_authentication_applications) THEN
        RAISE EXCEPTION 'Authentication refresh downgrade requires empty new state';
    END IF;
END $$;
DROP TABLE execution_authentication_applications;
DROP TABLE authentication_refreshes;
