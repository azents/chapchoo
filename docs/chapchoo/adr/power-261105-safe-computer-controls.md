---
title: "Safe Computer Power Controls Decisions"
created: 2026-11-05
tags: [computer, lifecycle]
document_role: primary
document_type: adr
snapshot_id: power-261105
---

# Safe Computer Power Controls Decisions

Authority: [power-261105/REQ](../requirements/power-261105-safe-computer-controls.md).

## D1. Update the existing desired state during Stop

The requester selected one persistent desired state rather than separate pending
Start management. Start against the exact current Stop updates its Running target
while preserving the physical Stop's request identity, fingerprint, current and
expected execution, phase and native-call correlation. It is an idempotent target
update against the captured predecessor, not a new historical Start receipt.

Alternatives were a separate durable queued Start/receipt and a broader separation
of all effect and intent identities. Both add lifecycle state beyond the requested
single target. Replacing the active Stop request outright is not viable: it fences
its existing reports and drops Docker continuation correlation.

Existing original-request replay/fingerprint checks remain first; changed-payload
reuse of that Request ID still rejects. A later operation or Delete makes the
captured predecessor stale. No migration, new queue, new persisted version or new
wire method is required. Retirement completion selects Starting when the target
is Running; the existing successor engine still requires confirmed old retirement
and original saved storage. Completion-first Start remains safe under the same
transaction locks. Original-execution Running reports cannot fulfill the newer
Running target before retirement.

Decision owner: requester. Accepted 2026-11-05 by explicit implementation direction
and confirmation of desired-state ownership. Applies to REQ-2 only; it does not
expand operational recovery or permit uncertain native-effect replay.
