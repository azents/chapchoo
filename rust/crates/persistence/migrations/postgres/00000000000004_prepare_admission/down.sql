-- Removing an admitted-effect fence could authorize unsafe effect replay.
DO $$ BEGIN RAISE EXCEPTION 'Prepare admission down migration is not supported'; END $$;
