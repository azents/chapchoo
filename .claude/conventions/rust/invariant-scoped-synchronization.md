---
title: "Choose synchronization around the protected invariant and resource owner, not around an entire service merely to share it."
---

# Invariant-scoped synchronization

- Identify what must be atomic and which owner serializes an asynchronous resource.
- Separate protected state changes from unrelated external work; revalidate current authority before applying a later result.
- A short ordinary critical section and an async resource owner solve different problems; neither mutex flavor is universal.

## Bad

```rust
let mut all = service.lock().await;
all.unrelated_network_lookup().await?;
```

## Good

```rust
let intent = state.prepare(scope)?;
let observed = adapter.observe(&intent).await?;
state.apply_if_current(intent, observed)?;
```

Reference: [Tokio shared-state strategies](https://tokio.rs/tokio/tutorial/shared-state).
