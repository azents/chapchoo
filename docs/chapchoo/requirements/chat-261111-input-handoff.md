---
title: "Chat Input Handoff and Computer Status Requirements"
created: 2026-11-11
implemented: 2026-11-11
document_role: primary
document_type: requirements
snapshot_id: chat-261111
tags: [chat, lifecycle, web, computer]
---

# Chat Input Handoff and Computer Status Requirements

- Snapshot: `chat-261111`
- Document reference: `chat-261111/REQ`

## Problem

Delivered input records remain indefinitely and appear after the native conversation, making completed delivery look like pending work. Computer lifecycle and unavailable internal observations are conflated into an unconfirmed label, even when the applicable lifecycle or last known internal state is available.

## Primary Context

### Primary Actor

The private Agent owner using Chat.

### Primary Scenario

The owner sends a message, sees it pending until the Agent takes responsibility, then reads the actual session conversation while the surrounding Computer status truthfully reflects lifecycle progress and temporary observation gaps.

## Supporting Scenarios or Effects

- Input delivery, processing and transcript reflection remain separate lifetimes.
- Reentry and interrupted observation must not replay accepted work or invent a failed/stopped Computer.
- Restart may replace native resources without exposing replacement as a different user action.

## Goals

- Bound pending-input ownership to the pre-Agent handoff lifetime.
- Present pending and recorded user messages consistently without delivery captions.
- Preserve understandable known state during lifecycle changes and observation retry.

## Non-Goals

- Seamless same-message morphing or a new shared input/transcript key.
- Special duplicate-matching or suppression mechanisms as the solution.
- Automatic Computer restart, repeated model delivery, or fabricated native history.
- New model, authentication, provider, operator or configuration capabilities.
- Merge, deployment or operational infrastructure changes.

## Requirements

### REQ-1. Pending input presentation

A pending input uses the same visual language as a recorded user message, with a dimmed appearance matching the established input-buffer precedent. No lifecycle caption appears below either message.

**Acceptance criteria**

- Pending and recorded sample bubbles have equivalent typography, geometry and body presentation apart from dimming.
- Accepted, delivered and steered captions do not remain below the bubble.
- Mobile and desktop retain their surrounding Chat layout.

### REQ-2. Complete handoff lifetime

Server-owned pending input ends when the Agent takes responsibility for it. Its input-message record is then completely removed rather than hidden, archived as a delivered receipt or retained as a bodyless tombstone. The actual session conversation remains independently owned.

**Acceptance criteria**

- Successfully handed-off inputs no longer exist in the pending-input store or pending-input reads.
- Agent-side processing and native session capture can continue after removal.
- Handoff does not wait for the model response to complete.
- Failed or uncertain handoff is not assumed successful merely because a transport waiter ended.

### REQ-3. Natural transition and safe recovery

Pending and session representations follow their owners' ordinary lifetimes, minimizing the handoff-to-session-display gap without requiring exact visual continuity or message matching.

**Acceptance criteria**

- No permanent second delivery list remains after handoff.
- Input ownership is not resurrected or automatically replayed after a confirmed handoff.
- Active or uncertain work is not automatically resubmitted on reconnect, lost acknowledgements or source changes.
- Completed input records are not kept for indefinite exact-request recovery. After complete removal, an absent result does not authorize automatic resend and is not a retained success receipt.
- Do not introduce a replacement admission counter, capability ledger or permanent per-input deletion marker merely to retain the old completed-request recovery contract.
- No textual-similarity matching or manufactured transcript entry is used as identity.

### REQ-4. Known Computer lifecycle

Known off, starting, stopping and restarting states are displayed as such rather than unconfirmed. Native replacement is presented as restarting.

**Acceptance criteria**

- The owner sees the applicable Computer task/state during each transition.
- There is no separate user-facing replacement state.
- Computer lifecycle facts and internal execution observations remain independent.

### REQ-5. Internal observation refresh

When the Computer has been powered on but its internal state cannot currently be observed and observation is being retried, keep the last known state and add only a loading spinner. Do not replace it with checking-status or unconfirmed prose.

**Acceptance criteria**

- A known previous internal state remains visible while refreshing, accompanied by a spinner.
- A missing or delayed observation alone does not become a confirmed runtime failure or stopped power.
- Explicitly reported startup/operation failure remains distinct from observation loading.
- Observation recovery does not itself power-cycle the Computer or replay user input.

## Fixed Constraints

- Preserve private owner/Team/source/execution admission, external effect ownership and uncertain-delivery no-replay boundaries.
- Preserve actual session history and saved environment across ordinary reconnect/restart.
- Do not add legacy fallbacks, protocol versioning or hidden retained-input archives.
- Existing normal quality gates and representative assembled-product checks remain required.

## Open Assumptions

- A display before any last-known internal state exists must not invent one; its smallest truthful representation will be identified during feasibility analysis.
- Exact Agent pickup acknowledgement and recovery mechanisms require current-system analysis; product intent does not equate transport enqueue with ownership transfer.

## Confirmation

The requester explicitly confirmed the behavior above through the Chat and Computer-status discussion and corrected component previews on 2026-11-11, then instructed implementation of both Chat and status on 2026-11-11 at 17:19 KST. The requester subsequently rejected adding a replacement per-Agent admission counter at 17:27 KST and instructed complete removal instead. This explicitly ends permanent recovery of a completed input's old receipt; active/uncertain effect protection and absence-not-resend remain required. This records the confirmed requirements without treating visual approval as approval of an unexamined technical mechanism.
