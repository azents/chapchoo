---
title: "Implicit Conversation Reply Work Design"
created: 2026-11-19
tags: [backend, engine]
document_role: primary
document_type: design
snapshot_id: reply-261119
---

# Implicit Conversation Reply Work Design

[Requirements](../requirements/reply-261119-implicit-conversation-work.md) ·
[Decision](../adr/reply-261119-implicit-conversation-work.md)

## Design Authority

Revision 1 implements the requester's confirmed single, hidden reply Work
using the existing authorized conversation operations:

| ID | Mechanism | Authority |
| --- | --- | --- |
| M1 | Open or resume one Work under the existing conversation lock on a fresh input admission or failed-input replacement | `REQ-1`, `ADR-D1` |
| M2 | Project unplanned Work as an internal objective, and project a summary only after explicit plan publication | `REQ-1`, `REQ-2`, `ADR-D1` |
| M3 | Apply existing `send` actions to the same Work; pending input wins against wait and completion | `REQ-3`, `ADR-D1` |
| M4 | Require durable pickup for reply context and defer ending reports while the current launch is reporting a dispatched input | `REQ-1`, `REQ-3`, `ADR-D1` |

## Lifecycle and boundaries

The fresh input insert and reply Work transition share a database
transaction. Existing acceptance recovery and internal continuation do
not create another Work. A message joining Work neither changes its identity
nor deletes its plan. The existing counter resets on fresh accepted human input.

Work stores no synthetic title or checklist entry. The empty stored title
encodes an unpublished plan (public titles are nonblank); such a row has an
empty task list. The owner summary omits it. The bounded context instead
contains its reply objective and disposition once pickup is durably reported.
The first explicit `continue`
supplies a title and replaces the task list, keeping the same Goal identity.
Existing published plans and captured history are unchanged. Do not backfill
old conversations or change the source/launch fences. New writes require
compatible backend readers. Preserve legacy plans; migration rollback rejects
rows whose unpublished plan or readiness would be lost.

The existing conversation lock serializes accept and `send`. An earlier
pending input leaves Work active even when a requested message is published;
an input accepted after completion opens fresh Work. A consumed user input
is available to the native context under existing delivery guarantees.
Existing failure and uncertainty reporting for input delivery remain separate
from Work settlement. When pending input prevents wait or settlement, discard
readiness from the already addressed input and wait for successful pickup.
The eligible Goals share the bounded projection without requiring readiness
from unrelated Goals. The Connector may defer a live ending report while
the same launch has a dispatched input awaiting its report, but not while a
future input is parked behind that ending. `send` catalog refresh uses the
existing owner boundary.

## Authority and Feasibility

- REQ-1 uses M1 and M4: the queue admission, retry, and successful pickup paths
  already serialize on the conversation; the current native owner has a
  scoped input-report ticket before dispatch.
- REQ-2 uses M2: the summary and bounded native projection are already separate
  reads. The Work row remains canonical, and no RPC or browser state is added.
- REQ-3 uses M3 and M4: the `send` transaction can inspect pending ordinary
  inputs independently of appending Agent Chat receipts. Existing native Stop
  and forwarding fences retain authority; no receipt or decision is replayed.

## Test Strategy

Use the same SQLite and PostgreSQL tests for single identity, duplicate
acceptance, no summary, title validation, retained plan and pending-input
settlement. Prove the simple reply, bounded continuation and visible-plan
journeys using the existing deterministic native/Chat acceptance fixtures.

## Design Approval

Revision 1, mechanisms `M1`–`M4`: implements the requester-confirmed scope and
implementation direction of 2026-11-19 (KST). Pickup and ending ordering
implement the existing delivery guarantees; they do not change the user
contract. No merge or deployment is authorized.
