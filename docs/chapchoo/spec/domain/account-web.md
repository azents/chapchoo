---
title: "General and Admin Account Web"
created: 2026-10-30
tags: [accounts, web, authentication, frontend]
spec_type: domain
domain: accounts
code_paths:
  - typescript/apps/web
  - typescript/apps/admin-web
  - typescript/packages/ui
  - typescript/packages/web-core
  - typescript/apps/e2e
  - testenv/chapchoo
last_verified_at: 2026-10-31
spec_version: 1
---

# General and Admin Account Web

## Applications and shared presentation

General and Admin are independently built Next.js/React/Mantine applications.
The general app provides password/configured-code login, email-bound signup,
password reset, personal/team context, membership and invitation operations.
Admin provides initial setup, administrator login and signup/reset-link
inventories with issuance/revocation. Team ownership does not grant server-wide
administration. Backend operations remain the authority for current permissions.

Both apps use the same workspace UI theme, semantic CSS variables, provider,
form controls, application shell and confirmation/feedback primitives. The
shared theme/token sources implement the selected DESIGN.md values. Product
routes and stories replace Hello World; there is no second Admin theme or Refine
layer. App containers own data/state/navigation; shared presentation receives
props/callbacks and imports no business RPC, environment, cookie or token store.
Both web apps support System, Light and Dark and load locally bundled Pretendard
with Geist Mono reserved for code. Pretendard uses the complete official
unicode-range subset set; the browser fetches only ranges used by rendered text,
without a whole-font or unused mono preload. The runtime mount is passed into
font stylesheet, font and decorative mascot URLs. The 72px mascot uses a 216px
alpha-preserving production derivative with normal composition. Five transparent
PNG masters and small WebPs share both appearances; unchanged generated originals
and extraction provenance remain in common UI assets.
Same-size single-line form fields and buttons share common frame heights through
the theme's Input and Button variables: xs 36px, default/sm 44px, md 48px,
lg 56px and xl 64px; narrow screens retain a 44px minimum. Textareas and
multi-value fields can grow rather than clipping content. Workspace contexts
use the shared transparent Paper row variant, aligning their text with adjacent
sections without an edge-flush filled background.

Desktop workspace sidebar and content regions scroll independently within the
viewport below the variable-height global header. The sidebar footer stays
independent of main-content length; the entire sidebar can scroll in short
windows or with long navigation. Narrow layouts use ordinary document scrolling.

Narrow editable controls, including compound inner inputs and editable text
surfaces, use a 16px minimum while preserving larger user text and pinch zoom;
native selects retain their platform-owned line box. Visual principles remain
in root DESIGN.md. Shared button and chip labels use untrimmed intrinsic line
boxes; action controls grow for wrapped or enlarged text rather than clipping
glyph extents. Auth footers link only to other auth modes and omit the current
login/signup/reset destination. Desktop uses a side navigation pane; narrow views wrap
the same navigation above full-width content. Sectioned forms/rows/tables replace
the former scaffold layout without changing app-owned actions or permissions. General-purpose design work lives in the independent
[design reference workspace](design-reference.md); app Storybooks retain their
product-state review role.

## Appearance lifecycle

Both applications reuse the same sticky global AppHeader through their auth/setup
and workspace frames. Its brand is left-aligned and its 44×44 icon-only appearance
entry point remains at the far right on desktop and narrow views, including
workspace loading and recoverable errors. The entry point has a localized
accessible name/hover title and a monitor, sun or moon for the
selected preference. It opens a keyboard-operable System/Light/Dark menu with a
current-selection check and restores focus on close. Account identity/logout
join this header on desktop and remain in a separate context row below navigation
on narrow views; only one responsive account-action presentation is visible.
The shared flex frame accommodates the header's actual height; wrapped or
enlarged identity text grows the header without clipping or covering the content.
The common UI is prop-driven and owns no identity request or authentication
store. The former page-local native select is replaced, including its exports
and examples.

An absent or invalid preference
means System. Explicit Light/Dark overrides device changes; System follows current
browser media and live changes. Theme changes preserve application identity, query
state, entered work and focus rather than remounting the subtree.

Host-only cookies `chapchoo-color-mode` and `chapchoo-color-mode-resolved` store the
preference and last resolution for one year at path `/`, SameSite=Lax, with Secure
on HTTPS. They contain only presentation enums and are intentionally readable by
JavaScript. General/Admin share them across same-host runtime mounts and ports;
different hosts remain independent. They are not session credentials or account
preferences. Cookies are the only persisted appearance source; no localStorage
mirror or default Mantine storage manager participates. Cookie denial permits an
in-page change but cannot guarantee persistence. Already-open independent tabs
are not actively synchronized through a cross-tab service.

