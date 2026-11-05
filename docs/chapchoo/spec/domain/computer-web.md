---
title: "Computer owner and operator web"
---

# Computer owner and operator web

Server-rendered owner controls and redacted administrator inventory project backend authority rather than creating browser lifecycle state.

# Agent Computer Owner and Operator Web Contracts

The General app uses [canonical Team-qualified routes](workspace-navigation.md#canonical-routes-and-admission)
for the private Agent list, separate creation form and Agent/Computer detail.
The list uses Agent-oriented headings and
empty copy. Profile image/name lead each saved-profile card; handle and Computer
power/operation observations are subordinate, not an overall Agent health model.
Cards use large bundled artwork, the saved name/handle and an explicit Open
affordance. Power/operation and Connector facts remain small, independent
observations beneath the identity, not interchangeable readiness signals.
Two columns are used when the content has room; the grid collapses before a
desktop sidebar crowds the cards, and mobile uses one column with contained
long names.
The list has no routine Refresh button; existing automatic observation
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

Listing and creation carry the resolved stable Team ID through the browser DTO,
Team-qualified query key and generated RPC request. Detail admission resolves
the owned record by exact resolved Team and Agent handle. No Teamless or
ID-query content entry is retained. Pagination,
create-result navigation, force removal and normal disappearance retain that Team.
Membership, individual ownership and lifecycle fences are revalidated by the
backend; Team manager or installation Admin status does not widen private access.

Generated Node RPC clients use each app's explicit backend target. A missing
target fails configuration rather than inferring a local listener or another
app's target. No browser credential, owner token, endpoint or connection ID is
serialized into these views. Browser DTOs copy only approved fields and validate
required identities, enum meanings, timestamps and bounded pages. Timestamps
become JSON-safe milliseconds. Operator resources deliberately retain non-secret
native scope/name/original identity/execution values; Agent names/user content
and bootstrap data are excluded. Kubernetes retains namespace and UID. Docker
retains opaque daemon scope, full container ID and optional original volume
creation identity without synthesizing Kubernetes UIDs. Volume labels/options,
daemon endpoints, credentials and bootstrap content are not browser DTO fields.

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

Creation presents final handle, catalogue image and six-key palette selection
alongside Team/name/Provider. Its authenticated SSR snapshot independently draws
the initial image and palette once and hydrates those exact values. Bounded
automatic handle candidates preserve manual edits and cancel stale resolutions;
the backend still owns final uniqueness. See [profile contracts](agent-profile.md).
Desktop places a portrait-oriented preview beside the image grid and fields;
mobile stacks a horizontal preview above the compact image choices. Named
palette swatches and an explicit random-color action change only the unlocked
palette draft. The creation footer retains its primary action below the complete
form; Provider eligibility and all frozen-request rules still govern admission.

Creation retains its complete draft on rejection or unavailability and
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

Computer power and Connector connectivity share one compact summary below the
profile identity, with current operation shown separately when incomplete. One
primary action changes between Start for stopped power and Stop otherwise;
unknown power, stale reads, pending submission and backend eligibility still
disable unsafe actions. The inapplicable opposite action and its ordinary
requires-running/stopped explanation are not duplicated. Restart and details use
named icon controls. Safe preparation retry retains its separate admission.

Details opens a controlled modal with observed power/time, last-known power when
current power is unknown, independent Computer and management connections, saved
storage and the latest operation. Short explanations distinguish each observation
without implying harness readiness, file inspection or backup verification.
Requested intent is in an optional troubleshooting disclosure. Closing or Escape
returns focus; explicit refresh invalidates the same observations. No new query,
poller or mutation is created by opening details. Deletion and force removal sit in a
separate destructive-action area, retaining the same backend eligibility and
consent fences.

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

Resource URL cursors normalize only recognized scalar kind/scope/name fields.
Kubernetes requires namespace and empty native scope fields; Docker requires
empty namespace and explicit `DOCKER_DAEMON` scope. Opaque daemon IDs retain up to
1024 UTF-8 bytes rather than being truncated to generic label/namespace limits.
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
Provider inventory remains independent of live Agent associations. It shows the
declared Kubernetes/Docker substrate without exposing connection configuration.
It shows
the typed read-only creation capability independently from connectivity, and shows
management/Connector/power separately, terminal intent and unresolved versus
confirmed cleanup. Resource detail labels native daemon/container/volume facts
separately from Kubernetes namespace/Pod/PVC/UID, and preserves presence/observation
independently of original identity. Long native values wrap on narrow viewports.
It provides no adoption, cleanup,
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

Real Docker resource/lifecycle validation belongs to the separate browser-free
integration app through public RPC and actual Control/Provider/Connector. It does
not duplicate the existing owner UI browser journey. Detailed native DTO/copy
remains owner-layer coverage.

## Connections

- [Agent handles and saved profiles](agent-profile.md) — final creation values, catalogue paging and frozen uncertain-result recovery.
- [Computer owner and installation inventory RPC](computer-client-rpc.md) — Owner operations and redacted installation inventory are separate capabilities over current account authority.
- [Computer lifecycle admission and Provider keys](computer-lifecycle.md) — Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.
- [Primary rendering and URL-owned state](web-rendering.md) — Server admission supplies primary content and stable selections; client queries own subsequent interactions rather than a second initial truth.
- [Workspace context and navigation](workspace-navigation.md) — Navigation selects an accessible context without granting access to another owner’s private Agents.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.

## References

- [`typescript/apps/web/src`](../../../../typescript/apps/web/src)
- [`typescript/apps/admin-web/src`](../../../../typescript/apps/admin-web/src)
- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
