---
title: "Agent Computer Owner and Operator Web Contracts"
created: 2026-11-02
tags: [agent, computer, frontend, authorization, inventory]
spec_type: domain
domain: computer
code_paths:
  - typescript/apps/web/src/features/computers
  - typescript/apps/web/src/app/agents
  - typescript/apps/web/src/shared/server/computers.ts
  - typescript/apps/admin-web/src/features/computers
  - typescript/apps/admin-web/src/app/computer-providers
  - typescript/apps/admin-web/src/app/computers
  - typescript/apps/admin-web/src/shared/server/computers.ts
  - typescript/apps/admin-web/src/shared/server/resourceInput.ts
  - typescript/apps/admin-web/src/shared/server/resourceInput.test.mts
  - typescript/packages/web-core/src/computer.ts
  - typescript/packages/web-core/src/computer-messages.ts
  - typescript/packages/web-core/src/rpc.ts
  - typescript/apps/e2e/src/computers.spec.ts
last_verified_at: 2026-11-03
spec_version: 1
---

# Agent Computer Owner and Operator Web Contracts

The General app exposes the private Agent list at
`/agents?team=<Team ID>`, a separate creation form at
`/agents/new?team=<Team ID>` and Computer detail at
`/agents/<Agent ID>?team=<Team ID>`. The list uses Agent-oriented headings and
empty copy. Agent names link to detail, and Computer power/operation observations
are subordinate information in one Computer-status column, not an overall Agent
health model. The list has no routine Refresh button; existing automatic observation
continues and a stale read exposes an explicit retry. Its Create Agent entry
navigates to the dedicated form instead of embedding it. Every Agent has a required
Team while retaining its individual owner. Account/team pages share the selected
context navigation. Admin exposes read-only Provider inventory at `/computer-providers`,
Provider Computers under the selected Provider path, and exact resource inventory
at `/computers/<Computer ID>`. There is no harness, conversation, file-management,
team-public Agent or concrete Mac enrollment UI.

## Server rendering and scoped data

Each route authenticates and resolves primary content on the server through the
existing request-local account session boundary. The server HTML renders that
record/list; the same snapshot and query selection seed client continuation.
Initial read failure belongs to the route boundary, not an empty inventory or a
fabricated loading state. The list reads owned Agent records and their Computer projections; Provider
choices belong to the separate creation page. That route verifies current Team
membership and server-renders the initial bounded Provider-choice snapshot with
the form. Subsequent Provider read failures preserve the entered draft and expose
an explicit retry without claiming there are no eligible Providers.

Listing and creation carry the required Team ID through the browser DTO,
Team-qualified query key and generated RPC request. Detail admission resolves
the actual owned record and rejects a mismatched URL Team. A Teamless entry
canonicalizes before rendering; list entry uses the validated recent/personal
context and detail entry derives its Team from the authoritative Agent. Pagination,
create-result navigation, force removal and normal disappearance retain that Team.
Membership, individual ownership and lifecycle fences are revalidated by the
backend; Team manager or installation Admin status does not widen private access.

Generated Node RPC clients use each app's explicit backend target. A missing
target fails configuration rather than inferring a local listener or another
app's target. No browser credential, owner token, endpoint or connection ID is
serialized into these views. Browser DTOs copy only approved fields and validate
required identities, enum meanings, timestamps and bounded pages. Timestamps
become JSON-safe milliseconds. Operator resources deliberately retain non-secret
namespace/name/original UID/execution identities; Agent names/user content and
bootstrap data are excluded.

Existing account Origin, POST unary tRPC, cookie/renewal, identity-cache clearing,
display and error boundaries remain authoritative. Cleared/unverifiable account
identity hides prior Agent, resource and identity content behind generic recovery
instead of resurrecting the SSR snapshot. This feature adds no browser
stream transport, cross-tab scheduler or persistent offline queue.

Typed invalid-session failures from every protected `computers.*` query/mutation
enter that identity-withdrawal policy. Admin permission loss on its read-only
Computer inventory does likewise; ordinary General business denials and transient
transport failures do not sign out the user.

## Truthful projections and actions

