---
title: "Codex Conversation Phase 1: Domain and Preserving Storage"
created: 2026-11-09
tags: [agent, conversation, implementation, persistence]
---

# Phase Execution Plan

- Phase: 1/6, domain and preserving conversation storage.
- Branch/base: `feature/chat-text-1-foundation` → `main` at `82c96dadc43963b6ae15b3baf5d560d9897b9435`.
- PR boundary: completed private Store operations and tests for saved supply, initial conversation binding, admitted input, paired source ingestion and clear/refill; no public conversation RPC registration or native launch.
- Inputs: [Requirements r24](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md), [approved Design r9](../design/chat-261108-text-conversation.md), [implementation decomposition](chat-261108-implementation.md).
- Approved Design mechanisms: M01, M05, M07–12, M18, M20, M22–24, M26 (storage/domain portions only).
- Authority references: REQ-1–3/10–12/14–16/18, ADR-D10/11/16–22/29–31/33, existing Agent ownership and account transaction/identity Specs.
- Design delta: None.
- Deliverables: validated supply/source/input values; compact repository conversation IDs; paired preserving migrations; request-owned private admission; source-bound operation acceptance/recovery; initial versus replacement binding; atomic pair/cursor ingestion and coherent bounded metadata reads.
- Non-goals: model execution, artifact acquisition, external token refresh, public wire APIs, rendered chat, new idle policy, command replay, native gap repair or general versioning.
- Interfaces: Store DTOs distinguish not-started/loading/complete source, operation request identity and occurrence order. Captured raw/normalized bytes are opaque to persistence; typed normalization/record projection belongs to phase 3. Content is never formatted in Debug output. New conversation tables are scoped to an Agent, while a separate current-conversation mapping avoids a permanent one-conversation uniqueness constraint.
- Removal obligations: none of the production Chat UI activates here; retain it until phase 5's real replacement. Explicit source replacement removes all affected prior operation/pair data as part of this phase's tested Store contract, preserving account/Agent/Computer/native identities.
- Absence verification: tests prove no old-source pairs or independently admitted operations remain after replacement; first binding preserves its starting input. No actor may use another owner's private Agent.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Approved baseline and plans | `/root` | `docs/chapchoo/{requirements,adr,design,notes,plans}/chat-261108-*` | approved documents | tracked approved baseline + exact execution scope | doc validator, ID/approval assertions, links |
| Domain values | `/root` | `rust/crates/domain/src/conversation*`, domain exports | Design | validated supply, source/cursor, input values and state | focused domain tests, Rust gate |
| Storage | `/root` | new persistence conversation modules, paired migration 12, entity/preparation exports | domain values + current admission macros | atomic private operations and readiness/downgrade guards | SQLite/PostgreSQL integration and preserving migration tests |
| Current implementation description | `/root` | owning Spec pages only where new Store behavior needs publication | stable implementation | precise storage boundary, explicit absent public/native behavior | semantic lint + doc checks |

- Integration order: domain values → schema/readiness → typed Store API/admission → operations and paired ingestion → both-engine tests → focused/full Rust checks → stable independent review → corrections/affected checks → phase PR.
- Independent review: `/root/chat-implementation-reviewer`; root supplies the complete stable diff and actual evidence. Reviewer is read-only; root evaluates and fixes every valid finding.
- Final validation: root runs `cargo fmt`, focused tests with disposable PostgreSQL through the existing test runner, `bash scripts/check-rust.sh`, documentation checks and actual remote PR checks. No prerequisite absence is a pass. Scope changes invalidate affected evidence.
- Scope-drift check: no source/operation authority is derived from event ID ordering, no live authority from history alone, no new hidden command archive, no credentials in transcript, no public success path for unimplemented native work.
- Context checkpoint: implementation complete for the phase boundary. Domain values, migration 12, private saved-supply/input/source/pair/chunk Store operations and both-engine contracts are integrated. `bash scripts/check-rust.sh` passes, including the complete workspace tests, Clippy, rustdoc and dependency boundaries. `bash scripts/check-rpc.sh` passes generated-client verification and 129 live checks on each DB engine. Existing migration/profile downgrade tests were updated to traverse the new empty additive migration before checking their original populated guard. No native/model/live Docker claim is made. Exact reviewer `/root/chat-implementation-reviewer` completed read-only review of the complete stable phase diff with no blocking findings; PR and remote CI follow. Remaining feature work is phases 2–6. Design delta remains None.
