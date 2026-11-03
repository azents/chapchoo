---
title: "When shared behavior changes, check every supported implementation's owned contract while keeping provider-specific translation local."
---

# Backend contract parity

- Enumerate affected database/native adapters before changing shared semantics.
- Exercise the same owned invariant through each supported implementation; native representation may differ.
- Express unsupported behavior explicitly rather than silently omitting it or duplicating the browser matrix.

## Bad

```rust
// Change shared retirement semantics only in one native adapter.
```

## Good

```rust
// Shared retirement contract: both native adapters.
// Database authority/data preservation: SQLite and PostgreSQL.
// Native wire translation stays with its adapter.
```

Reference: [Current Rust workspace](../../../rust/Cargo.toml) and the global owned-contract testing rule.
