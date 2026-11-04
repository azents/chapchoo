-- Never erase Team ownership or restore invalidated Provider credentials.
DO $$ BEGIN RAISE EXCEPTION 'Provider authentication authority down migration is not supported'; END $$;
