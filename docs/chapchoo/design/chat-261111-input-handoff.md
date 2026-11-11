---
title: "Chat Input Handoff and Computer Status Design"
created: 2026-11-11
implemented: 2026-11-11
document_role: primary
document_type: design
snapshot_id: chat-261111
tags: [chat, lifecycle, web, computer]
---

# Chat Input Handoff and Computer Status Design

- Requirements: [chat-261111/REQ](../requirements/chat-261111-input-handoff.md)
- Decisions: [chat-261111/ADR](../adr/chat-261111-input-handoff.md)

## Design Authority

- Design revision: `1`

| ID  | Mechanism                                                                                                                                                             | Authority                                                                                | Classification |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | -------------- |
| M1  | Pending input uses the ordinary user bubble at opacity 0.6 with no lifecycle caption                                                                                  | REQ-1 and confirmed component preview                                                    | required       |
| M2  | Successful native Inserted/Steered acceptance ends the pending input and its dependent delivery/preparation records; no completed-input replacement ledger            | REQ-2/REQ-3, ADR-D1                                                                      | decided        |
| M3  | Same-source authenticated repeated completion closes an already absent pending operation without asserting native delivery or granting another dispatch               | REQ-3, existing exclusive native operation ownership and durable-report release contract | derived        |
| M4  | Exact bounded known-input existence reads remove closed entries from local/server view caches; paging absence alone is not deletion                                   | REQ-2/REQ-3, current source-bound reader and bounded continuations                       | derived        |
| M5  | Known Computer lifecycle takes precedence; retained native foreground plus a spinner represents temporary observation gaps without restoring current effect authority | REQ-4/REQ-5, ADR-D3 and existing owner/source admission                                  | required       |

## Current Behavior and Gaps

The current input table serves both pending-body ownership and permanent delivery receipts. Native acceptance updates a dependent delivery row, leaving the original forever. The renderer appends these receipts and uses message-ID matching to hide some of them. Same-source reader caches only merge updates, so deleting a database row alone leaves ghost messages.

Chat currently places unknown power/disconnected Connector before retained native foreground. Native foreground already survives ordinary same-execution reconnect, but this presentation masks it. Current power, native observation and effect admission remain distinct.

## Ownership and Transitions

1. Browser upload and server acceptance retain current private owner/Team/target validation. Existing process-owned tasks survive waiter loss.
2. Pending input retains the body while preparation and forwarding are active. Possible/failed outcomes are not successful pickup.
3. The native owner returns Inserted or Steered before its ordinary captured output/completion path. The authenticated completion transaction validates current source/execution/target and removes the pending input and its dependent rows atomically.
4. Later captured session records appear through the existing native reader. No input/transcript identity bridge, synthesized user record or duplicate-suppression heuristic is introduced.
5. Repeated successful completion after an ACK loss closes the absent pending operation under current source validation. This is closure, not a reconstructed success receipt. The existing Connector invocation permit remains held until the native known-result registry is released on its command-checking loop.
6. Missing pending input denies subsequent body pulls/forwarding. No automatic owner submission is made from an absent result. After removal, an arbitrary explicit stale OwnerSubmit request cannot be remembered indefinitely; permanent completed-request recovery is intentionally removed rather than replaced.

## Read and Presentation Contract

Existing bounded known-input hints must be checked for actual existence under the same source. A returned closure removes the exact acknowledged cached item, including locally accepted input consumed before its first list read. A 24-item page omitting an older row is not a closure. Stale in-flight reads must not resurrect a closed input; source replacement resets the relevant view authority.

Pending and native bubbles use the same typography and geometry. Pending alone is dimmed. Remove captions and the old native-ID duplicate hiding. Existing failures and their explicit recovery actions remain separate from successful consumption.

Known lifecycle operations and stopped power override retained native state. On temporary read/connection uncertainty, a retained same-source native state remains the label and only a spinner is added. Before any known native state exists, display no invented state text. Confirmed setup failures remain failures. Send, answer and Stop admission continue to require current authority; retaining a label never reenables them.

## Data Evolution and Safety

Use forward, semantically paired SQLite/PostgreSQL evolution for old successful receipts. Remove only proven successful handed-off input rows whose native source binding independently preserves session ownership. Preserve unresolved/failed preparation facts until their lifetime actually ends; do not erase retained-source or fresh-context fences by blindly deleting historical rows.

`inputs.n == 0` describes only the absence of currently pending input, not a
permanent historical claim. A later explicit not-started submission can join the
exact independently retained startup/source again. It neither replaces that
source nor authorizes fresh context: preparation checks the current native
binding and combined unresolved/rejected startup barriers. The both-engine
startup regression consumes the first input, admits another explicit input and
proves `fresh_native_allowed=false` with the identical retained native target.
No replacement request counter is needed.

