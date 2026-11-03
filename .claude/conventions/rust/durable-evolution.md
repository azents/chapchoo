---
title: "Treat persisted representations and applied migrations as historical contracts; preserve data and invariants in every supported database."
---

# Durable evolution

- Preserve existing records, keys, relationships and cleanup responsibility during schema changes.
- Keep paired database migrations semantically equivalent; do not copy a PostgreSQL-only representation into the shared contract.
- Edit an old migration only after verifying it is unshipped/unexecuted. Applied history requires a preserving forward migration.

## Bad

```sql
DROP TABLE computers; -- Existing ownership and cleanup state are lost.
```

## Good

```sql
-- Preserve rows, identities, dependencies and constraints through the change.
-- Verify the full migration graph with existing data in both supported engines.
```

Reference: [Current persistence migrations](../../../rust/crates/persistence).
