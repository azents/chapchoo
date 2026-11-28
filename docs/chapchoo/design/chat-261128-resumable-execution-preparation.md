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
- Status: incident correction under the accepted D1 direction. This does not complete the full snapshot or decide ADR-D2.

## Current gap and traceability

A connection can restore the exact retained native target and report Ready, yet the next accepted input can still require a distinct configuration preparation. The ordinary refresh path needs a complete owner-local work inventory. An arbitrary retained Resume lacks that proof, even when the effective optional supply is unchanged and no replacement or relay change is necessary.

| Requirement | Decision | Mechanism and coverage |
| --- | --- | --- |
| REQ-1 | ADR-D1 | M1 and M2 cover exact unchanged optional supply on the same route. Changed configuration without a proven work cut is not qualified. |
| REQ-2 | ADR-D1; existing delivery and retirement contracts | M2, M3 and M4 preserve the source, work fences and independent forwarding authority. |
| REQ-3 | Known error projection is fixed; ADR-D2 remains open | M4 reports known compound cleanup failure. Genuinely unresolved preparation is not assigned a new outcome. |

## Architecture and boundaries

The connection reconciler still prepares the native owner before opening the exact retained context. The native actor continues to serialize local setup, input and retirement. A deferred, input-scoped preparation may reuse its exact live App Server session when the retained and live identities, built-in tool catalogue, requested harness supply and instructions match. The actor must not be blocked or see an active foreground, outstanding requests, input reports, pending ending effects or unsettled tools. A nonempty applied supply must have its exact existing binding on the same route. This check never declares a complete native work inventory.

The Connector sends an exact operation, execution, route and native target to Control, with the applied selection identity and a canonical supply fingerprint when nonempty. Control discovers the prospective supply under existing bounded rules without retaining another callable selection. Incomplete or superseded discovery does not prove equivalence. For nonempty supply the fresh selection, original settings digests and catalogue, and actual native declaration set must match. The store rechecks all saved integration IDs and settings digests, including omitted integrations, while holding the same lock as settings mutations. It records Ready only for the still-selected preparation on its exact route. For a nonempty reuse it keeps the original callable predecessor selected, including across subsequent refresh attempts; the new preparation does not itself become callable. Each call still checks current saved settings. No native settings, grants or relay bindings change. The ordinary forwarding transaction still owns the input grant.

The old refresh path remains sole authority for changed configuration or a different relay binding. It requires complete owner-local work evidence and fallible physical retirement when replacement is needed. The published Resume/Idle observations in released Codex 0.171.0 do not order all active-goal and child activity before Resume returns. They are not permission to replace an arbitrary restored owner.

## Failures, rollout and recovery

Migration 36 records whether a Ready setup keeps its exact external predecessor. Previously written rows default to false. Rollout must use compatible Control readers before writing this evidence. Older Control code does not recognize active reuse evidence; drain those executions before downgrade. Both down migrations refuse to discard a current execution's active reuse marker. The additive RPC is not required for existing setup; an older Control leaves the owner on the previous full-fence path. A negative observation does not clear a prior ambiguous result or permit physical replacement. The exact Ready can be read back without reapplying settings or requiring the original owner still to match. Confirmed compound cleanup failures use the existing retirement-unknown preparation result; they grant no recovery, replay or successor on their own.

## Test Strategy and feasibility

1. In a disposable browser fixture, seed a publication, save an unreachable optional endpoint, physically stop the Computer, explicitly start its successor, and require an exact Resume and Ready for the same snapshot and native session before one Send. Require one additional publication in the same Chat. Baseline reproduced no delivery after Ready. Candidate passes with matching Control and Computer binaries and a deterministic App Server peer. This does not prove released-client behavior or a real MCP invocation.
2. SQLite and PostgreSQL exercise exact complete settings comparison, stale-route rejection, same-operation result recovery, original callable selection continuity, denial of new calls with edited settings, and independent forwarding. Service tests compare the original catalogue and saved settings with the actual applied supply. Existing harness regressions exercise active work, approval, child, input, relay and cleanup fences.
3. Run local workspace Rust, generated RPC interoperability and document checks. Review exact identity, effect ordering and unchanged delivery policy. Do not infer stock Codex or real upstream operation from separate component tests. Full REQ-3 coverage requires a decided D2 policy. This correction does not qualify arbitrary replacement.

## Design Authority

- Design revision: `2`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Keep connection-time native preparation and exact retained Resume | ADR-D1; REQ-1 | `decided` |
| M2 | Confirm exact unchanged supply against current saved settings; preserve any original callable predecessor without replacement, rebinding or a new forwarding grant | ADR-D1; REQ-1 and REQ-2; [Server Tools authority](../spec/domain/server-tools.md#authority-and-publication) | `derived` |
| M3 | Keep the existing full work and physical retirement fences for changed selections or a different relay binding | REQ-2; [Configuration refresh](../spec/domain/agent-instructions.md#common-refresh-and-continuing-input) | `existing` |
| M4 | Preserve known preparation failure and exact outcome recovery without inventing completion, non-admission or delivery replay | REQ-2 and REQ-3; [Retained native source and Setup](../spec/domain/native-harness.md#retained-captured-source) | `derived` |

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Requiring the replacement cut to qualify exact unchanged supply | ADR-D1; REQ-1 | Control confirms scoped reuse; the actor must still prove exact owner/configuration and settled observed effects | Deferred input-scoped Setup only; no new startup or input path | Browser regression with unchanged retained target and paired-store tests for the original callable predecessor |
| Compound cleanup falling through to generic preparation uncertainty | REQ-3 | Explicit existing retirement-unknown failure classification | Setup result; not delivery or physical retirement | Typed error mapping, existing cleanup ownership checks |

No persisted input, capture, configuration or forwarding contract is removed.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Incident implementation scope: previously accepted ADR-D1 and explicitly requested again on 2026-11-28 (KST) without another approval pause.
- Revision: `2`, `M1`–`M4`. This records the D1 implementation scope, not an approval of ADR-D2 or of the full snapshot. Do not mark the full snapshot implemented.
