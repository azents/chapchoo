-- Conservative admission is distinct from actual component dispatch time.
ALTER TABLE computers ADD COLUMN prepare_admitted_at BIGINT;
UPDATE computers SET prepare_admitted_at = initial_start_dispatch_at
WHERE initial_start_dispatch_at IS NOT NULL;
