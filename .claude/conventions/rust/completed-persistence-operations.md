---
title: "Repositories own complete database-only transactions; external preparation or effects cross them through explicit revalidated intent and results."
---

# Completed persistence operations

- Repository composition owns atomic database work and finishes its transaction before returning.
- Keep network, filesystem, broker and runtime effects outside that transaction, including hidden helper calls.
- Revalidate the authority required for a final write; splitting a transaction must preserve its actual atomicity and failure contract.

## Bad

```rust
transaction(|tx| async { reserve(tx).await?; provider.create().await }).await
```

## Good

```rust
let intent = store.reserve(scope).await?;
let observed = provider.create_once(&intent).await;
store.finalize_if_current(intent, observed).await?;
```

Reference: [Current repository contracts](../../../rust/crates/persistence/src/lib.rs).
