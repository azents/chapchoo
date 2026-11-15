---
title: "Default Project Implementation Plan"
created: 2026-11-16
tags: [agent, implementation]
---

# Default Project Implementation

- Requirements: [prompt-261115/REQ](../requirements/prompt-261115-system-prompt-management.md)
- ADR: [prompt-261115/ADR](../adr/prompt-261115-system-prompt-management.md)
- Approved Design: [prompt-261115/DESIGN](../design/prompt-261115-system-prompt-management.md), revision 4, exact IDs M1–M8.
- Design delta: None.
- Delivery: one integrated feature PR including schema, protocol, UI, native orchestration, tests and current Specs.
- Root owns integration, verification, corrections and PR/CI observation.
- Single independent reviewer: refresh-reviewer, read-only complete stable integrated diff.

## Execution decomposition

1. Settings and evidence: owner-authorized saved prompt API, preserving SQLite/PostgreSQL migration, immutable operation/Computer-execution source bundle and exact component read/result checks (M1/M3/M6).
2. Project publication and refresh: bounded common selected-source DTO; existing default workspace; atomically published Codex AGENTS.md with first-original preservation; exact operation transport and live actor refresh; supported task/child tracking and complete old tool ownership; same-context restore and initial title persistence (M2/M3/M4/M8).
3. Owner web: shared system-prompt row/modal/fullscreen, Save-only, source read view and input/native readiness separation; regenerate transport clients from proto (M5/M6/M7).
4. Integrated tests and current Specs: lower layers for all state combinations; one minimal core E2E; real released-program fixed-response delivery evidence; full Rust/TypeScript/RPC/doc checks, stable review, corrections, PR/CI (M1–M8).

Self-contained settings/evidence and owner UI slices may use delegated implementation support, with root integrating and validating. Native orchestration is root-owned. Concurrent work uses nonoverlapping files; shared proto consumers are coordinated through explicit contract messages.

## Removal obligations

Replace configuration-blind live Setup reuse for changed selected content; replace obsolete catalogue mismatch-only behavior at the new-owner boundary; separate refresh-unready from UI input eligibility; keep historical sources outside consumable rows; preserve initial unmanaged instruction file once and all ordinary user files. Verify unchanged lookup behavior remains, failed evidence cannot launch, and no native event/capture migration is introduced.

## Verification and checkpoints

Before implementation: exact Design approval recorded; Requirements include the general-purpose project and autonomous delivery scope. Checkpoint after storage/API, runtime integration and UI integration. At each, compare forward requirements and reverse M1–M8 authority, affected removal and source-preservation paths. New material mechanism returns to the delegated decision owner before implementation.

Run scripts/validate_docs.py and its tests; scripts/check-rust.sh including isolated SQLite/PostgreSQL; scripts/check-rpc.sh; repository TypeScript format/lint/typecheck/unit/build and app component checks; designated native fixed-response checks and minimal E2E. Record missing prerequisites/failures honestly. Freeze diff for refresh-reviewer after integrated checks and apply findings at root. Open PR and observe exact-head CI. Remove this temporary plan after verified feature/spec promotion.
