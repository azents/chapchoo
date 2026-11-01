---
title: "Agent and Computer Connection Management Decisions"
created: 2026-11-01
tags: [agent, computer, architecture]
document_role: primary
document_type: adr
snapshot_id: computer-261101
---

# Agent and Computer Connection Management Decisions

Requirements authority:
[computer-261101/REQ](../requirements/computer-261101-agent-connection-management.md).

Mode: Collaborative. The requester owns unresolved material decisions.
Computer Provider and Computer Connector naming are accepted in D1 and D2.
Independent component-initiated server connections are accepted in D3.
Remaining implementation mechanisms are undecided.

## Current-System Framing

Research baseline: `5b1b723979aa1f3bad13283a98680c234656a6bf`.

- Account identity, current permission checks, General/Admin separation and
  authenticated RPC exist in `proto/chapchoo/accounts.proto`,
  `rust/crates/rpc`, `rust/crates/service` and `rust/apps/chapchoo`.
- SQLite/PostgreSQL account and display persistence and explicit preparation
  exist in `rust/crates/persistence`; these do not implement computer operation
  admission or external-result reconciliation.
- General/Admin web, account E2E and RPC listener tests provide reusable
  boundaries, not evidence of Agent or provider behavior.
- The inspected tracked Rust, protocol and product-web implementation has no
  HO/HP component contract or established name to preserve. Binary asset byte
  matches are not naming evidence.
- `infra/charts/chapchoo` packages the existing service; it is not an
  implemented computer provider.

## Decision Map

### Fixed or Existing Outcomes

- One personal Agent has one dedicated cloud computer in this scope.
- Infrastructure-configured authenticated provider registration; read-only
  cloud observation and no independent web provision-enablement switch.
- Computer power, CC connection, CP management and browser observation are
  independent facts.
- Manual power control and saved files/settings retention; no inactivity
  shutdown or promise of live process/memory continuity.
- Normal computer-then-Agent deletion, immediate explicit force deletion,
  provider-specific external cleanup guarantees and Kubernetes best effort.
- Existing account permissions, display behavior, SQLite/PostgreSQL support
  and explicit schema preparation remain applicable.

### Material Technical Decisions

- Accepted: Computer Provider (CP), replacing HO (`D1`).
- Accepted: Computer Connector (CC), replacing HP (`D2`).
- Accepted: independent component-initiated server-facing CP and CC
  connectivity (`D3`).
- Accepted: owner-based CP authentication (`D4`) and independent CP
  authentication keys whose refresh/rotation requires user or system
  authentication (`D5`).
- Accepted: bounded overlap for routine CP key rotation, no grace for
  explicit revocation (`D6`).
- Pending: CP key expiry and active-connection enforcement.
- Accepted current connector flow: provider/user creates an authentication
  session, connector exchanges its initial key and refreshes directly (`D9`).
  D9 refines D7's credential delivery; D8's provider-approved renewal proposal
  was not selected.
- Accepted: reusable initial session key with complete prior operational-key
  invalidation on re-exchange (`D10`).
- Pending: key lifetimes, stable identity and active-connection enforcement.
- Accepted: unary gRPC authentication operations and independent bidirectional
  component control streams (`D11`).
- Superseded proposal: general management-operation journal (`D12`).
- Accepted: current target/observation convergence (`D13`) and provider
  inventory visibility across Agent force deletion (`D14`).
- Pending: stream message contracts, reconnect ownership and uncertain-result
  recovery mechanisms.
- Pending: Kubernetes computer lifecycle and retained-environment mechanism.
- Pending: deletion fencing and remaining-resource cleanup ownership.
- Pending: authorized query/observation contracts and initial/live web state.
- Pending: process/deployment boundaries, rollout and verification support.

These are decision topics, not accepted mechanisms. Each topic requires an
evidence-backed options discussion before acceptance.

### Product Questions and Local Details

- Naming and topology are resolved in D1-D3. No new product-scope question is
  currently blocking the CP authentication discussion.
- Equivalent helper/file layout, fixture composition, local identifiers and
  translation-key names remain Agent-owned after contracts are decided.
  They do not authorize additional modes, authorities or persisted state.

## Naming Exploration Before D1

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-4`,
  `computer-261101/REQ-5`, `computer-261101/REQ-9`, and Terminology.
- Decision owner: requester.
- State: superseded historical proposals; accepted D1/D2 below govern current names.

### Question

Are Harness Provider and the orchestrator terminology appropriate for the
intended responsibilities? Rename a component only if a demonstrated semantic
mismatch outweighs retaining its domain meaning.

### Responsibility Baseline

The requester clarified that a Computer is the environment in which HP runs,
and HP connects that Computer to the server. A physical machine, VM or
container-based environment can satisfy that concept; the name does not
select an isolation mechanism or an HP-process-to-environment cardinality.

The wider HP purpose of providing/managing harnesses is compatible with this
connection responsibility. Neither the connection role nor the future harness
role implies a separate new component. Only the connection and computer
management subset is implemented in this milestone.

The role currently called HO provides Computers and manages their lifecycle.
Orchestration is how it achieves that responsibility, not the resource it
provides. A user-run HP can connect an already available computer without that
computer having been provisioned by HO; that wider product flow is not added
to the current milestone.

### Criteria

- Convey the difference between managing an environment hosting HP and
  providing/managing a harness within that environment.
- Remain intelligible without remembering an acronym.
- Do not imply that model execution or a particular harness is already
  included in this milestone.
- Avoid naming the infrastructure process another unqualified "Agent".
- Keep component naming distinct from choosing binary split, daemon mode,
  commands, protocols or container/storage mechanisms.

### Assessment of the Existing Name

**Harness Provider is appropriate for the intended role.** It identifies what
is offered to the Agent, works for both user-owned and orchestrated execution
environments, and distinguishes the provider from the harness itself and from
the product Agent. It does not require a particular model vendor or harness.
Implementing only its connection boundary in the first milestone is not a
reason to rename the enduring role.

Its limitations are definitional: "provider" can mean a model API vendor or a
catalog-only adapter in other systems. Define HP as the local execution and
management provider, not a model vendor, installer or remote API catalog.
These ambiguities do not currently establish a reason to replace the name.

The orchestrator name is appropriate for environment/HP-instance lifecycle.
The HO abbreviation alone does not establish an approved long-form name.
If "Harness Orchestrator" is used, it can be mistaken for the component
orchestrating harness sessions, which would overlap with HP's responsibility.
"Harness Provider Orchestrator" explicitly identifies HP instances as the
managed subject, at the cost of length and a different natural abbreviation.
That is a naming alternative, not a selected extra component or architecture.

### Revised Recommendation

- The current candidate is **Computer Provider (CP) / Harness Provider (HP)**.
  CP describes the resource offered, rather than the orchestration technique.
  HP connects the resulting environment to the server and retains the wider
  harness-provider purpose.
- This replaces the proposed Harness Provider Orchestrator wording: providing
  a Computer is a clearer boundary than describing only HP-instance management.
- The earlier Computer Runtime replacement for HP remains withdrawn. Runtime
  may be confused with the harness itself or an OCI runtime. The requester's
  explicit connection definition does not by itself request a Connector rename.
- The requester has clarified the concepts, but has not yet accepted CP as
  the formal replacement for HO. Do not treat this clarification as naming,
  transport, binary-layout or component-split approval.

This is a revised proposal, not a requester-accepted naming decision.

### Reference and Limits

The official [Coder architecture documentation](https://coder.com/docs/install/plan/architecture/)
separates infrastructure modification (`provisionerd`) from the process inside
the remote workspace (workspace agent). It is an analogy for different
responsibilities, not evidence that Chapchoo should replace Harness Provider:
the products expose different domain capabilities. The requester-defined
Harness Provider responsibility governs this naming assessment.

### Consequences Awaiting Choice

After the requester chooses names, align Requirements terminology and use that
vocabulary for later interface, configuration and documentation discussions.
Do not choose executable names, command syntax, an additional component split
or a transport solely from this naming choice.

## D1. Computer Provider

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-5`,
  `computer-261101/REQ-9`, and Terminology.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

Use **Computer Provider (CP)** for the component that provides Computers and
manages their lifecycle. This replaces the HO working name. A Computer is the
environment in which the computer-side component runs; that component connects
the Computer to the server.

### Rationale and Alternatives

Computer Provider identifies what is provided. Orchestrator describes a means
of providing/managing the resource, while Harness Provider Orchestrator frames
the responsibility around a subordinate process rather than the Computer.
The requester confirmed Computer Provider independently of the computer-side
component name.

### Consequences

Use CP in current Requirements and subsequent technical decisions. The initial
implementation is a Kubernetes CP; cleanup guarantees remain CP-specific.
This naming choice does not choose a transport, binary, deployment topology,
storage mechanism or additional product capability.

## D2. Computer Connector

- Related authority: `computer-261101/REQ-4`, Terminology, and `D1`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

Use **Computer Connector (CC)** for the component running in a Computer that
connects the Computer to the Chapchoo server. This replaces HP. The requester
accepted the **Computer Provider / Computer Connector** pair.

The requester clarified that this component connects the Computer to the
server and that retaining Harness Provider alone introduces an inconsistent
harness-level term into the Computer-centered model. This changes the basis
of the earlier recommendation to retain HP; it is not a request to change
the component's responsibilities.

### Alternatives Considered

- **Computer Connector (CC), selected:** names the confirmed relationship,
  connecting a Computer to the server. It does not claim to be the Computer,
  the harness or the user-visible Agent. The definition should make clear
  that it is the resident software endpoint, not a passive cable or a separate
  relay. Providing harness access can remain a capability through this
  connection without putting Harness in the component name.
- **Computer Agent:** common language for resident management software and
  broader than connectivity, but conflicts with Chapchoo's existing
  user-visible Agent concept.
- **Computer Daemon:** describes a resident process, but not the relationship
  it provides, and suggests a process/lifetime form not yet selected.
- **Computer Runtime:** describes execution broadly, but is ambiguous with
  both the Computer environment and the harness/runtime inside it.

The argument for Connector now rests on the requester-defined relationship,
not on temporarily limiting the first milestone to connectivity. Future
capabilities must still be explicitly designed; this name neither forbids
them nor adds them to the current scope.

### Consequences

Use CC in current requirements and subsequent technical decisions. Historical
naming exploration retains its original vocabulary as superseded context.
Computer remains the environment in which CC runs. CP provides/manages that
environment; CC connects it to the server. Providing harness access can be a
later capability, without introducing another component or adding harness
execution to this milestone.

This decision does not select binaries, protocols, trust credentials,
deployment topology or process supervision.

