---
title: "Agent Computer Implementation Plan"
created: 2026-11-02
tags: [agent, computer, plan]
---

# Agent Computer Implementation Plan

- Requirements: [computer-261102/REQ](../requirements/computer-261102-agent-computer-foundation.md), revision 4.
- ADR: [computer-261102/ADR](../adr/computer-261102-agent-computer-foundation.md), D1-D26 accepted.
- Design: [computer-261102/DESIGN](../design/computer-261102-agent-computer-foundation.md), approved revision 2, M1-M22.
- Design delta: None.
- Implementation owner: primary agent `/root`.
- Exact single independent reviewer: `/root/computer-reviewer`, read-only, every stable integrated phase diff; root requests reviews and makes all corrections.
- Authorization: direct requester instruction to proceed through implementation after the complete Design and two recommendations were presented. Merge/deployment/production provisioning remain excluded.

## Delivery Stack

| Phase | PR boundary                                                                                                | Mechanisms                    | Dependencies and integration                                                                              |
| ----- | ---------------------------------------------------------------------------------------------------------- | ----------------------------- | --------------------------------------------------------------------------------------------------------- |
| 1     | Account domain wire contracts and scoped transport adapters; generated callers migrated atomically         | M1/M16/M20 (contract portion) | Existing account operations unchanged; runtime listener groups remain until phase 3 activates replacement |
| 2     | Computer data, owner/credential authority and lifecycle admission                                          | M7-M10/M12-M15/M17/M19/M22    | Phase 1 contracts; equivalent SQLite/PG migrations, precise authorization and lifecycle fixtures          |
| 3     | Runtime groups, memory/peer routing, current Control/Provider/Connector streams and Kubernetes convergence | M1-M6/M10/M11/M13/M14/M18/M19 | Phase 2 persisted identities; activate group/listener replacement with new component clients              |
| 4     | Owner/admin screens, bounded observation, chart/config integration and end-to-end validation               | M15/M16/M18/M20/M21           | Prior phases; minimal core product E2E and separate real-cluster gates                                    |

Each phase opens its PR before the next phase starts; later branches depend on the preceding branch. Use `Agent Computer [n/4]: <phase>` titles. Do not create documentation-only PRs merely to satisfy the workflow. Formal design documents travel with phase 1.

Requester delivery-order update (2026-11-02 KST): finish and submit the remaining
PRs 3 and 4 before checking the stack's CI/review results and batch corrections.
Do not pause forward implementation for repeated PR2 review rounds or request
additional full automatic reviews during this construction pass. Mandatory
independent review and exact-head CI remain completion gates after all four PRs
exist; merge/deployment remain unauthorized. This changes delivery ordering only,
not Requirements/Design authority or feature scope.

## Fixed Interfaces and Removal Boundaries

D22 domain service names and typed messages are the RPC contract. Phase 1 removes Auth/General/Admin wire services and catch-all RpcService; update every Rust/Node caller and generated artifact together. It does not prematurely expose an empty Control group. Phase 3 replaces fixed backend role/listener configuration, implicit group routing and role-specific health/chart wiring when actual Control implementation exists. Phase 2 extends exact migration readiness/root-binding guards without adopting a new root. Phase 4 completes web/client configuration and Spec promotion. No legacy fallback, dual write or numeric generations.

## Validation and Prerequisites

Root runs docs validation, focused Rust tests/clippy, generated-client checks, live Node RPC against SQLite/PG, affected TypeScript lint/typecheck/tests/build. Each phase records exact evidence. CI on exact head, independent review and findings resolution precede phase completion. Later phases add deterministic two-Control, revocation, uncertainty and lifecycle tests; minimal E2E covers core owner/operator journeys. Live Kubernetes needs explicitly authorized disposable prerequisites; absent credentials are NOT EXECUTED, never success. No concrete Mac implementation or Mac runtime test.

## State, Specs and Rollout

Current account Specs remain true until each owning replacement is verified, then update them in that phase. Mark Requirements/Design implemented together only after all feature behavior and required verification complete. Preserve existing account/team/session/display/root data. Explicit prepare and coordinated contract/schema cutover; no startup migration, automatic down migration, unsupported mixed-old/new rollout, deployment, database copy or backup here.

## Context and Scope Checkpoints

At each stable phase record changed interfaces, removals, tests/review/CI, PR/base and remaining work. Check both missing approved behavior and unauthorized additions. New material mechanisms return to technical design; local file/type/fixture choices remain implementation work. Preserve unrelated worktrees. Keep plans until validated implementation and Spec promotion, then remove them and clean only owned finished branches/worktrees.

## Current Checkpoint

Phase 1 PR #40 is open/unmerged with complete CI and independent/automatic review.
Phase 2 PR #41 is open/unmerged at `09ba1ad`: equivalent Computer schema, ownership,
credentials, current lifecycle/resource evidence and application service are
implemented. Root integrated checks pass (82 Rust tests, clippy/boundaries,
74 live RPC checks on each DB engine, docs18 and normal commit hooks). Independent
full review and targeted corrections are complete. Exact-head remote CI/automatic
review are deferred until the remaining stack is submitted. Phase 3 builds on
that exact head under `computer-261102-phase-3-control.md`: runtime groups,
current-owner streams and peer/memory delivery, supported Kubernetes effects,
component processes, owner-scoped reconciliation and guarded restoration/cleanup
are implemented. Final root checks pass (107 Rust tests, clippy/boundaries,
67 live RPC checks on each DB, generated clients and docs18/catalog). PR3
submission is next; phase4 web/chart/E2E and final stack reviews remain.
The overall feature is not implemented yet.
