---
title: "Accounts Phase 1: Rust Domain and Cryptographic Foundation"
created: 2026-10-29
tags: [accounts, implementation, rust, security]
---

# Phase Execution Plan

- Phase: 1/5, Rust domain and cryptographic foundation.
- Branch/base: `feat/accounts-foundation` → `chore/bootstrap-chapchoo` at `173fafe`.
  The bootstrap is still open PR #1; this phase does not merge it or fold its
  existing scaffold diff into the account implementation review.
- PR boundary: approved design baseline plus executable typed foundation and its tests.
- Inputs: confirmed Requirements, accepted ADR D1-D37, approved Design revision 36.
- Deliverables: Cargo workspace, typed secret-safe domain values, password policy,
  bcrypt verifier, common root/purpose-separated crypto, session-proof verifiers,
  key-binding checks and deterministic unit tests.
- Non-goals: database schemas/operations, network listeners, CLI services, frontend
  changes, packaging/deployment, key replacement and unapproved generic frameworks.
- Interfaces: the domain crate owns validated inputs and roles without Tonic or
  Diesel; the crypto crate accepts those values and returns owned typed secrets,
  verifiers or sanitized errors. No service-locator/global mutable credentials.
- Approved Design mechanisms: M23, M24, M30, M32, M36, M37, M41.
- Authority references: REQ-1/REQ-3/REQ-10/REQ-11 fixed constraints; ADR-D22,
  ADR-D26, ADR-D29, ADR-D33, ADR-D34.
- Design delta: None.
- Removal obligations: None in this phase; Hello World and theme replacement belong
  to phase 4. No existing product backend or data is replaced.
- Absence verification: new crates must have no DB/protocol dependencies; no root
  secret formatter or plaintext persistence API; no replacement/rotation entrypoint.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Approval/plan | /root | docs/chapchoo/{requirements,adr,design,plans} | requester approval | exact authority baseline | catalog and 18 tests |
| Domain | /root | rust/crates/domain | approved policy | validated values, roles, typed errors | tests and dependency graph |
| Crypto | /root | rust/crates/crypto | domain types | common credential and verification primitives | tamper/purpose/input/secret-safe tests |
| Quality wiring | /root | rust/Cargo.*, rust/rust-toolchain.toml, scripts/check-rust.sh, .github/workflows/ci.yaml, DEVELOPMENT.md, .gitignore, .pre-commit-config.yaml | crates | pinned toolchain, locked builds and CI | root-owned fmt/Clippy/tests/hooks |

- Integration order: domain → crypto → quality wiring → frozen diff review → PR.
- Independent review: `/root/accounts-reviewer`, read-only, requested by root only
  after integrated implementation and validation are stable. Root applies fixes.
- Final validation: cargo fmt --check; cargo clippy --locked --all-targets -- -D
  warnings; cargo test --locked; dependency-direction and forbidden-import checks;
  documentation validation; required pre-commit hooks and exact-head CI.
- Scope-drift check: compare exposed APIs and dependencies with the approved IDs;
  reject key-change support, auth services, storage/transport, and later-phase code.
- Context checkpoint: foundation implementation complete; only library APIs are
  reachable. Remaining phases provide persistence, RPC/CLI, web and deployment.

## Integrated Checkpoint

- Scope drift: None. Implemented the assigned foundation mechanisms only; no DB,
  network service, key-change mode or new product flow was introduced.
- Root validation: Rust 1.99.1; rustfmt; all-target warning-denying Clippy; 14 unit
  tests and 2 compile-fail tests; Cargo dependency boundaries; documentation
  catalog plus 18 tests; full pre-commit including existing TypeScript checks.
- Independent review: `/root/accounts-reviewer` found a repeated-NUL bcrypt login
  input equivalence. Root added printable-ASCII/length admission before bcrypt
  and regression tests for both reproduced inputs. The same reviewer performed
  targeted security re-review and independently ran both password tests; no
  remaining findings.
- Initial formatting hooks removed extra TOML terminal blank lines; rerun passed.
  A catalog test invocation from the Rust directory failed import resolution;
  the canonical repository-root command passed. Neither was bypassed.
- PR base: existing bootstrap PR #1, `chore/bootstrap-chapchoo@173fafe`; no merge
  permission is inferred. Remote exact-head CI remains a separate delivery gate.
- Next phase: both-engine persistence/migrations and transaction tests, starting
  only after this phase PR is opened.
