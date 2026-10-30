---
title: "Server Account and Team Onboarding Decisions"
created: 2026-10-29
tags: [accounts, architecture, protocol, administration]
document_role: primary
document_type: adr
snapshot_id: accounts-261029
---

# Server Account and Team Onboarding Decisions

- Snapshot: `accounts-261029`
- Mode: Collaborative; the requester owns unresolved material decisions.
- Authority: [confirmed Requirements](../requirements/accounts-261029-server-team-onboarding.md).
- State: architecture decisions in progress; D1-D37 recorded; D26 replaces D25's
  input-range policy and D33 supersedes D32 before implementation; remaining
  topics pending.

## Existing Baseline

The current checkout contains a Next.js Hello World app and development tooling.
There is no implemented server, Admin UI, authentication, team model, or client
protocol to preserve or migrate. The bootstrap PR is independent of the
architecture decisions recorded here.

## Fixed Inputs

- General web and Admin UI are distinct user-facing experiences.
- A web UI server, not the browser, speaks the Chapchoo client protocol.
- Admin operations have a distinct client protocol and server-side administrator
  checks. User identity is shared; team roles are not server administrator roles.
- Server deployment supports external PostgreSQL, Docker images, Helm packaging,
  and local-cluster verification.
- Local development uses SQLite, with PostgreSQL selectable for deployment/QA
  in this cycle (D13); D28 selects Diesel for the database layer.
- Rust is the backend language, not a preselection of transport or libraries.
- Future local/native clients constrain reusable boundaries but do not expand
  current implementation scope.

## Material Decision Map

The list tracks pending topics and accepted boundaries. Pending topics are not
accepted choices or a mandate to add components.

1. **Application and deployment boundaries:** how the general web experience,
   Admin web experience, and core server are packaged and deployed; logical
   separation does not automatically select separate processes. The two web apps
   are independent deployment units under D1. Under D2, one server binary can run
   the general role, Admin role, or both, selected by invocation. D3 requires
   separate general/Admin protocol ports, including in combined execution.
   D6 adds unified distribution and one central CLI without removing independent
   web deployment.
   D30 scopes this cycle's installable package to Linux x86-64 and ARM64;
   macOS support follows in the next step.
   D31 uses the main product's Next.js/React/Mantine foundation and shared UI
   components for Admin, with app-owned routing/authentication instead of Refine.
   Browser requests use tRPC/TanStack Query; Node web servers call backend gRPC.
2. **Data and authority ownership:** which component owns durable account/team
   operations, what UI clients may persist, and where authorization is evaluated.
   D4 selects common backend domain logic and shared data authority for both
   roles, with no direct business-database access by web UI servers. D13 extends
   the earlier PostgreSQL scope to SQLite local development in this cycle.
   D18 makes the Team owner reference the sole ownership authority; effective
   OWNER is derived rather than duplicated in membership role storage.
   D19 clarifies the MANAGER role name and atomically assigns that role to the
   former owner during collaborative ownership transfer.
   D28 retains SQLite/PostgreSQL and selects Diesel; repository operations own
   pooled connections and complete transaction lifetimes.
3. **Identity and lifecycle boundaries:** user-facing authentication, server/client
   session responsibility, role revocation, and installation/operator recovery
   authority across the distinct client surfaces. D5 selects the operator recovery
   CLI's shared-binary, common-domain, direct-database execution boundary.
   D8 selects opaque random login tokens backed by database session records;
   D13 applies that model to either selected backend.
   D9 follows Azents for independently managed web logins over one common
   authentication/session model, with current server-administrator role checks.
   D10 clarifies that one client session can serve both protocols without a second
   Admin login; web cookie boundaries are not separate token authorities.
   D11 stores the server token in HttpOnly web cookies rather than introducing
   an additional web-session-to-server-token store.
   D14 makes password reset atomically revoke all prior sessions and requires a
   fresh login rather than issuing a session from reset completion.
   D20 selects a default 180-day renewable validity window with no default
   absolute lifetime cap; expired or revoked sessions cannot be renewed.
   D21 rotates the opaque token on eligible renewal after 10 minutes and accepts
   the immediately previous value for a 5-minute overlap.
   D22 selects hash verification plus an encrypted latest-token copy, using a
   separate backend key, to recover that value within the overlap.
   D23 fixes default signup/reset/code lifetimes from issuance without sliding
   extension on preview or attempts.
   D24 selects whole-address case-insensitive email lookup with outer-whitespace
   trimming, separate display/delivery spelling, and no provider alias rules.
   D26 replaces D25's initial input range with 8-to-64 printable ASCII passwords
   and selects bcrypt, retaining any three of four classes across bootstrap,
   signup, and reset.
   D33 fixes the installation encryption key for this cycle, superseding D32's
   session-invalidation replacement procedure. D34 applies the common one-way
   verifier versus recoverable-secret storage principle now.
4. **Protocol contracts:** general and Admin operation boundaries, transport
   options, errors, completion evidence, retry safety, and reconnect behavior.
   D7 selects gRPC with Protocol Buffers for both backend client surfaces;
   detailed method, identity, and lifecycle contracts remain pending.
   D12 selects ordinary request handling without automatic mutation replay or a
   generic persisted request-result/deduplication subsystem for this cycle.
5. **Deployment lifecycle:** configuration, migrations, startup/readiness,
   secrets, and failure behavior for a distributed server deployment.
   D15 separates schema mutation into an explicit central CLI command; normal
   server startup validates prerequisites without automatically migrating.
   D16 selects foreground CLI-owned component execution without an application
   daemon or detached-instance controller in this cycle.
   D17 terminates the combined invocation on startup or unexpected component
   failure, without restarting components or affecting separate invocations.
   D35 runs the explicit schema command in a pre-deployment Job for direct Helm
   and Argo CD, using prerequisites available before that deployment phase.
   D36 supports loopback-only HTTP for CLI-launched web apps and HTTPS for
   externally exposed web origins, independently of Node's build mode.
   D37 terminates external gRPC TLS at an operator-managed HTTP/2 proxy, with
   plaintext backend hops confined to loopback or explicitly trusted networks.

Only after these outcomes are coherent will Rust stack selection and idiomatic
implementation patterns be decided. Product-scope changes return to Requirements.
Reversible internal identifiers and helper layout do not need requester decisions.

## Accepted Decisions

### D1. Independently deploy the general and Admin web applications

- Reference: `accounts-261029/ADR-D1`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-9`, `accounts-261029/REQ-11`.

**Decision**

The general web UI and Admin web UI are two independently runnable and deployable
applications. Their separate UI roles are not merely route prefixes in one
inseparable web application.

**Rationale and consequences**

Operators can expose the general web application while restricting or not exposing
the Admin web application. Deployment and process failures of one web application
do not inherently require deployment or restart of the other. Two web runtime
components must be built, configured, and operated; independent deployment does
not guarantee isolation from common infrastructure or backend failures.

Shared libraries and frontend tooling may be reused. This separation does not
create a separate administrator identity store or replace server-side authorization.

**Alternative not selected**

One web application hosting separate general/Admin entrypoints would reduce runtime
component count but couple their deployment, process failure, and exposure.

**Not decided here**

The core server's process/binary layout, Admin frontend framework, protocol
transports, listener topology, session format, and common-library layout remain
unselected.

### D2. One server binary with separately or jointly runnable roles

- Reference: `accounts-261029/ADR-D2`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-9`, `accounts-261029/REQ-11`.

**Decision**

Distribute one Chapchoo backend server binary. Its invocation selects the general
server role, the Admin server role, or both roles together. Operators can run the
roles separately using independent invocations of the same binary, or start both
with one invocation.

The independent general/Admin web applications from D1 remain separate from these
backend roles. This decision does not put Next.js web applications inside the Rust
binary.

**Rationale and consequences**

One server artifact provides the same implementation for compact installations
and independently deployed general/Admin roles. Small deployments need not
operate separate backend role invocations; operators can separate them when
exposure or lifecycle requirements justify it.

Role separation must not create divergent account/team authority or permission
rules. Shared implementation can be reused by both entrypoints. State ownership,
database-operation boundaries, and cross-instance correctness still need design.

**Alternatives not selected**

- A backend that always runs both roles together would prevent independently
  running them when desired.
- Separate general/Admin binaries would multiply distributed server artifacts
  without satisfying the request for one command-selectable binary.

**Not decided here**

Command names, the internal process/task organization of combined mode, listener
ports, shared-state mechanisms, shutdown/failure policy, and migration execution
remain unselected. No automatic failover or restart behavior is implied.

### D3. Separate general and Admin protocol network ports

