---
title: "Preserve expected rejection, absence, operational failure and unknown physical outcomes instead of collapsing them into one fallback."
---

# Outcome and error meaning

- Use the owning contract's expected outcomes; `Result` is normal recoverable failure transport.
- Convert only the specifically handled absence/rejection. Preserve other causes for the handling boundary.
- A timed-out mutation is not proof that nothing happened or permission to repeat it.

## Bad

```rust
let found = store.lookup(id).await.ok();
```

## Good

```rust
match store.lookup(id).await {
    Ok(value) => Ok(Some(value)),
    Err(StoreError::NotFound) => Ok(None),
    Err(cause) => Err(cause),
}
```

Reference: [C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html).
