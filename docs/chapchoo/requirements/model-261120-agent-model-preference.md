---
title: "Conversation Model Selection Requirements"
created: 2026-11-20
updated: 2026-11-21
implemented: 2026-11-21
tags: [agent, conversation, harness]
document_role: primary
document_type: requirements
snapshot_id: model-261120
---

# Conversation Model Selection Requirements

- Snapshot: `model-261120`
- Document reference: `model-261120/REQ`

## Problem

An owner occasionally needs a different model without leaving the same
conversation, changing connections, or losing ongoing work. The owner should
be able to start a conversation without choosing a model first.

## Primary Context

### Primary Actor

The authenticated owner of a private Agent with a supported harness connection.

### Primary Scenario

An owner opens conversation detail, chooses another model, and explicitly
saves it. An execution already underway continues unchanged. The next new
conversation execution is requested with the saved model in the same session
and connection. The choice remains in effect if that session is resumed after
a restart. A saved choice does not assert which model a particular turn used.

## Supporting Scenarios or Effects

- A session starts with a model chosen by the connected harness.
- The owner can choose a replacement in the same conversation if the selected
  model is confirmed unavailable before a request starts.

## Goals

- Show and change one concrete model for the prepared session.
- Preserve explicit changes during same-session, same-connection recovery.
- Reuse the generic single-select list from the Chapchoo design system.

## Non-Goals

- No Default option, saved initial model, reset mode, or model reservation for
  an unprepared session.
- No model picker in the composer or Agent creation, per-message controls,
  or per-answer model indicator.
- No automatic selection, substitution, preference reset, or request replay
  as recovery from an unavailable model.
- No model guarantee or forwarding changes for harness-native commands.
- No new connection or sign-in flow.

## Requirements

### REQ-1. Let the harness choose the initial model

- The owner can send the first message without a model selection. A genuinely
  new session starts with the model chosen by its connected harness.
- When no user selection exists and the actual session model is known, show
  its name. When it is unknown, do not present a guessed model or Default.
- The initial model is not saved as a Chapchoo preference or restoration
  target. If still selectable, it can later be chosen by its actual name.
- If the session is already prepared and its model is known, the owner can
  change it before the first message through the ordinary conversation detail
  control.

### REQ-2. Choose one concrete model from conversation detail

- Desktop opens conversation detail through `…`; mobile uses the existing
  character/status control. The existing Agent detail link remains available.
- For a prepared session with a known model, the **Model** row opens **Change
  model** in the same panel. Offer concrete model names selectable for the
  current connection and mark the current choice when it is among the
  available options.
- The list uses adjacent rows with dividers, whole-row selection, keyboard
  operation and one trailing square check. Korean labels are **모델** and
  **모델 변경**. Model names, next-execution guidance and save controls remain
  feature-owned, separate from the shared list.
- Distinguish model-list lookup failure from a successful empty list.

### REQ-3. Save explicitly and retain the choice for its scope

- Changing the draft does not change the effective choice. Cancel or close
  before saving discards the draft and returns to conversation detail. A
  pending save prevents duplicate submission and dismissal.
- Confirmed save returns to conversation detail, updates the displayed saved
  model, and explains that it is requested for new conversation executions.
- Confirmed non-save preserves both the prior choice and the draft for explicit
  retry or cancel. An uncertain save is not a confirmed failure or an
  authorization to resubmit automatically.
- Reopening the page or restoring the same session with the same connection
  after a restart preserves the explicitly saved model. A reconnect alone
  does not discard it if the session and harness/account connection are
  unchanged. An actually new session or a different harness/account
  connection does not inherit it.
- Failed restoration is not permission to replace the existing session.

### REQ-4. Request the choice for new conversation executions

- Chapchoo requests the latest confirmed choice for new executions of
  ordinary conversation input and work continuation in the same session and
  connection, including queued inputs not yet submitted to start a new turn.
- The harness owns actual execution. If it joins an existing turn during
  submission or does not disclose the model it used, Chapchoo does not claim
  that a turn ran with the requested choice.
- Harness-native commands remain usable as before and can use a different
  model. The picker only needs brief new-turn guidance; do not add turn model
  history or diagnostic notices.
- An already started execution keeps its model. Input joining an active
  execution does not change the model of that execution.
- Saving does not start, stop, resume, or resend work or replace the session,
  conversation or history.

### REQ-5. Recover explicitly from confirmed unavailability

- When the selected model is confirmed unusable and a request is known not
  to have started, show the model and the impact in the same conversation.
  Provide a **Change model** action opening the picker.
- Do not preselect an alternative, even if there is only one. The owner must
  select and save; cancel preserves the prior choice. Saving alone does not
  resend the previous request.
- A stale, failed, or otherwise inconclusive lookup does not establish
  removal or non-start and does not change the saved model. An uncertain
  dispatch is not presented as non-start and is not replayed.

## Fixed Constraints

- Preserve current private Agent ownership and Team admission on model reads,
  writes and use. Model selection does not grant access to credentials.
- Use supported interfaces of unmodified external harness releases. Codex
  is the first integration exercised by this snapshot, not the public
  definition of a harness.

## Open Assumptions

- Validate selectable model observation, confirmed pre-start failure, and
  application of a saved choice on the same session after process restart.
  A displayed model name or a successful save does not by itself prove that a
  subsequent request used that model.

## Confirmation

The requester confirmed this repository snapshot on 2026-11-20 (KST) before
material technical decisions were accepted. Later the same day, the requester
confirmed the first integration's scope clarification in `REQ-4`, after
released harness behavior showed that native commands can use another model.
The requester then selected delivery against the existing unmodified Codex
release: request the selected model for new turns without claiming actual
per-turn model use from a message acknowledgement.
