---
title: "Consistent Team and Administration Navigation Design"
created: 2026-11-03
document_role: primary
document_type: design
snapshot_id: shell-261103
tags: [frontend, navigation, authentication, architecture]
---

# Consistent Team and Administration Navigation Design

- Snapshot: `shell-261103`
- Requirements: [shell-261103/REQ](../requirements/shell-261103-team-navigation.md)
- Decisions: [shell-261103/ADR](../adr/shell-261103-team-navigation.md)
- This draft records fixed implementation boundaries and current evidence.
  ADR-D1 is resolved by the requester's Admin-only API correction; final
  verification and the complete authority audit remain in progress.

## Current Behavior and Gaps

General and Admin are independent web processes and protected RPC service unions.
TeamManagement is registered only on the Client union. Combined development
fixtures expose both unions on one listener and therefore cannot prove a
split-listener topology. Existing account sessions have no app audience, while
web cookie policy previously separated General and Admin browser credentials.
Per-feature navigation supplies different subsets on different routes. The root
General page combines profile, contexts, invitations and team creation.

The requested shell separates workspace selection, features and personal account
controls. It also corrects the prior missing Agent-to-Team association without
turning private Agents into Team-public resources.

## Design Authority

- Design revision: `2`

| ID  | Material mechanism                                                                                                  | Authority                                                                      | Classification |
| --- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | -------------- |
| M1  | Required Agent Team association plus independent individual ownership                                               | REQ-1                                                                          | required       |
| M2  | Preserving owner-personal-Team migration and historical creation context                                            | REQ-1; existing explicit preparation and creation receipts                     | derived        |
| M3  | Departure/removal/Team deletion accepts existing terminal Computer cleanup and fences admission                     | REQ-1; existing asynchronous cleanup gates                                     | derived        |
| M4  | Shared selector and stable feature navigation; left mobile Drawer; existing mascot                                  | REQ-2, REQ-3, REQ-6                                                            | required       |
| M5  | Upper-right account Popover/right mobile Drawer reuses account/device preference controller                         | REQ-4; existing display persistence and recovery                               | derived        |
| M6  | Creation modal selects a Team only after confirmed ordinary creation                                                | REQ-5; existing non-replayed Team creation boundary                            | derived        |
| M7  | App entry validates remembered context against current identity/membership; URL selects working content             | REQ-1, REQ-2, REQ-7; web URL-authority convention                              | derived        |
| M8  | Same-host root session cookie, request-owned rotation, independent current Admin admission                          | REQ-8; existing shared backend session authority and General root-cookie scope | derived        |
| M9  | Early subset discovery and optional font replacement with readable fallback                                         | REQ-9                                                                          | required       |
| M10 | Remove old hub and its duplicated grouping boundary                                                                 | REQ-7, REQ-10                                                                  | required       |
| M11 | Admin-only workspace read/create with current-admin and membership checks; public peer web navigation metadata only | REQ-4, REQ-5, REQ-6, REQ-8; ADR-D1 resolution                                  | decided        |

## Admin API Boundary

The requester explicitly requires Admin web to communicate only with the Admin
API (ADR-D1 resolution). `AdminWorkspaces.ListTeams/CreateTeam` is a narrow
Admin-only contract reusing existing Team messages and storage operations. Each
operation verifies current administrator status under the same authenticated User
lock as its Team read/create. Results contain only that user's accessible Teams;
creation creates ordinary owner membership. No Team-management mutation surface,
private-Agent read or privilege over other users is added. General retains its
existing TeamManagement contract.

Admin Node uses its sole configured Admin backend. Public General/Admin web URLs
are validated runtime navigation destinations, forwarded by the supervisor and
Helm, never compiled or inferred from an API host/path. They carry no credentials.
Choosing a Team from Admin performs browser navigation to the General Team URL,
whose ordinary SSR/API admission runs independently. Standalone surfaces omit an
unconfigured peer destination rather than inventing one. Split-listener tests
prove this narrow contract and the continued Client/Admin business boundary.

## Agent Data, Admission and Deletion

`Agent.team_id`, `CreateAgentRequest.team_id` and `AgentListRequest.team_id` are
required. Agent listing carries a nested bounded Page and filters both current
Team and authenticated owner. Detail and lifecycle operations resolve the owned
Agent's actual Team and validate current membership. Team ownership/manager/Admin
status never grants another user's private Agent access.

Migration 5 backfills Agents and creation receipts through each owner's existing
personal-context mapping. A missing mapping fails atomically. Computer IDs,
executions, storage and Agent identities remain unchanged. Agent-to-Team deletion
is non-cascading. Receipt Team IDs are historical request context without a live
Team FK; replay compares that original context as well as the existing canonical
creation fingerprint. The down migration rejects erasing required scope.

