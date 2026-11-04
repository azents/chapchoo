---
title: "Provider Authentication Decisions"
created: 2026-11-04
tags: [provider, authentication, security, bootstrap]
document_role: primary
document_type: adr
snapshot_id: provider-261104
---

# Provider Authentication Decisions

- Snapshot: `provider-261104`
- Requirements: [provider-261104/REQ](../requirements/provider-261104-authentication.md)
- Mode: Collaborative.
- Material decision owner: requester.

## Fixed Inputs

The confirmed Requirements already establish the following. They are inputs to
decisions below, not alternatives awaiting another approval.

- Bootstrap is a registration context for server ownership. Later server-owned,
  user-owned and team-owned registration is capability-only in this snapshot.
- Authentication exchanges for Provider-scoped operational credentials.
- Default same-cluster Kubernetes bootstrap uses TokenReview, the expected
  audience and an explicitly permitted reviewed ServiceAccount identity.
- The root key stays server-side. Protected bootstrap Provider credentials bind
  a unique Provider identity and random salt. The allowed-ID list is bootstrap
  configuration, with no separate enabling boolean.
- Later registered Providers use independent random per-Provider tokens. Root-key
  encryption protects database storage, not random-token generation.
- Authentication method and execution substrate are independent. Trust and
  root-key-protected authentication can be prohibited by the server.
- Ordinary affected product settings belong to database/Admin authority;
  bootstrap identity/trust settings remain deployment inputs.
- Existing data, native effects, ownership and Connector authority are preserved.
- `REQ-9`, added by the requester before dependent mechanisms, requires
  Kubernetes-specific explicit opt-in, disabled by default. Once enabled, chart
  defaults provide installation/authentication without additional mandatory
  Provider values after common installation bootstrap prerequisites are supplied.
- `REQ-10` separates distributed and Standalone bootstrap. Distributed protected
  credentials use explicit issuance automated by Helm; Standalone defaults to
  Trust on a process-internal endpoint. Docker Helm deployment is out of scope.

## Current-System Evidence

Inspected base: `4c20865baa8c84d9d5c14182101840f960afca2d`.

- [Provider key lifecycle](../spec/domain/computer-lifecycle.md) and
  `rust/crates/persistence/src/component_keys.rs`: current defaults are 24-hour
  validity, proactive rotation after 12 hours and previous-key overlap of at most
  five minutes, capped by the old key's expiry. Explicit key revocation has no
  overlap, discards protected recovery and withdraws the Provider route.
- `rust/apps/chapchoo/src/provider.rs`: the current Provider holds the configured
  owner bearer, uses it for metadata configuration and key issuance, and
  proactively renews under owner authority. Operational keys do not self-rotate.
- `rust/crates/rpc/src/control_streams.rs` and
  [Control admission](../spec/domain/computer-control-rpc.md): heartbeat and
  current-source checks validate operational authority; native effect ownership
  is not cancelled merely because the transport closes.
- [Deployment boundaries](../spec/domain/deployment.md): the current chart uses
  a preexisting owner Secret for Control/Provider. The proposed SA path replaces
  that requirement rather than adding a second mandatory shared secret.
