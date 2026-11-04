-- Removed replay history cannot be reconstructed or safely served by old callers.
SELECT RAISE(ABORT, 'Provider request retention down migration is not supported');
