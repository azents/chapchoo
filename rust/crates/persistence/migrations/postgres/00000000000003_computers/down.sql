-- Preserve Computer intent, residual cleanup and credential authority on rollback.
DO $$ BEGIN
    RAISE EXCEPTION 'Computer schema down migration is not supported';
END $$;
