---
title: "Server-Rendered Web Page Refactor"
created: 2026-10-31
implemented: 2026-10-31
tags: [web, ssr, navigation, refactoring]
document_role: primary
document_type: design
snapshot_id: rendering-261031
---

# Web Page Refactor

Authority: [Requirements](../requirements/rendering-261031-web-page-state.md)
and [authority record](../adr/rendering-261031-web-page-state.md).

## Design Authority

Revision: `3`.

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Read-only server loading and rendering of primary page data | REQ-1, REQ-4 | required |
| M2 | Resolved snapshot props; initial-data queries for interactive content | REQ-2 | required |
| M3 | URL-selected pagination and authentication method, push navigation | REQ-3 | required |
| M4 | Independent supplementary section state | REQ-2 | required |
| M5 | Independent cookie/session authority and finite no-retry RPCs | REQ-4, account-web Spec | existing |
| M6 | Shared read-only read scope, initial-query policy and typed URL codecs/navigation | REQ-5, M1–M3 | required |
| M7 | Explicit UTC issuance/expiry formatting on server and client | ADR-D1, Requirements Date Display Scope | decided |

## Ownership and Interfaces

App server-only loaders use existing generated RPC methods and validators with
request-local credential access after validated identity admission. They never
invoke writable RequestSession renewal. Safe rendering failures retain sanitized
diagnostics. Required absence/invalid session is handled before feature rendering.
Public methods/proof/setup data is resolved once and passed to renderers.
Workspace and Admin inventory initial values seed matching client queries with no
mount/reconnect repeat. Later invalidation stays client-owned.

Admin offset uses the existing canonical decimal/int64 domain with fixed page
size 25. URL selection is parsed for SSR and read by client continuation. Login
method uses a bounded password/code value. Form/challenge state remains local.
General selected team and Admin inventory kind retain their current pathnames.

Shared web-core units own read-only credential-scoped operations, consistent
initial-query options and typed URL selection codecs. App adapters own framework
headers, authenticated admission and navigation integration. Page loaders name
their primary data explicitly; shared abstractions do not aggregate unrelated
requests or persist an additional copy of query/URL state.

## Removal and Replacement

| Existing unit | Removal authority | Replacement | Verification |
| --- | --- | --- | --- |
| Client-only first business reads | REQ-1 | Server initial data + hydrated queries | script-blocked document checks |
| Snapshot method/setup/proof queries and artificial loading states | REQ-2 | Required resolved renderer props | unit/component and rendered HTML checks |
| Local pagination/method selection | REQ-3 | URL selection | direct entry, reload, Back/Forward |
| Whole-workspace supplementary error aggregation | REQ-2 | Section-local feedback | failed supplementary read check |

## Test Strategy

Extend the existing representative General/Admin browser journeys rather than
adding a combinatorial E2E matrix. Verify actual content while application scripts
are blocked and hydration continuity with scripts enabled. Preserve existing
signup/team/reset and session-isolation journeys. Unit tests own URL parsing,
range bounds and read-only orchestration. App component checks own exact copy,
section failure, loading and transient confirmation states. Existing testenv
Assembly supplies isolated users/backend/listeners; do not use external fixtures
or real credentials. Run normal format/lint/typecheck/build, all unit suites,
Storybooks, components and E2E before PR; exact-head CI must complete. Missing
required prerequisites fail, never silently skip. Preserve safe summary logs.

## Feasibility and Risks

Generated read methods, validators and query initialData already exist. Runtime
mount handling is established. No schema/protocol change or credentials in props.
Concurrent browser cookie response-order limits remain unchanged. Visual geometry
must remain stable; incidental supplementary feedback is localized to its section.
Admin inventory dates use explicit UTC with a visible UTC suffix in both runtimes.
The requester deferred a full user-timezone policy; no preference cookie or
hydration warning suppression is introduced.

## Design Approval

Mode: direct implementation under the requester's approved Web principles and
2026-10-31 instructions to refactor current code and establish reusable patterns.
The requester additionally selected UTC inventory date display on 2026-10-31.
Revision `3`, exact authority set `M1`–`M7`. New material gaps return to the requester.
