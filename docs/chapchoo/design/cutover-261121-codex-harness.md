---
title: "Codex Harness Cutover Design"
created: 2026-11-21
updated: 2026-11-21
tags: [architecture, backend, migration]
document_role: primary
document_type: design
snapshot_id: cutover-261121
---

# Codex Harness Cutover Design

- [Requirements](../requirements/cutover-261121-codex-harness.md)
- [Decision record](../adr/cutover-261121-codex-harness.md)

## Design Authority

- Design revision: `1`

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| C1 | Codex uses one official App Server owner for start or exact resume. | `REQ-1`, `ADR-D1` | decided |
| C2 | Retain the same roots of authority for private admission, Connector credentials, capture and scoped responses. | `REQ-1`, `REQ-2`, [Connector credentials](../spec/domain/connector-credentials.md) | required |
| C3 | Share conversation operations and failures; keep adapter-specific wire messages and optional capabilities local. | `REQ-2`, `REQ-3` | required |
| C4 | Candidate-specific controls and a single durable save under the existing source and start fence. | `REQ-4`, [model-261120/ADR](../adr/model-261120-agent-model-preference.md) | derived |

## Ownership and delivery

The Connector owns one process and protocol driver in the conversation's
private scope. Setup binds the admitted credential and tool declaration before
the native conversation can accept product input. API-key startup uses an
explicit native login; the external-token path uses the existing private
file handoff. Neither writes credentials to capture. A saved file is not
proof of adoption.

New sessions bind the returned ID. Existing sessions use the retained ID and
capture snapshot; App Server notifications extend the same journal without
rewriting ACP records. Only matched live root events qualify current work.
Receipt correlation, prompt insertion, exact Stop and normal terminal evidence
are separate. A pending request handle is bound to the live owner, turn, and
captured occurrence; a retired handle cannot authorize another response.

The current publication path sends once through the actor after product
admission. Active steering carries an expected turn. Work continuation uses
the recorded ending association and existing one-use publication fence. An
ambiguous delivery never authorizes replay or a replacement conversation.

## Preferences and recovery

The applied connection's model list supplies the options for each model.
The edit keeps model, reasoning and speed on one page and resets the draft
tuning when switching candidates. Save validates the tuple against the
current applied scope and retains a receipt for the exact request. Legacy
model-only records have inherited tuning. A retained exact same-account,
same-thread restart can retain the draft but must validate it again before
dispatch. The save and publication fence commits a tuple without changing the
in-flight turn. Omitted speed inherits the native default; Standard is an
explicit baseline request. Fast and Ultrafast require advertisement by the
chosen model. Advertised capability and request serialization do not demonstrate
provider entitlement, billing or latency.

## Removal and Replacement

| Authority | Removed from Codex | Replacement and absence evidence |
| --- | --- | --- |
| C1 | ACP launch and ACP fallback for new and existing Codex conversations | After predecessors are physically retired and admission reopens, both an existing seeded ACP-origin conversation and a new conversation must launch only the direct owner. |
| C4 | Deriving another candidate's choices from the current model | A model switch reveals only its own capabilities, without saving or starting work during lookup. |

ACP support for other harnesses and interpretation of old retained events are
not removal targets. Database downgrade must not silently erase recorded
preferences or uncertain starts.

## Test Strategy

1. Rehearse the coordinated boundary with a retained ACP-origin conversation
   and a new conversation. Verify retirement, connector/credential authority,
   exact thread and workspace, retained context and records, approval/form,
   stop and continuation through the app. Verify the Activity sheet's one-save
   path and the absence of a legacy Codex owner after admission reopens.
2. Use released-binary loopback tests to attribute the real provider payload
   to a particular input, including explicit Standard over a nonstandard
   inherited speed. Synthetic advertisements test serialization, not billing
   or real account availability.
3. Use deterministic adapter and persistence tests for late/repeated replies,
   stale requests, no replay, per-candidate validation, atomic reservation,
   and historical receipt recovery on SQLite and PostgreSQL.

## Design Approval

The Requirements direction was confirmed. Revision-bound Design approval and
the final same-head rehearsal have not been recorded. This document does not
authorize merge or deployment.
