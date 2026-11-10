---
title: "Failed message recovery actions"
created: 2026-11-10
implemented: 2026-11-10
tags: [chat, frontend, recovery]
document_role: primary
document_type: requirements
snapshot_id: message-261110
---

# Failed Message Recovery Actions

- Snapshot: `message-261110`
- Document reference: `message-261110/REQ`

## Problem

A message that failed before delivery has no direct retry or delete action. Separate pending and retained rendering can reorder messages, and ambiguous presentation does not help the sender recover.

## Primary Actor

The private Agent owner sending text in Chat.

## Primary Scenario

The owner sees a failed outgoing message, understands whether it reached the harness, then explicitly retries it or deletes it without accidentally duplicating uncertain work.

## Supporting Scenarios or Effects

- Reentry preserves the resulting message state and order.
- Sending, failed and delivered text retains one consistent user-bubble treatment.

## Goals

- Make confirmed failed delivery recoverable through explicit actions at the message.
- Keep uncertainty, execution cancellation and deletion distinct.

## Non-Goals

- Automatic retry or replay.
- Undoing native work or external effects.
- Deleting successful native conversation history.

## Requirements

### REQ-1. Explicit retry of confirmed failure

A sender can intentionally retry text that is confirmed not delivered.

**Acceptance criteria**

- A failed message has a reachable retry action when current ownership, source and execution allow sending.
- Retry retains the text and consistent bubble presentation while showing progress and its outcome.
- Lost responses and possible forwarding do not authorize duplicate dispatch.
- A new attempt does not erase the outcome of the previous attempt or inherit authority from an old source.

### REQ-2. Delete failed outgoing messages

A sender can delete a failed message from the conversation.

**Acceptance criteria**

- A confirmed failed message exposes an explicit delete action.
- A successful deletion persists after reload and reentry.
- Delete is not a promise of cancelling running work or undoing effects.
- Retained operation evidence needed to prevent duplicate dispatch remains valid.

### REQ-3. Consistent message presentation and ordering

Sending and failed messages use the same user-message form as successful text.

**Acceptance criteria**

- Status and actions remain visually subordinate to message text.
- New messages follow older messages instead of appearing above retained failures.
- Reconciliation does not duplicate the same logical message.

## Fixed Constraints

- Exact private-owner, Team, conversation, source and operation checks remain authoritative.
- No uncertain input is automatically replayed.
- The previously requested authentication correction and startup recovery fixes remain independent.

## Open Assumptions

- Existing copied/backed-up database pages are outside the promise of logical body removal.

## Confirmation

The requester confirmed on 2026-11-10: deletion removes the message body and visible entry while retaining the used request identity and outcome; an explicit retry uses a new request identity and the same outgoing-bubble progress presentation. Repeated old request identities never start another execution. The requester explicitly directed implementation after this explanation.