Each root layout validates cookies and emits initial HTML/body mode attributes.
Request-time viewport metadata emits `color-scheme` in the initial head before
the CSS-dependent synchronous bootstrap, available while stylesheets are pending,
using a concrete explicit mode or `light dark` for System. The bootstrap and
live appearance bridge keep that metadata aligned with the selected preference;
it informs the browser's initial UA canvas independently of CSS evaluation.
The HTML element also carries inline `color-scheme` and a canonical-token
`light-dark()` canvas background, so the document canvas does not depend on
external stylesheets, the head bootstrap or hydration. System advertises both
schemes so the browser selects the canvas from current media even when the
cached resolution is absent or stale; explicit modes advertise only that mode.
The existing bootstrap
and live bridge update that same color scheme; the canvas follows automatically.
Explicit preferences win over a cached resolution. System uses the cached hint
or a light server placeholder until a synchronous head bootstrap resolves current
browser media before visible paint. A body-start prelude mirrors the result, and
the app provider maintains attributes and the resolution cookie through hydration
and live changes. The server cannot independently know a first-visit device
preference. With JavaScript disabled, explicit cookie modes still render correctly;
unknown System uses the server placeholder. Adaptive CSS variables use the HTML
mode as their sole styling authority. Shared presentation remains cookie-free;
the browser-safe web-core policy and app bridges own persistence/synchronization.
Browser acceptance checks computed canvas colors while external stylesheets are
held, then SSR body colors with application scripts blocked, and actual
content-frame background colors through refresh/hydration. Mode attributes alone
are not treated as proof of initial paint. These checks do not establish the
absence of a pre-content white frame on a physical iPhone Safari.

Light and warm olive-charcoal dark maps retain common layout/type metrics and
use separate filled-accent ink and readable accent-text roles. Menus/dialogs use
raised surface tones. Storybooks expose both appearances with static fixtures.

Routes remain locale-less. The locale cookie, then Accept-Language, then English
select en-US, ko-KR, ja-JP or fr-FR. Forms, statuses and failures use aligned
localized messages. Normal UI shows the consequence/next action rather than raw
protocol reasons, stack traces or implementation metadata.
Collaborative context badges use localized Team terminology; personal contexts
retain the distinct personal-space label.

## Node transport and request ownership

Browser operations use app-owned tRPC routers and TanStack Query. Node calls
[the generated account RPC contract](account-rpc.md) using request-local bearer
metadata. General/Admin use their corresponding backend endpoint; reusable
channels hold no mutable caller credential. Browser exports are type-only DTOs
and safe enum constants, without runtime grpc-js or server configuration imports.

The HTTP adapter admits POST only and validates the exact configured Origin
before constructing a backend or dispatching work. Query HTTP links override to
POST. Browser links include the compiled public base path; Route Handlers use
the internal `/api/trpc` endpoint because Next removes that path prefix before
dispatch. Server redirects use application-relative paths and let Next add the
prefix once. Admin inventory query inputs use canonical decimal offset strings
and bounded numeric page sizes, then convert to backend int64; BigInt values do
not enter TanStack Query's JSON-hashed keys. Each HTTP request carries one public procedure; compound batches are
rejected before dispatch so login creation cannot inherit another operation's
cookie deletion or renewal state. Missing/mismatched Origin and unsupported
streaming format are rejected before mutation/renewal. Automatic query/mutation
retry and gRPC request retry are disabled. A lost response does not authorize repeating a potentially
committed change.

Public login-method/setup/proof-preview routes do not implicitly renew. A
protected writable request renews at most once in its request-local context,
then uses the committed current token for subsequent business calls. Authentication
responses expose User to the browser, never AuthSession or its credential.
Fresh manual grant issuance intentionally returns the new bearer link; later
inventory reads omit proof/URL. Signup/reset links are user-presented proof flows,
not browser login-session storage.

Public authentication pages resolve sign-in methods, setup availability and
proof previews on the server before selecting their form/closed/error state.
These read-only loaders are request-bound, not build-time backend calls.
Snapshot form renderers consume resolved required data directly, without an
initial browser query or artificial initial loading state. Infrastructure failures
reach a sanitized route-level error boundary; invalid/closed proof and setup
states remain explicit business outcomes. Error recovery reruns the route read.
Proofs and session credentials retain their existing authority and disposition.

