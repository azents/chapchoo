-- Canonical IDs and preserved proofs cannot be safely downgraded.
DO $$ BEGIN RAISE EXCEPTION 'Migration downgrade requires operator restore'; END $$;
