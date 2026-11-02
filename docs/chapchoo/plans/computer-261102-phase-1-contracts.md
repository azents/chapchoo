---
title: "Agent Computer Phase 1: Domain RPC Contracts"
created: 2026-11-02
tags: [agent, computer, plan, grpc]
---

# Phase Execution Plan

- Phase: 1/4, account-domain contract and adapter replacement.
- Branch/base: `design/agent-computer-management` -> `origin/main` (`5b1b723979aa1f3bad13283a98680c234656a6bf` at start).
- PR boundary: existing account behavior using eight scoped domain wire services and scoped adapters, with every generated Rust/Node caller migrated together.
- Inputs: Requirements r4; ADR D1-D26; approved Design r2; implementation plan.
- Deliverables: Authentication, PersonalPreferences, AccountLifecycle, Teams, Invitations, Installation, AccountGrants, DisplayDefaults; no old broad services/catch-all adapter; server registration and service health names follow actual contracts.
- Non-goals: new Computer APIs/schema/UI/credentials/Kubernetes/Control listener; fixed role configuration replacement activates with working Control in phase 3.
- Interfaces: message payload/auth/status behavior retained; new domain service paths are a coordinated wire cutover, not aliases.
- Approved Design mechanisms: M1 (contract portion), M16 (existing account domains), M20.
- Authority references: REQ-10/15, ADR-D2/D22 and current account RPC/storage/web Specs.
- Design delta: None.
- Removal obligations: broad Auth/General/Admin wire grouping, unrelated transport-trait implementations on one RpcService, corresponding generation/caller references and obsolete layout tests.
- Absence verification: repository search finds no old generated service/client registrations or product callers; generation check and live Node/Rust tests exercise replacement services.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Protocol/adapters | /root | proto/chapchoo/accounts.proto; rust/crates/rpc; rust/crates/protocol | Approved domain map | Scoped generated traits/adapters | Format, clippy, RPC contract tests |
| Runtime/callers | /root | rust/apps/chapchoo; typescript/packages/rpc-client; typescript/packages/web-core; typescript/apps/web/src/**/server; typescript/apps/admin-web/src/**/server; testenv; scripts | New protocol | Atomic caller/health migration | Rust and live Node tests, affected TS tests/typecheck/build |
| Integration/docs | /root | docs/chapchoo; DEVELOPMENT.md; applicable chart health service literals | Prior work | Current Spec and checkpoint | Catalog/tests/search/CI |

- Integration order: protocol -> scoped adapters -> registration/health -> generated Node -> caller/test migration -> integrated validation -> reviewer -> PR -> CI.
- Independent review: `/root/computer-reviewer`, complete stable phase diff only; root owns all changes.
- Final validation: root-owned repository Rust/RPC checks and applicable TypeScript lint/typecheck/test/build; docs validation plus 18 catalog tests; exact SHA CI.
- Scope-drift check: account semantics preserved, no new data/permission/product scope, no placeholder Computer/Control APIs.
- Context checkpoint: implementation and integrated local validation complete. Independent read-only reviewer `/root/computer-reviewer` reported no concrete findings after reviewing all 47 staged files and independently comparing 37 handler/message contracts. PR/CI are next.

## Integrated Checkpoint

- Replaced all 37 RPC method registrations with eight domain services and eight
  scoped Rust adapters. TeamManagement/InvitationManagement avoid Protobuf's
  package-wide service/message name collisions with the existing Teams/Invitations
  DTOs; names were illustrative, not a changed product boundary.
- Generated Node contracts and every product/test caller changed together.
  AccountServices is a composition factory, not a handler implementing all traits.
  Application/persistence authorization is unchanged. Exact body comparison
  confirms all 37 methods differ only by scoped-context access and formatting.
- Health reports only actual common/listener domain service names. Chart probes
  and contract tests use TeamManagement/Installation; old service aliases removed.
- Validation: Rust full quality script (67 tests, both DB prerequisites, clippy
  and dependency boundaries); updated listener test rerun separately; SQLite and
  PostgreSQL live Node-to-Rust each passed 74 checks; affected lint/typecheck
  14 tasks; both web production builds; General tests 11, Admin 15, web-core
  suites 51+7 and testenv 5; chart contracts 13; docs tests 18/catalog passed.
- Local SWC native-cache prerequisite: use a 0700 current-user cache outside the
  group-writable workspace. `SWC_NATIVE_BINDING_CACHE=/home/agent/.cache/swc` was
  used only for this Runtime's verification; no deployment default was changed.
- Initial compiler service/message naming collision and two new-fixture mistakes
  (duplicate variable and nested User display assertion) were corrected and the
  affected checks rerun. No known remaining local test failure.
- Scope drift: None. Later-phase runtime group selection/Control, schema and
  Computer product behavior remain unimplemented, not claimed complete.