Both roots initialize authentication from validated read-only Identity, shared
with protected route admission through request-local React caching. Cookie
presence alone never establishes authentication. Missing/invalid session,
validated User and infrastructure failure remain distinct. Admin also validates
current administrator access.

Each app adapts its independent tRPC client to the shared web-core authentication
Context and query/mutation policy. The Context exposes one identity-query
projection plus logout, explicit recheck, authentication completion and local
identity clearing; it does not keep a second mutable User. SSR success seeds that
query without mount, focus or reconnect rechecks. Explicit recheck can recover an
initial error and is the only authentication refresh requested by the page-level
recovery actions. Workspace/inventory containers own their business invalidation,
not identity queries or session navigation. Header identity can remain visible
while business data loads or fails.

Both QueryCache and MutationCache handle typed invalid-session errors from their
app's protected procedures. An identity read returns `null` for invalid/missing
session while preserving server cookie disposition; a common identity observer
ends an expected browser session on that result. Admin identity permission loss
ends only the Admin context. Incorrect credentials/codes, proof rejection,
ordinary business permission errors and transport failures do not sign out the
user. No automatic operation replay or refresh timer is introduced.

Session-ending navigation is single-shot, clears identity-bound query/mutation
data, cancels outstanding queries and uses that app's configured login mount.
Authentication success clears previous account data and performs a full
navigation so server admission runs again. Explicit local clearing after
password reset preserves its terminal result and no longer treats the SSR
initial User as an expected current session.

Logout immediately clears client identity and account caches and submits a native
same-origin POST to the current app's logout route. Exact configured Origin
admission precedes credential handling. The route deletes only that app's
HttpOnly cookie and returns a no-store 303 to its configured login mount without
renewing the session. Backend revocation is best effort, runs through the server's
post-response lifecycle with a finite RPC deadline, and never blocks cookie
disposal or preserves a signed-in failure screen. There is no confirmation or
automatic replay. If the browser cannot reach the same-origin web server, client
JavaScript cannot physically delete an HttpOnly cookie; local identity clearing
alone does not claim that offline credential disposal succeeded.

## Primary content and page state

General home contexts, selected team details/membership, and the selected Admin
inventory page are loaded and rendered on the server. A request-local read scope
shares validated Identity and lends the app credential only inside server reads;
it never renews or writes cookies. Primary-data failures reach safe route-level
error/not-found handling. Required snapshot values do not gain artificial
client-only initial loading states.

Interactive queries receive the same initial data and selection for SSR and
hydration, without mount/reconnect repeats. Later updates and invalidation remain
client-owned. Received invitations and team management invitations are
supplementary: their loading/failure stays in the corresponding section, rather
than replacing otherwise usable primary content. Context navigation is also
supplementary on a selected-team route. Existing visual hierarchy is preserved.

Admin inventory offset is a canonical decimal URL search value with the existing
25-row page size and int64 bound. Admin login method is URL-owned and constrained
by the server-advertised methods. Server and browser use the same first-value
policy for duplicate keys. Native history push updates primary selections and
Next search-parameter subscribers; direct entry/reload resolves the same selection
on the server. Back/Forward restore the selected view. Minor URL state uses
replace; no current feature needs an invented minor selection.

Admin issuance/expiry values use the selected locale with explicit UTC formatting
and a visible UTC suffix, independent of server or browser local timezone. A full
user-timezone preference policy remains outside this implementation.

Existing selected-team and inventory-kind routes remain pathname-owned. Form
inputs, authentication challenge bindings, newly issued proof links and destructive
mutation confirmations remain transient and never become URL state. Shared
read/initial-query/URL-codec patterns are described in the
[implementation guide](../../notes/web-page-patterns.md).

## Cookie disposition

There is one backend opaque session, not a web mapping database or separate
access/refresh pair. HttpOnly cookies persist for at most 30 days and never beyond
the returned backend expiry. Names distinguish the apps because ports do not
isolate cookies:

| App/transport | Name | Path / SameSite |
| --- | --- | --- |
| General local HTTP | chapchoo-general-session | root / Lax |
| General HTTPS | __Host-Chapchoo-General | root / None |
| Admin local HTTP | chapchoo-admin-session | configured public base path / Lax |
| Admin HTTPS at root | __Host-Chapchoo-Admin | root / Lax |
| Admin HTTPS with prefix | __Secure-Chapchoo-Admin | configured public base path / Lax |

