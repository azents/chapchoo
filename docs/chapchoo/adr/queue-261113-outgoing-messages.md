---
title: "Server-Owned Outgoing Message Decisions"
created: 2026-11-13
document_role: primary
document_type: adr
snapshot_id: queue-261113
tags: [conversation, queue, web]
---

# Server-Owned Outgoing Message Decisions

## ADR-D1. Use the server outgoing queue as the pending-message authority

The requester selected a server queue and consumer, successful row removal,
retained failures without automatic retry, and explicit delete-then-requeue.
This replaces browser-owned uncertain-message lists for sending. Reuse the
existing durable `conversation_inputs` and exact native preparation/forwarding
authority rather than creating another queue or generic worker service.

Queue acceptance is not successful delivery. New normal user receipts are
committed at successful Inserted/Steered pickup, atomically with queue removal.
Previously committed receipts remain immutable. Existing preparation and delivery
facts distinguish reported failure from unknown physical outcome.

## ADR-D2. Make explicit replacement a completed persistence operation

Retry performs deletion and fresh admission in one database transaction, then
hands off only the newly admitted item to the existing process-owned consumer.
The replacement has a fresh identity. Repeated recovery of that replacement
does not dispatch it again. This is the direct implementation of the requested
delete-then-requeue contract, avoiding loss of retained text between browser calls.
Delete does not mean cancellation or rollback of an external native effect.

## ADR-D3. Preserve the confirmed presentation

The requester confirmed icon-only Retry/Delete with tooltips, original refresh
geometry, 28-CSS-pixel targets, zero button gap and 18-pixel icons. This explicitly
overrides the shared comfortable mobile target for these two controls only.
Both waiting and failed bubbles use opacity 0.6; recovery controls retain their
ordinary contrast. Composer enqueue failure uses the approved adjacent error
notice and preserves the draft.

## ADR-D4. Keep queue-attempt termination separate from physical outcome

To satisfy `queue-261113/REQ-1`, a consumer that ends without successful pickup
must leave an actionable error rather than an indefinitely pending item.
The existing outgoing row owns its PENDING/ERROR status and a renewable consumer
lease. Reuse the existing component validity/heartbeat policy for ownership
liveness; this is not a model task deadline. Fresh admission alone renews, while
an independent process-owned reaper closes expired owners. Neither expiry nor
AttemptError asserts native rollback or authorizes automatic replay. A valid late
success may still commit the normal receipt while its outgoing row exists.
Separate permanent history would conflict with `queue-261113/REQ-6`.

## ADR-D5. Order native publication behind the successful user receipt

Moving user publication from admission to pickup must preserve the ordinary
user-before-response ordering under `queue-261113/REQ-1` and `REQ-5`.
Use a bounded memory-only fence for the actual native owner and input, installed
before prompt dispatch. Only a successful report acknowledgement that confirms
the existing normal user receipt opens that fence. A mere absent-row closure
after explicit Delete does not. Response loss retains the fence through report
recovery; failure rejects existing dependent calls without disabling later manual
Retry. Actual native retirement closes that owner's calls. No causal input
history, deleted-row record or model-completion wait is introduced.