- Reference: `accounts-261029/ADR-D3`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-9`, `accounts-261029/REQ-11`.
- Related decision: `accounts-261029/ADR-D2`.

**Decision**

The general and Admin client protocol endpoints use distinct network ports,
including when both backend roles are started together. They are not separated
only by request paths on one shared port.

**Rationale and consequences**

Operators can distinguish the endpoints in Kubernetes Services, network policy,
and firewall configuration, exposing the general endpoint while restricting the
Admin endpoint. Configuring two protocol ports introduces additional endpoint
configuration and deployment wiring.

Port separation is not authentication or authorization. Both endpoints retain
their required server-side identity checks, and Admin operations require
server-administrator authority regardless of source network.

**Alternative not selected**

Using a single port with general/Admin path separation would reduce port
configuration but require application-aware routing or filtering to distinguish
their network exposure.

**Not decided here**

Port numbers, bind addresses, transport, TLS termination, ingress rules, health
probe exposure, and startup/shutdown failure policy remain unselected. Distinct
ports do not imply distinct machines or require separate processes.

### D4. Share backend domain logic and PostgreSQL data authority

- Reference: `accounts-261029/ADR-D4`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-1` through `accounts-261029/REQ-12`.
- Related decisions: `accounts-261029/ADR-D2`, `accounts-261029/ADR-D3`.

**Decision**

General and Admin backend roles call shared account, team, and permission
implementation and access the same PostgreSQL business data. The Admin role does
not forward its domain operations through a running general server.

Web UI servers are protocol clients, not direct readers or writers of the business
database. They do not implement a second authoritative account/team store.

**Rationale and consequences**

Both backend roles observe one account/team authority and enforce consistent
domain invariants without duplicating their implementation. An independently
running Admin role does not require a live general-role process to perform its
domain operations. Both still depend on their shared database and required
infrastructure.

Concurrent mutations must preserve invariants across separate processes and
instances through database-backed transactions, constraints, or other mechanisms
to be specified in Design. An in-process mutex or process-local copy alone cannot
establish cross-instance correctness.

**Alternative not selected**

Having the Admin backend forward operations to the general backend would
centralize execution behind that server but add a network/authentication boundary
and make Admin operations depend on its availability.

**Not decided here**

Database schema, repository interfaces, transaction boundaries, database access
roles, connection pooling, client-session storage, caches, and the operator CLI's
recovery access path remain unselected. Shared domain implementation does not
give general requests permission to invoke Admin-only operations.

### D5. Include operator recovery in the single backend binary

- Reference: `accounts-261029/ADR-D5`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-12`.
- Related decisions: `accounts-261029/ADR-D2`, `accounts-261029/ADR-D4`.

**Decision**

Backend server execution and operator commands are provided by one binary.
Administrator recovery is a dedicated invocation of that binary, using common
domain logic and database access rather than invoking a running general or Admin
network server.

An authorized operator supplies the required execution environment and database
credentials. The command grants server-administrator authority to an existing
user through the common permission service; it is not an instruction to edit SQL
manually or bypass domain validation.

**Rationale and consequences**

Recovery does not require a live server process, web UI, or working administrator
login. It does still require database availability and operator-level access.
The binary shares the server's implementation rather than distributing a separate
recovery program.

No new remote break-glass authentication method is added to the Admin protocol
for this recovery operation. Database credentials and the execution environment
must therefore remain restricted to authorized operators.

**Alternative not selected**

A CLI connecting to the Admin server using a separate recovery credential would
depend on that server's availability and introduce an additional remote recovery
authentication boundary.

**Scope**

This backend-binary decision does not reverse D1: general and Admin web
applications remain independently deployed. It does not choose command names,
configuration delivery, database privilege granularity, or implement future Mac
application packaging.

### D6. Unified distribution with one central CLI and packaged web runtimes

- Reference: `accounts-261029/ADR-D6`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D1`, `accounts-261029/ADR-D2`,
  `accounts-261029/ADR-D5`.

**Decision**

Provide a unified installable package whose PATH-registered Chapchoo command is
the central execution/control entrypoint. The package may contain multiple files
and runtimes and start multiple processes; the goal is not to eliminate all
runtime dependencies inside a single native executable.

Include built web artifacts and their runtime in that distribution. With the
existing Next.js server foundation, standalone output and a target-compatible
Node.js runtime provide this packaging direction. Operators do not need pnpm,
Turbo, a web build, or a separate Node.js installation on the target host.

The CLI can launch web and backend components separately or together. The same
built web applications remain independently deployable under D1. Development and
independent image builds may use pnpm/Turborepo.

**Rationale and consequences**

One installed command provides a compact deployment experience without forcing
independently deployable components into one process. Bundled runtime maintenance,
platform compatibility, artifact completeness, and child-process lifecycle handling
become distribution responsibilities.

Next.js standalone output still requires Node.js and accompanying static assets.
It is not a pure Rust executable. The existing request-dependent locale handling
also means static export is not a drop-in equivalent of the current web app.

**Alternatives not selected**

- Requiring operators to install and invoke Node.js or package managers manually
  would not provide the requested installation/control experience.
- Requiring one native executable with no packaged runtimes is not the stated
  objective and would impose additional architecture changes.

**Evidence and remaining validation**

