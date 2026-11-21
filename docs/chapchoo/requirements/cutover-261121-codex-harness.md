---
title: "Codex Harness Cutover Requirements"
created: 2026-11-21
document_role: primary
document_type: requirements
snapshot_id: cutover-261121
tags: [architecture, backend, migration]
---

# Codex Harness Cutover Requirements

- Snapshot: `cutover-261121`
- Document reference: `cutover-261121/REQ`

## Problem

The current Codex integration cannot expose the reasoning and speed choices of a candidate model before it is saved, or distinguish all supported speed selections. Its protocol-specific execution and event contracts also make it difficult to integrate another harness without changing conversation behavior.

## Primary System Outcome

After one coordinated cutover, both new and existing Codex conversations use the replacement execution path. Existing conversations continue with the same native identity, retained history, workspace and ownership. The product can integrate another harness without requiring it to reproduce Codex-specific capabilities.

## Supporting Scenarios or Effects

- A user edits the model, reasoning effort and one supported speed together and saves without sending a message.
- A different harness can provide its supported subset of Chapchoo conversation behavior without emulating Codex's settings or claiming unavailable capabilities.

## Goals

- Replace the Codex execution path for all Codex conversations in one cutover.
- Preserve authorization, exact conversation and operation continuity, and established behavior through the change.
- Offer model-dependent choices from the applied connection, not from a product-owned model table.

## Non-Goals

- A production canary, legacy Codex fallback, per-conversation migration switch, or concurrent owners for one conversation.
- Adding a named non-Codex harness, translating a Codex thread into another harness, or erasing historical records.
- Adding a separate inference credential, provider billing policy, or changes to the conversation entry point.

## Requirements

### REQ-1. Single cutover with exact continuation

The replacement applies to both new and existing Codex conversations at the same release boundary.

**Acceptance criteria**

- A rehearsed cutover exercises both a new Codex conversation and one created before the cutover. Neither uses the retired Codex path after admission reopens.
- An existing conversation uses the same native identity and private workspace, retains its prior transcript without duplicate messages, and includes prior context in subsequent work. Capture and transcript synchronization continue from the retained evidence.
- If exact resumption or preflight cannot be established, report the failure without creating a substitute conversation, independently switching back, or discarding its recorded history.
- Running work, pending responses and unresolved external effects are not treated as retired merely because the user interface shows idle. Prior ownership must be settled before admitting a successor. An uncertain submission is never automatically replayed.

### REQ-2. Preserve authority and conversation behavior

Replacement execution must continue to enforce the current product's conversation, credential and operation boundaries.

**Acceptance criteria**

- Private ownership and Team admission remain enforced, including when opening an existing conversation. Both configured credential kinds keep the authorized account and refresh authority; no additional credential, implicit account fallback, or credential disclosure is introduced.
- Ordinary prompt submission, live steering, Stop, approvals and forms, server tools, and one-use Work continuation preserve their source-bound targets and failure semantics.
- A submission acknowledgement, exact input insertion, a new execution and completion remain distinct. Capture replay and child or stale events cannot complete a current operation or mint a continuation.
- Tests exercise duplicate, delayed, lost and rejected operations; in particular, an ambiguous delivery or retired responder cannot become permission to resend or target another generation.

### REQ-3. Harness-independent conversation contract

Shared conversation behavior remains available through either supported integration family, while optional capability differences remain observable.

**Acceptance criteria**

- A conformance suite verifies shared lifecycle, submission, observation and response semantics against both adapter families without tying the product-facing contract to either wire protocol.
- A missing, unknown or unverified capability is never exposed as selectable or converted into a successful action. One adapter's state, credentials and request identities cannot satisfy another owner's operation.

### REQ-4. Model-dependent preferences in one edit

The existing Activity → Model → Change model sheet allows one edit and save for model, available reasoning effort and one supported response speed. It does not start work or change an in-flight turn.

**Acceptance criteria**

- Selecting a candidate model reveals the options verified for that model and applied connection before Save, without starting inference or changing the live model as a lookup technique. Back or Cancel discards the draft.
- The speed selector accommodates Standard, Fast and Ultrafast where supported. Explicit Standard remains distinct from inheriting the native default. Changing models never silently carries a paid speed to the new model.
- An explicit supported combination is applied to the intended new execution, not live steering or native commands. Validation and tests cover a stale or unsupported request and distinguish a saved preference from a confirmed application; unsupported choices are never reported as applied.
- Unknown candidate capabilities do not borrow options from a different model, and do not by themselves prevent the existing model-only operation.

## Fixed Constraints

- Use released, unmodified harness artifacts and supported interfaces.
- Keep existing ownership checks, retained operation fences and data recovery guarantees unless separately approved.
- The one-time production cutover is distinct from implementation sequencing and local rehearsal. The repository still requires explicit merge and deployment approval.

## Open Assumptions

- Local exact-thread resumption and tier serialization do not establish production authentication, retained-capture compatibility, or an account's service-tier entitlement. Those remain validation conditions.

## Confirmation

The requester confirmed proceeding with this snapshot on 2026-11-21: one cutover for new and existing Codex conversations, retaining ACP for other harnesses.
