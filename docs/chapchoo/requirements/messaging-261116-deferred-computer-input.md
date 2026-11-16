---
title: "Messages Before Computer Readiness Requirements"
created: 2026-11-16
document_role: primary
document_type: requirements
snapshot_id: messaging-261116
tags: [conversation, computer, messaging]
---

# Messages Before Computer Readiness Requirements

- Snapshot: `messaging-261116`
- Document reference: `messaging-261116/REQ`

## Problem

A private Agent's composer currently becomes read-only when its Computer is stopped or preparing. Merely accepting a message does not ensure that it waits for Computer readiness. Owners should be able to write normally without managing the Computer's preparation timing.

## Primary Context

### Primary Actor

The currently authorized private Agent owner.

### Primary Scenario

The owner sends a message through the ordinary composer before the Computer is ready. The message is retained until delivery can proceed. A stopped cloud Computer starts as part of this flow; a local Computer is not powered on automatically. Once ready, the Agent receives the accepted message without another user submission.

## Supporting Scenarios or Effects

- Several messages can be submitted while the Computer starts or restores.
- A local Computer can receive retained messages after it is made ready through its existing owner-controlled mechanism.
- Delivery-attempt failures retain their existing explicit recovery boundaries.

## Goals

- Make Computer startup/restoration transparent to ordinary message composition.
- Preserve accepted text and prevent duplicate native delivery.
- Preserve the owner's explicit lifecycle intent.

## Non-Goals

- Automatically powering on local Computers.
- Browser-persistent network-offline submission or automatic replay of uncertain requests.
- Changing AI authentication, private ownership, Team admission or destructive-operation authority.
- Introducing a separate queue-composition mode or queue-specific placeholder.

## Requirements

### REQ-1. Ordinary composition before readiness

The owner can submit messages while the Computer is stopped, starting or restoring, subject to unchanged account and Agent authority.

**Acceptance criteria**

- These Computer states alone do not disable ordinary text submission.
- Accepted original text remains available until delivered or explicitly resolved.
- The owner does not need to submit the same accepted message again after startup.

### REQ-2. Cloud-only automatic startup

Sending to a stopped cloud Computer initiates its startup. Sending to a local Computer never initiates power control.

**Acceptance criteria**

- Startup and restoration already in progress continue without duplicate startup requests.
- Cloud/local behavior follows the Computer's authoritative kind, not a Provider name or runtime technology.
- Local messages remain retained until their Computer is made ready through existing controls.

### REQ-3. Retained messages deliver after preparation

Messages accepted while the Computer is preparing wait for readiness and then use the established delivery path.

**Acceptance criteria**

- Multiple accepted messages retain their original text and submission order.
- Computer restoration does not require resubmitting accepted, not-yet-delivered messages.
- A delivery attempt with an uncertain outcome is not automatically replayed as if it had never been attempted.
- Deletion and authority withdrawal retain their existing terminal boundaries.

### REQ-4. One user experience

The same composer and normal placeholder are used before and after readiness.

**Acceptance criteria**

- No queue-specific or waiting-mode placeholder is introduced.
- Composer clearing still follows acceptance, and a rejected submission retains the draft.
- Background waiting is not presented as a different message-composition mode.

### REQ-5. Explicit Stop precedence

If the owner explicitly stops the Computer while accepted messages are waiting, retain those messages but do not restart solely for that older backlog. A later manual Start or a new message may resume the flow.

**Acceptance criteria**

- An older waiting message does not undo a newer explicit Stop.
- The message text remains available rather than disappearing due to Stop.

## Fixed Constraints

- Cloud-only startup and ordinary composer UX were explicitly requested on 2026-11-16.
- Retain current account/private-owner admission and no-replay boundaries for already attempted delivery.
- Actual startup failure and required recovery must not be disguised as successful delivery.

## Open Assumptions

- Existing same-source/native attempt uncertainty remains distinct from waiting before an attempt; technical ownership and recovery decisions remain to be resolved in the ADR and Design.

## Confirmation

The requester confirmed the complete Requirements, including explicit Stop precedence, on 2026-11-16 (KST). REQ-1 through REQ-4 also record the requester's direct instructions earlier that day. Material technical decisions and the complete implementation Design require their own approval.
