---
title: "Bound blocking work by its physical lifetime, because timing out or aborting a waiter does not stop a started blocking closure."
---

# Blocking work ownership

- Account for queued versus started blocking work and its input/resource lifetime.
- Move the capacity permit into work that can continue after caller cancellation.
- Use the existing blocking owner for finite work; indefinite workers need an explicit long-lived owner and shutdown contract.

## Bad

```rust
timeout(budget, spawn_blocking(move || hash(input))).await?;
drop(permit); // The timed-out computation may still be running.
```

## Good

```rust
let work = spawn_blocking(move || {
    let _permit = permit;
    hash(input)
});
owner.retain(work);
```

Reference: [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).