HTTPS cookies have Secure; permitted local HTTP cookies do not. Both modes are
host-only and HttpOnly. Deletion uses the same name/path/Secure/SameSite policy.
Credential values never enter React props, tRPC result objects, client query
caches, JavaScript persistence or diagnostic logs. Successful login/account
switch/logout clears identity-bound client query state.

Cookie outcome is independent of ordinary business success:

- Successful renewal survives later validation/permission/domain failure.
- Invalid session at renewal clears the app cookie and blocks business dispatch.
- Invalid session detected after renewal overrides its candidate cookie with
  deletion.
- Authentication infrastructure failure retains a potentially valid old cookie
  and prevents business dispatch.
- Server-rendered identity reads do not rotate, discard or refresh cookies.

There is no browser-wide monotonic token registry or background renewal job.
Delayed responses can overwrite/clear newer browser state. The backend's fixed
previous-token overlap bounds recovery; after overlap or expiry, fresh login can
be required. A delayed cookie never un-revokes backend state. In particular,
another tab's already-dispatched renewal response can restore a cookie after
local logout deleted it. Successful backend revocation prevents that credential
from authenticating; failed revocation does not provide browser-wide monotonic
logout across delayed responses. This refactor does not introduce a durable
cross-tab logout registry or change the backend credential protocol.

## Configuration and failure boundaries

Typed app configuration reads the public URL, bind host and backend URL in its
configuration boundary. Local HTTP requires localhost/parsed loopback public
host and loopback listener; supported local start/dev commands explicitly bind
127.0.0.1. Remote public origins require HTTPS. Host/Forwarded headers do not
replace configured origin trust. Backend HTTPS uses grpc-js's CA/hostname
verification without plaintext fallback; internal plaintext requires the
operator's trusted network boundary.

Next builds have no deployment public path. The documented production-server
configuration phase reads the runtime mount and asset prefix. A public custom
server accepts only that mount, strips it before Next dispatch, and uses the
documented webpack configuration extension with automatic chunk public paths.
Shared Pretendard/Geist Mono files and mascot art are staged from common UI
assets and served with runtime-prefixed URLs. Browser navigation and server redirects use the
safe runtime path rather than the framework's compiled basePath. Shared assets
are included through tracing/transpilation configuration.
GET `/health` under the runtime prefix validates runtime web configuration and
returns no-store process health. It does not establish backend/schema readiness
or an authenticated product journey. [Linux delivery](account-delivery.md)
describes executable-relative standalone execution and its acceptance boundary.

Canonical generated reason metadata is validated before conversion to typed
localized failures. Unknown/missing reasons retain generic transport failure.
Required decoded message presence, timestamp bounds and enum membership are
validated at the Node boundary. Public errors omit original message/stack/cause.
Node-origin infrastructure has one safe reporting boundary; known backend
failures are not redundantly logged. SSR uses a safe typed error retaining the
original source in a private diagnostic field, preventing default Next/Node
inspection from enumerating native credentials, SQL or proof material.

## Verification boundary

Focused tests cover Origin/POST admission, cookie attributes/caps/disposition,
renewal ownership and concurrent context isolation, post-renewal domain failure,
revocation and authentication infrastructure failure, decoded response validation,
read-only SSR, safe source preservation and runtime prefix validation.
A bounded-overlap response-loss/reordering authority model remains lower-layer
evidence. A separate controlled real-Chromium/HTTP test verifies actual delayed
cookie overwrite/deletion and HttpOnly JavaScript non-exposure using the product
cookie functions; it does not claim real-backend time travel or TLS deployment.
Component stories and SSR tests cover presentation states.

Three root-executed product journeys passed against a fresh private PostgreSQL
installation and both production Next applications at `/app` and `/admin`:
bootstrap/signup/personal, invitation/transfer/leave, and reset/relogin/Admin
rejection. Confirmed reset completion remains visible after identity cache
clearing consumes/removes the proof preview. Owned process/DB cleanup succeeded.
Desktop1280×900 and mobile375-wide native PNGs were inspected with the real
GeistSans font, light theme, en-US, Asia/Seoul and DPR1; mobile had no document
horizontal overflow. No proof/password screen was captured.

Both apps' format/lint/typecheck, unit tests, production build and Storybook build
have passed in root-integrated validation, together with backend/protocol gates.
Independent review precedes the phase PR; exact-head CI follows PR creation.
The same three journeys also pass from the extracted x86-64 distribution using
bundled Node, with no host Node/pnpm/Turbo on the product PATH. This is local
artifact evidence, not a native ARM64 or deployed Helm/Argo acceptance claim.
