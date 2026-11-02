-- Reverting required context identity would discard Agent ownership scope.
DO $$ BEGIN RAISE EXCEPTION 'Agent team association down migration is not supported'; END $$;
