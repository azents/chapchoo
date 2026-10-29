---
title: "Account and Team Onboarding Implementation Plan"
created: 2026-10-29
tags: [accounts, implementation, backend, frontend]
---

# Account and Team Onboarding Implementation Plan

- Snapshot: `accounts-261029`.
- Authority: [Requirements](../requirements/accounts-261029-server-team-onboarding.md),
  [ADR](../adr/accounts-261029-server-team-onboarding.md),
  [approved Design revision 36](../design/accounts-261029-server-team-onboarding.md).
- Approved mechanisms: M1-M34 and M36-M43; M35 retired.
- Design delta: None.
- Implementation owner/integrator: `/root`.
- Single independent read-only reviewer: `/root/accounts-reviewer`.
- Current Specs: credential primitives, account storage and RPC/CLI behavior;
  later web and delivery surfaces are not implied by those backend Specs.
- Approval and explicit implementation request received on 2026-10-29 KST.

## Delivery Phases

| Phase | Branch/base | Approved mechanisms | Deliverable and boundary |
| --- | --- | --- | --- |
| 1/5 | `feat/accounts-foundation` / `main` after bootstrap merge | M23, M24, M30, M32, M36, M37, M41 | Rust workspace, typed domain/credential primitives, common crypto boundary, deterministic tests and quality commands; adopt approved snapshot and plans in the same PR |
| 2/5 | `feat/accounts-persistence` / phase 1 | M2, M4, M10-M12, M16-M22, M25, M26, M29, M36, M40 | Both-engine Diesel migrations and completed operations, constraint/concurrency tests, explicit preparation and fixed-key checks |
| 3/5 | `feat/accounts-rpc-cli` / phase 2 | M1, M3, M5, M6, M8, M13-M15, M18-M22, M25, M28, M30-M32, M43 | Generated RPC contracts, shared services, mail adapter, operator/foreground CLI, owned shutdown and protocol tests |
| 4/5 | `feat/accounts-web` / phase 3 | M1, M3, M4, M7, M25, M27, M34, M38, M42 | Shared UI extraction, general/Admin account/team screens, generated Node gRPC adapters, writable-cookie renewal and representative browser journeys |
| 5/5 | `feat/accounts-delivery` / phase 4 | M7, M9, M28, M33, M39, M40, M42, M43; integrated M1-M34/M36-M43 | Linux packages/images/chart, Helm/Argo CD preparation hooks, integrated validation, current Specs, approved live QA when prerequisites are supplied, plan cleanup |

Open each integrated phase PR before beginning the next phase. Do not merge
without separate explicit permission. Earlier phase review corrections propagate
to dependent branches through the stacked-PR workflow. No phase adds Design
mechanisms or resets the approval boundary.

## Interfaces and Ownership

- Domain primitives do not depend on Tonic or Diesel. Persistence owns complete
  DB-only transactions and returns detached typed results. Protocol adapters
  depend on application capabilities, not DB implementations.
- Both selected databases enforce the same domain outcomes with explicit
  engine-specific SQL, transaction admission and migration fixtures.
- Generated Protobuf contracts are the Node/Rust transport source; credentials
  remain per request. Web code has no business DB or installation-root access.
- `/root` owns code, integration, validation, PRs and review corrections. The
  exact reviewer above reviews every frozen integrated phase diff read-only.

## Validation and Prerequisites

Phase checks include rustfmt/Clippy/compiler/unit tests, real SQLite/PostgreSQL
integration, generated-contract consistency, TypeScript quality/build checks,
minimal E2E and package/chart checks as the surfaces become available. Preserve
exact commands, SHA, environment and honest failures in phase checkpoints.

Required external live prerequisites: agreed Kubernetes cluster, namespace,
exposure and data-resource scope; existing database and fixed-key Secret; TLS
proxy/Ingress; image availability; Argo CD full-application sync authority. Do not
write to a cluster before those boundaries are authorized. Missing live QA is
reported as blocked, never passed from chart rendering.

## Removal and Spec Obligations

Phase 4 replaces scaffold Hello World product routes/stories and extracts the
app-local theme to shared UI, verifying consumers/assets and removing superseded
definitions. Reserved placeholders are removed only when occupied. No legacy
account data or protocol conversion exists. Phase 5 promotes reachable current
behavior to Living Specs, sets matching implemented dates only after complete
verification, then removes this plan and all phase plans.

## Context and Scope Checkpoints

Bootstrap PR #1 was merged by the requester at `9031127`. Its original commit
is preserved as a merge parent. PR #2 was reopened and retargeted to main after
its deleted base closed it; head `486c962` was unchanged and new CI passed.
PR #3 remains open at `563e9a9` with phase 1 as base and successful exact-head
CI. No implementation PR has been merged by the implementation agent.

At every phase boundary record changed interfaces, approved coverage, unauthorized
addition check, removal evidence, integrated test results, reviewer findings,
PR/base/head and remaining work. Return any new product or material mechanism to
Requirements/ADR/Design approval; keep identifiers and equivalent local layout
agent-owned. No key replacement, generic replay, native client, broad CRUD or
unrequested infrastructure work is included.