- [Kubernetes SA authentication](https://kubernetes.io/docs/concepts/security/service-accounts/#authenticating-in-code):
  TokenReview observes bound-object invalidation, while offline JWT verification
  alone can accept deleted-object tokens until expiry. Neither automatically
  invalidates separately issued Chapchoo operational credentials.
- `rust/apps/chapchoo/src/control_plane.rs` and `runtime.rs` already use
  `memory_transport` for local peer delivery, but that peer listener is not the
  Provider enrollment listener. A process-internal Provider Trust entrypoint
  must be composed explicitly; the existing internal peer path is not evidence
  that Provider Trust is already implemented.

## Decision Map

| Decision | State | Consequence |
| --- | --- | --- |
| ADR-D1 | Accepted | Operational validity, re-authentication and bootstrap-policy withdrawal latency |
| ADR-D2 | Accepted | Trusted-cluster/SA identity binding, review failure handling and bootstrap wiring |
| ADR-D3 | Accepted direction; lifecycle elaborated in Design | Distributed credential issuance; no Standalone file handoff or Docker Helm deployment |
| ADR-D4 | Accepted | Standalone process-internal Trust default; no implicit network Trust |
| ADR-D5 | Accepted | Existing operational credential cutover and current-auth removal; owner-scope capability remains fixed by Requirements |

Cryptographic library selection, equivalent local types, file layout and fixture
organization are agent-owned details only where they introduce no new authority,
state lifetime or behavior. Mechanisms affecting those boundaries return here.

## ADR-D1. Operational Authority After Bootstrap Authentication

Related Requirements: `provider-261104/REQ-3`, `REQ-4`, `REQ-8`.

### Question

How soon must withdrawal of bootstrap trust stop an already authenticated
Provider, without verifying external bootstrap evidence for every work message?

### Options

1. Retain current long-lived operational defaults and re-authenticate on renewal.
   Least lifetime change, but bootstrap withdrawal can leave up to a day of
   existing authority. Native explicit operational-key revocation remains faster.
2. Use bounded short-lived operational authority and re-authenticate for renewal.
   Recommended proposal: 15-minute validity, proactive renewal after five
   minutes. Renewal uses the original selected method and current server policy,
   never just the operational key. No implicit method fallback. Existing keys
   remain usable only through their current expiry when re-authentication is
   unavailable or denied; neither result extends authority. Immediate explicit
   operational-key revocation remains a distinct existing capability.
3. Revalidate bootstrap policy/evidence on every heartbeat/admission, or build a
   separate immediate policy-withdrawal broadcast/watch path.
   Faster withdrawal but adds verifier availability coupling or another
   distributed invalidation mechanism to the normal connection path.

### Proposal Presented Before Decision

Option 2 was the recommendation presented for requester choice. It makes the bound
explicit: after the changed bootstrap policy is applied to all relevant Control
instances, an existing operational key can survive for at most 15 minutes.
SA object invalidation must be observed at re-authentication; that is not a
promise that already-issued operational keys disappear when a Pod is deleted.
SA tokens are re-read for re-authentication so projected-token rotation is usable.

The exact treatment of prior-key overlap, limiting expiry to underlying proof
expiry, exchange-result recovery and verifier retry budgets must be specified
before final Design approval; this proposal does not silently accept those
independent lifetime details.

### Outcome

Accepted by the requester on 2026-11-04 (KST): option 2, presented as option B.

- Operational validity is 15 minutes; proactive re-authentication starts five
  minutes after issuance. These are product defaults, not new mandatory inputs.
- Renewal verifies the original authentication method and current permit policy.
  An operational token alone cannot extend its authority.
- Verification rejection or outage does not issue a replacement token, extend
  the existing expiry, or permit automatic fallback to another method.
- Once policy withdrawal has reached every relevant Control instance, a
  previously issued token has at most 15 minutes of remaining authority.
- Explicit operational-key revocation remains separately available without a
  grace overlap.
- Authority expiry/revocation stops admission of new commands, reports and
  connections. It neither proves a started native effect was cancelled nor
  authorizes replay of an uncertain effect.

The residual implementation-lifetime details identified above remain explicit
Design obligations; they cannot weaken the accepted maximum authority bound.

## ADR-D2. Kubernetes Bootstrap Identity Binding

Related Requirements: `provider-261104/REQ-2`, `REQ-4`, `REQ-8`.

### Question

Does a permitted ServiceAccount denote its stable name within the trusted cluster
and namespace, or one immutable Kubernetes object UID that must be rebound after
recreation?

### Fixed Verification Behavior

TokenReview and the expected audience are already confirmed. ADR-D1 determines
failure behavior: no successful review means no new operational token, without
extending existing authority or selecting a fallback.

### Options

1. Bind the configured local trusted-cluster context and reviewed ServiceAccount
   username (namespace/name), with the expected Provider-authentication audience,
   to one permitted Provider ID. The chart supplies this bootstrap mapping.
   TokenReview still validates object/token UIDs according to Kubernetes rules;
   Chapchoo does not independently pin a historical SA UID. Recreating the SA
   under the permitted name can authenticate fresh tokens. Authority to create
   Pods using that SA or recreate the SA in that namespace is therefore within
   the trusted deployment administration boundary.
2. Include a specific SA UID in the permitted binding. Recreation with the same
   name is denied until bootstrap trust is updated to the new UID. This provides
   object-instance pinning but introduces an installation/recreation dependency
   on a server-generated object identifier. First-seen automatic UID binding is
   not equivalent to this policy and is not implicitly proposed.

### Recommendation and Proposed Wiring

Option 1 is recommended for ordinary chart bootstrap. Bind the tuple of trusted
local verifier context, namespace, SA name and expected audience to Provider ID.
Do not select a verifier URL or fetch trust material from unverified JWT claims.
Foreign-cluster federation management remains outside this snapshot; another
permitted authentication method remains usable for external Kubernetes execution.

The chart would project a dedicated Provider-authentication-audience token into
the Provider Pod. Native Kubernetes resource operations retain their API-audience
credential; the authentication audience does not grant Kubernetes API access.
Only the Control workload receives the necessary cluster-scoped
`tokenreviews.create` permission and API credential for verification. The Provider
does not receive review permission, and no process receives a newly shared owner
secret or the installation root key.

The server uses the authenticated TokenReview result and returned audiences, not
an unverified subject from locally decoded input, for the permit mapping. An empty
or mismatched authenticated result cannot create a Provider. TokenReview outages
remain verification unavailability, distinct from a valid reviewed-but-forbidden
identity; neither issues new operational credentials.

### Outcome

Accepted by the requester on 2026-11-04 (KST): option 1 and the proposed Helm
bootstrap wiring.

- Permission binds the trusted local cluster context, reviewed namespace/SA name
  and expected audience to the permitted Provider ID. No historical SA UID is
  pinned by Chapchoo; valid credentials from a recreated same-name SA are eligible.
- Kubernetes TokenReview still performs its object/token identity validation.
  Unverified input never chooses a trusted verifier or establishes an identity.
- The chart supplies the name-based permission mapping, dedicated projected
  authentication-audience token and Control-only review permission. Provider
  native API credentials and the authentication audience remain distinct.
- Review outage or failed/forbidden identity grants no new operational token and
  does not extend existing authority or cause method fallback.
- Deployment authority to use/recreate the permitted SA is explicitly within the
  trusted namespace administration boundary.

### Subsequent Requirement Addition

The requester then explicitly required convenient defaults after
Kubernetes-specific opt-in (`REQ-9`), not implicit or generic Provider enablement.
The accepted bootstrap wiring must therefore be supplied by coherent chart
defaults rather than newly mandatory Provider-specific operator values.
Specific render mechanisms are to be recorded in Design against this requirement.

## ADR-D3. Bootstrap Deployment Context and Credential Issuance

Related Requirements: `provider-261104/REQ-2`, `REQ-5`, `REQ-8`.

### Question

How do distributed and Standalone bootstrap supply the initial Provider
authentication authority without distributing the installation root key?

### Fixed Credential Meaning

The Provider receives its ID and root-key-protected credential, never the root
key. Purpose and Provider identity must be integrity-protected along with the
random salt. The server validates the protected contents and its bootstrap
allowed-ID list. It does not need a per-credential external shared secret.
This credential is a reusable authentication secret, distinct from the
15-minute operational token exchanged using it.

### Alternatives Considered and Requester Correction

The initial proposal compared automatic same-host file export with explicit
operator credential issuance. The requester rejected the premise that
Standalone needs a separate local credential handoff: its Provider endpoint is
inside the server process. The explicit issuance alternative applies to
distributed components, with Helm automating the operation.

### Outcome

Accepted direction from the requester on 2026-11-04 (KST), after updating
`REQ-10` and the Docker chart non-goal:

- Distributed protected-credential bootstrap uses a server-side explicit
  issuance operation; Helm automates issuance/delivery rather than requiring
  operators to mint and copy a new secret manually.
- Default Kubernetes Helm authentication remains SA/TokenReview. A protected
  credential is not additionally issued for that default path.
- Standalone uses the process-internal default Trust path in ADR-D4, not an
  automatic shared-directory credential exporter.
- Docker Provider Helm deployment support is not included in this snapshot.
- Provider processes never receive the root key. A trusted issuance operation
  using that key is server-side authority, not the Provider runtime.
- This does not authorize an unauthenticated credential-download endpoint.

### Remaining Issuance and Revocation Obligations

A stateless verifier based on root key and allowed Provider ID cannot distinguish
two independently minted valid credentials for the same ID. Reissuing a file
therefore does not revoke a stolen old credential. The existing permitted action
is removal of that ID from bootstrap trust, with ADR-D1's bounded operational
expiry. This direction adds no database credential registry or root-key-rotation
feature to create per-artifact revocation authority.

The exact Helm issuance trigger, idempotent reuse/replacement, output ownership
and restricted delivery, upgrade/rollback behavior and failure handling remain
Design/decision obligations. The chart must not mint hidden credentials for
disabled or unsupported Providers merely to exercise this capability.

## ADR-D4. Standalone Trust Boundary

Related Requirements: `provider-261104/REQ-3`, `REQ-6`, `REQ-10`.

### Outcome

Accepted from the requester on 2026-11-04 (KST): Standalone defaults to Trust
because the Provider endpoint is only available inside the server process.

- Internal composition supplies the eligible Provider identity. A network
  request cannot establish this identity by claiming to be local.
- The internal path exchanges for scoped operational tokens and retains ADR-D1
  validity/renewal, current-source checks and native-effect ownership semantics.
- Public/component/peer TCP listeners do not acquire Trust merely because
  Standalone is selected or a request comes from loopback/private networking.
- Trust remains prohibitable by server policy. Explicit prohibition is not a
  signal to silently expose an alternate listener or switch authentication.
- No distributed network Trust endpoint is authorized by this decision; such a
  boundary would require a separate explicit decision if later requested.

## ADR-D5. Existing Operational Credentials at Authentication Cutover

Related Requirements: `provider-261104/REQ-3`, `REQ-7`, `REQ-8`.

### Fixed Capability Scope

The requester confirmed testable server/user/team-scoped random-token
authentication/exchange capability without complete new registration products.
No extra production enrollment API or registration UI is authorized by this
decision. Existing ownership and accepted native identities must be preserved.

### Question

At the upgrade boundary, may existing long-lived operational Provider credentials
retain their original expiry, or must they be invalidated and replaced through
the newly authorized authentication path?

### Options

1. Perform a coordinated cutover: stop old Control/Provider processes, invalidate
   old Provider operational credentials and routes while retaining Provider and
   Computer/resource/Connector records, and start the new mutually compatible
   server/Provider composition. New registration authentication resolves the
   existing permitted Provider identity without creating another Provider.
   The maintenance interval temporarily loses Provider management connectivity.
   Native processes/effects are not assumed stopped by credential invalidation.
2. Accept old operational credentials until their original expiry, while removing
   the old owner-bearer issuance method. This reduces immediate reconnections but
   retains an old-credential acceptance path and can preserve up to 24 hours of
   authority outside the new 15-minute rule.

### Recommendation

Option 1 is recommended: data-preserving coordinated authentication cutover, not
a destructive Provider reinstallation and not a legacy dual-authentication
window. Existing Connector authority must not be revoked just because Provider
management credentials change.

The existing explicit key revocation implementation clears protected recovery,
previous-key overlap and Provider routes under scoped authority. This is evidence
for which authority records are separate, not authorization to call live
revocation or blindly reuse that operation inside a schema migration.

Design must specify the atomic migration boundary, multi-Control drain/start
ordering, rollback limitations and uncertain native-effect recovery. Restart
cannot fabricate a never-dispatched proof or replay native effects.

### Outcome

Accepted by the requester on 2026-11-04 (KST): option 1, presented as option A.
This is approval of the coordinated transition design, not execution.

- Stop old server Control and Provider owners before the preparation/cutover.
- Invalidate existing Provider operational current/previous keys, protected
  recovery and Provider routes without deleting Provider/Computer/resource
  identities or Connector authority.
- Restart mutually compatible new components and re-authenticate the same
  permitted Provider identities through the new methods.
- Do not keep an old-owner-bearer fallback or a 24-hour old-key acceptance window.
- Accept a temporary Provider-management interruption. Do not describe it as
  zero downtime or as termination of every native process.
- Actual maintenance, rollout, migration and production writes require their
  own explicit authorization.
