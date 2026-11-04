---
title: "Provider Authentication Requirements"
created: 2026-11-04
implemented: 2026-11-04
tags: [provider, authentication, security, bootstrap]
document_role: primary
document_type: requirements
snapshot_id: provider-261104
---

# Provider Authentication Requirements

- Snapshot: `provider-261104`
- Document reference: `provider-261104/REQ`

## Problem

Provider authentication currently requires separately supplied system-owner
credentials even for Providers installed with the server. Adding functionality
must not continually add mandatory operator-supplied secrets, distribute the
installation encryption root to Providers, or make deployment configuration a
second authority for ordinary product settings. Registration context, ownership,
execution substrate and authentication method need distinct meanings.

## Primary Context

### Primary System Outcome

A deliberately permitted bootstrap Provider authenticates without possessing the
installation root key, obtains Provider-scoped operational authority and connects
through the existing Control boundary. The standard same-cluster Kubernetes
installation needs no manually provisioned Provider connection secret. Other
permitted bootstrap methods serve their declared deployment contexts without
silently weakening authentication.

## Supporting Scenarios or Effects

- An operator can exclude a bootstrap identity or authentication method through
  the server's bootstrap policy.
- The authentication foundation can later support server-owned Providers added
  after startup and individually user-owned or team-owned Providers without
  treating their credentials as bootstrap proofs.
- Kubernetes and Docker remain execution substrates, not exclusive selectors of
  authentication method; external Kubernetes Providers can use another permitted
  method.

## Goals

- Deliver the authentication and deployment design for existing bootstrap
  Kubernetes and Docker Providers.
- Separate initial/re-authentication from Provider operational communication.
- Preserve least privilege, data and native-resource ownership boundaries.
- Bound future registration work to reusable, verifiable capability in this
  snapshot rather than complete new management products.

## Non-Goals

- Complete post-start server-owned or user/team Provider registration workflows,
  registration/credential-management web screens or public management entrypoints.
- Moving every existing Chapchoo setting to DB/Admin in this snapshot; the
  bootstrap-versus-product-setting principle applies to the affected Provider
  authentication design.
- New Computer substrates, execution behavior, external-cluster federation
  management or unrestricted public Trust authentication.
- Helm deployment support for Docker Providers.
- Implementation or production deployment as part of the design request.

## Requirements

### REQ-1. Explicit registration context and ownership

Bootstrap registration and later server-managed registration have different
admission policies. A bootstrap Provider is server-owned; bootstrap is its
registration context rather than an additional tenant owner.

**Acceptance criteria**

- Authenticated registration resolves one Provider identity and authoritative
  ownership; request assertions cannot select a different owner.
- Bootstrap identity cannot take over an existing Provider belonging to another
  owner or replace its accepted native substrate/daemon binding.
- Capability coverage distinguishes bootstrap server ownership, later server
  ownership, user ownership and team ownership without making team membership an
  implicit grant of server-wide authority.

### REQ-2. Minimal bootstrap inputs and one product-setting authority

The installation root key, database access and identity/trust inputs needed to
bootstrap components remain deployment inputs. Other affected product settings
belong in the database and are manageable through Admin when their management
feature is in scope.

**Acceptance criteria**

- The default same-cluster Helm installation requires no operator-generated
  Provider connection token and provisions the bootstrap identity wiring.
- The SECRET_KEY authentication allowlist is bootstrap configuration, not an
  Admin-editable database allowlist.
- Bootstrap configuration does not continuously overwrite ordinary database-owned
  settings during server startup or upgrades.
- Future management capability does not introduce mandatory environment secrets
  for each additional registered Provider.

### REQ-3. Authentication followed by operational-token exchange

A successful supported authentication method grants operational credentials
restricted to the authenticated Provider. Registration authentication and ongoing
communication are distinct capabilities.

**Acceptance criteria**

- No operational token is issued for a failed, disallowed or mismatched identity.
- Operational credentials cannot register or manage a different Provider or act
  as an owner/root credential.
- The design specifies re-authentication, renewal, expiry, disablement and the
  effect of policy changes on already-issued tokens and live connections.
- Authentication outage or rejection does not authorize fallback to a weaker
  method or replay of uncertain Computer effects.

