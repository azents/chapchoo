---
title: "Deferred Computer Input Implementation Phase"
created: 2026-11-16
tags: [conversation, computer, implementation]
---

# Phase Execution Plan

- Phase: one focused feature PR.
- Branch/base: `feat/deferred-computer-input` → `fix/chat-message-editor` at `852ed8507495031905dfc3790f8964d62cafab94` (editor PR #139).
- Inputs: [Requirements](../requirements/messaging-261116-deferred-computer-input.md), [accepted ADR](../adr/messaging-261116-deferred-computer-input.md), [approved Design revision 1](../design/messaging-261116-deferred-computer-input.md).
- Approved mechanisms: `M1, M2, M3, M4, M5, M6, M7, M8`.
- Authority: messaging-261116/REQ-1 through REQ-5, ADR-D1/D2, preserved current source/lifecycle/native-grant and account authority.
- Design delta: None.
- Deliverables: ordinary submission while stopped/starting/restoring, durable ordered waiting, Cloud-only automatic Start, explicit Stop precedence, safe pre-attempt restart recovery and no uncertain native replay.
- Non-goals: second message store/broker, new versioning/runtime mode/configuration, browser-owned wake, production mutations, merge or deployment.
- Interfaces: retain public upload/acceptance/result contracts; internal claim-fenced Store/service native admission; existing completed lifecycle Start remains sole power authority.

| Workstream                                                         | Owner | Owned paths                                                    | Depends on           | Validation                                                                           |
| ------------------------------------------------------------------ | ----- | -------------------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------ |
| Durable acceptance, session/wake/order/claim, preserving migration | Root  | persistence conversations, paired migration, persistence tests | M1/M3/M4/M5/M6/M8    | SQLite/Postgres migrations, queue/claim/source/Stop/auth tests                       |
| Bounded dispatcher, owned shutdown and native-grant fences         | Root  | service conversation, RPC processor, composition               | persistence contract | service/RPC tests, Rust quality                                                      |
| Ordinary composer, exact placeholder and native-control separation | Root  | web chat, component fixtures/tests                             | acceptance contract  | root frontend gates and components                                                   |
| Core real flow, current Wiki and integrated shipping               | Root  | E2E/testenv only as needed, owning Specs                       | integrated feature   | minimal browser/Docker core flow, docs validation, independent review, exact-head CI |

Integration order: persistence → service/processor → web → fixtures/tests → Specs → stable integrated review → corrections → commit/PR/CI. Temporary phase plan is removed after validation and spec promotion in the same PR.

Removal obligations: replace immediate process-owned acceptance as readiness lifetime, unconditional pre-attempt expiry-to-Error and readiness-only Send/source-loading admission. Prevent unowned process-only wake. Preserve native POSSIBLE, late authenticated result, failed-item explicit recovery and current lifecycle gates. Verify removed behaviors through tests and reference searches.

Independent review: `/root/motion-review`, read-only complete feature review after root freezes the integrated diff; root applies every correction and requests only required targeted re-review.

Final root-owned validation: Rust fmt/clippy/tests with disposable PostgreSQL; root frontend format/lint/typecheck/build and affected units/components; minimal model-free Docker/browser flow; docs and hooks; full final-head GitHub CI. Missing mandatory prerequisites fail rather than becoming passes.

Scope-drift check: every Requirement and M1–M8 covered, no additional authority/config/mode/replay path. New material findings return to dedicated design decision owner; fixed product scope is not expanded.

Context checkpoint: implementation and connected Spec edits complete. Local
SQLite/PostgreSQL conversation tests (23), RPC component-authority tests (2),
service conversation unit tests (20), web components (217), web units (46),
root frontend typecheck/lint/build and document validation passed. Independent
review corrections and targeted authority re-review completed. Commit hooks,
complete exact-head CI and the extended existing model-free Docker/browser
journey remain required. Snapshot implementation dates and this temporary plan's
removal follow verified completion; no production or live-model acceptance is
claimed.
