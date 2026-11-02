-- Removing an admitted-effect fence could authorize unsafe effect replay.
SELECT RAISE(ABORT, 'Prepare admission down migration is not supported');
