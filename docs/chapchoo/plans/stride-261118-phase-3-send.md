---
title: "Conversation Work phase 3: send and Chat summary"
created: 2026-11-18
tags: [backend, frontend]
---

# Phase execution plan

- Branch/base: `feat/stride-conversation-work-send` → PR #143.
- Inputs: [approved revision 17](../design/stride-261118-conversation-work-continuation.md),
  [backend phase](stride-261118-phase-2-admission.md), and the approved summary design reference.
- PR boundary: server-owned Conversation Work, scoped `send`, its exact native
  catalogue cutover, the private Chat summary, and core product acceptance.
- Deliverables: one atomic tool transition per successful request; active work
  projected above the existing composer without changing receipt ordering.
- Approved mechanisms: `M1`, `M6`–`M10`. Authority: `REQ-1`, `REQ-3`,
  `REQ-5`–`REQ-7`, `ADR-D6`, `ADR-D11`–`ADR-D22`, `ADR-D26`, `ADR-D27`.
- Design delta: `None` for this phase. No managed compaction restore activation,
  general Goal lookup, new composer, or Admin UI. `M5` remains separately open.
- Interfaces: existing authenticated native tool scope and Chat/API authority;
  `send` supplies an explicit work action and optional publication as approved.
- Removal obligations: replace `send_message` catalogue, dispatch, instructions
  and matching product fixture assumptions together. Preserve prior receipts
  and native records. Verify absence using effective New and exact Resume as
  well as source/catalogue inspection.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Work transition | Primary agent | Rust domain, persistence migrations and conversation Store | Phase 2 Goal and count | Atomic Work, goal eligibility, and optional receipt; fresh user acceptance closes wait | SQLite/PostgreSQL parity, both wait race orders, silent versus publication, reset semantics, rollback |
| Tool and projection | Primary agent | Service, RPC, generated contracts, native catalogue/instruction call sites | Work transition | One scoped `send` contract and owner summary read | ACL, exact native scope, New and Resume cutover, loss without blind retry |
| Chat and acceptance | Primary agent | Private conversation web and existing browser/native fixtures | Owner projection and catalogue | Approved compact summary above unchanged composer | Initial and live render, interaction/reduced motion, core Docker browser journey |

- Integration order: domain and Store, tool and projection, native cutover, web,
  product acceptance, Spec alignment, stable integrated review.
- Independent review: `/root/stride-reviewer`.
- Final validation: Rust and RPC repository checks; frontend format, lint, types,
  tests and build; deterministic native and browser acceptance for the approved
  flow. Identify the real native and fixture versions separately from synthetic
  model evidence. Report missing live prerequisite coverage explicitly.
- Scope-drift check: `send` only; no status inferred from native output, no
  alternate legacy tool, no waiting-state side effects for message-only cases.
- Context checkpoint: backend phase #143 at `8b5e881` passed Rust/dual-DB/RPC
  checks and review. Managed compaction restore is not implemented in that base.
- Cutover evidence: isolated Docker retained the conversation, snapshot,
  native session, saved volume, and receipts across the catalog transition
  after the legacy effect settled. Paired SQLite and PostgreSQL backup
  restoration was verified using the original installation root. This is
  not mixed-version or uncertain-effect handover evidence.
- Full history checkpoint: the Docker browser journey proves automatic input
  remains absent from Chat receipts but readable with its body in Full history.
  Retain captured items regardless of whether their origin is known.
- Storage verification follow-up (`M1`, `M7`): exercise explicit failure of
  the exact Work-associated goal using the existing Store operation on both
  database engines, including the shared summary/context disposition, no
  publication, unchanged unrelated goals, and source and retry fencing.
  The test does not measure the admission count. This follow-up changes no
  production path or authority.
- Work context amendment (`M1`, `M4`, `M5`, `M6`, `M10`; `REQ-1`,
  `REQ-2`, `REQ-4`, `REQ-5`, `ADR-D20`, `ADR-D26`): project the already
  authoritative Work title, ordered task list and task states with the
  goal's disposition, without storing another copy or changing the fixed
  completion condition. Check the existing full-context byte bound inside
  each Work write transaction. Verify order/status, empty versus independent
  context, size-rejection rollback and input-wait retention on both engines.
  Verify the deterministic product continuation receives the latest tasks.
  No change to the `send` catalog, public authority, or restore activation.
- Open release gates: the standalone Goal transitions have no product
  caller. The `send` actions manage one associated goal but cannot add or
  amend an independent goal, or express an explicit harness Work failure.
  Revision 17 does not specify a distinct authorized producer and lifecycle
  for those commands. Do not expose a goal control, infer failure from a
  turn, or mark the snapshot implemented.
- Managed restore follow-up (`M5`, `REQ-4`, `ADR-D7`, `ADR-D8`, `ADR-D25`,
  `ADR-D28`): configure official Codex `SessionStart(compact)`, call the
  existing current-source context RPC through a bounded per-launch local
  listener, and verify application before the next inference. No added
  caller authentication, separate process isolation, or patched native
  dependencies. Qualify absence, timeout, Stop and source replacement.
