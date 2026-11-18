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
- Open release gates: qualify the actual `send_message` to `send` cutover and
  rollback boundary with retained native ownership, not just New and Resume
  within the new catalog. Independently settable/amendable Goals and explicit
  harness Work failure remain unqualified in the integrated product; the
  storage transitions alone do not satisfy those requirements. Do not add a
  legacy alias, infer failure from a turn, or mark the snapshot implemented.
