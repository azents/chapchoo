-- Never discard native identity, original storage observations or daemon binding.
SELECT RAISE(ABORT, 'Native Computer authority down migration is not supported');
