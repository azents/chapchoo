---
title: "Web authentication, request ownership and cookies"
---

# Web authentication, request ownership and cookies

Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.

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
query without mount rechecks. Loaded identities, including a known guest, are
rechecked on window focus and reconnect. A changed account clears prior scoped
caches and navigates through server admission. Explicit recheck can recover an
initial error and is the authentication refresh requested by the page-level
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

Protected Computer query/mutation invalid-session errors also withdraw their
app's identity and scoped caches. Current-Admin permission loss on Computer
inventory ends only the Admin context; ordinary owner business denials retain
General authentication.

General [external-authentication](authentication-web.md) queries participate in
the same invalid-session policy. Its imperative secret mutations keep raw keys
out of mutation-cache variables and explicitly clear the shared identity on a
typed invalid-session response. They do not replay the write or treat ordinary
Team/provider denial as logout.

Session-ending navigation is single-shot, clears identity-bound query/mutation
data, cancels outstanding queries and uses that app's configured login mount.
Authentication success clears previous account data and performs a full
navigation so server admission runs again. Explicit local clearing after
password reset preserves its terminal result and no longer treats the SSR
initial User as an expected current session.

Logout immediately clears client identity and account caches and submits a native
same-origin POST to the current app's logout route. Exact configured Origin
admission precedes credential handling. The route deletes the shared host-root
HttpOnly cookie and returns a no-store 303 to its configured login mount without
renewing the session. Backend revocation is best effort, runs through the server's
post-response lifecycle with a finite RPC deadline, and never blocks cookie
disposal or preserves a signed-in failure screen. There is no confirmation or
automatic replay. If the browser cannot reach the same-origin web server, client
JavaScript cannot physically delete an HttpOnly cookie; local identity clearing
alone does not claim that offline credential disposal succeeded.

## Cookie disposition

There is one backend opaque session, not a web mapping database or separate
access/refresh pair. HttpOnly cookies persist for at most 30 days and never beyond
the returned backend expiry. General and Admin intentionally share one host-root
cookie; ports do not isolate cookies:

| App/transport            | Name                     | Path / SameSite |
| ------------------------ | ------------------------ | --------------- |
| General/Admin local HTTP | chapchoo-general-session | root / Lax      |
| General/Admin HTTPS      | __Host-Chapchoo-General  | root / None     |

HTTPS cookies have Secure; permitted local HTTP cookies do not. Both modes are
host-only and HttpOnly. Deletion uses the same name/path/Secure/SameSite policy.
Credential values never enter React props, tRPC result objects, client query
caches, JavaScript persistence or diagnostic logs. Successful login/account
switch/logout clears identity-bound client query state.

Cookie outcome is independent of ordinary business success:

- Successful renewal survives later validation/permission/domain failure,
  including current non-administrator denial on Admin. The rotated credential
  is published before that denial while Admin business dispatch remains blocked.
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

## Connections

- [Account RPC boundaries](account-rpc.md) — Account transport owns service admission, caller credentials and validated wire outcomes. Business transactions and process lifetime have separate owners.
- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Primary rendering and URL-owned state](web-rendering.md) — Server admission supplies primary content and stable selections; client queries own subsequent interactions rather than a second initial truth.
- [Runtime web configuration and failures](web-runtime.md) — Web deployment configuration is resolved at runtime; decoding and reporting preserve safe, truthful failure boundaries.

## References

- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
- [`typescript/apps/web/src/features`](../../../../typescript/apps/web/src/features)
- [`typescript/apps/admin-web/src/features`](../../../../typescript/apps/admin-web/src/features)
