---
title: "Agent chat surface boundaries"
created: 2026-11-05
tags: [agent, chat, web]
document_role: primary
document_type: adr
snapshot_id: chat-261105
---

# Agent chat surface boundaries

Requirements: [chat-261105/REQ](../requirements/chat-261105-product-surface.md).

## Decision map

The requester has fixed the delivery boundary: promote the approved UI, connect
existing functionality and retain unavailable chat capabilities locally. Current
web conventions fix SSR admission, URL-selected Agent identity, pure renderers and
query invalidation. No remaining material alternative needs a new architecture
decision in this snapshot. File layout, local fixture composition and reusable
controller composition are implementation details.

## Fixed boundaries

- Reuse the existing Agent detail loader and Computer action controller. Do not
  introduce another authority for ownership, lifecycle or request-bound consent.
- Local transcript/activity is component-lifetime state, not an execution service,
  transport, credential store or durable history.
- Add a handle-aware chat destination while retaining the existing detail route.
  The Agent is the URL-selected conversation context; there is no conversation list.
- Existing profile administration remains on detail. No backend or migration change.
