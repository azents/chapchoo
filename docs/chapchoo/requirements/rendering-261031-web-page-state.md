---
title: "Server-Rendered Web Content and Restorable Page State Requirements"
created: 2026-10-31
implemented: 2026-10-31
tags: [web, ssr, navigation, refactoring]
document_role: primary
document_type: requirements
snapshot_id: rendering-261031
---

# Server-Rendered Web Content and Restorable Page State

- Snapshot: `rendering-261031`

## Problem

Protected pages currently receive server authentication but load primary business
content only in the browser. Snapshot authentication forms retain redundant query
loading states. Admin pagination and login-method selection are lost on reload.

## Primary System Outcome

Existing General and Admin pages comply with the shared Web rendering and
navigation principles without changing their business permissions or workflows.

## Requirements

### REQ-1. Render primary content on the server

The first server-rendered document contains the actual primary content for the
selected route. Passing initial serialized values without rendering is insufficient.
Initial primary-data failure reaches a safe page-level error or not-found boundary.

### REQ-2. Respect data and update ownership

Snapshot-like renderers receive required resolved values without artificial initial
loading states. Interactive content server-renders initial values and hydrates the
same state before client-owned updates. Supplementary loading or failure does not
invalidate otherwise usable primary content.

### REQ-3. Restore user-selected views

Existing route selection, pagination and login-method selection restore from URL
on direct entry, reload and Back/Forward. Primary-content selections push history;
minor URL adjustments replace. Draft forms, issued bearer links, mutation outcomes,
authentication challenges and mutation confirmations remain transient.

### REQ-4. Preserve operational boundaries

General/Admin identity, independent cookies, access control, read-only SSR,
no automatic mutation replay, runtime prefixes, localization and approved visual
geometry remain intact. No credentials or issued bearer proofs enter URL state.

### REQ-5. Make the principles reusable

The refactor establishes shared patterns and abstractions for server-only reads,
resolved renderer contracts, interactive initial-state handoff and URL-owned
selection. General and Admin use these patterns, rather than separate copies of
the policy. Primary versus supplementary ownership remains explicit at the page.

## Non-Goals

New product pages, filters or detail features; backend protocol or persistence
changes; new client frameworks; merge, deployment; proof of physical Safari
refresh white-frame resolution.

## Confirmation

The requester approved these principles and explicitly requested refactoring the
current code to comply on 2026-10-31. This snapshot records that instruction;
it does not introduce a separate approval gate for the approved behavior.
The requester additionally required reusable patterns and abstractions on the
same date.

### Date Display Scope

On 2026-10-31 the requester selected UTC for current Admin issuance/expiry
display. The UI identifies UTC explicitly and uses it in server and client
rendering. A complete user-timezone policy is separate follow-up work; no
preference cookie or browser-local detection policy is introduced here.
