-- Removed replay history cannot be reconstructed or safely served by old callers.
DO $$ BEGIN RAISE EXCEPTION 'Provider request retention down migration is not supported'; END $$;
