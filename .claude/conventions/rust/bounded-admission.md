---
title: "Tie acknowledgement and capacity release to the owned acceptance contract; overload must not silently discard accepted work."
---

# Bounded admission

- Bound payloads, concurrency and retained state by the actual owner/resource budget.
- Admit and acknowledge at the contract's real acceptance point; distinguish not-admitted from admitted-but-unfinished work.
- Define overflow/closure behavior explicitly. A wakeup signal and an accepted effect are not interchangeable queue items.

## Bad

```rust
acknowledge(request.id);
queue.try_send(request)?; // The accepted request may never enter the queue.
```

## Good

```rust
let admission = owner.admit(request)?;
acknowledge(admission.receipt());
// Owner retains accepted work until completion or explicit unresolved outcome.
```

Reference: [Tokio owned shutdown](https://tokio.rs/tokio/topics/shutdown).
