---
title: "Connection-Time Session Start Design"
created: 2026-11-10
document_role: primary
document_type: design
snapshot_id: session-261110
tags: [conversation, connector, harness]
---

# Connection-Time Session Start Design

- Snapshot: `session-261110`
- Document reference: `session-261110/DESIGN`
- Requirements: [Connection-Time Session Start Requirements](../requirements/session-261110-connection-start.md)
- ADR: [Connection-Time Session Start](../adr/session-261110-connection-start.md)

## Current behavior and gaps

A selected conversation is created only when a nonempty input is accepted. Setup admission, fresh-context accounting, actual-start authentication, and initial source binding all join that input. Connector authentication does not require desired `RUNNING`, and constructing the idle native owner does not launch it. Opening the owner conversation does not start work.

The gap is that [session-261110/REQ-1](../requirements/session-261110-connection-start.md) requires preparation before the first message when the Computer is desired running and its current Connector is authenticated. [session-261110/REQ-2](../requirements/session-261110-connection-start.md) requires a real empty conversation for that first session. Existing input coupling cannot do either without a synthetic prompt.

## Traceability

| Requirement | Decision | Mechanism |
| --- | --- | --- |
| REQ-1 | ADR-D2 | M1, M2 |
| REQ-2 | ADR-D1 | M3, M6 |
| REQ-3 | ADR-D1 | M4, M5, M10 |
| REQ-4 | ADR-D1 | M3, M7 |
| REQ-5 | ADR-D1 | M3, M8 |
| REQ-6 | ADR-D2 | M2, M9 |

## Architecture and ownership

The Client group owns startup admission and Setup dispatch. Control continues to own the Connector stream and does not await setup before returning it. The Connector still owns native execution and reports setup separately from queue admission.

The source of truth for a connection startup is `conversation_startups`. It uses `ConversationOperation` identities that are not input IDs. Input preparation remains the source of truth for message-triggered setup. Fresh and retained fences count both histories.

No owner RPC starts preparation. Page reads remain reads.

## Data and state

Migration 21 adds `conversation_startups` in both databases:

- `operation_id` primary key, 16-byte identity in SQLite and UUID in Postgres
- `conversation_id` foreign key
- `execution_id`, sealed supply, selection request, `fresh_native_allowed`, `require_retained_source`
- `preparation_state` `POSSIBLE`, `READY`, or `FAILED`, with the same ready and failure checks as input preparation
- one `POSSIBLE` row per conversation

Downgrade refuses while any startup row exists. Existing input rows and preparation rows are not rewritten.

The fresh barrier is the existing predicate over the union of input preparation and startup rows:

```text
NOT (FAILED AND (
  failure_kind IN (NOT_ADMITTED, BEFORE_NATIVE_LAUNCH, NATIVE_REJECTED)
  OR (failure_kind=SOURCE_UNAVAILABLE AND fresh_native_allowed=FALSE)))
```

Any historical `NATIVE_REJECTED` still seals `require_retained_source`.

## Runtime

A Client reconciler lists Computers whose desired state is `RUNNING` and whose Connector route resolves, in bounded pages. It coalesces one startup task per Computer. The task revalidates desired `RUNNING`, current execution, owner membership, and the current route inside the admission transaction. Team rows are locked before the Computer so removal cannot deadlock. A Computer can have more than one eligible Agent; each Agent is admitted separately. More than one Agent is not an integrity failure.

When an Agent has no selected conversation, that transaction creates a `NOT_STARTED` conversation and its selection pointer. It does not insert an input. A `POSSIBLE` startup whose execution is no longer current is completed as `RETIREMENT_UNKNOWN` before another grant, so fresh permission stays sealed and the single `POSSIBLE` slot can admit a retained resume. Uncertain dispatch still does not grant another setup for that same operation.

Saved authentication is required before a startup grant. Absence leaves the empty conversation unlaunched. A later tick may grant startup after a selection appears. The grant copies the saved supply and seals fresh and retained permission for the current execution.

Dispatch uses reconciliation authority. The authority owner must be the current Connector route. The call is not awaited inside stream admission. Queue admission is not native completion. The task does not send an input command, prompt, or historical replay.

A `POSSIBLE` grant for the current execution is recovered, not duplicated. `NOT_ADMITTED` and `BEFORE_NATIVE_LAUNCH` may be retried by a later operation while the start conditions hold. A retained-source failure, native rejection, or retirement-unknown result is not turned into a fresh context. If neither fresh permission nor a retained source is available, no Setup is dispatched and the settled failure stays visible.

The first NotStarted input may join a startup-owned `LOADING` or matching `READY` source when no prior input exists. Its stored target remains NotStarted. Forwarding uses the startup's ready target, which existing delivery already allows for a NotStarted input. A snapshot target must still match. A Ready source created by an input setup does not accept a later NotStarted target.

If startup is `POSSIBLE` on the current execution, or already `READY` for the current source and current Connector connection, input processing observes that operation and does not grant a second setup. A READY outcome retains the reporting connection identity. A different connection may represent a replacement native owner even within the same execution, so it receives a new retained-only Setup for the exact source. An already-live native owner returns the existing session; a replacement owner resumes it. Historical READY input preparation validates the source but does not suppress that connection's retained startup. After a retryable startup failure, input setup uses the normal path and the combined barrier.

## Interfaces

