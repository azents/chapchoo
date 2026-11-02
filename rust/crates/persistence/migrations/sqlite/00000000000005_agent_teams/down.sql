-- Reverting required context identity would discard Agent ownership scope.
SELECT RAISE(ABORT, 'Agent team association down migration is not supported');
