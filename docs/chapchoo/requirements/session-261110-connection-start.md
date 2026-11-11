---
title: "Connection-Time Session Start Requirements"
created: 2026-11-10
document_role: primary
document_type: requirements
snapshot_id: session-261110
updated: 2026-11-10
tags: [conversation, connector, harness]
---

# Connection-Time Session Start Requirements

- Snapshot: `session-261110`
- Document reference: `session-261110/REQ`

## Problem

A running Computer with a connected Connector still leaves harness preparation until the owner sends a message. The first reply therefore waits for process and session startup.

## Primary Context

### Primary Actor

The owner of a Computer.

### Primary Scenario

The Computer is desired running and its current Connector is authenticated. Before the owner sends a message, Chapchoo prepares that Computer's harness session. The owner's first message does not wait for that startup. When no conversation exists yet, an empty conversation exists so the session can be prepared. When a conversation already exists, only that retained session is resumed.

## Supporting Scenarios or Effects

- Missing saved authentication remains an authentication-required state and does not launch the harness.
- A retained session that cannot be restored stays failed rather than becoming a different session.
- An idle prepared harness remains available while the Computer stays desired running and the Connector remains connected.
- Stopping the Computer or ending the Connector ends that prepared harness.
- Opening the chat page by itself does not start the harness.

## Goals

- Remove harness startup from the wait before the first reply when authentication and a restorable session are available.
- Make the prepared session a real conversation, including the initial empty conversation.

## Non-Goals

- Do not submit a synthetic prompt, run model work, or resend earlier messages in order to start the harness.
- Do not replace a missing or unrestorable retained context with a new session.
- Do not change who may refresh or select saved authentication.
- Do not start a harness for a Computer that is not desired running or whose current Connector is not authenticated.

## Requirements

### REQ-1. Prepare before the first message

When a Computer is desired running and its current Connector is authenticated, prepare the harness session before the owner sends a message.

**Acceptance criteria**

- With usable authentication and a restorable session, the first message is not delayed by harness process or session startup.
- The chat page opening alone does not start preparation.

### REQ-2. Empty conversation for a first session

When those start conditions are met and the Computer has no conversation, create an empty conversation and prepare its session before the owner speaks.

**Acceptance criteria**

- The owner can see a conversation with no user message before sending one.
- That conversation is the session used by the later first message.

### REQ-3. Resume only the retained session

When a conversation already exists, resume only its retained session. If that context cannot be restored, do not create or switch to another session.

**Acceptance criteria**

- A successful resume uses the retained source.
- A failed restore remains a failed or unavailable session state and does not become a fresh session.

### REQ-4. No prestart model work

Preparation does not perform model work or resend prior owner messages.

**Acceptance criteria**

- No model request occurs before the owner sends a new message.
- Earlier messages are not submitted again as part of preparation.

### REQ-5. Authentication remains required

Without saved authentication that can be applied, preparation does not launch the harness and the owner still sees that authentication is required.

**Acceptance criteria**

- Missing authentication does not start a native process.
- The visible state identifies authentication as required rather than reporting a successful start.

### REQ-6. Idle lifetime follows Computer and Connector

An idle prepared harness remains available while the Computer is desired running and the Connector remains connected. It ends when the Computer is no longer desired running or the Connector ends.

**Acceptance criteria**

- Absence of messages does not by itself stop the prepared harness while both conditions hold.
- Computer stop or Connector end does not leave that prepared harness running.

## Fixed Constraints

- Existing owner, Team, application, route, execution, and source fences remain in force.
- Applied authentication stays immutable for the preparation that used it.
- A failed restore is not a permission to start a fresh native context.
- Preparation must not block Connector connection from becoming active.

## Open Assumptions

None. Repository evidence shows each Agent has one selected conversation, and production conversation creation inserts that selection with the conversation. No second eligible conversation requires a selection decision.

## Confirmation

Confirmed by the requester on 2026-11-10. The requester selected creation of an empty session so the harness can start before the first message, and requested immediate implementation. The same confirmation accepted no synthetic prompt, no historical resend, no fresh replacement after a failed restore, authentication-required behavior when authentication is missing, and retention of an idle harness until the Computer stops or the Connector ends.
