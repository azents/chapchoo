---
title: "Keep started external work and its capacity owned beyond a cancelled waiter; cancellation is not evidence that an effect stopped."
---

# Async effect ownership

- Inspect cancellation semantics at each awaited effect, including race/timeout losers.
- An owner that outlives the waiter retains started work, permits and uncertain progress.
- Stop new admission and drain or fence unresolved work; do not reconstruct permission from a dropped future.

## Bad

```rust
tokio::select! { _ = cancelled() => return, result = write_once() => result }
```

## Good

```rust
owner.spawn(async move {
    let _permit = permit;
    write_once().await
}); // Existing owner retains the task independently of its caller.
```

Reference: [Tokio cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).
