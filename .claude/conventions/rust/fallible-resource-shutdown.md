---
title: "When completion includes asynchronous or fallible cleanup, await an explicit finish and preserve both primary and cleanup failures."
---

# Fallible resource shutdown

- Make observable cleanup part of the resource owner's completion contract; destructor fallback is not completion evidence.
- Preserve the primary error when teardown also fails.
- Continue independently authorized cleanup stages; an unverified authority/source still fences mutation.

## Bad

```rust
let primary = operation().await;
owner.finish().await?; // Discards the primary failure if finish fails.
primary
```

## Good

```rust
let primary = operation().await;
let cleanup = owner.finish().await;
combine_primary_and_cleanup(primary, cleanup)
```

Reference: [C-DTOR-FAIL/BLOCK](https://rust-lang.github.io/api-guidelines/dependability.html).
