---
title: "Choose time from the contract's authority and units: durable acceptance timestamps and process-relative deadlines are different values."
---

# Time authority and units

- Identify who supplies an acceptance instant and when authority must be rechecked, especially after expensive work.
- Use process-relative elapsed time for local deadlines; a persisted timestamp or remote clock is not a monotonic budget.
- Keep units and expiry comparisons explicit across storage, protocol and policy boundaries; use controlled instants for policy tests.

## Bad

```rust
let accepted_at = before_password_work;
authorize(expensive_result, accepted_at)?;
```

## Good

```rust
let verified = password_work.await?;
let accepted_at = acceptance_clock.now()?;
authorize(verified, accepted_at)?;
```

Reference: [Current acceptance-time contract](../../../rust/crates/service/src/time.rs).
