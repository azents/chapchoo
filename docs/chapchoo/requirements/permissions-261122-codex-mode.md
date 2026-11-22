---
title: "Codex Conversation Permission Mode Requirements"
created: 2026-11-22
implemented: 2026-11-22
document_role: primary
document_type: requirements
snapshot_id: permissions-261122
tags: [conversation, codex, security]
---

# Codex Conversation Permission Mode Requirements

- Snapshot: `permissions-261122`
- Document reference: `permissions-261122/REQ`

## Problem

Conversation owners can choose a Codex model but cannot see or choose how Codex handles permission requests. An unset permission choice needs a predictable default among the modes available to that conversation.

## Primary Context

An authorized conversation owner opens the Activity settings of a Codex conversation, finds its permission mode immediately below Model, and chooses how new work obtains permissions.

## Goals

- Place a Codex permission-mode selector directly below Model in the existing conversation settings.
- Select a predictable available default when there is no explicit choice.

## Non-Goals

- Adding equivalent modes for other harnesses, changing model settings, or altering the Computer's outer isolation and Chapchoo server authority.

## Requirements

### REQ-1. Select a conversation permission mode

- The Codex conversation shows a permission-mode control immediately below Model.
- The control identifies **Approve for me**, **Full Access**, and **Ask for approval** as distinct choices. It does not imply that automated review approves every request or that Full Access grants host or server privileges.
- Saving a choice does not send a message or change permissions of an already running turn. A subsequent eligible turn in that conversation uses the selected mode, including if the conversation has been resumed.

### REQ-2. Resolve an unset mode in the requested order

- If the conversation uses the default, pick the first eligible mode in this order: **Approve for me**, **Full Access**, **Ask for approval**.
- An explicitly selected and eligible mode is not replaced by the default ordering.
- A failure to read policy or confirm a change is not evidence that a broader mode is allowed.

## Fixed Constraints

- Existing account, Agent, Computer and conversation authority, Computer isolation, and server-tool publication checks remain in place.
- Full Access changes the Codex sandbox as well as approval behavior. No mode change can revoke work or effects already dispatched.

## Confirmation

The requester specified the placement and fallback order and explicitly excluded new operational policy or administrator settings. The same default order applies to unset choices; eligibility comes from the connected Codex, not a new Chapchoo policy.
