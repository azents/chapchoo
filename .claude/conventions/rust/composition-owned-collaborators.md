---
title: "Let the composition owner supply collaborators with the operation's required authority; introduce traits for real substitution boundaries."
---

# Composition-owned collaborators

- Wire concrete services, configuration and clients at their composition boundary.
- Inject an existing collaborator rather than finding a global mutable service during an operation.
- Use traits/generics for actual alternative implementations or a narrower authority contract, not constructor ceremony. Pure immutable constants need no injection.

## Bad

```rust
fn send(&self) { global_installation().mailer().send(...); }
```

## Good

```rust
let service = Service::new(store, mailer, policy);
service.send(...);
```

Reference: [C-GENERIC and caller control](https://rust-lang.github.io/api-guidelines/flexibility.html).
