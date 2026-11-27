---
title: "Resumable Chat Execution Preparation Design"
created: 2026-11-28
document_role: primary
document_type: design
snapshot_id: chat-261128
tags: [chat, execution, reliability]
---

# Resumable Chat Execution Preparation Design

- Snapshot: `chat-261128`
- Document reference: `chat-261128/DESIGN`
- Requirements: [Resumable Chat Execution Preparation Requirements](../requirements/chat-261128-resumable-execution-preparation.md)
- Decisions: [Resumable Chat Execution Preparation Decisions](../adr/chat-261128-resumable-execution-preparation.md)
- Status: candidate for the verified subset, awaiting revision-bound approval. This does not complete the full snapshot or decide ADR-D2.

## Current gap and traceability

A connection can restore the exact retained native target and report Ready, yet the next accepted input can still require a distinct configuration preparation. The ordinary refresh path needs a complete owner-local work inventory. An arbitrary retained Resume lacks that proof, even when the effective optional supply is empty and no replacement or relay change is necessary.

| Requirement | Decision | Mechanism and coverage |
| --- | --- | --- |
| REQ-1 | ADR-D1 | M1 and M2 cover equal empty optional supply. Equal nonempty rebinding and changed configuration without a proven work cut are not qualified. |
| REQ-2 | ADR-D1; existing delivery and retirement contracts | M2, M3 and M4 preserve the source, work fences and independent forwarding authority. |
| REQ-3 | Known error projection is fixed; ADR-D2 remains open | M4 reports known compound cleanup failure. Genuinely unresolved preparation is not assigned a new outcome. |

## Architecture and boundaries

The connection reconciler still prepares the native owner before opening the exact retained context. The native actor continues to serialize local setup, input and retirement. A deferred, input-scoped preparation may reuse its exact live App Server session when the retained and live identities, built-in tool catalogue, requested harness supply and instructions match. The actor must not be blocked or see an active foreground, outstanding requests, input reports, pending ending effects or unsettled tools. The applied optional supply must be empty. This check never declares a complete native work inventory.

The Connector sends an exact operation, execution, route and native target to Control. Control may discover the prospective optional supply using its existing bounded discovery rules, but does not retain a callable selection. Capacity exhaustion, elapsed shared discovery budget and superseded settings do not prove emptiness. If no tools qualify, the store rechecks the entire set of saved integration IDs and settings digests, including omitted integrations, while holding the same lock as settings mutations. The transaction records Ready for the still-selected operation on its exact route. No native settings, grants or external relay bindings change. The ordinary forwarding transaction still owns the input grant. The optional path does not alter permission UI, destination policy or call retries.

The old refresh path remains sole authority for changed configuration or nonempty relay application. It requires complete owner-local work evidence and fallible physical retirement when replacement is needed. The published Resume/Idle observations in released Codex 0.171.0 do not order all active-goal and child activity before Resume returns. They are not permission to replace an arbitrary restored owner. Nonempty comparison alone would not protect predecessor calls before Control accepts the successor.

## Failures, rollout and recovery

There is no schema change. The additive Control RPC is not required for existing setup; an older Control that does not implement it leaves the owner on the previous full-fence path. No model-side changes, new user settings or migrations are required. An explicitly negative result, including ingress admission refusal before any operation, is not evidence of a committed setup. A lost result can hide a committed exact Ready. Further negative observations do not clear that earlier uncertainty or permit physical replacement. The exact Ready can be read back without reapplying settings or requiring the original owner still to match. Confirmed compound cleanup failures use the existing retirement-unknown preparation result; they grant no recovery, replay or successor on their own.

## Test Strategy and feasibility

1. In a disposable browser fixture, seed a publication, save an unreachable optional endpoint, physically stop the Computer, explicitly start its successor, and require an exact Resume and Ready for the same snapshot and native session before one Send. Require one additional publication in the same Chat. Baseline reproduced no delivery after Ready. Candidate passes with matching Control and Computer binaries and a deterministic App Server peer. This does not prove released-client behavior or a real MCP invocation.
2. SQLite and PostgreSQL verify exact complete settings comparison, stale-route rejection, same-operation result recovery, absence of an external call grant, and independent forwarding. Existing harness regressions exercise active work, approval, child, input, relay and cleanup fences. These establish feasibility for the candidate subset, not arbitrary replacement or nonempty reuse with unknown native work.
3. Run local workspace Rust, generated RPC interoperability and document checks. Review exact identity, effect ordering and unchanged delivery policy. Full D1 coverage requires a verified nonempty authority handoff without revoking a predecessor. Full REQ-3 coverage requires an accepted D2 policy and observably distinct unresolved cases. Neither is established by a longer wait or a synthetic idle event.

## Design Authority

- Design revision: `1`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Keep connection-time native preparation and exact retained Resume | ADR-D1; REQ-1 | `decided` |
| M2 | Confirm exact empty-supply reuse against current saved settings without a callable relay, physical replacement or forwarding grant | ADR-D1; REQ-1 and REQ-2; [Server Tools authority](../spec/domain/server-tools.md#authority-and-publication) | `derived` |
| M3 | Keep the existing full work and physical retirement fences for changed or nonempty selections | REQ-2; [Configuration refresh](../spec/domain/agent-instructions.md#common-refresh-and-continuing-input) | `existing` |
| M4 | Preserve known preparation failure and exact outcome recovery without inventing completion, non-admission or delivery replay | REQ-2 and REQ-3; [Retained native source and Setup](../spec/domain/native-harness.md#retained-captured-source) | `derived` |

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Requiring the replacement cut to qualify exact unchanged empty supply | ADR-D1; REQ-1 | Control confirms scoped reuse; the actor must still prove exact owner/configuration and settled observed effects | Deferred input-scoped Setup only; no new startup or input path | Baseline and candidate browser regression with unchanged retained target; store checks no new grant |
| Compound cleanup falling through to generic preparation uncertainty | REQ-3 | Explicit existing retirement-unknown failure classification | Setup result; not delivery or physical retirement | Typed error mapping, existing cleanup ownership checks |

No persisted input, capture, configuration or forwarding contract is removed.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Design revision and authority set proposed: `1`, `M1`–`M4`
- Status: not approved. ADR-D1 approval is recorded separately; it does not constitute approval of this Design revision. ADR-D2 remains open, and the snapshot must not be marked implemented.
