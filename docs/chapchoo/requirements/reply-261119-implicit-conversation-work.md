---
title: "Implicit Conversation Reply Work Requirements"
created: 2026-11-19
tags: [backend, frontend, engine]
document_role: primary
document_type: requirements
snapshot_id: reply-261119
---

# Implicit Conversation Reply Work Requirements

- Snapshot: `reply-261119`
- Document reference: `reply-261119/REQ`

## Problem

An Agent can stop at a normal turn ending without publishing its reply.
If it has not published a Work plan, the conversation has no pending reply
objective and cannot continue using its existing safeguards.

## Primary Context

### Primary Actor

The authenticated owner sending a message in a Chapchoo conversation.

### Primary Scenario

The owner sends a message. Work to reply begins without a progress summary.
The Agent sends its answer and completes that work directly, without
publishing a checklist first.

## Requirements

### REQ-1. Start one reply objective on user input

- A newly accepted user message starts reply Work if none is open. More
  messages join the same open Work without creating per-message goals or
  checklist entries. Existing user inputs keep their identities and order.
- A duplicate acceptance does not start or reopen Work. Input accepted
  after closure can start the next Work.
- A normal turn ending does not complete Work. Unfinished reply Work may
  use the existing bounded continuation and compaction context.

### REQ-2. Keep the implicit objective out of the progress summary

- A basic reply objective alone does not render a work summary. The
  existing summary shows progress only when the Agent explicitly supplies
  a titled task plan. Publishing that plan updates the same Work.

### REQ-3. Settle Work through existing actions

- A final reply completes the Work without requiring a prior plan. A
  question can pause the Work and a genuine new message resumes it.
  Explicit silent completion remains available.
- A terminal or waiting action must not dispose of a user message still
  pending delivery. A requested message is still published intentionally.

## Fixed Constraints

Preserve existing Stop, access, input delivery, Chat receipt and Full history
contracts, and the existing continuation safeguards. No separate failure
state, per-message task, new UI status, or new tool.

## Confirmation

The requester specified the default reply objective, one active objective
across multiple messages, and no implicit task bubble on 2026-11-19 (KST),
then directed implementation.