The existing web build already produces a standalone server entrypoint. Official
[Next.js output documentation](https://nextjs.org/docs/app/api-reference/config/next-config-js/output)
describes the traced runtime files and the need to include static assets.
[Static export documentation](https://nextjs.org/docs/app/guides/static-exports)
describes the request-dependent features unavailable in a static-only build.
The unified package and launcher have not yet been implemented or tested.

**Not decided here**

Installer format, installation locations, process supervision, background/service
integration, status/stop control transport, upgrades, signatures, and supported
artifact targets remain unselected. Central CLI control does not by itself
authorize a new remote administrative protocol.

### D7. Use gRPC and Protocol Buffers for general and Admin client contracts

- Reference: `accounts-261029/ADR-D7`
- Accepted: 2026-10-29 KST by the requester, explicitly requiring documented reasons.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-11`.
- Context: future native-client interoperability described in Requirements.
- Related decisions: `accounts-261029/ADR-D1`, `accounts-261029/ADR-D3`.

**Decision and scope**

Use gRPC with Protocol Buffers service/message definitions for general and Admin
backend client protocols. Keep their service surfaces and network ports separate.
The web UI servers are gRPC clients, and future native clients can implement the
same general client contract.

This decision does not select the browser-to-web-UI interface, a future harness
provider protocol, or a local process-control protocol. No parallel HTTP/JSON
backend API or gRPC-Web gateway is introduced by this decision.

**Decision criteria**

- A language-neutral, explicit contract shared by the Rust backend, Node-based
  web clients, and future native clients.
- One RPC model capable of ordinary operations and, where later required,
  streaming without inventing another method/message definition system.
- Standard call lifecycle primitives with explicit application-level semantics.
- Fit with web-server-as-client deployment, rather than browser-native transport
  being the deciding constraint.

**Reasons for selection**

1. **Shared generated contracts fit heterogeneous clients.** Protobuf defines
   service methods and request/response messages independently of a Rust web
   framework or TypeScript UI implementation. Language-specific generated clients
   reduce duplicate manual definitions. Semantic validation, optional-field
   meanings, and authorization still require explicit design.
2. **Request/response and streaming share a service model.** Current account/team
   operations can use unary RPC. Future authorized event subscriptions or
   bidirectional interactions can use streaming RPC when needed. This is a
   capability criterion, not approval to implement future agent/harness features
   or put all traffic on one permanent stream.
3. **RPC lifecycle conventions reduce protocol assembly work.** gRPC defines
   call status, metadata, deadlines, cancellation, and stream behavior. Chapchoo
   must still define business errors, authentication metadata, timeout policy,
   and mutation completion evidence.
4. **The actual web peer is a server.** A Node web UI server can act as a native
   gRPC client. The architecture does not require browsers to speak native gRPC,
   so a browser bridge is not necessary merely for the current web UI design.

**Alternatives considered**

- **HTTP/JSON with OpenAPI:** viable, inspectable with common HTTP tools, and also
  capable of typed generated clients. It is not rejected as untyped or incapable
  of native clients. Streaming/event mechanisms and their contract integration
  would need additional choices; gRPC better matches the selected common RPC
  contract direction.
- **A WebSocket-based application protocol:** viable for long-lived bidirectional
  messages, but WebSocket alone does not select method schemas, request-response
  correlation, structured call errors, or per-operation deadline semantics.
  Selecting or designing that application protocol is unnecessary for the chosen
  gRPC client surfaces. This does not ban WebSocket in other future boundaries.

**Costs and limitations**

- Requires schema/code-generation tooling and suitable language runtimes;
  exact Rust/Node/native libraries and generators are not yet selected.
- Deployment must support the required HTTP/2 path and compatible ingress/TLS
  handling. Long-lived streams also require timeout and draining design.
- Binary payloads are less convenient to inspect with a plain HTTP text tool;
  suitable gRPC diagnostics and safe reflection/exposure policy need consideration.
- A deadline or transport failure is not evidence that a mutation did not commit.
  Official gRPC documentation explicitly allows server/client completion views
  to differ. Retry and reconnect do not establish exactly-once application effects.
- Retries, idempotency, resumable subscriptions, authorization changes, and
  database transactions are not solved by transport selection.
- No benchmark was performed. Performance or payload-size superiority is not
  the primary justification or a verified project result.

**Primary references**

- [gRPC service definitions and RPC types](https://grpc.io/docs/what-is-grpc/core-concepts/#service-definition)
- [gRPC call termination semantics](https://grpc.io/docs/what-is-grpc/core-concepts/#rpc-termination)
- [gRPC retry behavior](https://grpc.io/docs/guides/retry/)
- [OpenAPI HTTP API contract specification](https://spec.openapis.org/oas/latest.html)

**Remaining decisions**

Method boundaries and message semantics, authentication/session authority, error
mapping, deadlines, retry safety, streaming/resume requirements, TLS/exposure,
schema compatibility rules, and implementation libraries remain pending.

### D8. Use opaque random tokens with database-backed login sessions

- Reference: `accounts-261029/ADR-D8`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D4`, `accounts-261029/ADR-D7`.

**Decision**

Issue an opaque, cryptographically random token for a login session. The backend
validates it against the authoritative PostgreSQL session record, rather than
using a self-contained JWT as the login access token. User/session association,
expiry, and revocation are server-side state, not client-interpreted token claims.

Email/password and future external-provider login can authenticate the user before
issuing this common session representation. External-provider access tokens are
not implicitly Chapchoo session tokens.

**Rationale and consequences**

The general and Admin backend roles already share database authority and domain
logic. No confirmed requirement calls for unrelated services to authorize requests
using self-contained claims without consulting current server state. An opaque
token therefore avoids adding signing-key and JWT claim validation solely for
the login access representation.

Server-side session records provide an explicit place to enforce expiry and
revocation. The cost is a current-state lookup on authenticated requests and
dependence on database availability. This decision does not promise that revoking
a session cancels an already-authorized in-flight operation.

**Alternative not selected**

Short-lived JWT access tokens with database-backed refresh/session state are a
viable alternative and are used by the Azents reference implementation. JWTs can
support independently validated claims, but add signing/claim validation and a
refresh lifecycle. Prompt revocation still requires current-state enforcement.
Azents itself checks active User/Session state in addition to JWT validity, so it
is not evidence that JWT adoption eliminates database lookups.

**Primary references**

- [OWASP Session ID content and server-side state](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#session-id-content-or-value)
- [OWASP JWT integrity, claims, and revoked-session considerations](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html#jwt)
- [gRPC authentication mechanisms](https://grpc.io/docs/guides/auth/)

**Remaining decisions**

General/Admin session isolation, token storage and transport details, expiry,
renewal/rotation, database token protection, browser cookie/session bridging,
revocation semantics for active streams, and security-event session invalidation
remain pending. This choice neither creates permanent tokens nor selects a
separate refresh-token layer.

### D9. Follow Azents web-session separation and common authentication authority

- Reference: `accounts-261029/ADR-D9`
- Accepted direction: 2026-10-29 KST; the requester chose the existing Azents behavior.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`.
- Related decisions: `accounts-261029/ADR-D1`, `accounts-261029/ADR-D8`.

**Verified reference behavior**

Azents Admin Web authenticates the ordinary User through the common password
login mechanism, then calls the Admin identity endpoint using that issued token.
Only after the administrator check succeeds does it retain its separately named
HTTP-only authentication cookies. A failed administrator check revokes the
provisional login session. Main Web and Admin Web perform separate login flows
and manage their retained sessions independently.

The backend Admin guard uses the same current-user/session authentication as
general APIs and additionally checks the user's current system_admin assignment.
Its token payload and guard do not enforce an Admin-specific token audience.
Consequently, a valid ordinary login token belonging to a current system
administrator can pass the Admin guard. Independent web logins must not be
described as a cryptographic or server-enforced general-versus-Admin token scope.

**Decision**

Follow that relationship: retain one User identity and common authentication/session
model, while general and Admin web applications establish and manage their own
login sessions. Admin login requires administrator authority, and protected Admin
operations recheck current server-administrator authority.

Do not add a rule that rejects an otherwise valid system administrator's session
solely because it was issued through the general login flow. An ordinary user
without the role is denied regardless of which client supplies their session.

Chapchoo retains opaque database-backed tokens under D8; choosing the reference
session boundary does not revert to Azents JWT/refresh-token representation.

**Rationale and consequences**

The requested compatibility keeps separate web login/logout state without adding
another identity store or purpose-specific session class. It also avoids claiming
that UI cookie separation protects against use of a stolen administrator session
at another endpoint. Such bearer-token exposure remains a security concern.

Current backend role checks, not UI visibility or token origin, enforce Admin
authority. Each web logout targets that web application's retained session;
account-wide invalidation remains a separately designed security lifecycle.

**Alternative not selected**

Purpose-bound general/Admin sessions, where a general token is rejected by Admin
even for a system administrator, would provide an additional scope boundary but
would differ from the verified Azents behavior. The earlier recommendation
combined that stronger boundary with independent web logins; the two must not
be conflated.

**Evidence and remaining design**

Reference implementation: Azents commit
`e99ac29b6bbdc05ee7a75a03b78fca2047073205`,
`typescript/apps/azents-admin-web/src/app/api/session/route.ts`,
`typescript/apps/azents-admin-web/src/shared/lib/auth-cookies.ts`,
`python/apps/azents/src/azents/core/auth/deps.py`, and
`python/apps/azents/src/azents/core/auth/jwt.py`.

This was static source/test inspection, not an executed runtime test. Chapchoo
login method placement across the two gRPC endpoints, cookie/session bridging,
expiry/renewal, and browser credential storage details still require design;
the reference's HTTP routing is not a new general-server availability dependency.

### D10. Unify protocol tokens without requiring a second Admin login

- Reference: `accounts-261029/ADR-D10`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D9`.

**Requester clarification**

Cookie handling necessarily depends on domain configuration, but server tokens
should be unified. In particular, a future Mac client must not require a second
login to use administrator capabilities on a server it has already authenticated to.

**Decision**

A client can reuse the same valid Chapchoo session token for the general and Admin
protocols of the same server. Admin operations additionally require the user's
current server-administrator authority. There is no separate Admin-purpose token
or mandatory second authentication solely to switch protocol surfaces.

Web applications may have different cookie storage/delivery boundaries and may
establish separate sessions where that deployment requires it. Those browser
boundaries do not redefine the server token model. This does not mandate an
automatic cross-domain web SSO flow, one global token across devices, or token
reuse between distinct Chapchoo servers.

**Relationship to D9**

D9's common authentication authority and current Admin role checks stand. Any
reading that two independently deployed UIs require every client to establish
two login sessions is superseded by this clarification. A native client may retain
one session per connected server and use it for all authorized protocol surfaces.
Future local trusted no-login behavior remains a separate, unchanged requirement.

**Rationale and consequences**

The token proves an authenticated session, while current permissions determine
which operations that session can perform. Client presentation and cookie domains
do not introduce another login identity or purpose-specific credential.

If one token is reused at both surfaces, invalidating that session affects its use
at both; interface separation does not create two independent copies of server
session state. Separately established sessions can remain independently managed,
subject to later account-wide invalidation policy.

An administrator's stolen valid session token can be used at either accessible
surface. Secure transport/storage and current authorization checks remain
necessary; separate ports or cookie names do not eliminate this risk.

**Not decided here**

Cross-domain web sign-in transfer, cookie/session bridging, client storage,
expiry/renewal, and any explicitly justified step-up authentication for particular
sensitive operations remain unselected. No blanket Admin re-login is authorized.

### D11. Retain web authentication tokens in HttpOnly cookies

- Reference: `accounts-261029/ADR-D11`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-11`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D10`.

**Decision**

Web applications retain the Chapchoo session token in an HttpOnly cookie. The web
UI server reads the cookie on incoming browser requests and uses that token for
its gRPC calls to Chapchoo. The browser does not directly use the backend client
protocol or need JavaScript access to the token.

Chapchoo's database session record remains the authority. Do not introduce a
separate web-only session identifier and shared mapping store solely to keep the
backend token off the browser in this cycle.

**Rationale and consequences**

This follows the reference web architecture while retaining the opaque token
representation selected in D8. It avoids a second session namespace/store,
including its replication, expiry, and restart persistence lifecycle, in the
web layer.

Cookie domain/path configuration controls where browsers send a token; it does
not establish a distinct general/Admin token model. D10's shared-token authority
and no mandatory second Admin login remain unchanged.

HttpOnly prevents ordinary browser script from reading the cookie, but does not
stop a compromised page from making authenticated requests or replace CSRF
protection. Secure transport, appropriate Secure/SameSite/domain/path settings,
and server-side request-origin/CSRF controls must be covered by Design.

**Alternative not selected**

Giving the browser only a web-session ID and keeping the actual token in a web
server-side store would keep the backend credential outside the browser cookie,
but add another persistent/shared authentication-state lifecycle. That extra
boundary is not selected for this cycle.

**Primary references**

- [OWASP HttpOnly protections and limitations](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#httponly-attribute)
- [OWASP SameSite and CSRF considerations](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#samesite-attribute)
- Azents reference: `typescript/apps/azents-web/src/shared/lib/cookies.ts` and
  `typescript/apps/azents-admin-web/src/shared/lib/auth-cookies.ts` at
  `e99ac29b6bbdc05ee7a75a03b78fca2047073205`.

**Remaining design**

Cookie configuration and cleanup, CSRF mechanism, login RPC placement,
expiry/renewal, and token protection in the database remain to be specified.
Future native-app secure storage is not selected by this web decision.

### D12. Use ordinary request handling without automatic mutation replay

- Reference: `accounts-261029/ADR-D12`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: account/team operations in `accounts-261029/REQ-1` through
  `accounts-261029/REQ-12`.
- Related decision: `accounts-261029/ADR-D7`.

**Decision**

Use conventional web-server request semantics over the selected gRPC contracts.
Do not automatically reissue a potentially executed mutation because its response
was lost or its deadline elapsed. Safe read retries may be bounded appropriately.

A transport error is not proof that a database mutation rolled back. Clients may
refresh existing state where applicable, report an unconfirmed outcome, and let
the user choose a subsequent action. This is ordinary error handling, not a new
persistent Work/task model or a generic operation-status workflow.

**Rationale and consequences**

The first cycle is a conventional account/team server. A generic request-key
registry, durable response ledger, and replayable result contract would add
storage, retention, concurrency, and secret-result handling beyond its needs.
The requester explicitly selected the simpler option and described the target as
ordinary web-server design.

Domain transactions, uniqueness constraints, input validation, and required
authorization checks remain necessary. Where an operation is naturally
idempotent, that property can be specified for that operation without creating a
generic replay subsystem.

**Alternative not selected**

Persisting mutation deduplication keys and results to permit automatic retries
could improve response-loss recovery, but adds another durable state lifecycle.
That mechanism is not included in this cycle.

**Implementation implications**

Do not configure indiscriminate application retry/hedging policies for mutating
RPCs. Respect library guarantees for retries that are proven not to have reached
application execution, rather than treating all reconnects as permission to
repeat a mutation. Transport cancellation is not a database rollback guarantee.

Standard error mappings, transaction boundaries, and timeout settings will be
documented in Design with rationale. Established security/correctness practices
and reversible implementation details do not require repeated product questions.

### D13. Support SQLite local development and PostgreSQL deployment/QA now

- Reference: `accounts-261029/ADR-D13`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: updated `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D4`, `accounts-261029/ADR-D8`.

**Decision**

Support SQLite for routine local server development and PostgreSQL for deployment
and QA configuration in this first cycle. PostgreSQL must be runnable/testable
without replacing the account/team/authentication business rules. Selecting the
Rust database library is explicitly deferred.

SQLite support is not postponed until the Mac application. Conversely, SQLite
local development does not implement Mac Standalone mode or bypass normal server
authentication and administrator setup.

**Relationship to earlier decisions**

D4's common domain implementation, shared authoritative storage for general/Admin,
and prohibition on direct web business-database access stand. Its PostgreSQL-only
wording, and D8's PostgreSQL-specific session-record wording, are generalized to
the configured backend by this decision. Distributed PostgreSQL deployment remains
supported; this does not turn SQLite into a shared multi-host database service.

Within one configured server installation, general/Admin roles and operator
commands use the same selected data authority. This is backend selection, not
simultaneous dual writes, local/cloud replication, or automatic data transfer
between engines.

**Rationale and consequences**

Local development needs a lightweight database without a separate server process,
while deployment and QA need actual PostgreSQL validation. Both engines must
preserve required domain outcomes. Shared models/queries/migrations are desirable,
but library selection does not erase SQL, DDL, typing, or concurrency differences.

Targeted integration coverage must exercise critical account creation, personal
space, token/session, invitation, and ownership invariants on the supported engines.
There is no requirement to duplicate every UI test permutation on both engines.
PostgreSQL verification cannot be replaced with a SQLite-only test result.

**Alternatives not selected**

- PostgreSQL-only development would require a database server for the ordinary
  local workflow.
- Deferring SQLite implementation until a future Mac app would miss the current
  local-development requirement.
- Selecting SeaORM, SQLx, or Diesel now would exceed the request to defer libraries.

**Remaining design**

Configuration syntax, default database file location, schema/migration mapping,
per-engine transaction details, and database libraries remain unselected.
Backend switching is not an automatic conversion of an existing database.

### D14. Revoke all prior sessions on successful password reset

- Reference: `accounts-261029/ADR-D14`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: updated `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D10`.

**Decision**

Successful password reset atomically consumes the valid recovery grant, changes
the intended user's password, and revokes all existing authentication sessions
for that user. Reset completion returns a completion result, not a newly issued
login session. The user signs in again using the new password.

Prior sessions cannot subsequently authenticate at either general or Admin
surfaces. User identity, team memberships, personal context, and server-admin
role assignments remain unchanged.

**Rationale and consequences**

Password recovery must not leave potentially compromised bearer sessions usable.
The shared general/Admin token model makes the revocation policy uniform.
Revoking only the session used by a browser, or preserving existing sessions
after recovery, would not meet the selected account-wide recovery behavior.

This follows verified Azents behavior: its reset repository groups token claim,
password mutation, and revoke_all_by_user in one transaction; its service returns
success without issuing login tokens.

**Boundaries**

Atomic session revocation is required on SQLite and PostgreSQL. It does not
retroactively undo already committed work or promise forced cancellation of an
already-authorized in-flight RPC. Coordination with concurrent session creation
must ensure an old credential cannot race recovery to create a surviving session;
exact per-engine synchronization is for the transaction design.

**Reference**

Azents commit `e99ac29b6bbdc05ee7a75a03b78fca2047073205`:
`python/apps/azents/src/azents/repos/password_reset_token/operations.py`,
`python/apps/azents/src/azents/services/password_reset_token/__init__.py`,
and `docs/azents/spec/domain/user-auth.md`, section 3.6.
Inspected statically; no runtime reset test executed for Chapchoo.

### D15. Run schema migrations through a separate central CLI command

- Reference: `accounts-261029/ADR-D15`
- Accepted: 2026-10-29 KST by the requester for the current cycle.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D5`, `accounts-261029/ADR-D6`,
  `accounts-261029/ADR-D13`.

**Decision**

Database schema initialization and subsequent migrations run through an explicit
subcommand of the same backend/central CLI binary. General and Admin server
startup do not automatically mutate the schema. They check required database
readiness and fail clearly when required schema prerequisites are not satisfied.

The boundary applies to SQLite local development and PostgreSQL deployment/QA.
Local instructions include the preparation command. Deployment automation or a
Kubernetes Job can invoke the same command; exact Helm integration is not selected.

**Rationale and consequences**

Schema mutation has an identifiable operator/deployment execution boundary rather
than being a side effect of starting/restarting each server role or replica.
Migration privileges and logs can be separated from normal service execution,
although exact database credential/privilege policy remains to be designed.

The trade-off is an explicit preparation step in the local and deployment
workflow. This does not require another installed migration tool and does not
select an ORM or migration library.

**Alternative not selected**

Automatically migrating during every server startup would shorten initial local
startup, but require concurrent-start coordination and schema-changing authority
in the normal server startup path.

**Remaining design**

Command spelling, migration identifiers and contents, per-database serialization,
failure recovery, compatibility checks, and Helm Job/hook ordering remain pending.
Schema preparation does not itself create the first administrator; installation
token initialization remains a separate accepted product flow.

### D16. Use foreground execution without an application-owned background daemon

- Reference: `accounts-261029/ADR-D16`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: updated `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D2`, `accounts-261029/ADR-D6`.

**Decision**

The central CLI's service execution stays in the foreground and owns the lifecycle
of the selected components it starts. On an ordinary termination request, it
coordinates shutdown of those components instead of intentionally leaving them
as detached services.

Persistent hosting and restart are supplied by Docker, Kubernetes, or the OS
service environment. Do not add an application-owned daemon, detached instance
registry, local management IPC, or background start/status/stop subsystem in this
cycle. This does not prevent an external service manager from supervising the
same foreground command.

**Rationale and consequences**

One installed control entrypoint does not require a second durable control plane.
Foreground lifetime is a direct fit for container/service supervision and avoids
new instance discovery, orphan ownership, and privileged local control endpoints.
Interactive terminal use requires keeping the execution alive or using an external
service manager.

The CLI only owns components started by that invocation; it must not terminate
unrelated processes, servers, or user applications. Runtime cleanup must also
respect this ownership boundary.

**Alternative not selected**

Self-detaching execution with built-in background management would keep services
running after the invoking CLI exits, but require instance identity, control
transport, lifecycle persistence, and additional recovery/security behavior.

**Remaining design**

Combined-component failure policy, signal propagation, graceful shutdown bounds,
readiness, and platform-specific process ownership still require design.
Future Mac app packaging/execution remains outside current implementation scope.

### D17. Treat combined execution as one failure unit

- Reference: `accounts-261029/ADR-D17`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: updated `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D2`, `accounts-261029/ADR-D6`,
  `accounts-261029/ADR-D16`.

**Decision**

If any selected long-running component fails to start or exits unexpectedly,
the combined foreground invocation coordinates cleanup of its other started
components and exits with an error. It does not continue presenting a partially
running selection as a successful execution, or automatically restart components.

The failure unit is that invocation only. Independently started commands,
deployments, existing servers, and user processes are not terminated. Operators
who need failure isolation can use the already supported separate role/web
execution and deployment units.

**Rationale and consequences**

A compact combined deployment has a straightforward success/failure contract and
can be restarted by its external service manager. It avoids a partial-service
supervisor, per-component restart policies, and another recovery control plane.

The trade-off is intentional fate sharing: a failed web component stops a healthy
backend started by the same invocation. Separate deployment remains available
where that coupling is unacceptable.

**Alternative not selected**

Continuing or restarting only the failed component could preserve partial
availability, but would require additional component health, degraded-state,
restart, and readiness semantics in the central CLI.

**Remaining design**

Startup/readiness detection, bounded shutdown escalation, process/task ownership,
and concrete error reporting still require implementation design. Expected
completion of one-shot migration/recovery commands is not an unexpected service
failure. This policy does not roll back database commits.

### D18. Store canonical team ownership on Team

- Reference: `accounts-261029/ADR-D18`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-4`, `accounts-261029/REQ-5`,
  `accounts-261029/REQ-6`, `accounts-261029/REQ-8`.

**Decision**

Team has one canonical owner reference. Membership stores ADMIN/MEMBER role data;
the effective role returned by permission checks and client projections is OWNER
when that member is the Team's referenced owner. Do not independently persist an
OWNER membership role as another writable source of ownership.

The owner must always be a member of that same Team at committed operation
boundaries. Personal Teams retain their fixed owner and one-member restriction.
Collaborative ownership transfer validates the actor and target membership, then
atomically replaces the owner reference. Generic member role edits, leave, and
removal cannot bypass the owner-only transfer rules.

**Rationale and consequences**

A single reference explicitly identifies one owner rather than inferring that
authority by searching role rows. Client-facing owner/admin/member roles remain
unchanged. Foreign-key and transaction design must enforce owner membership,
especially during Team creation, transfer, and concurrent member removal.

**Alternative not selected**

Persisting OWNER alongside ADMIN/MEMBER on membership rows would keep stored roles
uniform, but need additional owner-cardinality and transition enforcement.
Persisting both models as independently writable facts is also excluded.

**Remaining design**

The exact owner foreign-key target and creation/deletion ordering must be chosen
with SQLite/PostgreSQL constraint feasibility. The former owner's stored
ADMIN/MEMBER role after transfer must be specified before implementation; this
decision does not silently choose that permission transition.

### D19. Retain the former owner as a manager after ownership transfer

- Reference: `accounts-261029/ADR-D19`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-6`, `accounts-261029/REQ-8`.

**Decision**

The team-management role is named MANAGER. This clarifies the role previously
written as ADMIN in D18; the accepted single-owner-reference model is unchanged.
Server-administrator assignment remains a separate authority.

Collaborative ownership transfer atomically changes the Team owner reference
and sets the former owner's membership role to MANAGER. That person remains
a member and can subsequently leave through the ordinary leave operation.
The new owner's effective role is OWNER under D18. Transfer does not change
either user's server-administrator assignments.

**Rationale and consequences**

The permission transition is explicit and independent of the former owner's
underlying stored role before transfer. It retains team-management capability
while reserving ownership-only operations for the new owner.

**Alternative not selected**

Changing the former owner to MEMBER would immediately remove team-management
capability. Restoring an incidental stored role could make the same visible
ownership transfer produce different permission outcomes.

**Remaining design**

Per-engine transaction and constraint mapping must enforce the owner-reference
change and former-owner role update as one outcome, including concurrent member
removal, leave, and role changes.

### D20. Use a renewable 180-day session window without a default absolute cap

- Reference: `accounts-261029/ADR-D20`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D10`,
  `accounts-261029/ADR-D11`, `accounts-261029/ADR-D14`.

**Decision**

The default login-session validity window is 180 days. Eligible renewal advances
the expiry using the configured window. By default there is no absolute lifetime
cap from initial login; operators can configure a validity window and an optional
absolute cap. When a cap is configured, renewal cannot advance expiry beyond it.

Expiry and renewal eligibility are enforced by the authoritative backend session
state. An expired or revoked session cannot be revived by renewal. Logout and
password-reset revocation retain their existing effect at both protocol surfaces.
Common general/Admin session authority and the opaque token representation remain
as selected in D8-D11.

**Rationale and consequences**

The selected policy favors long-lived login continuity without a mandatory
periodic reauthentication deadline for active, successfully renewed sessions.
An eligible session can continue across repeated renewal windows when no
absolute cap is configured. This intentionally provides less forced
reauthentication than a bounded absolute-lifetime policy.

This decision selects continuity policy, not a JWT access-token layer, a separate
refresh-token credential, token-rotation timing, or previous-token grace behavior.
Those mechanisms are not implied by the 180-day window.

**Alternative not selected**

A 30-day inactivity limit with a 90-day absolute cap, or a fixed 30-day session
without extension, would force more frequent authentication and was not selected.

**Remaining design**

Define the concrete renewal operation, cadence, concurrency and response-loss
handling, token-rotation policy, web-cookie expiry updates, and per-engine atomic
coordination with logout and password reset before implementation.

### D21. Rotate the opaque session token with a bounded previous-token overlap

- Reference: `accounts-261029/ADR-D21`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D10`,
  `accounts-261029/ADR-D11`, `accounts-261029/ADR-D12`,
  `accounts-261029/ADR-D14`, `accounts-261029/ADR-D20`.

**Decision**

Use one opaque session-token credential, without a separate JWT or refresh-token
credential. A renewal request with the current token rotates its value when at
least 10 minutes have elapsed since the last issuance/rotation. This is
request-driven, not a periodic timer that rotates an unused session.

Rotation replaces the current value and retains the immediately previous value
for a 5-minute overlap measured from that rotation. A current-token renewal
before the rotation interval returns the current state without another rotation.
A previous-token request within the overlap does not start another rotation or
extend the overlap. Older values and the previous value outside the overlap
cannot authenticate.

Successful rotation extends the session-validity window under D20. Returning
unchanged current state within the rotation interval or previous-token overlap
does not independently extend expiry. Session expiry/revocation takes precedence
over either token's overlap eligibility at both protocol surfaces.

Rotation, expiry updates, and previous-token transition must form one atomic
outcome. Concurrent requests must not create independently authoritative current
tokens. Do not replay business mutations to recover a token-exchange response.

**Rationale and consequences**

The bounded overlap accommodates requests already using the previous token while
limiting the continued use of superseded values. It does not stop an attacker who
can obtain or follow the current credential. The 180-day continuity policy and
optional absolute cap remain as accepted in D20.

**Alternative not selected**

Keeping one unchanged bearer value while extending only database expiry would
avoid rotation races but allow the same value to remain valid across many
renewal windows.

**Remaining design**

Define protected verifier storage, whether/how a previous-token holder obtains
the latest token after concurrent renewal or response loss, and cookie-update
ordering. Retrievable current-token material, encryption keys, or a recovery
mechanism are not authorized merely by accepting rotation and overlap.

### D22. Protect and recover the latest session token with a separate encryption key

- Reference: `accounts-261029/ADR-D22`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D10`,
  `accounts-261029/ADR-D11`, `accounts-261029/ADR-D12`,
  `accounts-261029/ADR-D14`, `accounts-261029/ADR-D20`,
  `accounts-261029/ADR-D21`.

**Decision**

Persist verification hashes for session-token matching and an authenticated-
encrypted copy of the latest random token. General/Admin backend roles share
the required encryption key, stored separately from the business database and
preserved across restarts. Web UI servers and browsers do not receive this key.

An eligible immediately previous-token renewal within D21's 5-minute overlap
returns the same latest token after verifying current session validity. It does
not rotate again, extend the overlap, or replay any business mutation.
Concurrent holders converge on the committed current credential.

The token hash, ciphertext, and rotation/expiry facts are updated together.
Ordinary session/user read projections omit both protected verifier material
and ciphertext. Raw tokens are not stored as ordinary plaintext DB columns or
logged. Recovery rejects revoked/expired sessions regardless of ciphertext
availability.

**Rationale and consequences**

A hash verifies a candidate token but cannot reconstruct a newly issued random
value after response loss. The encrypted copy supports continuity for concurrent
or interrupted renewal, while separating the recovery key from DB data limits
what a database-only disclosure reveals.

This adds a persistent deployment secret shared by backend roles. A missing,
incorrect, or lost key prevents decryption-based recovery; it does not erase
users, memberships, or password credentials. Possession of a current token can
still be checked against its hash. Database and key compromise together can
expose token values; encryption is not protection against full backend compromise.

**Alternative not selected**

Hash-only storage without latest-token recovery avoids a decryption key, but a
client that misses the new value can require reauthentication after the overlap.

**Remaining design**

Specify secure key provisioning and loss/change procedures, ciphertext binding
and format, cryptographic library selection, cookie-update ordering, and exact
renewal RPC/error contracts. This decision does not select a key ring, key
versioning, automatic key replacement, or an external key-management service.

### D23. Use fixed signup, password-reset, and email-code validity windows

- Reference: `accounts-261029/ADR-D23`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-2`, `accounts-261029/REQ-3`,
  `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D14`, `accounts-261029/ADR-D20`.

**Decision**

Default validity is 7 days from issuance for an email-bound signup link,
24 hours from issuance for a password-reset link, and 10 minutes from issuance
for an email login code. These windows do not slide on preview, failed attempts,
or page navigation. The backend checks expiry when accepting the operation;
an earlier successful preview is not redemption authority.

Successful signup consumes its link. Successful password reset retains D14's
atomic consumption/credential/session-revocation outcome. An expired signup or
reset link requires a new authorized issuance; an expired login code requires
a new code request with mail delivery configured. Login-session renewal under
D20 does not extend any of these independent proofs.

**Rationale and consequences**

The selected defaults allow manually delivered onboarding links more time than
recovery links, and keep the interactive mail-login window short. Fixed expiry
bounds unattended proof lifetime without changing the no-mail signup/recovery
requirement.

**Alternative not selected**

Shorter signup/recovery windows reduce exposure of leaked links but increase the
chance that manual delivery requires reissuance.

**Remaining design**

Define exact issuance fields and operator configuration, email-code issuance
and attempt controls, proof hashing and delivery failure handling. Invitation
expiry, configurable multi-use links, and an expanded recovery-management UI are
not selected by these lifetime defaults.

### D24. Use one case-insensitive email identity key across account flows

- Reference: `accounts-261029/ADR-D24`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-2`, `accounts-261029/REQ-3`,
  `accounts-261029/REQ-7`, `accounts-261029/REQ-10`.
- Related decisions: `accounts-261029/ADR-D4`, `accounts-261029/ADR-D13`.

**Decision**

Use one shared email-identity normalization rule for signup issuance/redemption,
login and login-code matching, recovery lookup, and invitation-recipient matching.
Trim outer whitespace and fold letter case across the whole address into the
lookup key. Persist that key with engine-independent uniqueness enforcement.
Case-only variants resolve to the same identity and cannot create two accounts.

Preserve the trimmed display/delivery address separately. The lookup key does
not authorize rewriting the delivery address's local-part spelling. Dots and
plus tags remain significant; no provider-specific alias collapsing or
different-address account merging is introduced.

**Rationale and consequences**

Users can vary address capitalization without making account or invitation
lookup depend on SQLite/PostgreSQL collation. This is the selected application
identity policy, not a claim that all SMTP mailboxes are case-insensitive.
A provider that actually distinguishes local-part case cannot register its
case-only mailboxes as separate Chapchoo accounts.

**Alternative not selected**

Preserving local-part case as distinct account identity accommodates those
mailboxes but requires users and inviters to match that spelling exactly.

**Primary reference**

[RFC 5321 section 2.4](https://www.rfc-editor.org/rfc/rfc5321.html#section-2.4)
distinguishes case-sensitive SMTP local-parts from case-insensitive domains and
discourages exploiting local-part case sensitivity for interoperability.

**Remaining design**

Bind one normalization routine to the validated email grammar and use it at all
ingress and persistence boundaries. Verify uniqueness and recipient matching on
both backends. Future external-provider identity linking remains outside this
decision; matching email alone is not account-link authority.

### D25. Require 8-to-128-character passwords with three of four character classes

- Reference: `accounts-261029/ADR-D25`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-1`, `accounts-261029/REQ-2`,
  `accounts-261029/REQ-3`, `accounts-261029/REQ-10`.

**Decision**

Passwords contain 8 through 128 characters inclusive and satisfy at least
three of four classes: uppercase letters, lowercase letters, digits, and special
characters. Apply the same policy to initial administrator creation, signup,
and password reset. Four classes are also valid; no particular class is
individually mandatory when the other three are present.

Password matching is case-sensitive and does not use the email-identity
normalizer. Do not silently trim whitespace, truncate overlong input, or verify
only a password prefix. Invalid creation/reset input must not consume its
associated grant or change credentials.

**Rationale and consequences**

This retains the selected minimum length while relaxing the four-class
composition requirement to any three classes. The maximum is an explicit
128-character product contract, not a hashing-library byte-limit workaround.
The chosen verifier must support the entire accepted input.

**Alternatives not selected**

Requiring all four classes would retain the stricter composition rule.
A 15-character minimum without composition rules would select a different
password policy and was not chosen.

**Remaining design**

Define one shared length/class validator and select a compatible password hash
and resource limits. Common/compromised-password checks and authentication-attempt
controls are not selected by this composition policy.

### D26. Restrict passwords to 8-to-64 printable ASCII characters and use bcrypt

- Reference: `accounts-261029/ADR-D26`
- Accepted: 2026-10-29 KST by the requester before implementation.
- Requirements: `accounts-261029/REQ-1`, `accounts-261029/REQ-2`,
  `accounts-261029/REQ-3`, `accounts-261029/REQ-10`.
- Replaces: D25's initial input-range policy; its original record is retained
  above as decision history.

**Decision**

Accept 8 through 64 printable ASCII characters (U+0020 through U+007E)
inclusive. Require any three of uppercase ASCII letters, lowercase ASCII
letters, ASCII digits, and the special-character set defined in the Design.
Apply this rule consistently to bootstrap, signup, and password reset.
Preserve case, whitespace, and the entire input; reject disallowed or overlong
values rather than trimming or truncating them.

Store passwords using bcrypt with an independently generated random salt for
each password and a cost-bearing encoded verifier. The accepted input is at
most 64 UTF-8 bytes, below bcrypt's 72-byte input limit. No preliminary password
hash, extra pepper secret, or Argon2id verifier is selected.

**Rationale and consequences**

The requester selected the narrower ASCII input contract and bcrypt after the
128-character proposal exposed a byte-limit conflict. This is a product-input
restriction, not silent truncation by a hashing library. Non-ASCII passwords
are rejected explicitly. bcrypt remains a deliberate selection despite the
new-application preference for Argon2id in the referenced security guidance.

**Alternatives not selected**

Argon2id would support the previously discussed wider input range and a
memory-hard verifier. Retaining Unicode with a separate byte ceiling would
produce a less straightforward visible length rule.

**Primary reference**

[OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#bcrypt)
documents bcrypt's input limit and a work-factor minimum of 10.

**Remaining design**

Choose the Rust bcrypt implementation and validate its full-input behavior.
Benchmark the cost and bound concurrent hashing work for the target runtime,
with a work factor of at least 10. Authentication-attempt controls and
common/compromised-password checks remain independent decisions.

### D27. Ground unfinished behavior in the reference without feature expansion

- Reference: `accounts-261029/ADR-D27`
- Accepted: 2026-10-29 KST by the requester.
- Authority: confirmed Requirements non-goals and fixed outcomes.

**Decision**

Resolve reference-aligned details from verified Azents implementation, contracts,
tests, and configuration within the confirmed account/team scope. Preserve
explicitly approved Chapchoo differences, including its transport, deployment,
ownership, token representation/storage, and revised password policy.

Do not introduce features or user-visible restrictions absent from the
reference merely because they are common security or product recommendations.
The unaccepted proposal for a 5-failures/15-minute password-login restriction
and administrator-unlock workflow is not part of this design.

Reference-derived details are to be researched and incorporated without another
individual policy interview. Unresolved product behavior, permissions, and user
journeys still belong to the requester and must be asked when their outcome
matters. This is not blanket delegation of remaining product decisions.

Keep evidence and distinguish verified behavior from unknowns. A negative keyword
search alone does not establish stack-wide absence. Continue independent work
and report material conflicts with concrete evidence.

**Rationale and consequences**

The requester selected reference alignment over a sequence of speculative
feature choices. Standards and external recommendations can explain risks but
do not expand product scope or supersede explicit choices.

**Remaining design**

Complete reference-to-design mapping and the authority/feasibility audit.
This instruction neither grants implementation approval nor supplies evidence
for an unverified behavior. Required Rust/gRPC/dual-engine mechanisms still
need credible implementation design; blockers must be disclosed.

### D28. Use Diesel with local SQLite and deployment PostgreSQL

- Reference: `accounts-261029/ADR-D28`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`,
  and the confirmed whole-convention foundation constraints.
- Related decisions: `accounts-261029/ADR-D4`, `accounts-261029/ADR-D13`,
  `accounts-261029/ADR-D14`, `accounts-261029/ADR-D18`.

**Decision**

Retain SQLite for local operation without a separate database server and
PostgreSQL for deployment/distributed execution. Use Diesel for the Rust
database layer. This resolves the database-library choice deferred in D13;
it does not replace D13 with a PostgreSQL-only installation.

Preserve common application/domain contracts while explicitly adapting
backend SQL types, transaction admission, and schema operations. Native
PostgreSQL enums and constrained SQLite representations enforce the same domain
variants. SQL dialect portability is not assumed to be automatic.

The asynchronous integration uses diesel-async's supported PostgreSQL and
SQLite connection paths with an official pool adapter. Each complete repository
operation acquires and owns its pooled connection internally, including the
transaction lifetime. Services receive typed detached results, not live
connections. Ordinary errors roll back through the library; cancellation drops
the operation-owned lease so an open-transaction connection is not reused.

**Rationale and evidence**

Diesel provides a mature typed query DSL and an established ecosystem consistent
with the requested long-term foundation. Isolated probes verified typed queries,
native PostgreSQL enum mapping, SQLite constraints, and normal error rollback.

The documented diesel-async cancellation behavior does not by itself disqualify
the library. Its official adapters already classify open non-test transactions
as broken for pool reuse. A PostgreSQL probe with diesel-async 0.9.2 and bb8
0.9.1 verified that cancelling an operation owning its lease replaced the
backend connection, left no committed insert, and allowed subsequent work,
without custom rollback or a new task supervisor.

**Alternatives not selected**

SQLx and SeaORM remain valid evaluated alternatives, but the requester selected
Diesel. PostgreSQL-only local packaging would add DB-process/data-directory
lifecycle responsibility; PGlite's tested socket path introduced session/pool
differences. Neither is added to this cycle.

**Remaining verification**

The PostgreSQL pool result is not evidence for every backend, pool, query or
commit-cancellation boundary. Verify SQLite pool retirement, complete migrations,
schema typing, locking and connection reuse with the selected integration.
Exact dependency versions and pool choice follow compatibility verification;
bb8 is the tested baseline, not a reason to implement a custom pool. The
broader foundation bundle and final Design remain subject to approval.

### D29. Establish the shared Rust foundation with explicit ownership and standard tooling

- Reference: `accounts-261029/ADR-D29`
- Accepted: 2026-10-29 KST by the requester.
- Authority: confirmed whole-convention foundation constraints; REQ-1 through
  REQ-13 where the shared backend/CLI and verification support those outcomes.
- Related decisions: D4, D7, D12, D15-D17, D26-D28.

**Decision**

Use explicit constructor/function dependency injection and composition at the
executable boundary. Runtime calls follow protocol adapter, application service,
and complete repository operation responsibilities. Enforce exposed capabilities
with Cargo dependency direction and Rust visibility. Introduce traits for actual
substitution/capability boundaries, not automatically for every struct. Do not
add a runtime DI container or service locator as the default assembly mechanism.

Use Tokio for asynchronous execution and Tonic/Prost for the accepted gRPC
transport and generated Rust contracts. Own and join spawned work, coordinate
shutdown, and bound blocking CPU operations rather than treating async syntax
as cancellation or resource control. Keep the Diesel repository connection
ownership selected in D28.

Model domain values and outcomes with explicit structs/enums/newtypes and typed
errors. Validate external input and protocol presence at ingress. Preserve
unexpected infrastructure causes and explicitly capture reporting context/
backtraces at useful boundaries. Use tracing structured fields/spans and log
each handled failure once, without leaking secrets or internal details into
ordinary protocol errors.

Make rustc, targeted Clippy, and rustfmt required quality gates, with a pinned
supported toolchain, reproducible locked dependencies, generated-source
discipline, and layered tests. Preserve the complete relevant convention
baseline, including existing TypeScript/web, infrastructure, documentation,
dependency, testing, and delivery rules.

**Rationale and alternatives**

The direction combines established widely adopted Rust tooling with visible
dependency and resource ownership. Independent probes demonstrated constructor
substitution, rejected invalid type/layer/lint cases, generated two-endpoint RPC
behavior, owned shutdown, and explicit one-time structured failure reporting.

A runtime DI framework would add registration/lookup machinery without a
demonstrated requirement. Folder-only layering and convention-only type/error
discipline would provide weaker enforcement. Neither is the selected default.

**Remaining work and authority limits**

Resolve compatible versions and equivalent local crate/helper layout without
individual policy interviews. This decision does not approve a custom linter,
an unrequested service, a task scheduler, or new product behavior. Complete
integration design, operational verification planning, authority/feasibility
audit, and revision-bound final Design approval remain required. Implementation
has not been requested.

### D30. Ship Linux x86-64 and ARM64 packages before macOS

- Reference: `accounts-261029/ADR-D30`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-13`.

**Decision**

Provide the unified installable CLI distribution for Linux x86-64 and ARM64
in this cycle. Defer macOS package support to the next step. Windows packages
remain outside the current scope.

Each supported package includes the matching backend executable, built general
and Admin web artifacts, and the Node runtime required by D6. Verify installation,
startup, readiness and owned shutdown on both supported architectures.
Document the tested Linux runtime prerequisites with the release artifacts.

**Rationale and boundaries**

Linux-first distribution bounds the initial build, native dependency, runtime
bundling and process-lifecycle verification matrix while retaining both approved
CPU architectures. Adding macOS now would expand that matrix; it is deferred by
the requester. This package support boundary does not constrain local source
development to Linux or change Docker/Helm deliverables.

The future Mac desktop application and a macOS CLI package are separate
deliverables. This decision approves neither their implementation nor the
complete current Design. Exact build commands and artifact layout remain local
implementation details; final revision-bound Design approval is still required.

### D31. Share product UI across the two web apps without Refine

- Reference: `accounts-261029/ADR-D31`
- Accepted: 2026-10-29 KST by the requester, modifying the proposed web stack.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-11`, `accounts-261029/REQ-13`; frontend fixed constraints.

**Decision**

Build both independent web apps on Next.js, React, TypeScript and Mantine.
Admin shares the main product's components and look and feel. Use app-owned
Next.js routing and authentication integration; do not add Refine.

Share theme/tokens, typography, reusable presentation components and their
interaction-state tests through workspace packages. Keep application navigation,
credentials, permission-sensitive queries and server-side adapters within the
owning app. Sharing UI code does not merge deployments or authentication stores.

Retain the proposed browser-to-web tRPC/TanStack Query boundary. Each Node web
server uses generated Protobuf/gRPC clients to call its backend endpoint.
Request-local opaque credentials come from HttpOnly cookies under D11; Rust
domain authorization remains authoritative. The shared UI does not access DB
connections or server credentials.

**Rationale and alternatives**

The requester chose direct reuse of the main product's UI over the proposed
Refine administration integration. This keeps visual and interaction changes
in one maintained component source while allowing different navigation and
permissions in the independently operated apps. The apps own their small
authentication/navigation adapters rather than relying on Refine providers.

The reference's Refine layer coordinates authentication and routing; its actual
data calls use tRPC rather than a Refine DataProvider. Removing that framework
does not require changing the selected backend protocol or rebuilding a generic
administration framework.

**Verification and limits**

Verify shared component states and assembled views in both apps, including
keyboard behavior, loading/empty/error states and responsive layouts.
Verify generated TypeScript/Rust protocol interoperability, request-local token
isolation, response-cookie delivery, and independent standalone packaging.
The choice introduces no additional Admin CRUD, automatic cross-origin login,
or shared web session store. Product code changes remain pending final Design
approval and a separate implementation request.

### D32. Replace the recovery key with session invalidation and fresh login

- Reference: `accounts-261029/ADR-D32`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-3`, `accounts-261029/REQ-9`,
  `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.

**Decision**

Preserve the same separately supplied recovery key across normal restarts and
updates. For deliberate replacement or recovery from loss of that key, stop all
backend roles and other session-issuing processes for the installation. Use the
central operator CLI against the same database to invalidate all existing
sessions before restarting the roles with one new shared key.

Users then sign in again. Keep accounts, password verifiers, teams, memberships,
role assignments and unrelated onboarding/recovery grants intact. The operator
maintenance path requires database authority, not an existing web login or the
old recovery key. A configuration/decryption error by itself never triggers
automatic session invalidation or key generation.

**Rationale and alternatives**

This keeps key replacement bounded and avoids retaining old-key support or
re-encrypting existing active session copies. It sacrifices login continuity at
the explicit maintenance boundary. Re-encryption could preserve ordinary
sessions but would require both keys, additional transformation/recovery behavior
and a separate lost-key path; the requester selected fresh login instead.

**Operational consequences**

This is a stopped-service maintenance procedure, not a rolling key update.
Confirm all relevant service instances are stopped and cannot be automatically
restarted with old configuration. Commit and verify session invalidation before
switching the persistent secret and resuming service. A failed or uncertain
maintenance outcome leaves service stopped for inspection; a deployment or
secret rollback does not restore invalidated sessions.

Do not add automatic periodic key rotation, a key ring, a key-version registry,
or a general-purpose Admin session-management feature. Complete the per-engine
transaction tests and operator procedure before delivery. This is not final
Design approval or an implementation request.

### D33. Keep the installation encryption key fixed in this cycle

- Reference: `accounts-261029/ADR-D33`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`;
  current non-goals and future improvement boundary.
- Supersedes: `accounts-261029/ADR-D32` before implementation.

**Decision**

Key replacement is unsupported in this cycle. Treat the encryption key as
installation-level persistent secret material, shared through a common backend
cryptographic boundary. Use the same key on every role and across restarts,
updates and data restoration. Do not introduce a session-specific replacement
key or the D32 bulk session-invalidation command.

Bind the initially supplied key to the installation with a non-secret verification
record. Existing-installation startup and operator writes reject missing,
malformed or mismatched key material before admitting work. Normal startup reads
the binding; it does not replace or initialize it. The explicit initial
preparation path establishes the binding once, before encrypted product data is
written. Preserve stored data on failure.

**Rationale and operational contract**

Common encryption is a persistent-data concern rather than an independent
per-feature key lifecycle. The requester chose a coherent fixed-key contract for
the present scope, with a future common key-change capability instead of
feature-specific disposal policies. Operators retain the original key securely
alongside a recoverable installation backup; losing it is not repaired by
setting an arbitrary replacement. This introduces no automated backup service.

**Future boundary**

Common reversible-data re-encryption and recovery/re-entry of unrecoverable
one-way verifiers are deferred. A later compromise-response design must also
invalidate or replace credentials exposed to an attacker; changing ciphertext
does not undo that exposure. No key ring, version registry, rotation command or
new recovery system is authorized in this cycle.

### D34. Apply the common one-way verifier storage principle now

- Reference: `accounts-261029/ADR-D34`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-1` through `accounts-261029/REQ-3`,
  `accounts-261029/REQ-10`, and the current one-way storage fixed constraint.
- Related decisions: `accounts-261029/ADR-D8`, `accounts-261029/ADR-D22`,
  `accounts-261029/ADR-D26`, `accounts-261029/ADR-D33`.

**Decision**

If a user/client retains the original input and the server only needs to verify
it, store a one-way verifier. Do not make such inputs recoverable merely for
convenience. Choose verification appropriate to input entropy and existing
security authority; low-entropy passwords continue to use D26's bcrypt.

When an authorized server operation must later retrieve, use or return the
original secret, retain protected recoverable material through the common
cryptographic boundary. D22's latest-token recovery needs that representation
in addition to its hash-based matching; ordinary passwords do not.

**Consequences and future recovery**

The storage principle applies immediately, not only after a key-management
feature exists. Requirements for raw-value retrieval must be explicit at each
storage boundary. Do not assume hashing provides offline-guessing resistance for
low-entropy inputs.

A future key-change/recovery system may discard an unreconstructable verifier
and require the user/client to reset, re-enter or register the original value
again. A verifier independent of the affected key remains valid unless another
authorized security event requires invalidation. Reversible secrets are
candidates for future re-encryption. This distinction does not expand the
current cycle to new integrations, key replacement or additional reset flows.

### D35. Prepare schema through one Job supporting Helm and Argo CD

- Reference: `accounts-261029/ADR-D35`
- Accepted: 2026-10-29 KST by the requester, explicitly requiring Argo CD support.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D15`, `accounts-261029/ADR-D28`,
  `accounts-261029/ADR-D33`.

**Decision**

The chart supplies one pre-deployment Job invoking the central binary's explicit
schema/installation-preparation command. Direct Helm uses pre-install/pre-upgrade
hooks; Argo CD uses PreSync execution. Application rollout follows successful
completion. Local operators can invoke the same CLI directly; regular server
startup remains read-only with respect to schema and installation-key binding.

The database and fixed encryption-key Secret are prerequisites. Do not generate
or replace a key in the Job, or depend on ordinary chart resources that will only
be created in a later deployment phase. Serialize access to the installation
database and make repeat preparation a no-op for already applied migrations and
the same key binding.

**Controller semantics and failure behavior**

Use one Job template with explicit Helm and Argo CD hook/deletion annotations,
so each controller has an unambiguous supported path. Argo CD does not distinguish
Helm install from upgrade, and explicit Argo hooks cause its Helm-hook mapping to
be ignored. Treat each full sync as an opportunity to run the same idempotent
preparation command, not as two separate migrations.

Preparation failure blocks the dependent rollout and retains useful failure
evidence. A later application failure does not undo committed schema changes.
The Job does not stop already running old replicas; schema compatibility or an
explicit planned maintenance boundary remains necessary.

Argo CD selective sync does not execute hooks and is not the supported schema
rollout path. Use full application sync; server readiness still rejects
unprepared schema if an operator bypasses the required preparation path.

**Verification**

Verify both controllers independently: initial deployment, repeated upgrade/sync,
preparation failure blocking new application resources, missing prerequisites,
fixed-key mismatch, concurrent preparation and Job cleanup/recreation. Render
checks do not count as live Helm/Argo CD execution. Any real cluster operation
still requires the agreed namespace, exposure and data scope.

### D36. Support local loopback HTTP and externally exposed HTTPS

- Reference: `accounts-261029/ADR-D36`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-11`, `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D6`, `accounts-261029/ADR-D11`,
  `accounts-261029/ADR-D30`, `accounts-261029/ADR-D31`.

**Decision**

The general and Admin web applications bundled with the central CLI support
plain HTTP when their public origin and listening address are loopback-only.
External public web origins use HTTPS, with deployment-managed TLS termination.
This is the same web application, not another desktop app or product mode.

Derive cookie policy from validated public-origin and listener configuration,
not solely from `NODE_ENV`. Local HTTP retains HttpOnly, uses SameSite=Lax,
omits Secure and uses a host-only name without a `__Host-` prefix. HTTPS uses
Secure and the selected reference cookie boundaries. Distinguish general/Admin
cookie names even when the apps use different ports on the same host.

Reject public non-loopback HTTP and loopback-HTTP configurations listening on
non-loopback interfaces. Explicitly configured HTTPS behind a reverse proxy may
use ordinary internal HTTP between proxy and Node, without treating forwarded
headers as configuration authority.

**Rationale and verification**

The built Next.js standalone app runs in production mode even on a local
installation. Local use must work without a separately provisioned certificate,
while external credentials remain protected by the HTTPS deployment boundary.
Do not depend on browser-specific Secure-cookie exceptions for localhost.

Test localhost, IPv4/IPv6 loopback, default-port normalization, separate app
cookies, public HTTP rejection, reverse-proxy HTTPS cookies and exact Origin
checks. No certificate automation, public insecure-HTTP fallback or new key
management is introduced.

### D37. Terminate external gRPC TLS at the deployment proxy

- Reference: `accounts-261029/ADR-D37`
- Accepted: 2026-10-29 KST by the requester.
- Requirements: `accounts-261029/REQ-9`, `accounts-261029/REQ-11`,
  `accounts-261029/REQ-13`.
- Related decisions: `accounts-261029/ADR-D3`, `accounts-261029/ADR-D7`,
  `accounts-261029/ADR-D31`, `accounts-261029/ADR-D35`, `accounts-261029/ADR-D36`.

**Decision**

An operator-managed Ingress/reverse proxy terminates TLS for external gRPC.
The backend serves plaintext HTTP/2 gRPC on separate general/Admin ports.
Local execution uses loopback; distributed deployment permits plaintext hops
only within an explicitly trusted internal network. Do not expose backend
plaintext ports directly to public clients.

Remote clients validate the external endpoint's certificate and hostname.
The proxy forwards native gRPC over HTTP/2 while preserving authorization
metadata, status/trailers, cancellation and deadline semantics. Web-to-backend
traffic may use a private Service when both endpoints are inside the trusted
network; otherwise it uses the TLS edge.

**Rationale and alternatives**

This keeps certificate issuance, storage and renewal in existing deployment
infrastructure rather than adding a Rust server certificate lifecycle. Direct
Tonic TLS would encrypt the backend hop but require certificate/key provisioning
at each backend; the requester selected edge termination and accepted the
internal-network trust boundary.

Public TLS does not encrypt the proxy-to-backend hop or isolate workloads within
the trusted network. Operators must restrict access accordingly. This choice
does not claim end-to-end encryption, add mTLS/service mesh/certificate automation,
or weaken per-request session and current Admin authorization.

**Verification**

Verify external TLS validation, HTTP/2 forwarding, gRPC status/trailers and
authorization metadata, private backend exposure, and separate general/Admin
routing. Use a deployment-owned endpoint with explicit authorization for live
tests. Rendered ingress configuration alone does not establish these outcomes.

## D38 — Operator secret configuration simplification

Accepted 2026-10-30 by the requester's explicit implementation instruction after
the consolidated feedback brief. This refines the unimplemented deployment
contract; fixed key bytes, binding and setup transaction authority do not change.

- Rename the fixed server configuration to `CHAPCHOO_SECRET_KEY` and chart
  `secretKey`. Remove the prior configuration name rather than adding aliases.
  The requester subsequently clarified that Home's external path is the shared
  server scope `/chapchoo/SECRET_KEY`, not an authentication-only folder.
- Home keeps PostgreSQL PASSWORD as its sole external source and derives the
  password-bearing URL in its ExternalSecret template, with URI userinfo escaping.
- Without an explicitly supplied existing setup Secret, a distinct chart hook
  generates32 random bytes as43-character URL-safe proof, creates a persistent
  named Kubernetes Secret once and reuses/validates it on later deployments.
  No Helm render randomness, cluster lookup or Secret value in Git/render/logs.
- A dedicated bootstrap service account has get on the one named Secret and
  namespace-scoped Secret create only. Kubernetes RBAC cannot resource-name
  restrict create; no list/update/patch/delete is granted. Only that Job mounts
  its API token. The schema prepare Job retains its existing DB/key-only role.
- Order bootstrap RBAC before bootstrap Job before schema preparation in both
  Helm hooks and Argo PreSync. Concurrent create failure may accept only a
  separately verified existing valid Secret, never replace it or replay writes.
- Persist the proof independently of hook ownership; deleting a Job/release does
  not delete it. Existing DB setup consumption remains the final one-time gate.

Alternatives rejected: manual proof in Infisical burdens first installation;
random Helm render introduces Argo drift; `lookup` depends on rendering access;
mutating an existing proof risks invalidating operator-visible setup material.
The bootstrap uses pinned architecture-matched kubectl shipped only in the
backend image, not a shell-built HTTP client or a web image credential authority.
