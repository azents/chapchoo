---
title: "Safe Computer Power Controls Requirements"
created: 2026-11-05
implemented: 2026-11-05
tags: [computer, lifecycle, frontend]
document_role: primary
document_type: requirements
snapshot_id: power-261105
---

# Safe Computer Power Controls Requirements

- Snapshot: `power-261105`
- Document reference: `power-261105/REQ`

## Problem

A Computer owner needs explicit consent for interruption, useful progress feedback,
and a way to request running again while a previous shutdown is still completing.
A generic unconfirmed-outcome label does not explain the current task or what the
owner can do next.

## Primary Context

### Primary Actor

The current individual owner of an Agent within an authorized Team.

### Primary Scenario

The owner confirms stopping the Computer, sees shutdown progress, and requests
Start before shutdown completes. The Computer finishes stopping before another
execution starts with the original saved environment.

## Supporting Scenarios or Effects

- The owner cancels a Stop or Restart confirmation without interrupting processes.
- Delayed or unconfirmed shutdown remains distinct from completed shutdown.
- Lost connectivity or saved-environment verification must not become permission
  to start an overlapping execution.

## Goals

- Make process interruption deliberate and visible.
- Accept a later Start without requiring the owner to wait on the page for a
  stopped state before expressing that intent.
- Explain progress and impediments in actionable user language.

## Non-Goals

- Force-stopping an execution or manufacturing termination evidence.
- New operator recovery or storage-repair controls.
- Changing normal Agent deletion, ownership, Team authority or Admin capabilities.

## Requirements

### REQ-1. Confirm interrupting actions

Stop and Restart require an explicit confirmation before execution.

**Acceptance criteria**

- The first activation opens a confirmation rather than submitting the operation.
- Cancel and dismissal do not submit it.
- Confirmation describes process interruption and saved-file preservation.
- Polling cannot silently change the operation identity covered by consent.
- Mobile touch and keyboard activation obey the same boundary.

### REQ-2. Request Start during shutdown

While shutdown is incomplete, the owner sees shutdown progress and can request
Start to take effect only after that shutdown has actually completed.

**Acceptance criteria**

- Shutdown progress is shown without claiming that the Computer is already off.
- Start can express the subsequent running intent while the prior shutdown is
  still incomplete, subject to current ownership and supported capability.
- Acceptance of that intent is distinguishable from the successor starting.
- No successor runs before the exact previous execution is confirmed retired and
  its original saved environment is verified.
- Uncertain native work is not replayed to fulfill the subsequent intent.
- Agent deletion remains terminal and cannot be undone by an earlier Start intent.

### REQ-3. Useful progress and recovery language

Replace the generic unconfirmed-outcome label with the current operation and
information that helps the owner decide what to do.

**Acceptance criteria**

- Shutdown, startup, accepted later Start, completed operation and inability to
  verify completion are not represented by the same generic label.
- Delays and verification problems explain the consequence and available next
  action without inventing completed work or a guaranteed automatic recovery.
- General UI and its localized variants remain consistent; Admin continues to
  be read-only and truthful about the same underlying facts.
- Deletion consent states that the Agent, Computer and saved data are deleted;
  native cleanup order or internal processing steps are not part of that copy.

## Fixed Constraints

- Existing individual ownership, Team membership and expected-request fences.
- Exact native identity and definitive retirement evidence before a successor.
- Original saved storage preserved by Stop/Restart; normal Agent deletion cleans
  its Pod/container and saved storage.
- No automatic replay of possibly accepted native effects.
- No merge, deployment or production manipulation is authorized by this UI task.

## Open Assumptions

- The accepted Running target survives page closure, navigation and backend
  restart. The requester confirmed that it updates the existing desired state;
  there is no separate scheduled-start queue or pending-intent store.
- A dedicated cancellation control for the subsequent Start was not requested;
  any new cancellation behavior requires an explicit product decision.
- Existing eligibility rules for disconnected management or unverified saved
  storage remain unless a specifically approved change is necessary.

## Confirmation

The requester explicitly requested all three outcomes on 2026-11-05 and confirmed
persistent desired-state updates, without separate pending-start management, on
that date. The implementation request and this bounded technical direction are
confirmed; no broader recovery or cancellation controls are authorized.
