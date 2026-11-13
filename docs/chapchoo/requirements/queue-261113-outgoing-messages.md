---
title: "Server-Owned Outgoing Message Requirements"
created: 2026-11-13
implemented: 2026-11-13
document_role: primary
document_type: requirements
snapshot_id: queue-261113
tags: [conversation, queue, web]
---

# Server-Owned Outgoing Message Requirements

## Primary outcome

Sending a message must expose one server-owned outgoing item until delivery
succeeds, rather than making browser-local guesses look like accepted messages.

## Requirements

1. Submitting text enqueues a server-owned outgoing item. A consumer delivers the
   item to ACP. Successful delivery removes it from the outgoing queue and leaves
   the normal conversation message; a reported failure retains an errored item.
2. Failed outgoing items are not automatically retried. Retry removes the original
   item and enqueues a new item. Delete removes the failed outgoing item.
3. Pending-message rendering comes only from queried server queue items. Failed
   enqueue requests preserve the composer draft and show a composer error, without
   adding an optimistic message.
4. Both waiting and failed outgoing message bubbles are dimmed. Error feedback
   remains outside the bubble. Retry and Delete are icon-only controls with
   localized accessible names and tooltips, using the confirmed compact spacing.
5. Existing conversation messages and private ownership checks are preserved.
   Unknown physical outcomes are not recast as proof of nonexecution or rollback.
6. Remove obsolete local pending-message state, deletion-retention/recovery
   mechanisms and their superseded tests. Deleting an outgoing item removes
   that item and its dependent preparation/delivery records; it must not introduce
   a second persistent safety-history store.

## Scope and confirmation

The requester specified the server queue, consumer, success removal, retained
failure, no automatic retry, delete-then-requeue, table-only pending rendering and
composer failure behavior on 2026-11-13 KST. They then confirmed the real-component
visual proposal and requested implementation, correcting that failed messages,
like waiting messages, must also remain dimmed. This snapshot records that
instruction; it does not approve the withdrawn general result-verification design.

No live conversation invocation, merge, deployment, new task deadline, browser
offline queue, generic broker or replacement native session is authorized.