### REQ-4. Same-cluster Kubernetes ServiceAccount authentication

The standard Kubernetes bootstrap authenticates an explicitly permitted workload
identity for the intended Provider-authentication audience, using TokenReview
against the configured trusted cluster before trusting the supplied identity.

**Acceptance criteria**

- Provider identity is authorized from the reviewed ServiceAccount identity in
  its trusted cluster context, not merely an unverified JWT subject.
- Unexpected audience, unpermitted ServiceAccount/namespace, failed review,
  expired/invalid credentials or an unavailable verifier do not grant authority.
- Possessing any valid ServiceAccount token is insufficient to register an
  arbitrary Provider.
- Provider and server permissions are limited to their respective verification
  and execution responsibilities; neither requires exposing the root key.
- Kubernetes execution outside the default cluster is not forced to use the
  local-cluster ServiceAccount authentication path.

### REQ-5. Root-key-protected bootstrap Provider credentials

SECRET_KEY-based bootstrap uses a Provider-specific credential protected by the
server root key and bound to that Provider's unique identity and random salt.
The Provider never receives the root key itself.

**Acceptance criteria**

- Authentication requires both a valid identity-bound credential and membership
  in the server's bootstrap allowed-ID list.
- An empty list admits no identities through this method; no separate enabling
  flag is required.
- A credential for one Provider cannot authenticate another identity, and
  modification cannot produce an accepted credential.
- Credential creation/provisioning and root-key replacement consequences are
  defined without a newly mandatory independent operator-supplied shared token.
- A local Docker bootstrap can select this method; substrate selection does not
  implicitly authorize it.

### REQ-6. Explicitly bounded Trust authentication

Trust is a bootstrap admission policy with a defined trust boundary, not the
result of another method failing. Standalone uses Trust by default on its
process-internal Provider endpoint.

**Acceptance criteria**

- The server can prohibit Trust; prohibited requests receive no operational
  authority.
- The design explicitly defines allowed identities, entrypoint/exposure boundary
  and default policy before implementation.
- Trust does not allow arbitrary network callers to claim a server-owned
  Provider, and is never an implicit fallback from SA or protected credentials.
- Standalone's default Trust admission is reachable only within the server
  process. Loopback networking or a caller's claimed local address is not an
  equivalent boundary.

### REQ-7. Capability-only later server/user/team registration

Retain an extensible authentication and ownership foundation for additional
registration contexts without implementing their complete product workflows.

**Acceptance criteria**

- Later server-managed, user-owned and team-owned Provider credentials are
  independent cryptographically random per-Provider tokens, not identity- or
  root-key-derived bootstrap credentials.
- Such registration tokens are protected with SECRET_KEY when persisted in the
  database; storage protection is distinct from token generation.
- The core capability can establish the correct Provider/owner scope and exchange
  a valid scoped credential for operational authority in controlled tests.
- Cross-Provider and cross-owner credential use is rejected, including team scope.
- No new end-user/Admin registration UI, complete credential-issuance workflow or
  external-cluster trust-management UI is required by this snapshot.
- Schema/interface representation and the exact extension boundary are defined
  in Design, not assumed to require unused production endpoints.

### REQ-8. Preserve existing data and execution authority

Changing authentication does not recreate Providers, Computers, saved storage or
accepted native-resource identities and does not bypass existing lifecycle fences.

**Acceptance criteria**

- The design accounts for existing Provider records, owner credentials,
  operational credentials and reconnection without deleting owned data.
- Old authentication/configuration paths have an explicit replacement/removal
  boundary rather than an unrequested legacy fallback.
- Key revocation, current-connection checks and Connector-versus-Provider authority
  remain enforced unless a separately approved decision changes their mechanism.
- Credentials, root keys and raw SA tokens remain absent from public views,
  routine logs and diagnostic output.

### REQ-9. Explicit Kubernetes Provider enablement with convenient Helm defaults

For an installation with the common database/root-key prerequisites already
configured, explicitly enabling the Kubernetes Provider is sufficient to install
and authenticate it without additional mandatory Provider-specific values.
Starting the server or enabling generic Provider capability does not implicitly
enable the Kubernetes execution substrate.

