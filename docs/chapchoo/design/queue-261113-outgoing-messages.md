---
title: "Server-Owned Outgoing Message Design"
created: 2026-11-13
implemented: 2026-11-13
document_role: primary
document_type: design
snapshot_id: queue-261113
tags: [conversation, queue, web]
---

# Server-Owned Outgoing Message Design

## Design Authority

Requirements `queue-261113/REQ-1..6` and decisions
`queue-261113/ADR-D1..D5` record the requester's explicit flow and visual selection,
and the implementation boundaries needed to preserve that flow.
No withdrawn verification deadline or general operation ledger is included.
The implementation reuses existing input admission, owned processing, source
fences, typed failure facts, chunk bounds and private-owner admission.

Revision: 1. M1/M2/M3/M4 are required by the requester's queue lifecycle and
confirmed visuals. M5 derives from preserving schema evolution. M6 is required
by the requester's explicit removal instruction. These are the exact approved
material mechanism IDs. Equivalent local code/fixture organization adds no
additional source of truth or recovery mode.

The consumer-termination and receipt-order details below refine M1/M2/M3 under
REQ-1/REQ-2/REQ-5/REQ-6. They do not infer approval of a new task timeout,
historical ledger or changed visual/recovery flow.

## Mechanisms

- **M1 — Queue authority.** `conversation_inputs` remains the outgoing table.
  Activity reads retrieve the text of retained non-deleted items, not just
  failures. The browser renders these rows and no locally manufactured pending
  entries. The existing consumer starts only after committed admission.
- **M2 — Success boundary.** Remove the admission-time user-receipt write.
  Successful native Inserted/Steered reporting writes the user receipt and removes
  the queue item in one transaction. Failed reports retain the queue item and
  existing failure facts. Duplicate successful report ACK recovery remains safe.
- **M3 — Manual recovery.** A new typed RetryInput contract names the old item
  and fresh request ID under the existing input scope. Persistence verifies the
  old item's settled failure, deletes it, then admits its retained text under
  a fresh operation identity atomically. Only fresh admission dispatches a
  consumer. Delete physically removes a failed outgoing row and its child facts.
  Neither action deletes an already committed historical user receipt.
- **M4 — Presentation.** Composer enqueue errors retain the draft and use the
  approved adjacent alert. Queue waiting/failure bubbles use opacity 0.6,
  independent of their error/action row. Retry/Delete preserve the approved
  28px target and standard refresh glyph. Mutations have no automatic retry.
- **M5 — Preserving schema evolution.** A forward migration removes the old
  insert-time chat-receipt trigger, preserving existing queued bodies and Chat
  receipts. Obsolete bodyless rows from already completed explicit deletions
  are removed with their child facts.
  Its downgrade refuses unmatched outgoing rows instead of losing data.
- **M6 — Obsolete state removal.** Delete browser-local outgoing lists,
  acceptance-correlation helpers and tests that only defend those removed
  mechanisms. Failed-item deletion is a physical row deletion with dependent
  input preparation/delivery cleanup, not a retained bodyless result. No second
  outgoing-membership table or persistent deleted-item safety record is added.

### Owned consumer and ordered publication boundaries

The queue row's PENDING/ERROR state is independent of native outcome facts.
Only a newly admitted processing task renews its database-time lease, using the
existing component heartbeat/validity policy. Existing recovery never renews
or dispatches. Processor return/error finalizes its exact row; panic/process loss
stops its co-owned heartbeat and an independent reaper expires it. Both processing
and renewal continue polling while a DB lock is awaited, and normal shutdown
drains an already-started renewal before finalization. Post-dispatch observation
binds the original Control/connection, not any successor route. Renewal locks the
row before reading DB time and cannot revive an expired/error row. Old finalizers
cannot affect a new replacement ID. No model-task duration is imposed.

Connector reports a generic native worker failure as AttemptError, not as proof
of nonexecution. Authentication, exact current source/target and an existing
forwarding fact gate that report. Late valid success can still consume the retained
row, so manual Retry retains the original uncertainty rather than promising
exactly-once physical delivery.

One ephemeral publication owner is created only on an actual native launch.
An input ticket is registered before prompt dispatch; the serialized native actor
rechecks that launch's stable application binding before any effect. Tool calls
snapshot already-pending tickets and wait within the existing 50-second bounded
invocation task. Catalog/initialization, the native actor and report publication
remain independent. Inserted/Steered ReportInput acknowledges whether an ordinary
user receipt exists, including lost-ACK recovery by that receipt's existing
source-input provenance. Deleted-row closure alone cannot open the ticket.
Failure ends only calls already dependent on that ticket; later Send/Retry remains
available. Retirement closes the old owner, and an old ACK cannot open its
successor. No state survives that native lifetime.

## Removal and Replacement

The requester explicitly selected the new flow and required deleting mechanisms
that no longer serve it. Server queue queries replace local uncertain/sending
lists, draft-clearing receipt correlation and their synthetic fixtures. Physical
deletion replaces logical body clearing and indefinite delete-result recovery.
Success-time message publication replaces admission-time publication. Each
boundary is checked by source searches plus queue, message-lifetime and browser
tests; existing ordinary Chat messages remain immutable.

## Test Strategy

E2E retains the core native fixture send flow; detailed queue lifecycle and
concurrency belong to both-engine persistence and service/RPC tests. Verify
enqueue without a normal receipt, one receipt at success, failure retention,
no consumer replay, delete/replacement identity, atomic retry and failed retry
preservation, orphan expiry/no resurrection, processor error/panic, cancellation-safe
reaper drain and success-vs-deletion acknowledgement. Connector tests exercise
actual MCP transport, fast publication, lost ACK and successor isolation.
Web-core tests cover queried waiting bodies and closed-row races.
Component tests verify composer preservation/no optimistic row, desktop/mobile
waiting and failure dimming, tooltips, keyboard focus, icon-only controls and
replacement identity. Real-component renders use the product theme/fonts at
native resolution. These fixtures are not live model-side delivery evidence.

## Design Approval

On 2026-11-13 KST the requester specified M1–M3's queue lifecycle and recovery,
then confirmed M4's rendered visual proposal and requested implementation,
including the subsequent correction that failed bubbles remain dimmed. M5 is
required to evolve the existing trigger without rewriting historical migrations
or stored messages. No broader result-verification or lifecycle policy is
inferred from this confirmation.

Mode: Collaborative. Decision owner: requester. Date: 2026-11-13 KST. Approved
revision: 1; authority IDs M1–M6. The subsequent explicit instruction removes
obsolete state rather than authorizing a persistent safety-history replacement.
