---
title: "Web Page Rendering and State Patterns"
created: 2026-10-31
tags: [web, ssr, patterns, navigation]
---

# Web Page Patterns

Use the [Web conventions](../../../.claude/rules/web-conventions.md) for both
General and Admin. These patterns connect existing generated RPC and TanStack
Query infrastructure; they do not introduce a new page framework.

## 1. Request-local read scope

App adapters construct one React-request-cached `PageReadScope` from the app's
HttpOnly credential and generated Identity reader. `identity()` validates once;
`read(operation)` lends the credential only inside server operations. Neither
method renews a session or emits a cookie. Never serialize the scope or return a
credential from a page loader. `requiredPageRead` wraps initial failures for safe
route rendering while retaining the diagnostic source privately.

Load primary data explicitly in the page's feature-owned server loader. A missing
session redirects to the app login; absent primary records use the route's
not-found/error boundary. Keep supplementary requests outside this required-data
chain instead of building an all-page Promise.all dependency.

## 2. Resolved snapshot contract

Login methods, setup availability and proof previews are resolved before the
renderer. Give the renderer required typed props and keep mutation state in the
container. No initial client query or mounted guard is needed. Invalid/closed
proof or setup states remain business outcomes, not loading placeholders.

## 3. Interactive query handoff

Fetch initial primary data on the server and pass it to a Client Component that
actually renders it during SSR. Use `initialQuery(data, timestamp?)` only for the
matching query key/selection. It disables redundant mount/reconnect reads, not
explicit invalidation. Later client page selections may genuinely load or fail.
Use one server timestamp for time-dependent initial projections so hydration does
not change an expiry label merely because it ran milliseconds later.

The app's identity provider and account cache remain the shared authentication
boundary. Page loaders do not create another client identity store.

## 4. Typed URL selection

Define a `UrlCodec<T>` for a bounded selection. Both server `serverSearchValues`
and browser URLSearchParams implement `SearchValues`; `readSelection` therefore
uses the same parse/default policy, including first-value duplicate keys.
`selectionUrl` preserves the runtime mount, other parameters and hash.

`navigateSelection(..., "push")` updates Next's supported native history
integration, allowing `useSearchParams` to drive the client query without a full
reload. Use replace for minor state. Direct document entry still runs the server
loader for exactly that URL. Never copy selection into a competing useState.

Inventory offset is a canonical bounded decimal; login method is a bounded enum
further constrained by available methods. Confirmation dialogs and one-time issued
bearer links are transient mutation state, not URL detail selection.

## Verification

Check real primary DOM with application scripts blocked, then hydration and
client updates with scripts enabled. Verify direct URL entry, reload, Back and
Forward. Unit tests cover codecs, bounds and credential-scoped reads; component
checks cover section-local failures and copy. Do not duplicate every condition
through E2E or use serialized initial values as proof of rendered content.
