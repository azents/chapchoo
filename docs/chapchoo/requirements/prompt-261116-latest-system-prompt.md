---
title: "Latest System Prompt Page Requirements"
document_role: primary
document_type: requirements
snapshot_id: prompt-261116
created: 2026-11-16
implemented: 2026-11-16
tags: [agent, instructions, frontend, persistence]
---

# Latest System Prompt Page Requirements

- Snapshot: `prompt-261116`
- Document reference: `prompt-261116/REQ`

## Problem

The activity panel exposes prompt content through an inline expandable historical
browser. Its terminology is difficult to understand, and storing repeated prompt
payloads for every preparation unnecessarily accumulates content. Owners need a
straightforward page for the latest system prompt instead.

## Primary Context

### Primary Actor

The current private Agent owner with current Team membership.

### Primary Scenario

The owner opens System prompt from the Agent conversation, navigates to a
separate page, reads the latest system-prompt content, and returns to the
conversation.

## Supporting Scenarios or Effects

- Existing installations stop accumulating historical prompt payloads and discard
  previously accumulated older prompt content when upgraded.
- Saving owner-authored instructions and applying them to the continuing native
  conversation retain their existing behavior.

## Goals

- Use understandable system-prompt terminology throughout the affected interface.
- Make reading the system prompt a real page-navigation action.
- Keep only the latest system-prompt content for each Agent.

## Non-Goals

- Prompt history, historical execution selection, pagination, restore or rollback.
- Changes to owner editing, private ownership, default-project management,
  native-context continuity or the input-delivery policy.
- Operational database edits, rollout or deployment performed by this change.

## Requirements

### REQ-1. Understandable terminology

The affected interface consistently calls the capability System prompt.

**Acceptance criteria**

- The Korean entry and page title are `시스템 프롬프트`.
- Labels, descriptions, status, empty-state and error messages use system-prompt
  terminology in every supported locale.
- The affected interface never calls prompt content `출처` or uses the equivalent
  sources/provenance terminology in another locale.

### REQ-2. Separate page navigation

Opening System prompt changes the primary page rather than expanding content
inside the conversation activity panel.

**Acceptance criteria**

- The activity entry navigates to a separately addressable system-prompt page.
- Direct navigation, reload, browser Back and return-to-conversation navigation
  retain the correct Agent and Team scope.
- The page presents the latest content, or an accurate absent/read-failed state,
  without a historical-execution selector or earlier-content controls.
- Content remains read-only here; existing owner instruction editing is preserved.

### REQ-3. Latest content only

Retain only the latest selected system-prompt content for each Agent's current
conversation, independently of the latest saved owner-authored setting.

**Acceptance criteria**

- Repeated preparations and changes replace the retained prompt content instead
  of accumulating a prompt-content history.
- Older system-prompt payloads become unavailable and are removed from persistent
  storage, including already accumulated payloads during upgrade.
- The latest saved owner setting is preserved.
- Latest selected content is not presented as adopted merely because it was
  saved or selected. A missing preparation or failed read is not an empty
  successful result and does not reveal an earlier prompt as the current one.

### REQ-4. Preserve continuing execution and private access

The storage and page correction preserve existing instruction-delivery and
private-owner behavior.

**Acceptance criteria**

- Only the current private owner with current Team membership may read content.
- Save remains the sole submission action; empty text clears owner content and
  failed saving preserves the draft.
- Refresh preserves the exact native conversation and started effects.
- A pending, failed or obsolete preparation cannot authorize input forwarding
  under an older prompt. Uncertain delivery never authorizes automatic replay.
- Instruction or tool content does not expose operational credentials.

## Fixed Constraints

- Existing historical development snapshots remain immutable; current-system
  documentation must be updated with the implementation.
- Implement the correction through a focused pull request with preserving schema
  migration and automated verification for both supported database engines.
- Latest prompt content and current saved owner text are distinct existing
  concepts; preserving both does not introduce user-visible prompt versions.

## Open Assumptions

- None.

## Confirmation

Confirmed by the requester on 2026-11-16. The requester reiterated that the
direction was already decided and explicitly instructed immediate implementation
of the presented scope.
