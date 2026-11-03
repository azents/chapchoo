---
title: "Docker Computers Phase 1 Native Contracts"
created: 2026-11-03
tags: [computer, docker, implementation]
---

# Phase Execution Plan

- Phase: `1/4 native resource contracts`.
- Branch/base: `feat/docker-computers-1-native-contracts` → main `210db53`.
- PR boundary: shared domain native identity/kind/role contracts, their tests,
  existing K8s kind-parser integration and approved snapshot/plans.
- Inputs: confirmed REQ-1–REQ-6, accepted ADR-D1–ADR-D6 and approved Design
  revision 6 / M1–M16.
- Deliverables: validated discriminated native scopes and identities, native
  resource roles derived from kind, exact typed volume observations, one strict
  resource-kind authority consumed by existing storage.
- Non-goals: schema/wire/DTO changes, Docker admission, daemon IO, connection
  resolution, resource effects, new recovery state or Mac support claims.
- Interfaces: `chapchoo-domain` exports protocol/database-independent native
  values; persistence reexports the shared kind at its existing public path.
  Existing K8s serialized kind strings remain unchanged.
- Approved Design mechanisms: M3/M4/M7 foundations, M10 contract preservation,
  M12 owner-layer tests. Other mechanisms remain subsequent-phase work.
- Authority references: [REQ-1/3/4/6](../requirements/docker-261103-server-computers.md),
  ADR-D3, Design native resources/representation, current computer-storage and
  computer-kubernetes Specs, typed-ingress and test-owned-contracts conventions.
- Design delta: None.
- Removal obligations: replace storage-local kind enum and unknown-kind fallback
  in touched test ingress; leave schema/wire and activation removals in phases 2/3.
- Absence verification: search for duplicate enum and permissive fallback; test
  unknown kind rejection and unchanged existing kind values. No native identity
  converts a Docker object to a K8s identity.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Native values | `/root` | `rust/crates/domain/src/{lib,computer_resource}.rs` | Approved Design | Typed validated scopes/identities/roles | Domain tests, fmt, Clippy |
| Existing consumer | `/root` | `rust/crates/persistence/src/computer_executions.rs`, touched persistence/RPC fixture parsers | Native values | Single strict kind authority; unchanged K8s behavior | Persistence/RPC compile and focused existing regressions |
| External process boundary | `/root` | `rust/apps/chapchoo/tests/provider_cli.rs` | Requester clarification on external Providers | Regression proving no product DB/root startup dependency | Real child process reaches authenticated synthetic Control |
| Authority and evidence | `/root` | Snapshot trio, these plans, owning domain Spec | Stable integrated code | Accurate approval and implemented-contract boundary | Catalog, diff checks, exact-head CI |

- Integration order: domain types/tests → persistence kind reexport and strict
  fixture parsing → owning Spec boundary → integrated validation → freeze/review
  → direct corrections/affected checks → commit and phase PR.
- Independent review: `/root/computer-reviewer`, read-only, complete integrated
  phase against confirmed Requirements, accepted ADR, approved Design and this
  plan; root requests only after checks and freeze.
- Final validation: `cargo fmt --all -- --check`, workspace Clippy with
  `--locked --all-targets -- -D warnings`, domain tests and affected
  persistence/RPC tests using explicit disposable prerequisites; docs catalog,
  `git diff --check`, required CI at exact PR head.
- Scope-drift check: native values alone authorize no Docker reservation/effect;
  no material mechanisms beyond Design; no changed lifecycle/defaults; K8s
  string/role preservation; all later mechanisms remain explicitly incomplete.
- Context checkpoint: native contracts, strict kind reexport/fixture parsing and
  the external K8s child-process DB-independence regression are implemented.
  Base `210db53`; no schema/wire/API activation changes. Root validation passed:
  workspace format and Clippy; `python3 scripts/run-rust-tests.py` created and
  removed its own local disposable PostgreSQL and passed all 122 tests across
  44 groups, with zero failures/ignored tests. An earlier direct invocation of the
  persistence integration binary lacked `CHAPCHOO_TEST_POSTGRES_URL` and failed
  that prerequisite; SQLite passed. The authoritative isolated runner then
  passed both engines, rather than skipping PostgreSQL. Docs catalog and diff
  whitespace checks passed. Mac/real-daemon gates remain unavailable and unrun.
  `/root/computer-reviewer` reviewed the complete stable integrated phase and
  reported no findings; no fixes or re-review required. PR and exact-head CI
  remain pending.
