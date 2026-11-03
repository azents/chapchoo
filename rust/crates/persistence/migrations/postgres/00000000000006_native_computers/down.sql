-- Never discard native identity, original storage observations or daemon binding.
DO $$ BEGIN RAISE EXCEPTION 'Native Computer authority down migration is not supported'; END $$;