One backend-derived projection supplies desired intent, current phase, actual
power, last-known power, management connectivity, Computer Connector connectivity,
saved-environment evidence, observation time and available actions with typed
blocking reasons. The browser does not infer stopped power from disconnection or
query failure. Connector connectivity is not harness readiness. Display formatting
uses the user's region/timezone; English, Korean, Japanese and French copy share
the same typed key set.

Creation retains Team/name/Provider selection on rejection or unavailability and
navigates to the accepted Agent's detail. A pending/lost creation response retains
its exact Request ID and payload for explicit recovery; unresolved payload edits
are locked. An existing removed receipt is distinct from a newly created Agent.
Exact unresolved result recovery remains available even if fresh Provider choices
are absent or stale, using the original payload/Request ID rather than a new
eligibility decision. Its action is labeled as checking the creation result.
Provider choices use bounded pages, with explicit next/first reads rather than
unbounded background accumulation.

Start submits directly. Safe preparation retry, stop/restart, normal deletion and force removal use
accessible controlled confirmation. The confirmation captures its current
lifecycle Request ID when opened; polling cannot silently retarget that consent
to a newer request. Cancel sends no mutation. Every write still revalidates current
owner/session/expected request on the backend. A lost response is not rollback or
permission to replay an external effect.

Preparation retry is shown only when the backend's safe-retry projection permits
it. It calls the dedicated owner RetryPreparation contract with a fresh Request ID
and captured predecessor, retains the same Agent/Computer/execution/allocated
saved environment, and never exposes internal execution IDs to ordinary views.

Stop/restart warns that processes are interrupted and only saved files are
preserved. Normal deletion exposes the deleting intent until managed cleanup is
confirmed. Force removal warns that Computer/data may remain while independent
cleanup intent persists. Five-minute start delay comes from the backend's actual
dispatch interval, never a browser timer or reset on refresh. Confirmed normal
removal eventually yields owner not-found and navigation to the list; a failed
read alone does not remove an Agent.

## Bounded observation

While the view is visible and authenticated, query refresh is five seconds for
pending lifecycle projections and fifteen seconds for stable observations.
Resource detail/Provider choices use fifteen seconds. Hidden views pause interval
reads, and returning/focusing resumes observation. Query serialization/deduplication
bounds overlap; one list query replaces per-row browser calls. All page requests
use the RPC limit of twenty items. Primary page/selected detail/cursors live in
the URL; transient form edits and supplementary Provider-choice paging do not
become another authoritative lifecycle store.

Resource URL cursors normalize only recognized scalar kind/namespace/name fields.
Complete valid cursors preserve the selected page while unrelated decoration is
ignored. Partial, repeated or malformed cursor fields select the first page
instead of throwing before the route's required-data boundary. RPC input remains
strict independently of this browser URL normalization.

New reads are observations, not automatic retries of a failed mutation. They
consume persisted projections and do not initiate Kubernetes observation or
lifecycle effects. A browser read error retains last-known content with a warning;
unsafe detail controls and stale creation choices are disabled. Post-mutation
invalidation updates related queries without imperative per-hook refetch.

## Read-only operator inventory

Current installation-admin authority is required at SSR and every subsequent RPC.
Provider inventory remains independent of live Agent associations. It shows
the typed read-only creation capability independently from connectivity, and shows
management/Connector/power separately, terminal intent and unresolved versus
confirmed cleanup. Resource detail exposes exact reserved namespace/name/UID and
presence/observation without keys or user files. It provides no adoption, cleanup,
retry, force removal or other operator mutation control. Confirmed cleanup stays
visible under the backend's 24-hour retention; unresolved evidence does not expire
merely because it is old.

## Verification boundary

Web-core owns DTO validation/redaction, action mapping, intent/power separation,
resource identities and localization parity. App Storybooks and executable
component checks own meaningful copy/status/failure/mobile states. The minimum
assembled Computer E2E uses actual Control, Computer Provider and Computer
Connector processes, PostgreSQL, real web/RPC and synthetic HTTP beneath the
supported Kubernetes SDK. It verifies SSR, create/stop/start, one original PVC
identity, independent Connector connectivity, operator resource reads and confirmed
normal deletion. It does not create a real cluster, persist files on a real PVC,
prove CNI/TLS or replace live substrate release gates.