## D3. Independent Server-Facing CP and CC Connectivity

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-4`,
  `computer-261101/REQ-5`, `D1`, `D2`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Existing Evidence and Fixed Outcomes

Current account RPC registers Auth and the selected General/Admin service;
there is no implemented CP/CC transport or component authentication contract.
Account-session bearer authentication is existing user authority, not a
preselected credential for these infrastructure components.

REQ-4 distinguishes CP management connectivity from CC service connectivity.
It permits a valid CC connection while CP management is unavailable; it does
not by itself select physical topology. CP's computer provisioning remains
distinct from CC's computer-to-server connection.

### Decision

Both CP and CC initiate independent authenticated connections to the Chapchoo
server. CP is outside the ongoing CC communication path. The requester
confirmed that the connections are separate.

### Options

1. **Independent component-initiated connections (selected).** CP connects
   to the server for management work; CC independently connects from inside
   each Computer. Server communication with CC does not require a running
   CP relay. Server-originated work uses the established communication path
   or component retrieval; this decision does not choose streaming or polling.
2. **CC communication through CP.** CP aggregates/proxies CC traffic and can
   reduce direct server-facing endpoints. This adds intermediary identity,
   multiplexing, backpressure and recovery responsibilities. A single relay
   failure can remove CC connectivity even if the computers remain running.
   A resilient or independently supervised relay is possible but adds another
   boundary; do not silently assume it exists.
3. **Server-initiated component connections.** The server reaches CP/CC
   listeners directly. This can fit a routable private deployment but requires
   inbound reachability, discovery and endpoint protection for components.

### Consequences and Limits

CP and CC each need outbound access to a configured,
authenticated server endpoint, not separately exposed inbound listeners for
the Chapchoo control connection. This refers to logical peers and direction,
not the same port, same process, same credential or a selected RPC protocol.

CP may participate in provisioning/bootstrap later; being outside ongoing CC
traffic does not determine bootstrap authority. A shared network/cluster failure
can still interrupt both connections. Independent logical paths do not promise
independent physical availability or eliminate connection state/scaling work.

The official Kubernetes
[node-to-control-plane communication model](https://kubernetes.io/docs/concepts/architecture/control-plane-node-communication/#node-to-control-plane)
is a reference for component-initiated authenticated connections to a central
API. It does not decide Chapchoo's topology or imply copying Kubernetes
certificates/service-account tokens.

### Deferred Decisions

Transport, TLS termination, machine trust bootstrap, credential scope and
rotation/revocation, connection liveness, routing across server instances,
operation recovery and exact process boundaries require subsequent decisions.
This topology decision does not authorize implementation or new infrastructure.

## D4. Owner-Based CP Authentication and Rotation

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-2`,
  `computer-261101/REQ-10`, and `D3`.
- Decision owner: requester.
- Accepted direction: 2026-11-01 KST.

### Existing Evidence

Existing account authentication uses user-session bearers. The installation
root protects stored credentials and key binding; it is not an implemented
system-login credential or CP identity.
`rust/crates/crypto/src/token.rs` provides random 256-bit bearer generation,
digest verification and redacted secret formatting. These are reusable
primitives, not an implemented machine-authentication or authorization flow.
`rust/crates/persistence/src/sessions.rs` implements user-session renewal,
rotation, previous-token overlap, expiration and revocation checks. These
mechanisms are useful reference points; its persisted sessions currently
belong to users. There is no implemented system-principal session flow.

### Accepted Direction

CPs can be user-owned or system-owned. Their authentication should use the
corresponding owner's authentication with appropriate credential rotation.
The requester supplied this ownership distinction in response to a proposal
to configure individual CP static keys.

Do not impose a separate static-key login model on all CPs. The initial
delivery remains the system-owned Kubernetes CP; recognizing the general
ownership distinction does not add user-owned CP product flows to this
milestone. Neither owner identity nor CP identity should be defined by the
current rotating secret value.

### Superseded Proposal

The earlier uniform per-CP configured bearer-key recommendation was not
accepted. mTLS and static per-CP keys were considered before the owner-based
direction was clarified; neither was selected. User authentication must not
be ruled out merely because CP is an infrastructure component.

### Boundaries and Remaining Decisions

- Authentication of the owner and authorization for a particular CP are
  distinct checks. Self-declared ownership does not authorize a provider.
- Whether CP directly uses an owner session or exchanges owner authentication
  for a CP-scoped session is not determined by this direction.
- System authentication bootstrap is still undefined. Do not expose the
  installation encryption root or invent a system-user account to fill it.
- Rotation requires an explicit contract for delivery acknowledgment,
  concurrent renewal, bounded overlap, expiration, revocation and active
  connection enforcement. Existing user-session timings are not CP policy.
- Revocation of authentication and deletion of the CP's computers are separate
  outcomes; no automatic resource-deletion policy is added here.
- CC remains independently connected under D3. Its authentication requires a
  separate decision; owner/CP secrets are not implicitly shared with CC.
- No credentials are generated, requested or configured during this design.

### D4 Clarification Recorded with D5

The requester clarified that system authentication already exists in the
agreed design as the authentication configured when connecting a system CP.
The earlier statement that its bootstrap was undefined confused an incomplete
implementation with missing design authority. No new system-session login
scheme is required merely because the inspected account code does not implement
this flow. D5 records the exact separation of CP key use and renewal authority.

## D5. Independent CP Keys with Owner-Authenticated Refresh and Rotation

- Related authority: `D4`, `computer-261101/REQ-1`,
  `computer-261101/REQ-10`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Decision

CP uses a separate, independent authentication-key system. A CP key
authenticates the corresponding CP connection; it is not the user's login
credential or the configured system authentication credential.

Refresh and rotation of that CP key require the corresponding owner
authentication:

- User-owned CP: user authentication authorizes refresh/rotation for a CP
  belonging to that user.
- System-owned CP: the system authentication configured for CP connection
  authorizes refresh/rotation for the corresponding system CP.

Possessing the current CP key alone does not grant self-renewal authority.
Owner authentication and ownership authorization must be checked at the key
management boundary. Stable CP identity and ownership survive key rotation.

### Clarification of the Selected Alternative

The requester selected separate CP authentication (option B), with an explicit
correction: refresh/rotation uses user or system authentication, not autonomous
renewal authenticated only by a CP session. The prior proposed self-renewing
CP-session description is not accepted.

System authentication is an established design input, not a new authentication
concept to invent or reconfirm. Its credential is configured when connecting
the system CP. This does not equate it with the installation encryption root,
nor select a wire protocol or persisted system-user account.

### Consequences and Open Mechanisms

- Separate credentials used for CP communication from authority used to manage
  those credentials. Reuse existing cryptographic/rotation primitives only
  where they preserve this authority boundary.
- Define issuance, refresh and rotation as explicit operations in the later
  contract; this decision does not equate refresh with rotation or set timings.
- Failure of owner authentication must not fall back to self-renewal with the
  old CP key. Exact existing-key lifetime after loss of owner authentication,
  key revocation and enforcement on active connections remain to be decided.
- Key delivery, bounded transition overlap, concurrent rotation and recovery
  after an uncertain response require further design.
- CC's authentication remains independent and is not implicitly given CP or
  owner credentials.
- The initial implementation remains the system-owned Kubernetes CP. No
  credentials, runtime configuration or infrastructure are changed here.

## D6. Routine Rotation Handover

- Related authority: `D5`, `computer-261101/REQ-7`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Decision

Allow a bounded overlap for the immediately previous CP key during ordinary
owner-authorized rotation, instead of invalidating it at the instant the new
key is created.

An immediate cutover is simpler and avoids accepting an old key, but a lost
new-key response can interrupt CP communication until owner-authenticated
recovery succeeds. A bounded overlap tolerates handover and in-flight work,
but leaves a limited period when the old secret can still authenticate.

### Accepted Constraints

- The overlap has a fixed end and never extends because the old key is used.
  Existing old-key expiry remains an upper bound.
- The previous key can authenticate CP communication during that window; it
  cannot refresh, rotate or retrieve the new key by itself. Key recovery still
  requires user/system authentication and CP ownership authorization.
- Explicit security revocation has no routine-rotation grace. Active-connection
  enforcement and distributed propagation must be defined before implementation;
  this proposal does not claim instantaneous cross-process delivery.
- Concurrent rotation and lost-response recovery must not generate an unbounded
  chain of accepted keys. The exact serialization/recovery mechanism and
  numerical lifetimes remain subsequent design work.

Existing user-session code has bounded previous-token overlap, but also allows
an accepted old session token to recover the latest token. Only the handover
concept is comparable: copying that recovery authority into CP key management
would violate D5. No implementation reuse is authorized without this separation.

## D7. Initial CC Authentication for a CP-Created Computer

- Related authority: `computer-261101/REQ-3`, `computer-261101/REQ-4`,
  `computer-261101/REQ-10`, `D3`, `D5`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Fixed Context

CC connects independently to the server. A CP key must not be installed in a
Computer as the CC's credential. A running Pod or a claimed Computer identifier
alone does not prove that the connector belongs to that Computer.

This decision concerns the initial CP-created Computer only. User-launched CC
enrollment remains outside the milestone. CP-key refresh/rotation authority
from D5 does not automatically determine CC-key issuance or renewal authority.

### Options

1. **Server-issued Computer-scoped CC key delivered by the assigned CP
   (selected).** The server authorizes the authenticated CP for a specific
   accepted Computer creation and issues an independent CC key bound to that
   Computer. CP supplies it to the new environment. CC uses it to authenticate
   directly to the server.
2. **Kubernetes workload identity.** CC presents a suitable projected workload
   token. The server validates its issuer, audience, lifetime and workload
   binding, then maps it to the assigned Computer. This can use Kubernetes
   token projection/rotation but requires cluster trust and workload-to-Computer
   authorization, and makes initial authentication provider-specific.

