---
title: "Agent chat surface requirements"
created: 2026-11-05
updated: 2026-11-05
implemented: 2026-11-05
tags: [agent, chat, web]
document_role: primary
document_type: requirements
snapshot_id: chat-261105
---

# Agent chat surface requirements

- Snapshot: `chat-261105`
- Document reference: `chat-261105/REQ`

## Problem

The approved conversation study is separate from the usable product. Agent owners
need its conversation-first surface in the application while execution is built
in follow-up work.

## Primary Actor

An authenticated owner of an Agent in their selected personal or collaborative Team.

## Primary Scenario

Open an Agent from its list or after creation, use the approved conversation
surface, inspect activity, and reach existing Agent and Computer controls.

## Goals

Promote the approved responsive visual composition and reuse implemented product capabilities.

## Non-Goals

Harness execution, model calls, chat transport, durable messages, uploads and deployment.

## Requirements

### REQ-1. Approved conversation composition

Retain floating character/status and input, readable conversation content, activity
sheet, sidebar and mobile navigation, per-Agent palette, and shared mascot motion.

**Acceptance criteria**

- The status opens activity on the first activation and remains expanded while scrolling.
- Same-sender messages have 16px spacing; sender changes have 32px spacing.
- Profile, status and hamburger share the approved translucent glass treatment.
- Input clearance, fades and latest-follow preserve readable content on desktop and mobile.
- No additional mock labels, explanatory banners or demo settings are introduced.

### REQ-2. Existing functionality remains real

Connect existing account, workspace, Agent identity, detail and Computer behavior.

**Acceptance criteria**

- Agent list and successful creation enter chat; character links to existing detail.
- Saved identity/artwork/palette and Team/owner admission are preserved.
- Computer status, action eligibility, consent and refresh use existing product authority.
- Read failure or disconnection is not represented as confirmed stopped power.
- Authentication withdrawal removes protected conversation content.

### REQ-3. Local conversation interaction

Unavailable conversation functionality remains local UI for this delivery.

**Acceptance criteria**

- Sending appends local user messages without invoking execution or claiming a model response.
- Each Agent has its conversation surface; the sidebar contains Agents, not conversation history.
- Unavailable Computer state leaves history readable and input read-only.
- Supported English, Korean, Japanese and French presentation remains available.

## Fixed Constraints

Existing private-Agent isolation and server rendering remain authoritative. No new
backend contracts, credentials, persistence, deployment or scroll-dependent status sizing.

## Open Assumptions

The existing detail page continues to own profile editing and destructive administration.

## Confirmation

The requester explicitly approved promoting the selected mock, connecting existing
capabilities and retaining unavailable functionality locally on 2026-11-05. The
same request authorizes PR delivery and merge; follow-up correction excludes extra
mock disclosures. Scroll-dependent status sizing was explicitly removed. The
requester subsequently required the study's Agent sidebar and excluded conversation
lists on the same day before implementation was verified.
