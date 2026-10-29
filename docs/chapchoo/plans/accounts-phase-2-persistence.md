---
title: "Accounts Phase 2: Dual-Engine Persistence and Atomic Operations"
created: 2026-10-29
tags: [accounts, implementation, rust, database]
---

# Phase Execution Plan

- Phase: 2/5, SQLite/PostgreSQL persistence and atomic operations.
- Branch/base: `feat/accounts-persistence` → `feat/accounts-foundation` at `486c962`.
- PR boundary: complete both-engine storage, explicit preparation/readiness and
  repository-owned account/team/session operations; no network or web activation.
- Inputs: phase 1 PR #2, approved Design revision 36 and its domain/crypto types.
- Deliverables: reviewed per-engine embedded migrations, native PostgreSQL enum
  mappings and SQLite constraints, operation-owned official async pools,
  preparation serialization/fixed-key binding, read-only readiness, detached
  results and invariant/concurrency tests on both real engines.
- Non-goals: gRPC handlers, public CLI/network services, mail sending, UI, container
  images/charts, data conversion, key replacement and generic replay ledgers.
- Interfaces: repositories own lease and DB transaction for each completed
  operation; no live connection or transaction escapes into service code.
  Services provide already prepared password/token material and repositories
  revalidate all acceptance-critical authority/state inside the transaction.
- Approved Design mechanisms: M2, M4, M10-M12, M16-M22, M25, M26, M29, M36, M40.
- Authority references: REQ-1 through REQ-13 as applicable; ADR-D4, D8, D13-D15,
  D18-D24, D28, D33-D35. Current credential-primitives Spec remains preserved.
- Design delta: None.
- Removal obligations: replace only the selected reserved backend layout with
  actual Rust persistence; no existing database/data conversion or fallback.
- Absence verification: no DB access from domain/crypto, no external IO within
  transactions, no key rebinding, no mutation replay or cross-engine replication.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Schema | /root | rust/crates/chapchoo-persistence/migrations | domain/data inventory | engine-native constraints | actual SQLite and disposable PostgreSQL |
| Repository authentication/common | /root | rust/crates/chapchoo-persistence/src except teams.rs | schema + domain/crypto | completed auth/session/grant operations and shared transaction boundary | atomicity, authorization, races, cancellation |
| Repository teams | /root/accounts-team-storage | rust/crates/chapchoo-persistence/src/teams.rs and tests/teams.rs | root-owned Store, operation/authentication macros, migrations | completed team/membership/invitation operations | owner/manager/member rules, personal restrictions and concurrent acceptance |
| Integration tests | /root | rust/crates/chapchoo-persistence/tests, scripts | repository | deterministic both-engine fixtures | barriers/state, no sleep-based correctness |
| Quality/Specs | /root | rust/Cargo.*, scripts/check-rust.sh, .github/workflows/ci.yaml, docs/chapchoo/spec | integrated code | CI real PG service and current schema Spec | root-owned checks and review |

- Integration order: schemas → preparation/readiness → repositories → both-engine
  concurrency/cancellation tests → Spec/CI → frozen diff review → phase PR.
- Independent review: `/root/accounts-reviewer`, same exact read-only reviewer;
  root requests only after complete integrated diff and validation are stable.
- Final validation: rustfmt/Clippy/locked tests, actual PostgreSQL and SQLite
  migrations/constraints/atomic operations, root-owned rollback/cancellation and
  concurrent acceptance tests, pre-commit, docs and exact-head CI.
- Scope-drift check: all required account/team/session invariants represented;
  no additional CRUD, DB migration fallback, automatic mutation replay or
  feature-specific key-change path. Return material new findings to Design.
- Context checkpoint: phase 1 PR #2 exact head `486c962` passed Rust and full
  validation CI. This phase has integrated both-engine library operations and
  passed root-owned Rust validation (24 unit/integration tests plus 2 compile-fail
  tests). The same independent reviewer found no issues and independently
  reran all 24 plus 2 tests against real temporary databases. Full pre-commit
  and 18 catalog tests passed. Remote exact-head CI is still pending. No public
  RPC, CLI server, browser or live deployment acceptance is claimed.

## Integrated Discoveries and Corrections

- The actual PostgreSQL transfer-versus-target-leave test reproduced a foreign-key
  lock cycle with User `FOR UPDATE`. Root changed the User state lock to
  `FOR NO KEY UPDATE`, preserving credential serialization while allowing FK
  key-share checks. The same deterministic race passes on both engines; no
  automatic retry was added.
- Configured-mail public signup requires no administrator issuer, matching the
  approved reference behavior. Signup grants therefore allow a null issuer;
  password-reset grants still require an existing administrator issuer and target.
- A failed initial session insert rolls back the new User, personal context and
  signup-grant consumption. The same proof can then be redeemed successfully.
- Tests use private SQLite files and random PostgreSQL schemas. The integrated
  test launcher creates/removes a private socket-only PostgreSQL instance when
  no explicit disposable test-server URL is supplied. Missing prerequisites fail
  rather than silently skip PostgreSQL coverage.