No schema/protocol version feature, indefinite completed-input idempotency ledger or legacy behavior fallback is added. Generated contract surfaces, if changed for bounded existence reads, are regenerated from protocol sources.

## Removal and Replacement

| Existing behavior                                      | Removal authority | Remaining behavior                                | Boundary                 | Verification                                                   |
| ------------------------------------------------------ | ----------------- | ------------------------------------------------- | ------------------------ | -------------------------------------------------------------- |
| Permanent Inserted/Steered input receipts and bodies   | REQ-2, ADR-D1     | Actual native session capture/history             | Successful native pickup | Store deletion and body/grant absence tests                    |
| Delivery captions below user bubbles                   | REQ-1             | Dimmed pending versus ordinary native bubble      | Renderer                 | Component assertions and native-resolution images              |
| Captured-user-ID receipt hiding                        | REQ-3             | Ordinary pending/source lifetime                  | Handoff activation       | No matching helper/heuristic and assembled conversation        |
| Catch-all unknown/checking caption masking known state | REQ-4/REQ-5       | Known lifecycle or last native state with spinner | Chat presentation        | Pure status tests and rendered states                          |
| Add-only stale pending-input caches                    | REQ-2/REQ-3       | Source-qualified bounded current existence        | Reader merge             | Closed-before-first-read, paging and late-response regressions |

## Test Strategy

- Persistence/service tests on SQLite and PostgreSQL for acceptance, successful pickup deletion, source fencing, failed/uncertain retention, duplicate completion ACK loss, denied post-removal pulls and independent setup/context barriers.
- Reader/merge tests for exact known-item closure, local acceptance before first poll, pagination, source replacement and stale-read resurrection.
- Pure status tests for retained idle/working/action state during observation gaps, lifecycle priority, explicit failure and no-state cold load. Component stories/assertions cover dimming, absent captions and last-state spinner at mobile/desktop.
- Existing representative actual Docker/backend/official-adapter conversation browser scenario proves prestart, first-input same-source reuse, output, typed answer, reentry and owned cleanup. Add only the handoff/native-history assertion at this assembled boundary; exhaustive matrices stay below E2E.
- Normal format/lint/typecheck/build, generated RPC interoperability, independent read-only code review and connected Spec review. Exact-head CI must pass; no hook bypass, fixed sleeps, weakened gates, extra infrastructure or live-model credentials.

## Feasibility Evidence

- **M1 — feasible:** real Chat/ConversationBody stories and component checks verify
  the same native/pending bubble geometry and opacity at mobile/desktop sizes.
- **M2 — feasible:** SQLite/PostgreSQL conversation tests cover complete removal,
  failed/uncertain retention, source/execution refusal and post-removal body denial.
  The paired migration fixture seeds legacy successful and unresolved rows.
  Current native binding and independent startup facts preserve exact restoration
  even with no remaining pending inputs.
- **M3 — feasible:** repeated successful authenticated reports acknowledge absent
  operations; a Connector-owned test preserves the invocation lease through
  transport loss, report retry and command-loop-ordered registry closure.
- **M4 — feasible:** exact-presence tests cover cached/local closure independently
  of paging. Before/after-body identity checks reject a new operation submitted
  under the same consumed Request ID; source-qualified serial read owners avoid
  late competing continuations without a persistent closure ledger.
- **M5 — feasible:** pure status and real component checks retain native foreground
  during stale/error/disconnected observations, prioritize lifecycle states and
  keep effect controls read-only.
- The representative model-free Docker/backend/official-adapter browser journey
  verifies prestart, exact native-context reuse, pending-input absence, captured
  output, typed answer, reconnect, reentry and owned teardown. This is assembled
  plumbing evidence, not proof of real credentials or model behavior.

## Design Approval

- Mode: Collaborative.
- Decision owner: requester.
- Implementation-direction approval: 2026-11-11 at 17:19 KST, after real component
  preview review; complete removal without a replacement counter was confirmed at
  17:27 KST.
- Recorded Design revision: `1`; material authority IDs: `M1`, `M2`, `M3`, `M4`, `M5`.
- Scope: M1/M2/M5 implement the confirmed outcomes; M3/M4 preserve existing
  ownership, report-release and source-qualified read contracts under removal.
  None introduces an optional recovery mode or completed-input authority.
- The requester approved implementation, not a separate revision-bound technical
  document review. No such additional approval is claimed. This record preserves
  that process limitation rather than reconstructing approval from feasibility.
