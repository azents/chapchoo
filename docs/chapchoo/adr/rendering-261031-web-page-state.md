---
title: "Server Rendering and Page State Authority"
created: 2026-10-31
tags: [web, ssr, navigation, refactoring]
document_role: primary
document_type: adr
snapshot_id: rendering-261031
---

# Rendering Authority

Authority: [rendering-261031/REQ](../requirements/rendering-261031-web-page-state.md).

No unresolved material technical choice is introduced. The requested Web
principles fix server primary-data ownership, actual first-content rendering,
client continuation and URL selection. Existing generated RPC clients, TanStack
queries and independent app adapters are retained. SSR remains read-only and uses
request-local credentials; no new session mode, persistence or retry is added.

Main content: General home contexts; selected team details and membership;
Admin grant inventory at the selected offset; public methods/proof/setup state.
Navigation contexts, received invitations, and management invitation inventories
are supplementary to an already selected team and fail within their own sections.
Mutation confirmation and newly issued proof dialogs are transient operations,
not navigable detail records. These classifications follow the current UI and
the approved primary-content definition, rather than adding product features.

## Pending Display Policy

During verification, server and browser local zones produced different inventory
date strings and a real hydration mismatch. The requester owns the choice of
common explicit display zone versus browser-local policy. No decision is accepted
yet; this does not reopen the fixed SSR/URL ownership principles.

## ADR-D1. Explicit UTC Inventory Display

Accepted by the requester on 2026-10-31 after the pending display-policy question:
use UTC for issuance/expiry dates in this refactor, identify it in the rendered
value, and defer a complete user-timezone specification. Both runtimes format
the same instant with `timeZone: "UTC"` and the selected locale. This resolves
the mismatch without client-only content, warning suppression or a new cookie.