`OwnerConversation` gains optional startup preparation with the existing input preparation shape: possible forwarding, session ready, or setup failure. It does not include credential material. Native foreground still wins when a live observation exists. Before that, possible startup is starting, authentication absence on a not-started conversation remains authentication required, and a known startup failure uses the existing failure reasons.

Setup, authentication application, and source binding accept a startup operation when no input row owns that operation ID. An operation present in both tables is an integrity conflict.

## Security

Startup admission is not a browser or Connector RPC. It rechecks Computer desired state, current execution, route, Agent owner, and Team membership in the completed transaction. Applied authentication remains immutable for the operation that acquired it. Reconciliation dispatch proves the current Connector owner and cannot mint a user session.

## Failure, retry, and lifetime

Missing route, stopped or deleting Computer, or revoked Connector does not create a conversation or grant. A grant whose Computer is no longer desired running is withheld. This does not turn any earlier uncertain dispatch into known non-admission; native dequeue rechecks desired running as part of the sealed startup check. Connector end and Computer stop continue to finish the native owner through the existing process lifetime. Idle time does not stop it.

An uncertain result for the current execution does not become a known failure and does not grant another setup for that operation. Once that execution is no longer current, its unresolved startup cannot remain `POSSIBLE`: the Connector that could have completed it is gone. Recording `RETIREMENT_UNKNOWN` preserves the fresh barrier without redispatching that operation.

## Migration and rollback

Upgrade is additive. Rollback is refused while startup rows exist; deleting those rows is not a product downgrade path. Old servers ignore the new owner field. New servers tolerate its absence only before this snapshot is deployed together.

## Observability

Startup failure kinds remain the existing body-safe preparation failures. Logs do not include credential material, prompts, or session secrets. Owner reads expose the startup state needed to distinguish authentication required, starting, and failed restore.

## Test Strategy

Primary proof is the Docker conversation E2E. With saved authentication and a connected running Computer, the selected conversation exists and its startup reaches session ready before the first Send. The fixture must show thread setup without a model turn. The first Send then uses that session and is the first model-facing input.

A second E2E or deterministic adapter case covers missing saved authentication: the Computer connects, no native process starts, and the owner state is authentication required.

Persistence tests cover empty conversation creation, refused startup without authentication or while stopped, combined fresh and retained fences, one possible grant, and first NotStarted input joining the startup source. RPC tests cover reconciliation dispatch not being awaited inside Connector `open`, and setup check/report for a startup operation. Component tests cover the owner projection of starting, authentication required, and failed startup without an input.

Live model calls and new Computer provisioning are not required. Optional live tests stay skipped unless their existing credentials and services are already present. A missing Docker fixture fails the primary E2E rather than passing.

## Assumptions and non-blocking risks

- Computer stop reaching Connector SIGTERM remains the existing substrate behavior. This design does not add a second native cancellation path.
- Retry cadence for `BEFORE_NATIVE_LAUNCH` is a local reconciler interval, not a new product mode.
- Native adoption of a published credential file remains native behavior, as in the current Spec.

## Design Authority

- Design revision: `3`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Prepare only when desired running and the current Connector route resolves; page reads do not start it | REQ-1; connector connectivity is not harness readiness | `required` |
| M2 | Dispatch after stream installation with reconciliation authority and current Connector owner proof | REQ-1; ADR-D2; existing reconciliation authority | `derived` |
| M3 | For each eligible Agent, create the selected empty conversation and seal startup without an input or prompt | REQ-2; REQ-4; ADR-D1; current one-selected-conversation constraint | `required` |
| M4 | Count startup history in the existing fresh barrier and retained-source fence | REQ-3; ADR-D1; current preparation Spec | `derived` |
| M5 | Resume only the retained source; do not replace a failed restore | REQ-3; current native setup Spec | `existing` |
| M6 | First NotStarted input joins the startup-owned source and does not grant a second setup | REQ-2; existing NotStarted forwarding rule | `derived` |
| M7 | Startup dispatch sends Setup only | REQ-4 | `required` |
| M8 | Missing saved authentication does not launch and stays visible as authentication required | REQ-5; current owner projection | `required` |
| M9 | Idle retention follows the existing Computer and Connector lifetime | REQ-6; current native owner Spec | `existing` |
| M10 | A `POSSIBLE` startup whose execution is no longer current is completed as `RETIREMENT_UNKNOWN` and is not redispatched | REQ-3; ADR-D1; current fresh-barrier Spec | `derived` |
| M11 | A READY result witnesses its reporting Connector connection; replacement connections receive exact retained-only Setup | REQ-1; REQ-3; existing connection replacement and native-owner lifetime | `derived` |

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| None | None | Input-triggered setup remains for messages that are not covered by a startup grant | No startup-only deletion | Input preparation tests still pass |

No authoritative input, authentication, or native lifetime behavior is removed.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester for product scope
- Approved on: not approved as a Design revision
- Approved Design revision: none
- Approved authority IDs: none
- Approved scope: The requester confirmed the product package on 2026-11-10 and requested immediate implementation. That package is an empty selected conversation, harness preparation before the first message, no synthetic prompt, no historical resend, no fresh replacement after a failed restore, authentication-required behavior when saved authentication is absent, and idle retention until the Computer stops or the Connector ends. Revision `3` mechanisms `M1`–`M11` were not presented or approved as a separate Design package. They are required, existing, or derived from that confirmed scope and current repository constraints. This section does not record revision-bound Design approval.