The official Kubernetes
[ServiceAccount documentation](https://kubernetes.io/docs/concepts/security/service-accounts/#get-a-token)
supports expiring, rotated projected tokens with an appropriate audience.
It does not establish that a ServiceAccount token by itself proves ownership
of a Chapchoo Computer; that mapping remains application authority.

### Accepted Initial Flow

- Server authorizes the user's Computer creation and its assigned CP.
- Authenticated CP may obtain an initial CC key only for the Computer creation
  assigned to it. It cannot choose another owner's Computer by changing an ID.
- Server issues a distinct Computer-bound credential. Key delivery and creation
  are treated as one recoverable logical provisioning operation; retries must
  not silently allocate additional Computers or unconstrained credentials.
- CP injects that credential into the Computer; it does not pass its own key
  or the owner's user/system authentication material.
- CC authenticates directly to the server with the Computer-bound key.
  Subsequent CC traffic is independent of CP under D3.

### Trust and Limits

CP can see credentials it provisions and controls the target environment.
Therefore this model trusts CP for its assigned Computers; it does not claim
to protect a Computer from a malicious provider that created it. Scope limits
must prevent that CP from enrolling connectors for other providers' Computers.

Computer identity is independent of credential value and process lifetime.
This decision does not choose a static forever key, CC self-renewal, CP-driven
renewal, numerical expiry, storage format or another bootstrap-token exchange.
CC-key refresh/rotation authority and its behavior while CP is offline are
subsequent decisions. The user/system authorization required to rotate a CP key
remains unchanged.

Credential delivery must not enter images, logs or browser-visible output.
Exact secret injection, initial-key response recovery, late-creation fencing
and revocation must be specified before implementation.

## D8. Renewal Authority for CP-Created Computer Credentials

- Related authority: `D3`, `D5`, `D7`,
  `computer-261101/REQ-4`, `computer-261101/REQ-10`.
- Decision owner: requester.
- State: not selected; replaced by the requester-directed session flow in D9.

### Decision Question

Should an existing CC key authorize its own refresh/rotation, or must the
assigned CP authenticate and authorize that key-management operation?

### Options

1. **Assigned-CP authorization (recommended).** The assigned CP authenticates
   with its own valid CP key when requesting CC key refresh/rotation. The server
   verifies the CP-to-Computer assignment and current lifecycle state before
   issuing refreshed/replacement CC credentials. The old CC key alone cannot
   extend its lifetime or obtain its replacement.
2. **CC self-renewal.** A valid CC key can request a replacement while the
   server independently checks Computer existence and authorization. This
   continues renewal without a working CP, but a stolen CC key can potentially
   sustain access through renewal until effective revocation or an absolute
   limit. Preventing that persistence requires additional controls.

Both fit independent CC traffic under D3. D3 does not require renewal to work
without CP; renewal authority and ordinary communication are separate.

### Proposed Consequences

- CP refresh/rotation still requires user/system authentication under D5.
  CC refresh/rotation would require its assigned CP authentication, not the
  owner's broad credentials copied into the Computer.
- CP disconnection alone does not revoke a still-valid CC key or erase an
  independently valid CC connection.
- If the CP remains unavailable until CC credentials expire, CC loses
  authenticated service access until the assigned CP can authorize recovery.
  Do not silently keep an expired credential or allow CC self-renewal as a
  fallback. This loss of connectivity does not imply that the Computer is off.
- This is an availability/security trade-off, not a consequence already
  mandated by D3 or D7; it requires requester selection.
- CC-key loss or expiry can be recovered only within the assigned Computer
  boundary. Deletion/revocation must not permit key management to resurrect
  an Agent or Computer.
- This proposal selects authorization only. How a replacement reaches a
  running or stopped CC, rotation overlap, persistence across power operations
  and concurrent recovery are separate mechanisms still to be designed.

## D9. Connector Authentication Sessions and Direct Refresh

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-10`,
  the session invalidation constraint, `D3`, and `D7`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Session and Credential Roles

The provider creates a **session**, not merely a final connector access key.
Here session means a Computer Connector authentication session, not a user
login session, conversation or harness execution session.

For a provider-created Computer:

1. The assigned Computer Provider authenticates and requests creation of a
   connector session for its authorized Computer.
2. The session has an initial authentication key. The Kubernetes provider
   places that initial key in a Kubernetes Secret made available to the
   Computer Connector.
3. The Computer Connector presents the initial key to the server and obtains
   its operational authentication key.
4. The Computer Connector uses and refreshes that authentication key directly
   with the server under the session's authority.

For a user-run Computer Connector, user authentication authorizes creation
of the session; the connector then obtains and refreshes its operational
authentication key in the same model. This establishes the common conceptual
model without adding the user-owned-computer UI or deployment flow to the
current milestone.

The initial key and the operational authentication key have different roles.
Do not silently introduce another long-lived refresh credential, require owner
reauthentication at each connector refresh, or treat the initial key as the
operational key itself.

### Authority and Invalidation

The server remains authoritative for session validity and its Computer
binding. Key exchange, authenticated use and refresh must respect that
authority. Invalidating the entire session removes the connector's authority;
an earlier key, a newer key, the initial secret or refresh cannot restore
an invalid session.

The authority boundary supports invalidating a session if compromise is
suspected. It does not add credential-leak detection, an incident-response
console, automatic compromise classification or user-facing revocation
tooling to this milestone. The enforcement mechanism and handling of already
open connections still need an implementable design; invalidation is not a
promise of instantaneous distributed notification or rollback of prior work.

### Relationship to Earlier Decisions

- D3 remains: Computer Provider and Computer Connector connect independently
  to the server.
- D7's assigned-provider and Computer-scoping checks remain. Its directly
  supplied connector key is refined to a **session initial authentication key**
  delivered through a Kubernetes Secret, followed by connector-side exchange.
- D8's proposal to require the provider to approve each connector renewal is
  not selected. Provider unavailability alone does not prevent a valid
  connector session from refreshing with the server.
- D5/D6 govern Computer Provider keys and remain unchanged. The new connector
  session lifecycle does not make Computer Provider keys self-renewable.
- The provider can see the initial secret it injects and controls the created
  environment. Existing provider trust limits remain; session scope does not
  protect a Computer against its own malicious infrastructure provider.

### Deferred Mechanisms

Whether the initial authentication key is single-use, reusable for restart,
or time-limited is not specified by this decision. Token lifetime, refresh
timing, overlap, concurrent refresh and lost-response recovery remain open.
Kubernetes Secret delivery is selected; the exact file mount and reload
mechanism are not implicitly selected from earlier explanatory proposals.

Routine connector refresh occurs between connector and server, rather than
requiring Secret updates by the provider for every new operational key.
Restart recovery and local credential persistence must be specified without
weakening saved-environment continuity or session-wide invalidation.

## D10. Initial-Key Reuse Replaces All Operational Keys

- Related authority: `D9`, `computer-261101/REQ-4`,
  `computer-261101/REQ-10`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

The session's initial authentication key may be reused while the session is
valid. An exchange that issues a new operational authentication key invalidates
all operational keys previously issued under that session. There is no
old-operational-key overlap for this re-exchange.

The session itself and its reusable initial key remain valid; invalidating the
entire session is the distinct D9 authority-revocation operation. Keys belonging
to other sessions are not invalidated by this exchange.

### Consequences

- A restarted Computer Connector can use the injected initial Secret to obtain
  a new operational key without preserving the last operational key as a
  prerequisite.
- Previous operational keys cannot authenticate new work, refresh themselves
  or recover a newer key after re-exchange.
- Existing connections cannot retain authority merely because authentication
  succeeded earlier with an invalidated key. Enforcement on open connections
  must be part of the subsequent connection contract.
- Concurrent re-exchange or refresh must not leave older credentials valid
  contrary to this decision. Issuance and prior-key invalidation need one
  authoritative outcome; serialization and lost-response recovery remain to
  be designed.
- Re-exchanging the initial key is authority replacement, not a read-only
  lookup. Do not replay an uncertain exchange as though it had no side effects.
- D6's bounded grace applies to ordinary Computer Provider key rotation. It
  does not override this immediate replacement rule for Computer Connector
  initial-key re-exchange.
- This decision does not settle the overlap policy for ordinary connector
  refresh, credential lifetimes or a one-active-connection-per-session rule.
  Do not infer those additional contracts from key invalidation.

## D11. Component Control Transport

- Related authority: `D3`, `D9`, `D10`,
  `computer-261101/REQ-4`, `computer-261101/REQ-7`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

### Existing Evidence

The existing backend uses Protobuf with Tonic/Prost and account RPC services.
`rust/crates/protocol/Cargo.toml` confirms the current generation/runtime
dependencies. This is a reusable contract toolchain, not an implemented
component control stream.

### Decision

Use typed gRPC request/response methods for authentication-session creation,
initial-key exchange and refresh, and component-initiated bidirectional
streams for ongoing control/status. Computer Provider and Computer Connector
retain distinct authenticated roles and independent streams. This selects a
transport family, not shared credentials, a common listener or deployment unit.

gRPC's official
[core concepts](https://grpc.io/docs/what-is-grpc/core-concepts/)
describe bidirectional streams with independent send/receive directions and
ordering within each stream. They do not provide application-level durable
delivery, exactly-once effects or safe replay after disconnection.

### Alternatives and Trade-offs

- Polling/long polling can reuse typed RPC and simpler request lifetimes,
  but requires explicit scheduling/cursors and latency/load trade-offs for
  provider command delivery and current connection observation.
- WebSocket supports duplex traffic and can suit HTTP/1.1 infrastructure,
  but requires additional framing, typed dispatch/error/flow-control contracts
  beside the existing gRPC toolchain.
- gRPC streams reuse current protocol tooling and accommodate server-originated
  work over outbound component connections. They require HTTP/2-capable
  intermediaries, bounded buffering, liveness handling and stream ownership.

### Open Security and Recovery Contracts

Authentication at stream opening is insufficient for D9/D10 revocation.
The server must enforce current session/key authority during the connection;
the exact mechanism and propagation behavior remain subsequent decisions.
Transport reconnection is not authorization to replay uncertain mutations.
Do not configure automatic command retry or infer operation success from
stream write completion.

This decision does not cover harness conversations, bulk files, browser-native
transport, application message shapes, listener placement, heartbeat values or
multi-replica routing. Those are separately scoped decisions where needed.

## D12. Durable Management-Operation Authority

- Related authority: `computer-261101/REQ-3`, `computer-261101/REQ-5`,
  `computer-261101/REQ-7`, `computer-261101/REQ-8`, `D11`.
- Decision owner: requester.
- State: not selected as the general lifecycle model; superseded by D13.

### Current Boundary and Problem

Existing persistence operations perform complete DB-only transactions and
return detached results; external work is not performed while holding their
database transaction. Account mail delivery demonstrates that post-commit
external failure does not imply database rollback. It does not provide a
computer-operation journal or durable dispatch mechanism.

A successful stream write does not prove that a Computer Provider accepted,
executed or completed a command. A disconnect after an external effect can
leave the server uncertain about that effect. Volatile stream state alone
cannot provide the revisit and non-duplication outcomes in REQ-7.

### Options

1. **Server-owned operation records (recommended).** The server durably
   admits a management intent with a stable operation identity before
   returning acceptance or dispatching it. It owns the accepted intent and
   user-visible progress. The Computer Provider executes and supplies
   operation-correlated results and observed external state.
2. **Provider-owned operation records.** The server forwards the authorized
   intent and the Computer Provider durably accepts and owns its progress;
   the server projects provider results for users. This can put execution
   bookkeeping near the resources, but needs provider-local durable state,
   admission-result reconciliation and server-side handling of provider
   unavailability while users revisit or force-delete an Agent.

These options differ in authoritative admission and recovery ownership, not
whether a provider may keep execution receipts. A memory-only command queue is
not a sufficient alternative for the confirmed recovery requirements.

### Proposed Contract

- Admit the authorized intent and associated Agent/Computer changes in one
  DB-only transaction. Do not hold that transaction while talking to a
  Computer Provider or Kubernetes.
- Dispatch after durable admission. The server must be able to discover
  admitted work after a process restart; the dispatcher mechanism and exact
  persistent schema are not selected here.
- Track acceptance, dispatch uncertainty, provider acknowledgment, execution
  observation and terminal results distinctly. A transport send/acknowledgment
  is not external completion.
- Correlate reports and retries to the original operation, rather than creating
  a new restart/create request whenever a connection is restored.
- Repeated delivery of the same operation must not repeat the external effect.
  Provider-side recognition and resource-specific recovery evidence are
  required; a server record alone is not exactly-once execution.
- For an uncertain operation, first query/reconcile what happened. A running
  computer alone cannot prove that a particular restart was performed.
  If evidence cannot resolve the outcome, do not silently run the mutation
  again or report success.
- Provider-reported external facts do not grant ownership or permission. The
  server validates identity, assignment and current lifecycle before applying
  them to its operation/Agent state.

### Limits and Remaining Decisions

This selects the source of accepted intent and user-visible operation progress.
It does not select leases, queues, an outbox implementation, execution
receipt storage, resource annotations, timeouts or routing across server
replicas. It does not authorize blind retry, automatic re-creation or a
new user-visible recovery action.

Force deletion still completes user-visible Agent deletion without waiting
for external cleanup. How residual cleanup remains identifiable after that
point must be designed under the already agreed provider-specific policy.

gRPC's official [retry guidance](https://grpc.io/docs/guides/retry/)
describes replaying call history on another attempt. Transport retries are
not proof of application-level deduplication or outcome recovery. The actual
selected client's retry behavior must be checked in implementation rather
than assuming the absence of an explicit retry policy prevents all replay.

## D13. Current Computer Target and Observation Convergence

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-7`,
  `computer-261101/REQ-12`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST.

The server owns durable current Computer intent and observations. Computer
Providers execute authorized substrate actions and report observed resource
facts; Computer Connectors report their independent server connection.
Reconciliation uses that current authority to converge the Computer toward
its target. Server-composed presentation and action availability are derived
from these facts rather than stored as a second lifecycle authority.

### Evidence and Rationale

Azents' `runtime-260825` lifecycle design and current implementation at
`73236626c0e77130be7123e8784f7a08dd43bdc1` separate desired lifecycle,
provider observation/connection and runner state. The ordinary lifecycle
uses bounded current state rather than a permanent command-event ledger.
Its restart completion hands off to ordinary start convergence.

Adopt the state-management principle, not that application's complete schema
or every lifecycle semantic. No permanent per-command history is required as
the general Computer state authority. Limited admission, correlation,
deduplication and cleanup evidence may still be required; their representation
remains to be designed.

### Constraints and Differences

- Separate request acceptance, convergence, actual resource state and connector
  connectivity. Unknown observation is not proof of stopped or deleted state.
- Current-target changes and result application use short database-only
  transactions; external calls occur outside them.
- Connection loss or loss of process-local history does not authorize
  allocating a replacement Computer.
- Repeated convergence must be safe for the exact authorized target. This is
  not permission to replay an uncertain restart as a new operation.
- Existing preservation and explicit force-deletion requirements prevail.
  Azents' absence-gated permanent-removal completion is not copied over
  Chapchoo's immediate user-visible force deletion.
- Profile systems, reset, batch recreation, runtime-free Agents, Redis
  deployment assumptions and the reference's numeric generation/configuration
  machinery are not adopted by this decision.

Exact target identities, reconciliation cadence, restart handoff mechanics,
missing-resource recovery and stale-report fencing require further design.
The reference's actual stopped/unknown presentation must not weaken REQ-4/5.

## D14. Provider Inventory Survives User-Visible Agent Removal

- Related authority: `computer-261101/REQ-9`, `computer-261101/REQ-11`,
  `computer-261101/REQ-12`, and `D13`.
- Decision owner: requester.
- Accepted product direction: 2026-11-01 KST.

A Computer Provider has an authorized managed-Computer inventory page.
Force-deleting an Agent removes the user-visible Agent but does not erase the
remaining Computer's identity, provider assignment, deletion intent or evidence
needed to observe cleanup. This is tracking of a previously assigned resource,
not a new administrator-only Computer-creation capability.

The inventory consumes the same Computer lifecycle authority as Agent detail.
It must not introduce a second UI-owned cleanup queue or reconstruct all
authority from the provider's currently connected process.

### Observable Outcomes

- A remaining Computer is visible under its responsible provider even after
  its Agent disappears. The UI can identify that the Agent has been removed.
- An unreachable provider does not imply an empty inventory or deleted Computer.
  Last observations are distinguished from currently confirmed facts.
- A late running or connected report is external evidence, not permission to
  restore the Agent, cancel deletion or assign the Computer elsewhere.
- The existing Kubernetes best-effort cleanup policy remains unchanged.
  Visibility does not turn cleanup into a guaranteed-completion promise.
- The page exposes infrastructure lifecycle observations to authorized
  provider observers, not private Agent content or user execution authority.

### Scope and Open Details

The current system-owned provider page remains read-only. No manual cleanup,
retry, adopt-orphan or configuration-edit controls are added. A permanently
retained history of every completed deletion is not required.

Computer identity and cleanup evidence must remain usable without a live Agent
record. The concrete data relationship, minimum retained metadata, physical
deletion of completed tracking state and final display retention are not
selected here. Do not infer a new orphan cleanup service or tombstone schema
solely from the required observable outcome.

## D15. Restart Through Execution-Resource Replacement — Proposal

- Related authority: `computer-261101/REQ-5`, `computer-261101/REQ-6`,
  `computer-261101/REQ-7`, and `D13`.
- Decision owner: requester.
- State: proposed; not accepted.

### Question and Alternatives

Should restart durably retire the identified execution resource, preserve the
saved environment, and then hand off to ordinary start convergence?

1. **Recommended: server-owned replacement handoff.** Retain the final running
   target and bounded current restart evidence. Confirm retirement of the
   authorized old execution resource before ordinary start convergence can
   establish its replacement. Reuse the start path rather than duplicating
   allocation/readiness logic.
2. **Provider-owned complete restart.** The provider owns the entire replacement
   sequence and reports its correlated result. This can encapsulate substrate
   differences, but requires recoverable provider-side progress and a separate
   completion contract across provider restart and lost responses.

Both alternatives must preserve the same logical Computer and saved environment.
A disconnected connector, a successful delete-request response, or an arbitrary
running resource is insufficient evidence of restart completion.

### Proposed Contract

- Persist the accepted restart's relationship to its exact old execution
  resource before dispatch. Keep only the bounded evidence needed for current
  work and safe duplicate handling, not a permanent command journal.
- Keep the final running intent, with restart progress blocking ordinary
  start convergence until the old execution resource is confirmed absent.
  A user-visible stopped target is not an intermediate power command.
- Retire execution resources only. Saved files/settings and the logical
  Computer assignment survive. Concrete storage and controller topology are
  not selected by this proposal.
- A terminating resource is not yet absent. Provider observation must establish
  authoritative absence within the relevant resource scope, rather than infer
  it from a disconnected stream, timeout or missing local cache entry.
- Persist the handoff before ordinary start convergence. Server/process
  restart resumes from durable current progress and fresh observations;
  it does not initiate another restart.
- Resolve a lost deletion response by observing the identified old resource.
  Any safe continued deletion must remain constrained to that exact resource,
  never a replacement with a reused name. If evidence cannot resolve the
  result, keep it unknown rather than report success or blindly repeat it.
- After handoff, observe the replacement execution resource as running to
  confirm power-operation completion. Observe its authenticated Computer
  Connector connection independently.
- Apply results only while they belong to the still-authorized lifecycle.
  If deletion has superseded restart, a late completion cannot initiate start.
  Exact concurrency and stale-result fencing mechanisms remain pending.

### Decision Boundary

Acceptance would choose the replacement/handoff responsibility, not a numeric
generation scheme, resource schema, Kubernetes workload kind, retry cadence,
permanent operation history, or automatic recovery of unexpectedly absent
resources. Those are separate choices where material.

### Updated Remaining Decision Map

- Current: restart responsibility and durable handoff (`D15`, proposed).
- Pending: resource/intent identity, duplicate admission, stale-result fencing,
  and recovery evidence for uncertain external effects.
- Pending product-policy analysis: restoration after unexpected resource loss
  under a running target, distinct from an explicitly authorized restart.
- Pending: active authentication revocation, normal refresh races/overlap,
  connection cardinality, and initial/operational key persistence.
- Pending: saved-environment storage, terminal deletion boundaries, and
  completed inventory retention.
- Pending: stream/liveness and multi-replica ownership, deployment boundaries,
  observation contracts and verification support.

Accepted D1-D14 and their stated supersessions remain unchanged. This proposal
does not create an approved implementation Design.

### D15 Acceptance

Accepted by the requester on 2026-11-01 KST. The preceding proposal is retained
as the options discussion; D15 is now accepted.

Select alternative 1, server-owned replacement handoff, including the proposed
contract and decision boundary above. The server durably retains current
restart progress, confirms retirement of the exact authorized old execution
resource, and hands off to ordinary start convergence while preserving the
logical Computer and saved environment. No full implementation Design is
approved by this decision.

## Post-D15 Identity Evidence and Remaining Product Boundary

The accepted preservation and exact-resource rules require distinguishing the
logical Computer from replaceable execution resources. They do not yet choose
a database schema or a command/intent identity mechanism.

Kubernetes provides a native object UID that distinguishes historical objects
even when their names are reused. Its deletion API supports preconditions and
returns conflict when a deletion precondition cannot be fulfilled:

- [Object names and UIDs](https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#uids)
- [DeleteOptions](https://kubernetes.io/docs/reference/kubernetes-api/common-definitions/delete-options/)

UID-scoped deletion is a candidate substrate guard, not a complete lifecycle
fencing design. It does not by itself prevent a stale create request, establish
saved-storage ownership, or prove that all execution belonging to a workload
has stopped. Resource scope, controllers, concurrent authority and credential
binding still require design. No application-level version/generation scheme
is selected by this evidence.

Before selecting missing-resource reconciliation mechanics, resolve whether
the running target authorizes automatic restoration after an unexpected loss
of execution resources. D15 authorizes an explicit restart, not this separate
recovery policy.

The recommended product option is automatic restoration of execution resources
for the same Computer only when current running intent, authoritative absence,
saved-environment continuity and the absence of conflicting lifecycle work can
be established. Unknown observation, potentially still-live prior execution,
missing storage or ownership uncertainty must not become permission to create
a second execution or an empty replacement environment.

The alternative is to report confirmed execution loss and require a new
user-authorized recovery action. This offers more explicit control but requires
defining that recovery interaction and leaves a running target unmet until then.

This recovery recommendation is not accepted. If selected, record its observable
behavior in Requirements before adopting its material implementation mechanisms.
Identity/deduplication/fencing, authentication, storage/deletion/retention,
transport/deployment and verification decisions remain open.

## D16. Conditional Automatic Execution Recovery

- Related authority: `computer-261101/REQ-6`, `computer-261101/REQ-12`,
  `computer-261101/REQ-13`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST; Requirements review revision 8.

The requester selected the conditional automatic recovery option discussed
after D15 instead of requiring a new user-authorized recovery interaction for
every confirmed execution loss. The earlier unaccepted recommendation is now
resolved by REQ-13.

Current running intent authorizes restoring execution for the same Computer
when prior execution is confirmed absent, the existing saved environment is
usable, ownership is established and no conflicting lifecycle work exists.
This maintains the intended state without another user action while preserving
the explicit uncertainty and data-preservation boundaries.

Do not treat disconnection, missing observations or an API-object deletion
alone as proof that a partitioned execution has actually stopped. Do not
substitute new empty storage for a lost or unverified saved environment.
Recovery must respect current stop/deletion intent and explicit restart progress.

This accepts a product recovery contract, not a particular Kubernetes workload,
failure detector, storage system, reconciliation cadence or fencing protocol.

## D17. Per-Computer Persistent Storage Claim — Proposal

- Related authority: `computer-261101/REQ-6`, `computer-261101/REQ-8`,
  `computer-261101/REQ-9`, `computer-261101/REQ-13`.
- Decision owner: requester.
- State: proposed; not accepted.

### Evidence

The current chart values in `infra/charts/chapchoo/values.yaml` configure the
account backend and web applications. They do not provide a Computer storage
contract to preserve.

Kubernetes persistent storage supports a claim lifetime separate from Pod
lifetime, but the volume's reclaim policy determines what happens after claim
deletion. `ReadWriteOnce` allows multiple Pods on one node; it is not a
single-execution guarantee. `ReadWriteOncePod` offers single-Pod access with
compatible CSI storage and has deployment prerequisites:

- [PersistentVolume reclaiming](https://kubernetes.io/docs/concepts/storage/persistent-volumes/#reclaiming)
- [Volume access modes](https://kubernetes.io/docs/concepts/storage/persistent-volumes/#access-modes)

### Options and Recommendation

1. **Recommended: a dedicated persistent volume claim per logical Computer.**
   Keep that claim across stop/start, restart and execution recovery. This
   gives each Computer a distinct allocation and terminal cleanup boundary.
   Allocation latency, minimum allocation size and storage-provider capabilities
   remain operational constraints.
2. **A shared persistent volume with a per-Computer directory.** This can reduce
   allocation overhead, but requires directory-scoped ownership, safe cleanup,
   access isolation and accounting. The claim itself no longer identifies one
   Computer's saved environment.

Execution-resource removal must not garbage-collect saved storage. Computer
terminal deletion includes its dedicated saved environment under the existing
deletion requirements; force deletion preserves the cleanup identity until
the remaining cleanup can be observed.

### Decision Boundary

This proposal selects storage allocation/ownership granularity only. It does
not promise full root-filesystem persistence or silently restrict saved files
and settings to a home directory. Persistent path coverage and image/runtime
behavior must be designed to satisfy REQ-6.

StorageClass, capacity, access mode, controller topology and physical cleanup
evidence remain undecided. A PVC is not a backup or proof of exclusive live
execution. With `Retain`, removing a PVC can leave the backing asset and data;
PVC absence alone therefore cannot establish completed saved-environment
cleanup. Deployment prerequisites and completion evidence must match the
selected storage contract.

### D17 Acceptance and Operator Responsibility Boundary

Accepted by the requester on 2026-11-01 KST with an explicit scope correction.
Select one dedicated PVC per logical Computer, maintained independently of
replaceable execution resources across stop/start, restart and recovery.

StorageClass policy and underlying storage reclamation are Kubernetes operator
responsibilities. The preceding proposal's discussion of backing-data cleanup
does not become a Chapchoo completion requirement. Chapchoo does not enforce a
reclaim policy, directly delete retained PVs/backing assets, or keep a Computer
pending solely because operator policy retains them.

Within this boundary, the Kubernetes provider confirms removal of its managed
Computer resources, including the dedicated PVC. Deletion-request acceptance
or a still-terminating PVC is not yet confirmed removal. Force-deletion
tracking retains the managed-resource cleanup identity while that work remains;
it does not extend into an operator storage-reclamation queue.

Requirements review revision 9 records this ownership boundary. Persistent
file/settings coverage, access mode, capacity and execution topology still
require design. This acceptance does not select full root-filesystem
persistence or restrict preservation to a home directory.

## D18. Preserve PVC Contents Only

- Related authority: `computer-261101/REQ-6`, `computer-261101/REQ-13`, `D17`.
- Decision owner: requester.
- Accepted: 2026-11-01 KST; Requirements review revision 10.

The saved environment is the content on the Computer's dedicated PVC.
Stop/start, restart and authorized execution recovery reuse it. Changes outside
that persistent storage, including the container's writable OS layer, are not
part of the preservation guarantee.

This selects persistent-data continuity rather than full mutable-root
continuity. It does not introduce filesystem snapshots or image capture.
Concrete mount paths are still to be defined; do not interpret the decision
as selecting a particular home-directory layout. Connector credential handling
remains subject to the accepted authentication-session contract and further
key-persistence design.

## D19. Provider-Managed Standalone Pod — Proposal

- Related authority: `computer-261101/REQ-5`, `computer-261101/REQ-7`,
  `computer-261101/REQ-12`, `computer-261101/REQ-13`, `D15`, `D17`, `D18`.
- Decision owner: requester.
- State: proposed; not accepted.

### Question and Options

Should the initial Kubernetes provider directly manage a standalone execution
Pod per Computer, or reconcile a Kubernetes workload controller that manages
the execution Pod?

1. **Recommended: standalone Pod managed by the Computer Provider.** The
   server owns intent and restart handoff. The provider creates, observes and
   removes the identified Pod under that authority, reusing the independent
   PVC. Conditional replacement follows the accepted recovery gates.
2. **Single-replica StatefulSet or equivalent workload controller.** The
   provider reconciles the workload specification; the Kubernetes controller
   can replace Pods even while provider control is unavailable. Stop/restart
   must first constrain that controller so it cannot create a replacement
   before the server's accepted handoff and safety gates.

The first option aligns external creation responsibility with the existing
server/provider lifecycle. Its cost is that Pod replacement requires an
available authorized management path. An existing Pod can continue running
without that path; container restart within the same Pod is separate and
depends on its eventual restart policy.

The second option reuses built-in workload recovery, but its autonomous
replacement behavior and controller/Pod/PVC relationships must satisfy the
same absence, storage and lifecycle-authority conditions. It is viable only
with an explicit controller coordination design, not merely replicas=1.

### Evidence and Limits

Kubernetes distinguishes container restart from Pod replacement: a given Pod
UID is not rescheduled; recovery on another node uses another Pod. StatefulSet
provides stable workload identity and volume associations, but deletion alone
does not guarantee ordered graceful termination:

- [Pod lifetime and recovery](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-lifetime)
- [StatefulSet limitations](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/#limitations)

Neither option alone fences a partitioned old execution or proves safe
replacement. Node-loss evidence, stale create/delete requests, concurrent
provider workers and connector authority remain separate design obligations.
This proposal does not select a restart policy, numerical generation mechanism,
container image, resource limits, security context or user-visible new mode.

### D19 Acceptance

Accepted by the requester on 2026-11-01 KST. Select a standalone Pod directly
managed by the Computer Provider, with the Computer's PVC independent of Pod
lifetime. The preceding proposal's limitations remain applicable.

## D20. No Increasing Application Generation Numbers

- Related authority: `computer-261101/REQ-7`, `computer-261101/REQ-12`.
- Decision owner: requester.
- Accepted constraint: 2026-11-01 KST.

Revise the lifecycle and connection design without increasing application
generation/version numbers. Reference alignment with Azents does not authorize
copying its numeric desired-generation or connection-generation machinery.
Keep stale-work rejection, durable current intent and exact resource targeting.

The earlier review messages suggesting numeric connection and desired
generations were not approved. This constraint does not preapprove the
replacement mechanisms proposed below. Provider-native API metadata can still
be carried by supported clients; it is not an application lifecycle sequence
or a basis for comparing Chapchoo request recency.

## D21. Exact Current Identities and Conditional State Changes — Revised Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-7`,
  `computer-261101/REQ-9`, `computer-261101/REQ-12`, `computer-261101/REQ-13`,
  `D13`, `D15`, `D19`, `D20`.
- Decision owner: requester.
- State: proposed for review; not accepted.

### Model and Alternatives

Use unordered opaque identities for actual connections, accepted user requests
and planned execution instances, with exact current-value comparison in short
database transactions. Use random identifiers without time/lexical ordering.
There is no additional revision counter incremented on each row update, report,
heartbeat, retry or phase transition.

The alternatives are increasing generations (excluded by D20), or comparing
only power-state values and resource names (insufficient for old/new running
states and same-name resource replacement).

The minimum conceptual identities are:

- Stable Computer ID: ownership, provider assignment and saved environment.
- Connection ID: one authenticated stream registration, separately for provider
  and connector roles. Reconnection creates another connection, not another
  Computer or user request.
- Request ID: one accepted lifecycle intent. Repeated delivery and the restart
  removal-to-start handoff retain this ID. The current request remains
  identifiable after completion until another request is accepted.
- Execution ID: one reserved execution instance, recorded before Pod creation.
  Replacement reserves a new instance only after the previous execution and
  its unresolved mutations are settled. Its exact Pod locator is recorded,
  and its Kubernetes UID is captured when creation is observed.

Current dispatch/observation correlation also needs an effect ID identifying
the specific outstanding external call. This is bounded current work, not a
second lifecycle version or permanent per-command event journal. The authority
comparison is equality, never greater-than or creation-time order.

### User Request Admission

The client submits a Request ID, operation payload and the current Request ID
it observed. A database-only acceptance transaction authenticates current
ownership and checks the current operation's admissibility.

1. If the submitted Request ID is already current, return that request's state
   only when its payload matches. Do not re-execute it.
2. Otherwise, require the expected current Request ID to equal the stored
   current Request ID, then atomically install the new request and target.
3. If another request won, return a conflict/current-state response. Do not
   silently turn a stale retry into a new operation.

Changing a semantic target away and back does not restore an earlier Request
ID. Old restart requests therefore cannot become current merely because the
Computer is running again. A new explicit user restart may have a new ID;
refresh/revisit does not create one. Creation admission needs its own stable
Agent/create identity contract; this proposal does not claim that a lifecycle
request check alone solves Agent creation or deletion/recreation deduplication.

### Connection Admission and Replacement

Authentication establishes a candidate's component identity and current
credential authority. The candidate captures the current Connection ID.
After readiness, an atomic database compare-and-set replaces that exact value
with the candidate ID while rechecking authentication and assignment.

A candidate whose expected connection was replaced cannot promote itself
with an unconditional late write. It must end this attempt and re-register.
The durable pointer is the authority; volatile route installation must match
it, and a failed route publication leaves an unavailable route rather than
reviving an older one. Stream messages and effect admission revalidate current
authority; physically open superseded streams do not retain permission.

Heartbeat and close updates require exact current Connection ID equality.
An old close cannot clear a newer connection. Time is used only for expiry
and liveness, not as an alternative ordering/version mechanism.

### Reports and Restart Handoff

Accept a command result only when provider assignment, current Connection ID,
current Request ID, expected phase, current effect ID and affected execution/
resource identity all match. A stale result cannot advance current progress.
Its resource facts may still justify fresh authorized observation and cleanup;
ignoring lifecycle authority must not hide a leftover resource.

Restart retains one Request ID through old-execution retirement, confirmed
absence, durable handoff, new-execution reservation and start observation.
Each phase transition conditionally consumes the expected phase and effect.
A duplicate handoff sees that the phase already advanced and does nothing.
Completion requires the new execution evidence, not any running report.

Without ordered observation revisions, independent watch events are bounded
re-observation hints rather than unconditional authoritative overwrites.
Use a current correlated observation request for durable state updates.
Superseding intent, connection replacement or an incompatible effect invalidates
the old observation context. Coalesce repeated hints; do not allocate another
execution merely because a watch or transport restarted.

Connector reports additionally require current authentication-session/key
authority and binding to the authorized Computer/execution. The exact credential
handoff remains in the authentication-design lane; a matching Connection ID
alone cannot make a revoked or old-execution credential valid.

### External Side Effects and Unknown Results

Reserve a bounded current effect record before dispatch, including Request ID,
operation, exact execution/resource locator and dispatch certainty. Admission
is conditional and serialized for the affected Computer. External calls occur
outside the database transaction. The provider checks current authority before
starting work, but this check is not atomic with Kubernetes accepting a request.

Deletes target the exact Kubernetes object using UID preconditions. A late
delete must not be broadened to a replacement object. Start uses the recorded
execution locator; discovering an existing object requires assignment and
identity validation, not adoption by name alone.

Do not automatically retry an uncertain mutation as a fresh external effect.
Explicitly account for client/SDK retry behavior as well as application retries.
Observation can settle an uncertain create when the specifically identified
created object is found and ownership matches. A NotFound observation alone
does not settle a create that may still be accepted later.

If an old provider or external request may still create execution, do not
allocate a new execution instance, release the mutation slot merely because a
lease expired, or declare managed-resource deletion complete. Resolve the
original effect or preserve an explicit unknown state. Provider failover may
re-observe unresolved effects, not blindly redispatch them.

Deletion can become the current terminal intent while old work is unresolved.
Old success cannot resume start. Force deletion still removes the visible Agent
immediately, retaining the Computer and unresolved managed-resource evidence
for best-effort cleanup. This preserves D14/D17 and does not add tracking of
operator-retained backing storage.

This design prioritizes safe non-duplication over an unsupported promise that
every uncertain external effect can be recovered automatically. A database
compare-and-set, a random ID, or a numeric generation cannot independently
fence already-submitted Kubernetes work or a partitioned running process.

### Review and Verification Boundary

Review these interleavings: old connection close after replacement; delayed
candidate promotion; two requests with the same expected current request;
repeat request with changed payload; delayed restart completion after deletion;
duplicate restart handoff; delayed delete against a replacement Pod; uncertain
create with a temporary NotFound; process loss after effect reservation; and
revoked connector credentials on an otherwise current connection.

Database operations must retain SQLite/PostgreSQL support and short DB-only
transactions. The current repository provides this transaction pattern, but
no Computer implementation has been exercised. This is a revised mechanism
proposal, not an approved complete Design or a demonstrated distributed
exactly-once implementation.

### D21 Acceptance

Accepted by the requester on 2026-11-01 KST. Select the revised exact-current
identity and conditional-state-change design, including its explicit unknown
external-effect limits. D20's prohibition of increasing application generation
numbers remains in force. Request retries and restart handoff preserve the
accepted Request ID.

The acceptance does not close the separately identified creation-admission
and authentication-binding decisions or approve implementation.

## D22. Execution-Bound Connector Authentication Session — Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-10`,
  `D9`, `D10`, `D18`, `D20`, `D21`.
- Decision owner: requester.
- State: proposed for review; not accepted.

### Reference and Difference

Azents at `73236626c0e77130be7123e8784f7a08dd43bdc1` binds signed Runner
credentials to a Runtime and its current desired generation. The authentication
service verifies the signature and compares its claims to the durable Runtime
authority:

- `python/apps/azents/src/azents/core/runtime_runner_credential.py`
- `python/apps/azents/src/azents/services/runtime_runner_auth/service.py`

Retain the current-execution authority boundary, not Azents' signed-generation
credential format. Chapchoo uses D9/D10's authentication sessions and reusable
initial keys, and D21's opaque Execution ID.

### Selected Review Option

Bind a provider-created Computer Connector authentication session to the exact
Computer and Execution ID. Reconnecting or restarting the connector process
inside that same execution can reuse the session's initial key while the
session remains valid. D10 still invalidates all prior operational keys when
that initial key is re-exchanged.

Replacing the Pod allocates a new execution and a new session/initial Secret.
The prior session is invalidated as part of retiring its execution authority;
its initial key, operational keys and open streams cannot acquire authority
for the new execution. Session validity, authorized execution and key validity
must all be checked; client-reported Execution IDs cannot create authority.

A normal connector network reconnect using a valid operational key does not
create another session or execution. Process-start re-exchange is distinct
from stream reconnect. The initial Secret remains separate from PVC data and
is available only to its intended execution. PVC continuity does not retain
the authority of an old execution.

Do not give this component the provider/owner credential. The authorized
provider still creates the session and delivers the initial key under D9.
The new session can be prepared for the reserved execution before Pod creation;
connector registration requires that execution to remain authorized and the
Computer not to be stopping/deleting.

### Alternative and Open Details

A single session shared across successive executions would need another
per-execution secret boundary: an old Pod retaining its reusable initial key
could otherwise re-exchange and invalidate the new Pod's operational key.
Choose a new execution-bound session rather than silently retargeting an old
session or assuming old bootstrap-secret holders have lost the secret.

Exact idempotent session issuance and secure initial-key response recovery
remain contract work; one provider retry must not create multiple independent
active sessions for one reserved execution. Ordinary operational-key refresh
overlap/lifetimes remain separate from D10's no-grace re-exchange.
This proposal adds no session-management UI or secret-incident automation.

## D23. Enforce Current Authority Throughout Open Streams — Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-10`,
  `D6`, `D9`, `D10`, `D11`, `D21`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Check authority at registration and every accepted control/report/heartbeat
boundary, not only when opening the stream. Authoritative checks include current
connection identity, provider ownership/assignment or connector session/execution
binding as appropriate, key validity and session revocation. Mutating acceptance
and relevant state changes are serialized with credential revocation through
short database-only operations.

Revocation changes the durable authority first. Notify the stream owner to close
the old stream promptly, but correctness does not depend on that notification
arriving: a subsequently admitted message must recheck durable authority.
Routine key-overlap rules apply only where expressly selected; explicit
revocation and D10 re-exchange do not retain authority for old keys.

A message accepted before revocation may already have external consequences;
closing its connection does not undo them. Treat late external evidence using
D21 rather than reopening revoked authority. Database unavailability fails
authority-sensitive acceptance closed and reports control unavailability,
without claiming that the actual Computer stopped.

Retain the Azents boundary of durable authority distinct from physical stream
existence, adapted to Chapchoo's session keys and exact-ID comparisons.
Connection closure is prompt best effort, not a promise of instantaneous
cross-process network teardown.

## D24. Serial Provider Dispatch with Independent Liveness — Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-7`,
  `computer-261101/REQ-12`, `D11`, `D19`, `D21`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Use Azents' current single-in-flight lifecycle-command dispatch per active
provider stream, bounded transport buffers and an independent heartbeat task.
The reference is `docs/azents/spec/flow/agent-runtime-control.md`, Provider
report/dispatch contract, and
`python/apps/azents/src/azents/runtime/control_protocol/grpc/provider_server.py`.

Do not occupy the provider stream for an entire multi-minute convergence.
A command can return a verified in-progress substrate observation, followed
by later bounded observation requests. A rejected heartbeat or lost authority
stops new work and attempts cancellation of in-flight work. That cancellation
is not proof that an already-submitted Kubernetes mutation was cancelled.

Serial dispatch limits ordinary in-connection overlap, while D21 still governs
unknown effects and cross-connection concurrency. Do not assume that serial
stream processing replaces durable effect identity, target checks or Kubernetes
UID preconditions. Different providers may progress independently.

The alternative is bounded per-Computer parallel dispatch within one provider,
which can improve throughput but requires additional scheduling/fairness and
concurrency control. Retain the simpler reference behavior for this milestone.
Heartbeat/deadline numerical defaults and transport owner deployment are not
selected by this concurrency choice.

## Acceptance of D22-D24

The requester accepted D22, D23 and D24 together on 2026-11-01 KST:

- Execution-bound connector authentication sessions; replacement executions
  receive new sessions while prior execution authority is invalidated.
- Durable current-authority checks throughout open streams; best-effort
  teardown is not the correctness boundary.
- One in-flight lifecycle command per provider stream, bounded buffering and
  independent heartbeat processing.

Their stated limits remain applicable. Initial-key delivery recovery,
ordinary connector-key refresh overlap, numerical credential/liveness policy,
creation admission and deployment contracts still need design. This is not
approval to implement or deploy.

## D25. Idempotent Credential Results with Protected Recovery — Proposal

- Related authority: `computer-261101/REQ-7`, `computer-261101/REQ-10`,
  `D5`, `D9`, `D10`, `D21`, `D22`, `D23`.
- Decision owner: requester.
- State: proposed for review; not accepted.

### Evidence and Scope

Azents' generation-bound signed Runner credential is not the selected Chapchoo
session/initial-key contract. Its current-execution authorization boundary is
already adapted in D22; do not claim its credential issuance API can be copied.

Chapchoo's current `rust/crates/persistence/src/sessions.rs` serializes session
renewal and stores a protected latest-token recovery envelope.
`rust/crates/crypto/src/root.rs` uses authenticated encryption bound to the
session identity and token digest. That primitive protects storage, not
authorization; provider/connector-specific permissions remain separate.

### Proposed Contract

Give each mutating credential request an opaque Request ID and a canonical
payload identity. Atomically check current authority, apply the credential
change and retain its current recoverable result. An exact retry returns that
same result only while the corresponding output remains authorized; it does
not issue another key, repeat invalidation or extend expiration/overlap.
Reusing a Request ID with different content is rejected.

For provider-created connector-session issuance, the exact Computer/Execution
binding is unique. Retrying issuance for that authorized execution retrieves
the existing session and its protected initial-key result instead of creating
another session. A revoked/retired binding cannot silently be replaced by a
retry. Recovering the initial key requires the currently assigned authorized
provider, not knowledge of an execution or request identifier.

For initial-key exchange, a retry of the same exchange request retrieves the
same still-current operational key. A genuinely new exchange request performs
D10's complete prior-operational-key invalidation. A stale request cannot
reinstall or recover a superseded operational key.

For provider-key rotation/recovery, the corresponding owner authentication
and provider authorization are always required under D5. This is not permission
for a previous provider key to retrieve its replacement.

Current recoverable secret material is encrypted at rest and verified by
digest. Plaintext is returned only through the authorized TLS response needed
for delivery. Logs, errors and diagnostic DTOs exclude it. Keep only results
needed for the current credential/initial-key delivery contract, not a permanent
history of issued secrets. Discard superseded recovery material when it is no
longer an authorized recovery result.

This uses the existing installation credential-protection facility as an
internal cryptographic primitive; the installation encryption root is not
passed to providers/connectors or treated as their system authentication.

### Alternatives and Limits

Hash-only storage avoids recoverable server-side secret material but cannot
return the original randomly issued secret after a lost response. It would
require a new issuance or another derivation/delivery contract. Select protected
current-result recovery to preserve exact retries without repeated authority
replacement.

An identifier is not recovery authorization. Every retry rechecks session,
key, execution/assignment and revocation as applicable. A credential result
superseded or revoked after issuance cannot be made valid by response replay.
Delivery after acceptance remains subject to subsequent revocation; returning
a previously issued result does not guarantee it is still usable on arrival.

## D26. Bounded Previous-Key Overlap for Ordinary Connector Refresh — Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-10`,
  `D9`, `D10`, `D23`, and proposed `D25`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Ordinary connector refresh uses its current operational key directly with
the server. Serialize refresh against key re-exchange and session invalidation.
Retain the newly issued current key and at most the immediately previous
operational key for a fixed handover window, capped by that previous key's
original expiry. Use or retries do not extend the window.

The previous key can authenticate during that window but cannot create another
replacement key. It can recover the same successful refresh result only for
the exact still-current refresh Request ID under D25. A different refresh
request based on the superseded key is a stale/conflicting request, not another
rotation. Concurrent refresh requests therefore cannot create an unbounded
chain of valid keys. A connector uses one refresh operation at a time.

Initial-key re-exchange is deliberately different: D10 immediately invalidates
all earlier operational keys, including a previous key still inside an ordinary
refresh window. Session invalidation and explicit key revocation similarly
offer no grace. Neither refresh nor result recovery revives retired execution
authority.

The alternative is immediate invalidation on every ordinary refresh, which
reduces overlap but can interrupt the existing stream during response delivery
or reconnection. Select bounded overlap for ordinary handover, retaining
immediate invalidation for explicit authority replacement and revocation.

This adapts the bounded handover pattern already present in Chapchoo account
session renewal, not its user-session lifetime or its broader previous-token
recovery API. Numerical key lifetime, refresh schedule and overlap defaults
remain to be specified together; no 180-day user-session lifetime is inherited.

## Acceptance of D25-D26

The requester accepted D25 and D26 together on 2026-11-01 KST. Exact credential
request retries recover the same authorized result from protected current
material, and ordinary connector refresh allows fixed, bounded immediately
previous-key overlap. Initial-key re-exchange and explicit revocation retain
their no-grace semantics. Numerical time policies remain unselected by this
acceptance.

## D27. Machine Credential Time Defaults — Proposal

- Related authority: `computer-261101/REQ-10`, `D5`, `D6`, `D9`, `D10`,
  `D22`, `D23`, `D25`, `D26`.
- Decision owner: requester.
- State: proposed for review; not accepted.

### Defaults and Rationale

Use a 24-hour validity for newly issued provider communication keys and connector
operational keys, with proactive rotation/refresh at 12 hours after issuance.
Allow at most five minutes of immediately previous-key overlap for routine
handover, capped by that key's original expiry.

These are Chapchoo design defaults, not measured deployment values or a copied
Azents Runner policy. Azents uses the different signed current-generation
credential already discussed in D22. Chapchoo's existing account implementation
provides a five-minute overlap pattern but its 180-day user-session validity
is not inherited.

The proposed daily lifetime and halfway refresh leave a substantial retry
window without retaining a user-login-length communication credential.
Five minutes permits ordinary delivery/reconnect handover without indefinite
previous-key validity. Neither the lifetime nor refresh is an assurance that
stolen renewable credentials self-revoke; current authority checks and explicit
revocation remain the security boundary.

Provider rotation still requires owner authentication. Connector refresh still
uses the current operational key and session authority directly with the server.
The client uses server-returned validity/refresh deadlines, not a separately
assumed clock policy. A same-request retry does not move those deadlines.

### Expiry and Bootstrap

An expired communication/operational key cannot keep an open stream authorized
or renew itself through expiry grace. A provider can recover using its required
owner authentication. A connector can intentionally re-exchange its still-valid
initial key for the same authorized execution; this follows D10 and is distinct
from blindly replaying an uncertain exchange under a new Request ID.

Execution-bound connector sessions and their reusable initial keys have no
independent calendar-expiry default in this proposal. Their validity follows
explicit invalidation and the authorized execution lifetime under D22; stopping,
retiring or deleting the execution withdraws that authority. A process restart
inside the same still-authorized execution does not itself end the session.

Keep these defaults in validated server configuration, with server-declared
deadlines authoritative. Do not add web configuration UI. Validate positive
durations and refresh-before-expiry; clamp previous-key overlap to original
expiry regardless of configuration. Configuration changes apply to future
issuance rather than extending an already issued key or overlap deadline.

## D28. Reference-Aligned Liveness and Convergence Timers — Proposal

- Related authority: `computer-261101/REQ-4`, `computer-261101/REQ-7`,
  `computer-261101/REQ-12`, `D21`, `D23`, `D24`.
- Decision owner: requester.
- State: proposed for review; not accepted.

### Verified Reference

At Azents `73236626c0e77130be7123e8784f7a08dd43bdc1`:

- `services/runtime_connection_registration/service.py`: heartbeat 20 seconds,
  connection TTL 60 seconds.
- `runtime/control_server.py`: reconciliation loop 15 seconds, lifecycle retry
  delay 15 seconds, start timeout 300 seconds.
- `runtime/control_protocol/reconciler.py`: provider command deadline 10 seconds,
  observation eligibility interval 10 seconds and bounded candidate batches.

Paths above are under `python/apps/azents/src/azents/`. The actual loop cadence
and serial-provider queue mean a 10-second observation eligibility interval is
not a promise that every Computer is observed every 10 seconds.

### Chapchoo Application and Differences

Use those 20/60-second liveness defaults with exact current Connection ID
checks. Missing liveness makes the control path unavailable; it does not prove
that a Computer has stopped or authorize allocating another execution.
An expired connection requires a new registration rather than a late heartbeat
reviving it.

Use a 15-second bounded reconciliation sweep, 10-second observation eligibility,
15-second backoff eligibility for known-safe retries, and a 10-second individual
provider-command deadline. D21 overrides any broad retry interpretation: expiry
does not prove non-execution, release an unresolved mutation claim, or authorize
a fresh creation/restart. Long convergence uses subsequent observations rather
than holding one provider command indefinitely.

At five minutes from the first dispatch of the current start phase, surface
start delay/attention rather than silently continuing preparation. Keep the
current target and evidence. This is not proof of stopped state, not permission
to reset storage, and not a promise of a five-minute provisioning SLA.
Later valid evidence may complete the same request. Repeated delivery or
reconnection must not repeatedly reset that request's delay marker. Immediate
provider unavailability remains separately visible without waiting five minutes.

All delays are operational defaults with bounded work, not application version
numbers. Current state authority and exact identity checks remain independent
of timer values. These defaults still require real integration/operational
verification before release.

## Acceptance of D27-D28

The requester accepted D27 and D28 together on 2026-11-01 KST. Adopt the
24-hour communication-key validity, 12-hour proactive refresh/rotation and
at-most-five-minute routine overlap, with the stated bootstrap/revocation
boundaries. Adopt the reference-aligned connection and convergence defaults
without interpreting timeouts as proof of stopped state or non-execution.
No implementation or deployment is authorized.

## D29. Durable Minimal Agent-Creation Receipt — Proposal

- Related authority: `computer-261101/REQ-3`, `computer-261101/REQ-7`,
  `computer-261101/REQ-9`, `D13`, `D21`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Accept creation with a client-generated opaque Request ID scoped to the
authenticated owner. Persist a minimal receipt uniquely keyed by owner and
Request ID, containing the canonical creation-payload fingerprint and assigned
Agent ID. Atomically insert the receipt, personal Agent and dedicated Computer
intent in one short database transaction. Provision only after commit.

An exact retry returns the existing Agent/preparation outcome, not another
Agent or Computer. A different payload under the same key is rejected. When
the Agent has already been removed, the receipt produces an already-removed
outcome rather than recreating it. A genuinely new creation requires a new
Request ID. A fresh form opened by an explicit user action is different from
automatically replacing the ID after an uncertain submission.

Keep the receipt while its owner account exists, including after Agent and
Computer cleanup. This is deliberate minimal durable deduplication state:
owner/Request ID, payload fingerprint and opaque assigned Agent ID, not names,
conversation content, secrets, full responses or a general command event log.
It creates storage proportional to historical Agent creations; disclose that
cost rather than calling the state free or temporary. Existing account-removal
policy, if exercised, governs removal with its owning account; this decision
does not add account deletion behavior.

The alternative is expiring receipts, which needs an independently enforced
request-validity window to prevent an old accepted create from becoming new
after receipt expiry. Do not introduce form-expiry/creation-ticket behavior
implicitly. Select the minimal retained receipt for this milestone.

This is a Chapchoo REQ-7 adaptation, not a claim that the inspected Azents
Agent creation service implements this contract. The reference's
`services/agent/__init__.py` delegates creation to its repository; that alone
does not establish the required post-removal retry guarantee.

## D30. Independent Computer Tracking and Bounded Completed Retention — Proposal

- Related authority: `computer-261101/REQ-8`, `computer-261101/REQ-9`,
  `computer-261101/REQ-11`, `computer-261101/REQ-12`, `D14`, `D17`, `D21`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Persist Computer tracking separately from the live personal Agent. Retain the
responsible provider, stable Computer ID, nullable live Agent relation, original
opaque Agent identity, current terminal intent, execution/resource locators and
unresolved-effect evidence needed for cleanup. Agent removal clears the live
relation in the same database transaction; it does not cascade-delete Computer
tracking. Ownership/authorization and public DTOs remain role-specific.

The provider inventory reads this same lifecycle authority, not a second
cleanup queue. It can show a removed Agent association without retaining the
Agent's private content. Normal deletion removes the Agent after confirmed
managed-resource removal. Force deletion removes it first and keeps pending
Computer cleanup. Retained backing storage under operator policy remains
outside this record's cleanup-completion boundary.

After all managed-resource cleanup and unresolved effects are settled, mark
the Computer cleanup complete and retain a minimal completed entry for 24 hours,
then remove its completed tracking details. This is a proposed finite display/
diagnostic window, not an Azents-derived default or a permanent audit archive.
Do not start this retention clock on force deletion, a request timeout, provider
disconnection, or an unverified absence. Unresolved cleanup does not expire
merely because it is old.

Revoke obsolete execution sessions when authority is retired. Remove secret
recovery material once it is no longer authorized for recovery, without waiting
for the 24-hour display window. Keep only non-secret completion metadata during
that window. D29's separate minimal create receipt survives completed tracking
removal so an old creation cannot be replayed as new.

Alternatives are immediate removal of completed entries, which gives little
revisit visibility, or indefinite inventory history, which adds unrequired
retention. Select a bounded completed window while preserving pending cleanup
for as long as it actually remains unresolved.

## D31. Separate Control, Provider and Connector Deployment Roles — Proposal

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-2`,
  `computer-261101/REQ-10`, `D3`, `D11`, `D19`, `D23`, `D24`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Keep three distinct responsibilities:

- Server Computer Control: authenticated provider/connector gRPC streams,
  current-authority checks and lifecycle reconciliation, sharing the canonical
  product database with General/Admin APIs.
- Kubernetes Computer Provider: independently deployed cluster-side process,
  scoped Kubernetes permissions and outbound authenticated control connection.
- Computer Connector: process inside each execution Pod, with its mounted
  initial Secret and outbound authenticated connection. It receives no product
  database/root credentials or provider Kubernetes credentials.

Azents has separate Runtime Control and Kubernetes Provider processes and an
in-Pod Runner. Reuse this responsibility split, not its complete worker/gateway/
tool-execution stack. The existing Chapchoo delivery supports foreground
composition and separate General/Admin backend deployments. Extend the server
executable's composition with a selectable Control role; allow a combined
invocation for a small installation and a separate Control invocation for
independent deployment. Do not add a new daemon/supervisor framework.

Use a dedicated logical gRPC listener/endpoint for component control and
credential RPCs, separate from General/Admin user-facing listeners. External
transport requires authenticated TLS and HTTP/2; client CA trust is explicit.
Private trusted hops, ingress termination and public reachability must be
documented and tested rather than inferred from a rendered Ingress.

The provider and connector initiate connections; neither requires an inbound
control listener on each Computer. Provider system authentication is configured
through deployment as already agreed, not through web registration. Existing
explicit schema preparation remains the only migration entry point.

This selects role/process boundaries, not Redis, replica counts, autoscaling,
leader election, default resource limits or a numeric generation scheme.
Transient cross-replica routing/coordination must still be specified before
claiming multi-Control readiness. Kubernetes workload RBAC and runtime security
configuration also require the scoped implementation Design.

## Acceptance of D29-D31

The requester accepted D29, D30 and D31 together on 2026-11-01 KST:
minimal owner-scoped creation receipts retained across Agent removal;
independent Computer tracking with a 24-hour confirmed-completion window;
and separate Control, Kubernetes Provider and in-Pod Connector responsibilities,
with combined or split server composition. Their confidentiality, unresolved
cleanup and operator-owned storage-policy boundaries remain unchanged.

## D32. Optional Transient Coordination with Durable Database Authority — Proposal

- Related authority: `computer-261101/REQ-7`, `computer-261101/REQ-12`,
  `D21`, `D23`, `D24`, `D31`, and the repository Redis-optional convention.
- Decision owner: requester.
- State: proposed for review; not accepted.

Use one coordination interface with in-memory and Redis implementations for
live stream routing, bounded delivery buffers and transient wakeups.
Azents provides these adapter boundaries in
`python/apps/azents/src/azents/runtime/coordination/{store,memory,redis}.py`.
Retain the architectural split, replacing generation comparisons with D21's
exact current identities.

The relational database owns current Connection/Request/Execution identity,
authorization, durable effects and cleanup. Redis or memory cannot create
authority, infer effect success, unlock an uncertain mutation or delete a
Computer. All external coordination calls remain outside DB transactions.

In-memory mode supports one active Control process for an installation; it
supports the same lifecycle behavior without Redis. Use a database-backed
exact-ID owner admission guard to reject a second active in-memory Control
owner rather than silently isolating streams. General/Admin request processes
may remain separate: accepted intent is durable and the Control reconciler
discovers it without requiring an in-process callback.

Redis mode supports multiple Control processes, each owning its actual streams.
Route work only to the exact currently authorized stream owner; losing Redis
makes affected delivery unavailable rather than switching individual replicas
to isolated in-memory operation. Empty Redis recovery requires fresh
registration/rebuilding transient routes from durable authority, not restoring
old Redis keys. Unresolved durable effects remain unresolved during that repair.

Select the coordination backend explicitly in deployment. A single-Control
installation can use either adapter. Multi-Control deployment requires the
shared adapter and the corresponding validated PostgreSQL deployment topology;
SQLite remains supported for its existing single-installation topology.
Do not claim arbitrary shared-file SQLite HA.

Internal transport cursors are delivery mechanics, not increasing application
versions or evidence of newer authority. The alternative DB-only delivery queue
avoids Redis but adds a separate polling/mailbox implementation; prefer the
reference's interchangeable transient coordination boundary.

## D33. Namespace-Scoped Provider and Unprivileged Execution — Proposal

- Related authority: `computer-261101/REQ-1`, `computer-261101/REQ-10`,
  `D17`, `D18`, `D19`, `D21`, `D22`, `D31`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Deploy the provider with a dedicated ServiceAccount and a configured dedicated
workload namespace. Grant only required namespaced Pod/PVC observation and
create/delete operations, initial-Secret get/create/update/delete operations,
and leader-Lease operations. Additional verbs must be justified by an actual
selected SDK operation. Do not grant PV deletion, cluster-admin, Pod exec,
impersonation or TokenReview authority for this milestone. Infrastructure
administrators supply the namespace, storage and relevant deployment trust.

Kubernetes RBAC does not enforce resource-label ownership. The namespace is
the permission boundary; the provider also checks recorded identities and
ownership labels before operating. It must not claim that label filtering
makes shared-namespace Secret privileges safe. Keep application/owner credentials
out of the computer workload namespace except for deliberately mounted
execution-scoped initial Secrets.

Use Kubernetes Lease leadership for processes sharing one configured provider
identity, following the reference provider's active/standby pattern. Only the
leader opens the management stream and starts substrate mutations. Lease loss
stops new work and initiates cancellation; neither lease expiry nor promotion
of a standby proves old external requests or execution have stopped. D21's
durable effect and identity checks remain mandatory. Default to one replica;
the leader mechanism also protects an overlapping restart.

For the Computer Pod, use a non-root runtime identity, disable privilege
escalation, drop Linux capabilities, use RuntimeDefault seccomp and disable
automatic ServiceAccount-token mounting. Do not mount host paths/sockets or
enable privileged/host-network/PID execution. The reference's unprivileged
runner settings are in Kubernetes `provider.py::_unprivileged_security_context`;
its optional privileged engine is outside this milestone.

Retain a writable ephemeral container layer where the image requires it;
only the dedicated PVC persists under D18. Make the workspace mount configurable
in provider deployment and use it consistently for HOME and working directory,
following the reference pattern. The connector reports its actual resolved
path; no browser/server process invents a local path on its behalf.

StorageClass and capacity are explicit deployment inputs; use the reference's
ReadWriteOnce claim mode without asserting that it guarantees single-Pod or
single-process execution. D21's replacement/uncertainty rules still apply.
Network exposure and egress policy are installation controls, not a new
profile-policy product surface. This proposal is not a complete hostile-code
sandbox guarantee or authorization for tool execution.

## D34. Server-Composed Views with Read-Only Browser Polling — Proposal

- Related authority: `computer-261101/REQ-3`, `computer-261101/REQ-4`,
  `computer-261101/REQ-5`, `computer-261101/REQ-7`,
  `computer-261101/REQ-10`, `computer-261101/REQ-11`,
  `computer-261101/REQ-12`, `D13`, `D21`, `D30`.
- Decision owner: requester.
- State: proposed for review; not accepted.

Keep initial authenticated reads and mutations on the existing General/Admin
RPC plus web server/tRPC boundaries. Share a server-composed Computer view
contract containing target, current progress, provider/connector connectivity,
observed execution, observation freshness and authorized action availability.
Use role-specific projections rather than exposing infrastructure identifiers
or private Agent content across authorization boundaries.

After the initial server read, poll the visible Computer detail and relevant
lists with side-effect-free reads: five seconds while active work is pending,
15 seconds otherwise. Pause hidden-page polling and re-observe on return.
These are proposed client defaults, not faster substrate observations or an
availability SLA. Keep queries bounded/paginated, coalesce overlapping reads
and preserve the last view with a stale/unavailable indicator on read failure.
No polling callback creates, restarts, or retries a mutation.

Successful mutation responses include the accepted Request ID and current
view. Repeated action submission retains its Request ID and expected current
request under D21; server admission rechecks permissions and current facts
even if a button was enabled by a previous read. Do not add automatic mutation
retry. The existing account authentication query policy remains unchanged;
this polling applies to Computer domain observations.

Use the existing TanStack/tRPC web infrastructure rather than adding a separate
browser WebSocket/SSE channel for this observation-only milestone. Azents'
server lifecycle computation is the reference for one interpretation of state;
browser polling is the bounded Chapchoo transport choice, not a claim that
Azents uses the same web refresh implementation.

Normal and force deletion retain the confirmed confirmation/progress behavior.
Admin provider inventory remains observational; this contract adds no retry,
cleanup, creation-enable or configuration buttons. Existing display settings,
keyboard behavior and narrow-screen requirements apply to both web surfaces.

## D35. Redis Exclusion and Database-Owned Stream Routing — Revised Proposal

- Related authority: `computer-261101/REQ-7`, `computer-261101/REQ-12`,
  Requirements revision 11 Redis exclusion, `D21`, `D23`, `D24`, `D31`.
- Decision owner: requester.
- Accepted constraint: no Redis, 2026-11-01 KST.
- Mechanism state: proposed for review; not accepted.

### Supersession

D32's optional Redis coordination proposal was not approved and is withdrawn.
Do not implement Redis configuration, dependencies, queues or an optional
Redis adapter for this capability. D33 and D34 remain independent unaccepted
proposals; the Redis correction is not their approval.

### Proposed Mechanism

Use the existing product database as the durable current-intent/effect
authority and as the directory for current stream ownership. Each Control
process has a random process identity; current component connection records
name that owner and the exact Connection ID with a liveness expiry.
Only the actual stream and bounded write buffer are process-local.

General/Admin APIs accept user intent through the same DB-only transactions
whether or not they share a process with Control. They do not need to locate
the owning Control server, synchronously forward a command, or append to a
second shared delivery queue.

Each Control sweeps bounded indexed candidates for providers whose current
streams it owns. Claim the exact current request/effect in a short DB-only
transaction that also rechecks owner and Connection ID. Deliver through that
local stream outside the transaction. Revalidate correlated results before
changing durable progress. A local wakeup may reduce latency but is not
required for progress or correctness.

Use the D28 15-second sweep as the reliable discovery path. This makes queueing
latency and database polling load explicit trade-offs for removing a shared
message broker. It is not a separate permanent operation ledger: current
request, current dispatch/effect and retained cleanup state remain the records
already selected in D13/D21.

### Multiple Control Processes and Failure

Different Control processes can own different streams with PostgreSQL-backed
conditional admission. Two processes cannot successfully claim the same
current effect under different owners. SQLite remains supported in a single
Control installation; do not use a shared SQLite file as distributed routing
infrastructure or require Redis for either topology.

If a Control process dies, another process does not inherit its socket. The
provider/connector reconnects through the endpoint and registers a new
Connection ID on an available Control process. D21's compare-and-set prevents
a delayed candidate or stale close from overwriting a newer connection.

A crashed/expired owner does not prove that its external work never happened.
The new connection observes the existing reserved resource/effect; it cannot
clear unknown work and redispatch a fresh mutation solely on owner expiry.
Similarly, dropping a process-local outbound buffer does not prove that an
effect was undelivered.

Revocation remains durable. An owner can detect an idle stream's lost authority
on heartbeat/periodic checks, while every accepted message and effect admission
still checks current authority under D23. There is no correctness dependency
on another server receiving a close notification. Database unavailability
blocks new authority-sensitive work; existing Pod execution is not declared
stopped.

### Scope and Validation

This design needs no server-to-server stream proxy for this milestone:
lifecycle requests are durably asynchronous and browser reads use DB state.
It does not claim a transport for future interactive tool/file data paths.
Connection-owner directory queries and effect claims must be indexed and
bounded; no database transaction contains gRPC/Kubernetes work.

Verify two-Control concurrent claims, delayed connection replacement, owner
crash before/after dispatch, idle revocation detection, database interruption
and recovery, and the equivalent single-Control SQLite path. This is source-
and-contract-level feasibility analysis, not executed multi-process validation.

## Authority Transition to the Current Foundation Snapshot

Recorded on 2026-11-02 KST. The requester subsequently confirmed a broader
standalone/distributed foundation and requested repository-document consolidation.
The current planning authority is
[computer-261102/REQ](../requirements/computer-261102-agent-computer-foundation.md)
and [computer-261102/ADR](computer-261102-agent-computer-foundation.md).

This earlier snapshot remains append-only decision/proposal history. Its old
pending labels, numeric-generation alternatives, fixed-role listener assumptions,
Redis proposals and universal command-recovery mechanisms must not be treated as
current authority. The new snapshot explicitly carries applicable accepted
Computer/authentication decisions, records later foundation choices and lists
genuinely unresolved topics without promoting old proposals.

In particular, the previous D35 DB-only polling proposal is not the selected
general command transport. Current authority separates durable product intent
and a DB address/owner directory from direct gRPC request/response delivery.
There is still no implementation or deployment approval.
