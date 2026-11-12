---
title: "Server Tools Phase 1: Independent Chat Receipts"
created: 2026-11-12
tags: [tools, chat, implementation]
---

# Phase Execution Plan

- Phase: 1/3, independent chat receipts and bounded owner reads.
- Branch/base: `feat/server-tools-chat-storage` → `main`.
- PR boundary: atomic user input/chat acceptance, prepared Agent publication repository contract, preserving migration and protected message-only reads. UI activation and MCP relay remain later phases.
- Inputs: [approved Design revision 2](../design/tools-261112-server-provided-tools.md); [Requirements](../requirements/tools-261112-server-provided-tools.md); [ADR](../adr/tools-261112-server-provided-tools.md); [implementation plan](server-tools-261112-implementation.md).
- Approved Design mechanisms: M8–M11, M13–M14, M17–M18 and applicable M20 validation. M8's actual MCP entrypoint remains phase 2.
- Authority references: REQ-2, REQ-8–REQ-12; ADR-D9, D10, D12, D13; current conversation-storage/private-owner contracts; completed-persistence-operations and durable-evolution.
- Design delta: None.
- Deliverables: accepted original user body survives input consumption/deletion/source replacement; stable shared USER/AGENT order; retained non-deleted original input transfer; exact private metadata pages and body chunks without native payload reads.
- Non-goals: MCP listener/catalog/queue; native setup changes; UI activation; session-to-chat reconstruction; public retry key; models, operating-data changes, merge or deployment.

## Fixed interfaces

Repository message identity is a typed repository-issued opaque ID. Each immutable message belongs to the product conversation, has role/body/receipt time/positive int64 receipt order and optional input provenance without ephemeral ownership. Conversation allocation and append commit together. Existing input recovery returns its existing receipt without appending another message or dispatching another input. Agent append will require phase 2's current sealed component/source authority rather than model-selected targeting.

Owner read contracts extend the existing private Conversations service with bounded message metadata pages and exact message-body chunks. Reads revalidate Agent/conversation scope, include no native source lifetime, and preserve exact integer values through generated Node boundaries. Existing native record services are unchanged. Proto is authoritative and clients are regenerated.

Input provenance includes the existing input ID and request nonce for exact UI
acknowledgement correlation when the response is lost and native pickup has
already removed the temporary input. This is read-only provenance: admission
still checks only the retained input operation, never chat as a permanent retry
or deduplication index. Agent publication has no caller resend-key field.
Component publication authorization does not require native readiness or a
native session/snapshot ID; those remain capture/restoration provenance under D4.

Migration 23 on both engines establishes conversation-owned messages and allocation heads. It transfers only retained original non-deleted inputs into a deterministic prefix, preserving acceptance/provenance data without modifying delivery facts or replaying native input. Existing preparation/schema/root-binding guards recognize the schema; downgrade refuses to discard accepted message data.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Domain and completed storage | Root | rust/crates/domain conversation/identity contracts; rust/crates/persistence/src/conversations*, schema/preparation and both migration-23 directories | Approved data contract | Typed chat receipts/read operations and atomic input admission | Both-engine isolation/order/recovery/lifetime tests |
| Owner service/RPC | Root | rust/crates/service conversation entrypoints; rust/crates/rpc/src/conversations*; proto/chapchoo/conversations.proto | Store contract | Private metadata/body adapters | Auth, chunk bounds, source-independence, actual RPC tests |
| Generated clients | Root | typescript/packages/rpc-client generated output; Rust generated build contract | Proto | Exact typed client contract | Existing generator verification and client build |
| Integration/evidence/Spec | Root | persistence/service/RPC tests, existing testenv fixture only as needed, owning storage/RPC Spec pages and phase checkpoint | Integrated phase diff | Verified phase behavior and removal evidence | Canonical checks and single independent review |

- Integration order: read existing stores/transactions and schema guard → domain/schema/migration → atomic acceptance/Agent append → private reads → RPC/generation → focused both-engine/RPC checks → stable-diff review → commit/open PR.
- Removal obligations: replace the completed input-acceptance path that can persist without chat; upgrade schema/preparation membership checks; retain input cleanup itself. No main-chat UI filtering removal occurs before its phase-3 replacement.
- Absence verification: failure injection cannot leave an accepted input without its chat row; consumption/source-replacement/deleted-input fixtures preserve existing chat; downgrade with accepted messages fails; generated output has no hand-written duplicate API.
- Independent review: `/root/server-tools-reviewer`, read-only on the complete stable integrated phase diff. Root requests review/re-review, applies findings and owns reruns.
- Final validation: document validator/tests; relevant domain/persistence/service/RPC tests against disposable SQLite and PostgreSQL; `bash scripts/check-rust.sh`; generated RPC verification/live checks where phase interfaces require them; diff/private-information checks. Missing prerequisites are reported honestly and fixed locally when supported, not converted to skips.
- Scope-drift check: every change traces to the phase mechanism list; no native replay, generic result archive, resend API, new retention/delete UI or extra production tool; no premature UI-source switch.
- Context checkpoint: atomic user receipt and prepared Agent append, migration 23
  on both engines, protected message reads, generated client and owning Spec
  updates are implemented. Latest focused validation passed all 12 conversation
  persistence tests against SQLite/PostgreSQL and both component-auth RPC tests,
  including publication without native readiness, consumed nonce reuse, exact
  Unicode bytes and rollback on either side of input/chat acceptance. The final
  corrected-source Rust workspace gate passed (format, Clippy, tooling, docs,
  all workspace tests and dependency boundaries). Live Node/Rust RPC validation
  passed 129 checks on each database engine; client generation/build and
  document validation passed. The single independent integrated-diff review
  found no actionable material findings and verified the receipt-clock
  correction. One overlapped test invocation produced preparation-lock `Busy`;
  the final full run was isolated and passed. Commit/PR delivery is next.
  No merge, deployment, MCP entrypoint or UI activation is claimed.
