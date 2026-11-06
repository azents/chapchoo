DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM external_authentications)
       OR EXISTS (SELECT 1 FROM external_auth_attempts)
       OR EXISTS (SELECT 1 FROM external_auth_requests)
       OR EXISTS (SELECT 1 FROM agent_authentication_selections)
    THEN
        RAISE EXCEPTION 'Cannot erase external authentication data or accepted results';
    END IF;
END $$;
ALTER TABLE external_authentications DROP CONSTRAINT external_auth_active_attempt;
DROP TABLE agent_authentication_selections;
DROP TABLE external_auth_requests;
DROP TABLE external_auth_attempts;
DROP TABLE external_authentications;