Member leave/kick serializes on the Team, acquires sorted Providers and then
Computers, records deleting intent and revokes Connector authority before removing
membership. Team deletion records the same intent for all its cloud Agents and
marks the collaborative Team pending deletion. Pending Teams cannot be selected,
read, invited into or used for Agent admission. The marker is internal cleanup
bookkeeping, not a selectable archive or restore feature.

Provider reconciliation does not depend on continued owner membership. It retains
all existing dispatch fences, allocated-UID protection, absence evidence and
cleanup-completion requirements. Only after the final Agent FK is removed does
post-commit Team collection physically remove the Team and memberships. Empty
Teams are collected immediately. Team collection must not acquire Team locks
while holding Provider/Computer locks. Unresolved external resources have no TTL.
Local enrollment/execution remains unimplemented; this work adds no Local lifecycle.

## Web Data and Navigation Ownership

Primary data remains request-authenticated and server-rendered. Query handoff uses
the same initial data, identity, Team and pagination selection. Subsequent live
queries use Team-qualified cache keys. A mismatched detail/URL Team is not-found,
not a silent switch to another context.

The Agent list/detail URL includes `team`. An omitted Team is canonicalized to an
accessible context before rendering; the list uses the personal Team when no
validated remembered selection exists, and a detail can derive its Team only from
the authoritative owned record. Feature links and pagination retain context.
Selecting different primary content creates meaningful browser history.

The recent workspace value is only a browser preference. It must be bound to the
current identity and checked against the current accessible Team list on entry.
It never grants access, changes Agent ownership or becomes a global account
property. Missing/stale preference uses the existing personal context. Admin is a
separate destination and must never be stored or created as a Team.

The shell owns selection presentation and navigation; feature containers own
business state and mutations. Shared UI accepts props/callbacks, not RPC clients
or credentials. Selection and successful Team creation navigate only after the
corresponding authoritative outcome. Failed or uncertain creation retains truthful
feedback and does not automatically submit again.

The obsolete hub's invitation acceptance moves to a separately reachable
invitation destination. Team role/member/ownership settings remain reachable.
Profile and account display settings remain separate from workspace membership.

## Account Chrome and Authentication

The common account entry displays a user icon for an authenticated identity and
sliders for a visitor. Desktop uses a bounded Popover; narrow screens use a right
Drawer. Mini-profile, appearance, language/time-zone/region, profile and sign-out
remain in one entry. The existing display controller retains account/device
scope, async save, retry, unavailable state and proposal handling. A save failure
is not converted to a success message by shell navigation.

The app-provided header controls own account chrome, avoiding duplicated page
logout/identity rows. Drawers and Popovers must use their controlled change
callbacks, support Escape/outside dismissal and restore focus. Navigation and
account modal layers must not compete.

General and Admin use the existing General root cookie identity and HttpOnly,
Secure/SameSite policy. No credential enters React props, local storage, browser
events or logs. A committed renewal is published before Admin permission is
checked; denial blocks dispatch but does not erase a valid General credential.
Actual invalid-session responses still clear it. Read-only SSR does not rotate.
Focus/reconnect identity refresh detects a changed account and resets client
navigation instead of retaining another identity's cached primary data.
Exact-Origin POST checks and current Admin server authorization remain required.
This is not cross-installation federation or support for unrelated installations
sharing one browser-cookie scope.

## Fonts and Layout

Preload the existing core Pretendard WOFF2 subset from the caller's runtime mount
and use `font-display: optional` for all owned text subsets and the lazy mono face.
Slow cold views retain readable fallback rather than swapping metrics late; fast
and warm views retain Pretendard. Do not hide the page or await font readiness
before taking the cold measurement. Preserve the font license and text scaling.
Desktop sidebar/content geometry stays independent. Mobile navigation is hidden
until the left Drawer is requested; the existing 72px retriever remains available
without occupying the resting content area.

## Removal and Replacement