**Acceptance criteria**

- A Kubernetes-specific opt-in switch is required and defaults to disabled.
  The final value path is a Design detail; a substrate-neutral switch alone is
  not a substitute for explicitly selecting Kubernetes.
- Once explicitly enabled, the chart provides a coherent default Provider
  identity/name, ServiceAccounts, expected audience, server permission mapping,
  TokenReview permission, isolated managed namespace, process/execution images
  and resource/storage policy.
- The chart creates the default managed namespace instead of requiring an
  operator to discover and precreate it. Default execution resources remain
  separate from database/root/owner credentials.
- Provider and execution image defaults reuse the installation's selected
  compatible backend image rather than demanding separately maintained pins.
- Supported standard cluster prerequisites, such as existing usable storage and
  required API permissions, are documented. Unmet prerequisites are explicit
  failures, not successful deployment or implicit weaker authentication.
- Operators can override defaults for a deliberate advanced deployment without
  making those overrides prerequisites for the standard installation.
- A reusable public chart does not embed private deployment identities or hosts.
- Default bootstrap wiring does not acquire authority over subsequent
  database-owned ordinary product settings.

### REQ-10. Distinct distributed and Standalone bootstrap

Bootstrap supports two deployment contexts rather than assuming every Provider
is a separate local process requiring credentials copied from the server.

**Acceptance criteria**

- Distributed components authenticate across their process/network boundary.
  When protected bootstrap credentials are selected, an explicit server-side
  credential-issuance operation can be automated by Helm rather than requiring
  manual secret generation and copying.
- The default Kubernetes chart path continues to use ServiceAccount
  authentication. Automated protected-credential issuance is not a reason to
  switch that default or add Docker Provider deployment templates.
- Standalone Provider authentication is process-internal and defaults to Trust,
  without a separate credential-file handoff for that internal connection.
- Standalone Trust still exchanges for Provider-scoped operational authority;
  it does not bypass common connection, expiry or ownership checks.
- The server can prohibit Trust as previously required. Starting an external
  listener does not implicitly inherit Standalone's internal trust.
- A convenience same-host automatic-export directory is not required by this
  snapshot; it is not a substitute for the two confirmed deployment contexts.

## Fixed Constraints

- The requester-selected flow is authentication followed by operational-token
  exchange; methods are not reimplemented independently in every work message.
- Default same-cluster Kubernetes bootstrap uses reviewed ServiceAccount identity
  and explicit audience authorization.
- SECRET_KEY remains server-side. Its method's allowed Provider IDs are bootstrap
  policy and its protected credentials are distinct from later random tokens.
- Minimal external bootstrap inputs and DB/Admin-owned product settings are
  complementary, not mutually exclusive rules.
- Existing SQLite/PostgreSQL support and data-preserving upgrade obligations
  remain applicable.
- Public artifacts remain environment-independent; private installation details
  belong in deployment configuration.

## Open Assumptions and Pending Boundaries

- Capability-only means testable scoped credential authentication/exchange and
  ownership extension points, not merely enum placeholders and not full new
  registration products. The requester confirmed this scope interpretation.
- The corresponding ADR records accepted operational validity/renewal,
  name-based SA bootstrap, distributed protected-credential issuance,
  Standalone-only default Trust and coordinated old-credential cutover.
  Their complete implementation Design still requires revision-bound approval.
- Detailed mechanisms do not authorize silently adding bootstrap settings,
  DB management screens or root-key distribution to Providers. Any newly
  discovered material choice returns to ADR before implementation.

## Confirmation

The requester confirmed the complete Requirements snapshot and the
capability-only boundary on 2026-11-04 (KST), before ADR decision work began.
This confirms the design scope, not implementation or production deployment.
The requester additionally required convenient Helm defaults with an explicit
Kubernetes-specific enablement switch (`REQ-9`) on 2026-11-04 (KST). This addition
and clarification are applied before dependent ADR and Design mechanisms.
The requester further clarified distributed versus Standalone bootstrap
(`REQ-10`), Standalone's process-internal default Trust and exclusion of Helm
Docker Provider support on the same date, before dependent decisions.
