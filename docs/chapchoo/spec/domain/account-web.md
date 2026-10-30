---
title: "General and Admin Account Web"
created: 2026-10-30
tags: [accounts, web, authentication, frontend]
spec_type: domain
domain: accounts
code_paths:
  - typescript/apps/chapchoo-web
  - typescript/apps/chapchoo-admin-web
  - typescript/packages/chapchoo-ui
  - typescript/packages/chapchoo-web-core
last_verified_at: 2026-10-30
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
extracted theme/token sources preserve the scaffold's definitions. Product
routes and stories replace Hello World; there is no second Admin theme or Refine
layer. App containers own data/state/navigation; shared presentation receives
props/callbacks and imports no business RPC, environment, cookie or token store.
The inherited Azents-derived visual styling remains provisional, not an approved
Chapchoo design system. General-purpose design work lives in the independent
[design reference workspace](design-reference.md); app Storybooks retain their
product-state review role.

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

Server Components use Identity only, without renewal or cookie writes. A missing
or invalid session can redirect to login; an authentication infrastructure
failure remains an error rather than being converted to signed-out identity.
Admin read-only admission also rejects a currently non-administrator identity.
An explicit Admin client-error refresh invalidates identity and both grant lists,
so a transient identity failure can recover instead of leaving its error screen
pinned. An initial SSR infrastructure error refresh repeats the read-only page
admission rather than enabling client business queries without that precondition.

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
be required. A delayed cookie never un-revokes backend state.

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
Shared Geist font files are build inputs, served as ordinary public assets with
runtime-prefixed CSS font faces. Browser navigation and server redirects use the
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
