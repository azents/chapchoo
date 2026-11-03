---
title: "Before effects, bind an owner to one coherent validated runtime configuration; explicit malformed input does not authorize borrowing another source."
---

# Runtime configuration snapshots

- Resolve related settings as one source with their security and authority meaning intact.
- Freeze the selected target/material for the owning lifetime; a later environment/context change cannot retarget active work.
- Reject incomplete explicit choices rather than silently mixing defaults from another source.

## Bad

```rust
let endpoint = explicit.endpoint.unwrap_or(context.endpoint);
let credential = explicit.credential.unwrap_or(context.credential);
```

## Good

```rust
let selected = resolve_coherent_source(explicit, context)?;
let owner = Owner::new(selected.validate()?);
```

Reference: [Global runtime-only configuration](../global/runtime-only-configuration.md).
