---
title: "Agent Computer Phase 2: Data, Credentials and Lifecycle Admission"
created: 2026-11-02
tags: [agent, computer, plan, security, database]
---

# Phase Execution Plan

- Phase: 2/4, common/cloud domain persistence and application authority.
- Branch/base: `feat/computer-domain-state` -> `design/agent-computer-management` at `81a2403d353b86c9a38dd91b81ea82a61c1bdb44` (PR #40).
- PR boundary: typed Computer/lifecycle values, equivalent SQLite/PG records and atomic operations, Provider owner/key and Connector session/key authority, server projection and service-level admission with deterministic tests.
- Inputs: approved Design r2 M1-M22; Requirements r4; accepted D1-D26; phase1 domain contracts.
- Deliverables: owner-scoped creation receipt; permanent cloud dedication; current request preconditions/phase; independent cleanup; dual-dialect migrations/root guard; protected current credential recovery and explicit system-owner verifier. No external effect is performed by DB operations.
- Non-goals: component networking/peer routing/Kubernetes client; user/admin screens; actual Mac enrollment/runtime; runtime group/listener activation; production credential provisioning; deployment/merge.
- Interfaces: detached typed values, opaque exact identities, digest/fingerprint checks, caller-supplied generated drafts validated by services, current authority rechecked under short DB-only transactions. No universal command ledger, generation, Redis or implicit migration.
- Approved Design mechanisms: M3 (credential authority), M7-M10, M12-M17, M19/M20/M22; remaining connection mechanics activate in phase3.
- Authority references: REQ-1/3-13/18, ADR-D7/D9-D16/D20/D26, unchanged account-storage and crypto rules.
- Design delta: None.
- Removal obligations: extend exact migration readiness and pre-root product-data guard in both dialects. Account data, root binding, sessions and prior migration identities remain intact. No preexisting Computer schema to delete.
- Absence verification: startup stays read-only; new data prevents key rebinding; no current-operation history table/archive, cascade-erased creation receipt or global uniqueness that forbids future local sharing.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Typed domain | /root | rust/crates/domain/src | Approved lifecycle | Parseable enums/predicates/current state | Exhaustive transition/evidence tests |
| Persisted authority | /root | rust/crates/persistence/src; migrations; tests | Domain values | Atomic owner/current-request/cleanup/credentials operations in both engines | Real SQLite/PG fixtures, conflicting requests, no resurrection, exact recovery |
| Application boundary | /root | rust/crates/service; crypto only where existing primitive reuse needs a typed boundary | Persistence | Validated identifiers, owner verifier, key/session and lifecycle service | No secret Debug/diagnostics, direct authority tests |
| Integration/docs | /root | docs/chapchoo/spec; plan/check scripts as needed | Prior work | Accurate current schema/authority descriptions | Format/clippy/tests/dependency boundary check, docs tests |

- Integration order: domain -> schema/prepare -> DB operations/credentials -> service projection/admission -> both-engine tests -> stable independent review -> PR.
- Independent review: exact `/root/computer-reviewer`, root requests only after stable integrated diff and validation; root applies every correction.
- Final validation: full Rust quality suite with actual disposable PostgreSQL/SQLite; generation/live RPC regression; migration/prepare tests and domain/credential race evidence. Do not count unimplemented network/Kubernetes as verified.
- Scope-drift check: cover approved current intent/recovery/auth/cleanup semantics; preserve account behavior; no future Mac workflow, new user administration or public cleanup actions.
- Context checkpoint: domain values/admission/restoration predicates, scoped
  system-owner verifier and canonical fingerprints are implemented. Equivalent
  migrations, extended first-binding guard and actual SQLite/PG schema tests
  cover ownership, dedication, receipts, execution/resource scope, saved PVC and
  refusal of destructive downgrade. Current Store methods implement configured
  Provider metadata, eligible choices, atomic creation and owner reads, current-
  request power/deletion admission, force removal and Provider credential
  rotation/recovery/revocation. Qualified current power is separate from last-known
  evidence; deletion atomically withdraws Connector secrets and routes.
  Connector session issuance, reusable bootstrap exchange, direct operational
  refresh and irrevocable session revocation are now implemented and exercised.
  Connector self-auth uses Computer/session authority without a global Provider
  lock or a live Provider key/stream prerequisite.
- Integrated pre-review validation: full Rust quality script passes with 82 tests on real
  disposable SQLite/PostgreSQL, clippy and dependency boundaries. Same-engine
  operation exercises cover creation races/replay/isolation, power conflicts/
  stale evidence, force removal, deletion credential withdrawal, Provider key
  lost-result recovery/overlap/expiry/revocation/stale replay. Earlier unused
  fields are now used by actual operations; no warning suppression.
  Expanded Connector exercises pass on both engines: unique binding recovery,
  role separation, lost result, bounded previous-key handover, no stale exchange
  invalidation, Provider-independent refresh, same-request concurrent recovery,
  one accepted successor for competing distinct requests and terminal revocation.
- One full run encountered an existing executable-fixture ETXTBSY in
  `forced_cleanup_reaps_adopted_descendants_and_reports_failure`. The focused
  test and subsequent full quality run passed. No cause/fix is claimed and no
  unrelated executable/fixture behavior was changed.
- Phase1 PR #40 head `81a2403` has all remote CI including browser E2E and both
  architecture packages successful; automated review completed with no findings.
- Execution/resource reservation and current-source observations/retirement/
  restart/cleanup are implemented. Exact Control/Connection/route credential and
  current key are rechecked using DB time under locks. Resource names are reserved
  before effects, Provider-scoped uniqueness prevents cross-Computer ownership,
  and original UIDs/confirmed absence prevent same-name adoption and missing-
  storage replacement. Unknown preparation retains the current candidate.
  Stop/restart require actual exact execution retirement; an uncertain display
  phase cannot erase that gate. Connector authority is withdrawn on retirement.
- Independent managed inventory survives force removal. Normal confirmation
  requires all managed resources absent and no live execution; explicit collection
  uses confirmed completion plus 24 hours. Pending cleanup has no TTL and creation
  receipts survive Computer collection.
- `ComputersService` generates IDs/secrets and uses a composition-configured scoped
  owner verifier. It composes owner lifecycle/key methods and internal current-
  source managed operations without opening listeners or performing external IO.
- Both-engine managed tests cover candidate recovery/races, resource collisions
  with rollback, current-source/key/Control rejection, restart uncertainty,
  session retirement, current-source restoration/capabilities, storage namespace/
  UID continuity and loss, initial never-allocated continuation, stop/start,
  force/normal cleanup and completion retention. Fixture-only PostgreSQL mutations
  explicitly leave the async runtime; a nested-runtime fixture failure was fixed
  and both-engine plus full-suite reruns pass.
- Scope checkpoint: Design delta: None; approved phase-2 data, credentials,
  lifecycle admission and application boundaries are implemented. Current domain
  power/management projections are not working component/heartbeat/UI evidence;
  runtime directory activation, Connector connectivity projection, final
  user/Admin DTOs and their actions belong to phases 3/4.
- Generated-client check/build and live Node/Rust RPC regression passed on both
  SQLite and PostgreSQL (74 checks each). Documentation catalog validation and
  its 18 tests pass. No new Computer RPC or external lifecycle E2E is claimed.
- Independent review of the complete phase found two P2 lifecycle issues:
  initial CREATE dispatch classification changed after PVC confirmation, and
  issue phases could preserve an obsolete recovery dispatch timer. Corrections
  persist the candidate's initial-allocation classification and first actual
  running evidence independently of PVC observation/current display phase.
  Both-engine regressions cover create-only Provider continuation and direct/
  uncertain retirement into a fresh recovery interval, with same-attempt retry
  timestamps preserved. Corrected full Rust quality (82 tests/clippy/boundaries),
  generated/live RPC (74 checks per engine) and docs (18 tests/catalog) pass.
  The exact independent reviewer `/root/computer-reviewer` completed targeted
  re-review: both findings resolved, no correction-introduced material issues.
  Scope, authority, storage continuity and retry gates remain intact.
- PR #41 was submitted at `3d0fdd2` against PR #40; all remote CI including
  browser E2E and native packages passed. Automatic review then identified
  missing session/Computer route constraints and deletion Request ID checks after
  Agent removal. Both are corrected with composite ownership FKs, unique session
  routing, and retained minimal removed-Agent identity on residual Computer
  inventory. Wrong-ID removal retries/races reject; after inventory collection no
  deletion request can be proven and removal retry returns not-found. Creation
  receipts remain minimal and continue returning removed on creation replay.
- Corrected full Rust quality passes (82 tests/clippy/boundaries), live RPC passes
  (74 checks per engine), and docs18/catalog/diff check pass. The exact independent
  reviewer completed targeted re-review: both automatic findings resolved, no new
  material correctness/security/data-loss findings, and Design delta remains None.
  Correction commit `9ef8d64` passed all remote CI. Subsequent automatic review
  found two terminal-report paths: late unknown resource evidence after cleanup,
  and issue reports after completed stop. Both are rejected under current-Computer
  locks; both-engine regressions prove preserved cleanup/collection and next-start
  admission. Full 82-test Rust quality, live RPC74 per engine, docs18 and targeted
  same-reviewer review pass again with no new material issues. Latest correction
  commit/hooks and new-SHA CI follow. No component runtime or Mac flow is exposed
  in this phase.
