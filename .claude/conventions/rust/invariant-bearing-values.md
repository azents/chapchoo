---
title: "When primitive values carry different authority, validation or units, preserve those distinctions in the owning domain contract."
---

# Invariant-bearing values

- Use validated values or an owned context when exchanging same-primitive values could change meaning or authority.
- Keep syntax validity distinct from current authorization; a parsed ID alone is not permission.
- Plain fields remain appropriate inside an unambiguous owned record; avoid speculative wrappers.

## Bad

```rust
fn renew(session: &str, credential: &str);
```

## Good

```rust
fn renew(grant: AuthorizedRefresh<'_>);
// The owning boundary constructs the current, scoped grant.
```

Reference: [C-NEWTYPE and C-CUSTOM-TYPE](https://rust-lang.github.io/api-guidelines/type-safety.html).
