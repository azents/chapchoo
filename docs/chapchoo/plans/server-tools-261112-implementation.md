---
title: "Server Tools Implementation Plan"
created: 2026-11-12
tags: [tools, chat, implementation]
---

# Server Tools Implementation Plan

- Requirements: [tools-261112/REQ](../requirements/tools-261112-server-provided-tools.md).
- Decisions: [tools-261112/ADR](../adr/tools-261112-server-provided-tools.md), D1–D14 including the appended rendered selection.
- Approved Design: [tools-261112/DESIGN](../design/tools-261112-server-provided-tools.md), revision 2, exact M1–M20 set.
- Authority: requester instruction to implement and open PRs after the selected UI refinements.
- Design delta: None.
- Implementation/integration/validation owner: root agent.
- Single independent reviewer: `/root/server-tools-reviewer`, read-only one-way opinions on every complete integrated phase diff; root evaluates and applies all corrections.
- Repository baseline: `83a41877867f3602f741da0a9de819fa52f76929`, freshly verified as origin/main at implementation start.

## Reviewable delivery stack

1. **Server tools [1/3]: preserve independent chat receipts.** Base `main`, branch `feat/server-tools-chat-storage`. Completed atomic input/chat receipt, independent conversation-owned message storage and ordering, preserving transfer, protected bounded message reads and generated contracts. Scope M8–M11, M13–M14, M17–M18 and their M20 tests. No native output deletion or UI source activation yet. The service API for Agent publication is prepared for phase 2.
2. **Server tools [2/3]: provide bounded Connector MCP tools.** Base phase 1, branch `feat/server-tools-relay`. Public SDK local HTTP relay, native-owner capability, current component RPC authorization, bounded fair invocation lifetime/cancellation/shutdown and real publish_text domain receipt. Scope M1–M8, M15–M19 with actual capture compatibility and M20 evidence. Test-only second behaviors stay fixtures; no additional production tools.
3. **Server tools [3/3]: activate message-only chat and selected records.** Base phase 2, branch `feat/server-tools-chat-ui`. SSR message-only primary chat, independent full-record reads, exact selected uniform-row UI, normalized publication preview, product/component acceptance, Spec promotion and plan cleanup after final validation. Scope M7, M12–M14, M18–M20, with full cross-phase coverage.

Each phase gets a tracked phase plan before implementation and an opened PR before work starts on its successor. All planned PRs are opened before waiting for the stack's CI completion. No merge or deployment is authorized. The feature is not declared implemented or given an implemented date until the complete approved behavior and integrated checks are satisfied.

## Fixed boundaries and interfaces

- Existing input operations retain their delivery/uncertainty/recovery contracts; chat receipt is independent of consumption and source replacement.
- Conversation-owned immutable USER/AGENT messages use repository IDs and database-local serialized positive order. Input provenance does not own chat through a foreign key.
- Owner message metadata/body reads use current private scope, exact integer anchors, finite pages and bounded chunks; native record readers remain separate.
- The relay uses SDK public APIs, loopback capability headers and current component authority. The first production catalog contains publish_text only.
- Stream/cancel state and resource parameters follow the approved Design, not short Control helpers or an automatic retry/replay mechanism.
- Full-history publication stays one captured tool item; actual normalized tool identity determines presentation, never text resemblance or generic other kind.

## Validation and prerequisites

Root owns focused checks, both-engine preserving migration/domain contracts, RPC generation/live checks, canonical Rust checks, TS lint/type/build and focused real-component checks. The minimal browser acceptance covers accepted user input, intentional Agent publication and retained native output/history without automatic publication. Cross-product/race/error cases use lower layers, not an expanded E2E suite.

Use disposable SQLite/PostgreSQL and the existing isolated testenv fixtures; never operating data. Local rendering is not authenticated product or actual SDK/native evidence. Actual native compatibility must satisfy the approved gate without silently enabling protocol features or fresh-context fallback. Real model calls or credential-dependent external actions remain separately authorized; report unavailable live prerequisites honestly rather than claiming success.

## Removal and preservation

The phase activating a replacement removes its old implementation/expectations: phase 1 replaces acceptance-without-chat and schema guards; phase 3 replaces native-driven main-chat filtering, sender/robot/action-title presentation and obsolete expectations. Native history/capture, original raw tool fields, steering/Stop/responders and input delivery are preserved. Each phase records absence evidence. Generated surfaces are regenerated from authoritative proto, not patched by hand.

Current Specs change only with verified phase behavior. Final Spec promotion updates semantic owners and connections, not a second path inventory. Remove temporary plans after integrated validation and promotion, not as a standalone workflow-only PR.

## Context checkpoint

Planning baseline approved; review-only fixture files removed and original product
component restored. Six design/research documents accompany the first phase.
Phase 1 storage, atomic acceptance, preserving migration and protected reads are
implemented with focused both-engine tests, the final Rust workspace gate,
129 live RPC checks per engine and client/document validation passing. The single
independent review found no actionable material findings. Commit/PR delivery is
next; phases 2 and 3 have not begun. No merge or deployment has completed.
