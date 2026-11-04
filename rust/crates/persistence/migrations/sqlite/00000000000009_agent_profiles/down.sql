-- Canonical IDs and preserved proofs cannot be safely downgraded.
SELECT RAISE(ABORT, 'Migration downgrade requires operator restore');
