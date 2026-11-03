---
title: "Docker Computers Phase 2 Durable Native Authority"
created: 2026-11-03
tags: [computer, docker, implementation, database, grpc]
---

# Phase Execution Plan

- Phase: `2/4 durable native authority and protocol`.
- Branch/base: `feat/docker-computers-2-native-authority` →
  `feat/docker-computers-1-native-contracts` at `9b82d83`
  ([PR #49](https://github.com/azents/chapchoo/pull/49)).
- PR boundary: server-side data preservation and native authority, coordinated
  source-generated protocol/DTO contracts, existing K8s producer/consumer updates
  and owning Specs. No Docker SDK/runtime effect activation.
- Inputs: phase 1 shared native values, 122 passing tests, all hooks including
  generated RPC/Node interoperability passed, independent review without findings.
  PR #49 passed all nine exact-head CI checks and remains unmerged; this phase
  has no merge or deployment authority.
- Deliverables: lossless forward migration on SQLite/PostgreSQL; immutable scoped
  Provider substrate/daemon binding; typed native reservation and evidence;
  complete substrate-specific resource/role gates; bounded native inventory;
  native wire payloads and generated clients; server-side retirement CAS foundation.
- Non-goals: Docker native IO/context resolver/transport/effect supervisor, host
  changes, Mac claims, UI redesign, open enrollment, new third-party credential
  management, compatibility versioning, automatic uncertainty reset or replay.
- Interfaces:
  - External Providers use authenticated Control RPC only; product DB/root
    initialization stays exclusively in server commands.
  - Configure/bind uses existing SYSTEM-owner scope and Provider locking; an
    accepted kind/daemon cannot be silently changed by later metadata or keys.
  - Internal reservations/evidence use native ADTs; K8s wire field numbers retain
    meaning and normalize to one native representation at ingress. Docker payloads
    cannot mix with K8s identity, bootstrap Secret or namespace fields.
  - Saved PVC/volume roles are derived from kind; exact original identity survives
    unknown/absence evidence. Current source/request/execution checks remain.
  - Inventory cursor is the canonical bounded kind/native-scope/name tuple;
    nonsecret native identity is not paging or authorization authority.
- Approved Design mechanisms: M1/M3/M4/M5/M8/M10/M11 and the complete server-side
  M16 admission/correlation boundary; M12 owner-layer validation.
  M2/M6/M7/M9/M13/M14/M15 and native SDK stage execution remain phases 3/4.
- Execution refinement: server-side retirement correlation is integrated with
  native dispatch in this phase, not deferred until the SDK adapter. This prevents
  any new wire path from bypassing the approved once-only fence. M16 itself is
  unchanged: exact phase CAS before forwarding, bounded live source/call/native
  manifest, observation-only ambiguity/loss, and one coordinated Delete prefix/
  never-submitted suffix. No generic reset or new recovery authority is added.
- Authority references: [REQ-1/3/4/6](../requirements/docker-261103-server-computers.md),
  [ADR-D1/D3/D4](../adr/docker-261103-server-computers.md),
  [Design revision 6](../design/docker-261103-server-computers.md), current
  computer-storage/directory/control-rpc/web Specs, typed-ingress and
  schema-dependencies-in-living-specs conventions.
- Design delta: None.
- Removal obligations: replace K8s-only Provider/resource checks, shared
  namespace-only model/cursors and K8s-only reservation/evidence completeness in
  owning server paths. Preserve the actual K8s adapter and its native identities.
  Generated clients are regenerated, never hand-patched. Runtime removal of shared
  K8s effect ownership belongs to phase 3.
- Absence verification: both-engine fixtures compare all eleven migrated tables'
  original identities/credential BLOBs/routes and external receipts; SQLite
  foreign keys stay on, no unexpected incoming dependency or staging object
  remains; search and compile checks eliminate namespace-only shared DTO/cursor
  authority and unknown-kind-to-Secret mapping from touched ingress.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Preservation schema | `/root` | `rust/crates/persistence/migrations/{sqlite,postgres}/00000000000006_native_computers/`, `src/preparation.rs`, migration/schema tests | Approved closed eleven-table graph | Forward-only native checks/columns, exact preservation with rollback | Populated SQLite/PG migration fixtures, incoming-FK refusal, FK/uniqueness checks |
| Native store authority | `/root` | `rust/crates/persistence/src/{computers,computer_executions,component_keys,directory,delivery,lib}.rs` and owning tests | Schema + domain values | Immutable substrate/binding, typed reservation/evidence, resource-role completeness, retirement CAS | Both-engine current-source/identity/absence/no-replay tests |
| Service and wire | `/root` | `rust/crates/service/src/{computers,lib}.rs`, `proto/chapchoo/{control,computers}.proto`, `rust/crates/rpc/src/{computers,reports,native_resources,component_auth,control_calls,control_streams,peer}*`, server composition and tests | Native store contracts | Typed ingress/mixed-input refusal, shared dispatch/dequeue correlation and complete M16 server gates | Owner/peer/frame/codec/privacy and no-replay tests, generated Node/Rust interoperability |
| Existing producer integration | `/root` | `rust/apps/chapchoo/src/provider*`, `rust/crates/kubernetes/src/*`, affected native fixtures | New generated contracts | Behavior-preserving explicit K8s native mapping; no Docker SDK work | Existing K8s adapter/Control/child-process regressions |
| Generated artifacts and inventory | `/root` | `typescript/packages/rpc-client/src/generated/`, affected shared DTO/cursor mapping in web/Admin server boundaries, owning Specs | Source wire/DTO contract | Source-generated clients and native bounded inventory; unchanged page behavior | Generator check, format/lint/typecheck/build where changed, cursor/privacy tests |

- Integration order: exact schema fixtures and forward migration → server native
  values/queries/binding/reservation/evidence → wire and service/RPC normalization
  → existing K8s producers/fixtures → generated clients/DTO consumers and owning
  Specs → root integrated validation → freeze/review/corrections → phase PR.
- Independent review: `/root/computer-reviewer`, complete stable integrated phase,
  read-only one-way findings only after root validation. Root implements fixes
  and owns all affected reruns.
- Final validation: full `bash scripts/check-rust.sh`, `bash scripts/check-rpc.sh`,
  docs catalog and diff checks; both-engine migration/authority fixtures;
  applicable TypeScript workspace checks for changed consumers; exact-head CI.
  Use the authoritative isolated PostgreSQL runner, not a production URL.
- Scope-drift check: every new persisted/native field has approved authority;
  all DB access remains server-side; no new credential distribution product;
  Docker effects cannot be forwarded before the complete server once-only gate;
  native SDK execution is activated only with the phase-3 adapter;
  existing K8s identity/data/cleanup and generated field meanings remain intact.
- Context checkpoint: the child branch remains based on `9b82d83`; phase 1 PR #49
  passed all nine exact-head CI checks and remains unmerged. Forward schema 6,
  immutable native binding, typed row/identity ingress, role-aware reservations/
  evidence and native RPC/DTO payloads are integrated with existing K8s call
  sites. Root's full `bash scripts/check-rust.sh` passed format, all-target Clippy,
  all 142 tests across 47 groups (zero failures/ignored) with isolated PostgreSQL
  and SQLite, and dependency boundaries. Source-generated Node clients and
  `bash scripts/check-rpc.sh` passed generation comparison/build and 67 live
  Node/Rust interoperability checks on each engine. Both-engine native execution
  tests cover original storage, binding/identity collision, exact retirement CAS,
  late-Running refusal, direct/scoped Delete uncertainty and subset-absence
  cleanup refusal. The scoped test initially raced PostgreSQL's global prepare
  advisory lock; its two scenarios now run sequentially within one PG test,
  preserving the production lock and avoiding sleeps/retries.
  M16 now integrates bounded connection-local source/call/native correlation,
  exact late results, all Stop/Restart/terminal/Delete entry points, current-source
  absence and only known-never-submitted Delete suffix progression. Five synthetic
  Provider/real Store regressions cover normal progression, lost waiter, reconnect,
  NotStarted/ABA and early retirement paths. Browser DTO/cursor mapping and Admin
  native diagnostics retain long daemon IDs, full container IDs and original
  volume creation identity without endpoint/credential/volume-metadata leakage.
  Root validation passed workspace format/lint/typecheck, all Web/Admin/UI/
  web-core/testenv/E2E-unit suites, complete web build, Admin Storybook and all ten
  executable Admin component checks, including narrow native diagnostics/cursor
  restoration. RPC was rerun after M16 and passed 67 checks per engine. Owning
  storage/directory/control/web Specs and catalog/diff checks are updated.
  Independent review found one response-waiter/native-task lifetime mismatch:
  transport timeout incorrectly ended the exact running task's next-stage checks.
  Root preserves only the already-forwarded manifest across waiter loss, still
  refuses repeat dispatch, and ends continuation on explicit native settlement or
  source/correlation loss. A sixth regression crosses the actual short waiter
  deadline and the public authenticated CheckCommand RPC, then verifies exact
  continuation, observation-only convergence, no second Retire and Completed/
  Uncertain native settlement. Full Rust and RPC gates passed again after the fix;
  unchanged frontend evidence remains applicable. The corrected integrated diff
  received targeted same-reviewer re-review with no further findings; the P2
  lifetime correction is resolved. Commit, stacked phase PR and exact-head CI
  remain pending. No real
  Docker operation has occurred. Native Mac/real Desktop and selected remote-mode
  acceptance remain unavailable, not passed. Design delta: None.
