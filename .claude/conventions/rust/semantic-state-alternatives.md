---
title: "Model outcomes with different legal next actions as state-specific data; keep genuinely independent observations independent."
---

# Semantic state alternatives

- Represent mutually exclusive outcomes with their required evidence, rather than correlated flags and optional payloads.
- Keep unknown, rejected and completed effects distinct when they permit different actions.
- Do not turn independent facts into one enum merely to reduce a boolean count.

## Bad

```rust
struct Progress { complete: bool, receipt: Option<Receipt> }
```

## Good

```rust
enum Progress { Pending, Completed(Receipt), Uncertain(EffectRef) }
```

Reference: [C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/type-safety.html).