| Existing unit or behavior                          | Removal authority    | Replacement/remaining authority                             | Boundary                                                | Verification                                                           |
| -------------------------------------------------- | -------------------- | ----------------------------------------------------------- | ------------------------------------------------------- | ---------------------------------------------------------------------- |
| Teamless Agent create/list contract and callers    | REQ-1                | Required Team and private owner                             | Proto, generation, storage, service, Node/SSR, fixtures | Cross-Team and private-owner tests; generation/typecheck               |
| Team hard deletion while Agents still reference it | REQ-1                | Terminal cleanup then physical Team purge                   | Membership/Team transactions and cleanup callbacks      | Both-engine lifecycle, FK and concurrency tests                        |
| App-isolated Admin session cookie policy           | REQ-8                | Existing General root-cookie authority plus Admin admission | Shared web session adapter and Admin auth paths         | Renewal/denial and real two-app logout checks                          |
| Route-specific feature menu subsets                | REQ-2, REQ-6         | One stable context menu per surface                         | General/Admin features and shared chrome                | List/detail/settings/profile route checks                              |
| Old contexts/profile/create-team hub               | REQ-5, REQ-7, REQ-10 | Selector modal, Profile and invitations destination         | Root entry and obsolete hub sections/fixtures           | Root resumes validated context; invitation/team flows remain reachable |
| Separate header display/appearance/account entries | REQ-4                | Unified account entry and existing controller               | Shared header and both app providers                    | Component composition and production UI checks                         |
| Late text-face metric replacement                  | REQ-9                | Existing assets with early discovery/optional display       | FontFaces and licensed subset CSS                       | Cold delayed-font measurements                                         |

## Remaining Feasibility Checks

The pre-existing Team deletion released its handle immediately. Internal cleanup
rows must not change that public behavior into an indefinite handle reservation.
Migration 5 therefore enforces uniqueness only among live Teams. Reusing a handle
creates a distinct Team identity and never adopts old Agent cleanup or receipts.
The SQLite dependency-group rebuild preserves all Team, membership, personal
mapping and invitation rows with foreign keys enabled; PostgreSQL uses a partial
unique index. Both-engine preservation/reuse and PostgreSQL predicate evidence
close this feasibility check without a new deletion or retention policy.

Native physical tab activation is not established by Playwright's default focus
emulation. The production authentication experiment separately verifies explicit
focus-event delivery through real identity RPC and navigation; it must not be
reported as physical-device or operating-system focus verification.

## Test Strategy

Keep representative E2E flows minimal: one private Team-qualified Agent lifecycle
and independent Admin inventory journey; one context switch/create/membership flow;
and General/Admin identity continuity with current authorization. Use unit and
both-engine integration tests for ownership combinations, invalid scopes, receipt
conflict, rollback and cleanup races rather than multiplying E2E cases.

Use the existing disposable Assembly for actual source-built production web/Rust
execution and ordinary setup/signup APIs. It owns PostgreSQL and exact child
process groups; no production credentials, state copies or direct DB fixture writes
belong in browser tests. Missing prerequisites fail rather than silently skip.
Component checks own responsive rendering, keyboard/focus, overflow and copy.
Visual acceptance uses actual app components/theme/fonts at explicit viewport,
locale, appearance and DPR, not an inert preview patch or Storybook screenshot as
proof of production integration. Credential values and proof-bearing URLs never
belong in evidence.

Local verification: workspace Rust Clippy and 114 tests passed, including
SQLite/PostgreSQL migration preservation, atomic rollback, private Team scope,
departure cleanup, concurrent creation/deletion and PostgreSQL FK catalog proof.
The font experiment passed four delayed-font conditions. Web-core authentication,
DTO and navigation checks passed 66 tests; Admin unit checks passed 19; General
unit checks passed 13. Shared UI browser composition checks passed 17 cases.
Four assembled browser journeys passed with separate Client+Control and Admin
API listeners, including Admin Team creation, browser navigation to General and
Administration return, signup/membership/reset/shared-logout and the real
Computer component/PVC lifecycle over the supported-client synthetic fixture.
Helm lint and 15 offline chart tests passed, retaining the peer-backend exclusion.

Actual production screenshots were inspected at 1440×1000 and 390×844, DPR 1,
en-US/light and ko-KR/dark, with settled owned Pretendard faces and no document
overflow. The production experiment verified Admin selection/logo navigation,
Escape focus, both mobile Drawers and persistence of saved language/appearance
through reload and General/Admin navigation. The earlier seven-case auth
experiment remains explicit focus-event integration, not physical OS-tab proof.
An independent read-only review of the complete implementation found no required
correctness/security/data-integrity fixes. Exact-head remote CI remains pending;
these local results are not physical Safari, deployment or live-cluster evidence.

## Design Approval

- Mode: Collaborative.
- Decision owner: requester; ADR-D1 resolved by the explicit Admin API boundary.
- The requester authorized implementation of the selected visual behavior and
  mandatory Team/deletion corrections. This document does not reinterpret that
  as approval of an undisclosed listener/configuration change.
- Revision 2 mechanisms M1–M10 reflect the authorized visual implementation and
  Team/deletion corrections; M11 reflects the subsequent explicit Admin-only
  API instruction. No new topology choice remains open. Final authority and
  feasibility evidence is still being consolidated; this draft does not claim
  separately obtained complete Design approval or verified implementation.
