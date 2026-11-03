---
title: "Expose the consumer's owned contract at a crate boundary, not a live implementation handle or accidental internal layout."
---

# Consumer-owned API boundaries

- Choose exports from consumer authority and lifetime needs; an intentional `pub use` facade is valid.
- Return completed owned data or an explicit resource owner, not lazy database rows or a borrowed transaction.
- Private module layout may change without moving persistence or SDK authority into consumers.

## Bad

```rust
fn account_page(...) -> Result<Rows<'transaction>, StoreError>;
```

## Good

```rust
fn account_page(...) -> Result<AccountPage, StoreError>;
// AccountPage owns the completed result, not a database handle.
```

Reference: [Rust public re-exports](https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html#re-exporting-names-with-pub-use).
