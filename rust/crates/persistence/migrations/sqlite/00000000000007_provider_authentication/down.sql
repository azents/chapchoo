-- Never erase Team ownership or restore invalidated Provider credentials.
SELECT RAISE(ABORT, 'Provider authentication authority down migration is not supported');
