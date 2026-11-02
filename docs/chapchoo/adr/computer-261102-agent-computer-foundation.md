---
title: "Standalone and Distributed Agent Computer Decisions"
created: 2026-11-02
tags: [agent, computer, architecture, deployment, security]
document_role: primary
document_type: adr
snapshot_id: computer-261102
---

# Standalone and Distributed Agent Computer Decisions

Requirements authority: [computer-261102/REQ](../requirements/computer-261102-agent-computer-foundation.md).
Mode: Collaborative. The requester owns remaining material decisions.

This is the current decision baseline consolidated on 2026-11-02 KST from
requester confirmations on 2026-11-01/02. It is not a complete implementation
Design or implementation approval. Accepted decisions below are authoritative;
the explicit remaining-work section is not. Add future confirmations here in
the same working turn rather than leaving them only in session notes.

The earlier [computer-261101/ADR](computer-261101-agent-connection-management.md)
retains chronological decisions and proposals. This new snapshot supersedes it
for active planning where the foundation changed. It does not rewrite that
accepted history or turn unselected proposals into authority.

## Current-System Evidence

The inspected Chapchoo checkout remains at
`5b1b723979aa1f3bad13283a98680c234656a6bf`:

- `proto/chapchoo/accounts.proto` exposes broad Auth/General/Admin services.
- `rust/apps/chapchoo/src/runtime.rs` binds role-specific TCP listeners and
  registers Auth with General or Admin. Domain/account logic is shared.
- `rust/crates/rpc`, `service`, `persistence`, `protocol` and `crypto` provide
  typed RPC, current authorization, SQLite/PostgreSQL operations, code generation
  and protected credential primitives. They do not implement Computer control.
- `docs/chapchoo/spec/domain/account-rpc.md`, `account-storage.md`,
  `account-web.md` and `account-delivery.md` describe the implemented boundaries.
- The workspace uses Tonic 0.14.8 and Tokio. The installed Tonic source supports
  custom connectors and `Connected` for `tokio::io::DuplexStream`; the current
  server already uses `serve_with_incoming_shutdown`.
- Existing distribution is not evidence of completed Mac application integration
  or multi-Control behavior.

Azents reference implementation was inspected at
`73236626c0e77130be7123e8784f7a08dd43bdc1`. Reuse its verified boundaries only
where they fit the confirmed requirements. Its Redis transport, numeric
connection/desired generations and dedicated trusted-peer mTLS server are not
Chapchoo defaults.

## Current Decision Map

Accepted: D1-D26. D23/D24 resolve ownership and per-call transport policy.
P1-P6 are resolved at the policy level. P7 is implementation and verification
of common/cloud data and credentials, not a new Mac implementation scope.
The direct implementation request on 2026-11-02 KST accepts the presented D25
and D26 recommendations and complete Design. No material decision is pending.
This map is a working index; the accepted D1-D16 decision bodies remain unchanged.

- Composition: domain services, selectable groups, external/internal listeners.
- Runtime: standalone plus reachable single-cluster distribution; Control owns
  both connections and reconciliation.
- Delivery: DB directory, direct gRPC, exact opaque identities, bounded safe
  rerouting; no mandatory message persistence or broker.
- Computer: standalone Kubernetes Pod plus persistent dedicated PVC, conditional
  recovery, normal/force deletion and bounded completed visibility.
- Authentication: distinct Provider and execution-bound Connector credentials,
  protected idempotent issuance and bounded ordinary key handover.

## D1. Component Responsibilities and Independent Connections

- Authority: REQ-1/2/3/4/10.
- Accepted; carries forward prior ADR-D1/D2/D3/D4.

Computer Provider provides/manages Computers. Computer Connector runs inside a
Computer and independently connects to the server. A Computer is an execution
environment, not necessarily a physical machine. One-Agent dedication is the
initial cloud milestone scope, not the universal definition.

The Provider is not a mandatory relay for the Connector's ongoing traffic.
Both initiate outbound server-facing connections. Their active connections
may land on different Control instances. The initial Provider is system-owned
Kubernetes; user/system ownership exists conceptually without adding user-owned
attachment UI now. Register through infrastructure/authentication configuration,
not a web enrollment or independent enablement switch.

This rejects a topology in which all Connector traffic must traverse its
Provider. It does not choose future local-computer product scope.

## D2. Domain Services and Runtime-Selected Service Groups

- Authority: REQ-14/15.
- Accepted foundation, replacing fixed per-role port assumptions.

Refactor broad service contracts/handlers into appropriate domain services.
Client, Admin and Control are service groups selected at runtime, not mandatory
separate binaries, processes or listener ports. One instance can register all
or selected groups; separate deployments can independently scale subsets.

Keep domain/business logic shared. Compose common Auth/Health dependencies
once rather than registering the same service name twice. Each client manages
its required group's endpoint independently. Equal configured endpoints do not
permit inferring another group or falling back to it after failure.

Prefer composition over distinct permanent server products. Exact domain-service
names, group membership and replacement of existing generated contracts require
the Design's interface map. This decision is not permission for unrelated
account behavior changes or a speculative compatibility API.

## D3. Listener Boundaries and Internal Network Trust

- Authority: REQ-10/15/16.
- Accepted on 2026-11-02 KST.

Separate client-facing and server-peer listeners by trust boundary, independently
of service-group composition. Client-facing includes authenticated Provider and
Connector services; Control is not entirely an internal-only API.

The internal listener serves peer relay/coordination and is not externally
published. Initial Kubernetes deployment must enforce network restrictions
allowing only authorized server workloads. Trust that restricted network without
an additional application peer credential or mandatory mTLS. Allowed-server
compromise can access internal APIs; this is an accepted trust boundary, not an
assertion that a port number authenticates a peer.

Standalone has no cross-Control relay and does not start that internal listener.
Do not remove user/component authentication, DB credentials, key protection,
current connection/target checks or business authorization. Exact internal
transport encryption and external TLS termination remain deployment design work;
no universal plaintext exception is inferred.

This replaces the unselected shared-listener peer-key/mTLS alternatives. Port
separation by General/Admin/Control group remains unnecessary.

## D4. Shared gRPC Implementation and Deployment-Supplied Addresses

- Authority: REQ-14/15/17.
- Accepted direction; memory transport support is source-verified, not executed.

Reuse typed gRPC contracts and handlers locally and remotely rather than requiring
a second direct-call business API. Tonic can connect the same generated client
and server using bounded memory IO or TCP. Protobuf/HTTP2 remains present; this
is not zero-serialization method dispatch. Do not force an unnecessary remote
self-relay when the current process owns the target connection.

Initial distributed support is one Kubernetes cluster with directly reachable
server peers. Deployment supplies a reachable advertised endpoint, for example
through Downward API Pod IP and the configured listener port. Keep bind and
advertise addresses distinct. Product routing uses general endpoints and random
instance identities, not Kubernetes APIs or Pod IP as durable identity.

A common group endpoint may load-balance new connections. Owner-targeted relay
requires the exact instance endpoint. Ordinary Service/headless representative
DNS does not by itself select the owner of a live connection. Per-Pod DNS has
additional configuration prerequisites; this need alone does not require a
StatefulSet or per-Pod Service.

VM/routed-network expansion can supply endpoints under the same contract.
NAT/disconnected clusters need real routing, relay or another transport; merely
changing address discovery cannot make them reachable. Do not implement those
future modes in this milestone.

## D5. Control Co-Locates Connections and Reconciliation

- Authority: REQ-4/12/14/15.
- Accepted after inspecting the Azents execution owner.

The Control group runs component gRPC handlers and lifecycle reconciliation as
separate asynchronous tasks in the same execution unit. Do not introduce a
separate Worker role now. A standalone invocation can host all selected roles
in one process; a distributed Control instance can host that group separately.

Azents `runtime/control_server.py` constructs RuntimeLifecycleReconciler and
starts `runtime-lifecycle-reconciler` beside the gRPC server. It also passes the
reconciler to Provider observation-completion handling. This, rather than a
hypothetical Worker alternative, is the carried reference boundary.

Co-location does not imply that one Control owns all connections for a Computer.
D6/D8 cover delivery to the actual owner; shared product state remains the domain
source, not per-Control memory.

## D6. Database Directory and Direct Owner-Targeted gRPC

- Authority: REQ-12/14/17/18.
- Accepted on 2026-11-01 KST during the foundation revision.

Use the existing product database for two conceptual relations:

- instance identity to current advertised endpoint/registration;
- Provider or Connector target to current owner instance and Connection ID.

Actual requests/responses use direct gRPC to that owner. This is not a SQL command
queue, permanent message history, or an extra discovery server. Maintain target
kind explicitly: Provider and Connector ownership are independent.

In-process routing can use local/memory facilities, without advertising an
unnecessary external route. The registry records routing facts, not proof that
an incoming request is authorized or a previously sent command succeeded.

Select this after comparing direct RPC, temporary SQLite/PG queues, NOTIFY,
NATS and ZeroMQ. The chosen reachable-cluster deployment makes direct peer
routing practical while retaining the existing RPC stack. No Redis or optional
Redis mode is adopted. Concrete schema, registration lifecycle and cache behavior
remain in P3.

## D7. Exact Current Identities, Not Increasing Generations

- Authority: REQ-7/12/17; no-generation fixed constraint.
- Accepted; carries the applicable identity part of prior ADR-D20/D21.

Use opaque identities for actual instances, connections, accepted requests and
execution instances. Compare exact current identity, never a greater numeric
revision or a timestamp standing in for one. Kubernetes UID identifies the actual
resource, separately from logical Computer and request identity.

Request retries and restart phase handoff retain one Request ID. An exact repeat
with a matching payload returns current known progress rather than admitting a
new operation; reuse with different content is rejected. Current-intent changes
and connection replacement use short conditional database operations. Delayed
registration/close cannot overwrite a newer accepted connection.

These checks prevent stale authority from changing current state. They do not
cancel already-submitted Kubernetes work or make external execution exactly once.
The earlier proposal to give every external message an indefinitely retained
recoverable effect record is not carried as a universal requirement; D9/P2 define
the new boundary.

## D8. Safe Rerouting Before Dispatch, Uncertainty Afterwards

- Authority: REQ-7/17/18.
- Accepted on 2026-11-02 KST.

Carry expected target/owner connection identity on forwarded requests. When an
old owner authoritatively rejects before target forwarding, the caller may
re-resolve and retry the same Request ID within the original deadline and a
bounded attempt budget. Never return this safe rejection after uncertain dispatch.

After possible forwarding, timeout/disconnection does not cause automatic replay
of an effecting command. Return uncertainty. No current owner returns unavailable,
not an indefinitely retained offline request. Repetition-safe reads may have an
explicit separate API policy; do not infer safety from a generic transport error.

This chooses bounded correction of stale routing over unconditional replay or
requiring every stale address to be manually retried. Exact error envelopes and
stream/cancellation semantics remain in P4.

## D9. Durable Product Intent, Lightweight Command Delivery

- Authority: REQ-4/7/12/13/18.
- Accepted foundation refinement; supersedes stronger general command-history
  assumptions in the previous snapshot.

Persist desired running/stopped/deleting state because Control/app processes can
stop. Providers supply substrate facts, Connectors supply independently authorized
connection observations, and the server composes display/action availability.
Do not persist a second UI lifecycle authority.

Transient calls may be lost or time out. Message storage, guaranteed eventual
execution, indefinite unresolved-command retention and an outbox are not mandatory
transport mechanisms. Desired-state reconciliation is not a substitute for general
queries, commands and response/stream delivery.

Restart remains explicit work: preserve the logical Computer/PVC, identify the old
execution, confirm its retirement, and hand off under the same request to ordinary
start convergence. A running observation alone does not prove that restart occurred.
Preserve honest power semantics: deletion-request acceptance, disconnect or unknown
state does not prove stopped/removed execution.

Automatically recover unexpectedly absent execution only under REQ-13's confirmed
absence, saved-environment, ownership and non-conflict conditions. Minimum durable
restart/deletion progress and user recovery after uncertainty require P2; do not
silently revive the old universal effect-ledger design or abandon execution safety.

## D10. Standalone Pod and Independent Per-Computer PVC

- Authority: REQ-5/6/8/13.
- Accepted; carries prior ADR-D15/D16/D17/D18/D19 where applicable.

The initial Kubernetes Provider directly manages one standalone execution Pod per
Computer, not a StatefulSet/Deployment controlling replacements. Keep a dedicated
PVC independent of Pod lifetime and reuse it for power transitions and authorized
execution recovery. Preserve PVC contents only; no mutable-root snapshot or live
memory continuity is implied.

The Provider's replacement capability must not depend on remembered process-local
command history. A terminating resource is not confirmed absent. StorageClass
reclaim behavior/backing assets belong to the Kubernetes operator; Provider
completion stops at its managed-resource boundary, including PVC removal.

Controller topology, per-Computer allocation and preservation are settled.
Workspace mount path, capacity configuration, access mode, Pod security settings
and Provider RBAC/leadership details from prior unaccepted ADR-D33 remain in P5.

## D11. Independent Computer Cleanup and Bounded Completion Visibility

- Authority: REQ-8/9/11/12.
- Accepted; carries prior ADR-D14/D30.

Normal deletion confirms managed Computer removal before removing the Agent.
Force deletion immediately removes the visible Agent after explicit confirmation,
while the Provider continues best-effort residual cleanup. No separate Connector
cleanup prerequisite or user-facing internal stepper is added.

Track the Computer independently of a live Agent relation. Keep Provider, logical
identity, terminal target and useful resource/observation evidence after force
deletion; never restore the Agent on a late report. The Provider inventory reads
this same authority and exposes no private Agent content or lifecycle control.

Retain minimal non-secret confirmed-completion information for 24 hours, then
remove completed tracking details. Pending/unverified resource cleanup does not
age out. This is not an archive of command bodies or operator-retained backing
storage. Revoke obsolete credentials and discard no-longer-authorized secret
recovery material independently of the display window.

## D12. Minimal Creation Deduplication Survives Agent Removal

- Authority: REQ-3/7/18.
- Accepted; carries prior ADR-D29.

Use an owner-scoped opaque creation Request ID with a canonical payload fingerprint
and assigned Agent ID. Atomically create that receipt, Agent and Computer intent;
perform external provisioning after commit. Exact replay returns the existing
outcome, and replay after Agent removal returns already-removed instead of creating
again. Different content under the same ID is rejected.

Keep this minimal receipt while its owner account exists, without names, content,
secrets or full responses. The storage grows with historical creation count; that
cost was disclosed. Existing account-removal policy governs the owner relation,
not a new account deletion feature. This specific deduplication record is distinct
from the rejected universal permanent command/message archive.

## D13. Independent Provider Keys and Owner-Authorized Management

- Authority: REQ-1/2/10 and component-authentication constraints.
- Accepted; carries prior ADR-D4/D5/D6.

Provider communication uses an independent Provider key. Its issuance/rotation/
recovery requires the corresponding user or configured system-owner authentication
and Provider ownership check. The current Provider key alone cannot renew itself
or recover its replacement. Stable Provider identity does not depend on key value.

Routine rotation accepts the immediately previous key for a fixed bounded overlap
capped by its original expiry. Use does not extend it; explicit revocation has no
grace. Existing system authentication is a confirmed design input, not an invitation
to invent a system-user account or expose the installation encryption root.

## D14. Execution-Bound Connector Sessions and Active Revocation

- Authority: REQ-4/6/10/12 and session-revocation constraint.
- Accepted; carries prior ADR-D9/D10/D11/D22/D23.

The assigned authorized Provider creates a Connector authentication session bound
to Computer and Execution ID and supplies its initial key through Kubernetes
Secret. The Connector exchanges it for an operational key and refreshes directly
with the server, independently of Provider connectivity.

The initial key is reusable while its session/execution is authorized. A new
exchange invalidates all earlier operational keys in that session without grace.
Session invalidation also prevents initial-key exchange and refresh from restoring
authority. A network reconnect with a valid operational key is not a new session.
A process restart within the same execution may re-exchange its initial key.
A replacement execution receives a new session/Secret; retire prior execution
session authority so an old bootstrap-key holder cannot disrupt its replacement.

Use typed unary gRPC for session/key operations and independent component-initiated
bidirectional streams for ongoing control/status. Current session/key, execution
and connection authority is checked throughout accepted work, not just at stream
open. Durable revocation is authoritative; prompt stream teardown is best effort.
Already accepted external work is not undone by closing a socket.

Do not add a second long-lived refresh secret, incident-response UI, compromise
detection or user-owned-computer flow. Credential bootstrap/transport persistence
implementation must respect these fixed contracts.

## D15. Idempotent Protected Credential Recovery and Refresh Handover

- Authority: REQ-7/10; D13/D14.
- Accepted; carries prior ADR-D25/D26.

Mutating credential requests have opaque Request IDs and checked payload identity.
Atomically apply the change and retain its current recoverable result. Exact retry
returns the same still-authorized result without new issuance, repeated invalidation
or expiry extension. Retrying session issuance for an authorized execution retrieves
its unique session; it cannot silently replace a revoked/retired binding.

Protect current secret recovery material with the existing authenticated-encryption
primitive, scoped to identity/digest. Recheck current recovery authority; Request ID
alone grants none. Provider-key recovery still requires owner authentication.
Do not keep an archive of old secrets, expose plaintext in diagnostics, or send the
installation encryption root to components.

Ordinary Connector refresh retains at most the current and immediately previous
key during fixed overlap. A previous key can authenticate within that window and
recover the exact still-current refresh result, but cannot initiate another successor
rotation. Serialize against concurrent refresh, re-exchange and revocation. D14's
initial-key re-exchange and explicit revocation invalidate all old authority without
overlap, even if routine grace had time remaining.

## D16. Selected Credential, Liveness and Convergence Defaults

- Authority: REQ-4/7/10/12; D13-D15.
- Accepted; carries prior ADR-D24/D27/D28.

- Provider communication and Connector operational keys: 24-hour validity,
  proactive owner-authorized rotation/direct refresh at 12 hours.
- Ordinary immediately previous-key overlap: at most five minutes, capped by
  original expiry; issued deadlines are not extended by same-request retries.
- Connector session/initial key: no separate calendar-expiry default; validity
  follows authorized execution and explicit invalidation.
- Component heartbeat 20 seconds, connection validity 60 seconds.
- Bounded reconciliation sweep 15 seconds, observation eligibility 10 seconds,
  known-safe retry eligibility 15 seconds.
- Individual Provider command response deadline 10 seconds. One in-flight
  lifecycle command per live Provider stream, bounded buffers, independent
  heartbeat; different Providers can progress independently.
- At five minutes from initial dispatch of the current start phase, expose delay
  rather than resetting state or claiming failure-to-execute. Preserve the target;
  valid later evidence may complete it. Reconnect/retry does not reset the marker.

The numeric operational defaults are not application generations or throughput/
provisioning guarantees. A 10-second observation eligibility interval does not
promise every target is observed every 10 seconds. An expired owner/command does
not prove non-execution or authorize unsafe replay. The dates/limits for server
instance directory registrations are not implicitly selected by these component
connection values.

The liveness/convergence values were checked in Azents connection registration,
Control composition and lifecycle reconciler sources. Credential 24h/12h/5m values
were separately selected for Chapchoo, not copied from Azents signed Runner keys or
180-day user-login validity.

## Retired or Unselected Directions

These are not current implementation authority:

- prior general command journal and universal durable external-effect recovery;
- increasing connection/desired generations or timestamp versions;
- Redis or optional Redis adapters;
- fixed per-Client/Admin/Control listener/process topology;
- DB-only target polling as the transport for every query and command;
- assuming one Computer's Provider and Connector share a Control owner;
- PG queue, LISTEN/NOTIFY, NATS, ZeroMQ or hybrid-broker transport as the selected
  command path (they were compared, not adopted);
- separate peer application keys or mandatory mTLS for the restricted internal
  server listener;
- unapproved browser polling, Provider RBAC/Lease/Pod-security and mount defaults
  from the earlier exploratory batch.

## Remaining Decision Map

This working checklist records decisions through D24 and Requirements revision 4.
The seven rows are coverage areas, not seven additional approval questions. Closed items are
retained for traceability, not as repeated approval questions.

| ID | State | Topic | Current treatment |
| --- | --- | --- | --- |
| P1 | Closed by D17 and REQ revision 4 | Future local Mac compatibility | Preserve the Computer boundary and sharing model; defer concrete Mac enrollment/authentication and implementation |
| P2 | Closed by D18 | Reconciliation continuity | Connector-owning Control recreates its reconciler from desired state; lossy delivery policy remains |
| P3 | Policy resolved by D23 | Instance/connection ownership lifecycle | Translate accepted registration, renewal, replacement and failure rules into schema and race verification |
| P4 | Policy resolved by D21/D24 | Relay/request-response contract | Translate accepted typed calls, correlation, deadline/cancellation and bounds into messages and verification |
| P5 | Closed by D19 | Kubernetes operational policy | Operator owns policy; agent supplies sound configurable Helm defaults within confirmed constraints |
| P6 | Policy resolved by D22/D25 | Domain API/group/observation map | Implement approved domain map and bounded browser projection reads |
| P7 | Design and feasibility work | Common/cloud data and credential integration | Specify schema/migrations and approved credential delivery; preserve future local compatibility without a concrete Mac workflow |

P3/P4/P6/P7 are design work areas, not permission to turn every field, helper,
default or test fixture into an interview. Only newly discovered material choices
require discussion. No Mac inclusion, UI framework or broad durability question
remains to be reapproved.

## Document and Verification Work Still Required

- The approved [primary Design](../design/computer-261102-agent-computer-foundation.md)
  revision 2 covers M1-M22, including accepted D25/D26. Implementation is
  authorized by the direct implementation request; verification remains required.
- Trace every Requirement to mechanisms, and every material mechanism to accepted
  authority. Explicitly cover replacement of broad current API/listener contracts.
- Preserve existing explicit DB preparation and both supported database modes.
- Verify same-handler memory/network gRPC, split Provider/Connector ownership,
  two-Control races, uncertainty/cancellation, component revocation and internal
  listener access boundaries. Do not substitute document tests for these checks.
- Verify common standalone composition, SQLite and memory/network transport.
  A concrete Mac runtime/enrollment smoke test is deferred with that future
  delivery. Kubernetes network/rolling behavior requires appropriate integration
  verification for the current cloud scope.
- Keep E2E focused on core user outcomes and use lower-level contracts for detailed
  race/transport combinations. No new broad E2E matrix is implied.
- Design authority/feasibility review and requester approval are recorded for
  revision 2. This snapshot has no `implemented` date until verified delivery.

## Reference Basis

- Current local Specs: account RPC/storage/web/delivery, named above.
- [Tonic memory IO example](https://github.com/hyperium/tonic/blob/master/examples/src/mock/mock.rs)
- [Tonic 0.14.8 custom connector](https://docs.rs/tonic/0.14.8/tonic/transport/struct.Endpoint.html#method.connect_with_connector)
- [Kubernetes Downward API](https://kubernetes.io/docs/concepts/workloads/pods/downward-api/)
- [Kubernetes networking](https://kubernetes.io/docs/concepts/services-networking/)
- [Kubernetes DNS behavior](https://kubernetes.io/docs/concepts/services-networking/dns-pod-service/)
- [Azents pinned lifecycle/control source](https://github.com/azents/azents/blob/73236626c0e77130be7123e8784f7a08dd43bdc1/python/apps/azents/src/azents/runtime/control_server.py)

References establish observed mechanisms and constraints, not approval to copy
all reference behavior. No private discussion provenance or deployment values
are required to read this snapshot.

## D17. The Mac Itself Is a Computer; UI and Packaging Are Out of Scope

- Authority: REQ-3/5/14 and revision 2 scope clarification.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

The local Mac is a first-class Computer in the future standalone application.
Design the shared system for local execution, not merely a locally hosted UI/
backend that requires a cloud Computer. Do not require Kubernetes, a Pod or a PVC
to make the Mac recognizable as a Computer.

Mac UI technology, implementation, packaging and distribution are outside this
work. Their later integration motivates the foundation; they are not pending
decisions that block it. P1 is closed.

Kubernetes Pod/PVC provisioning and its dedicated-Agent cloud journey remain the
cloud implementation, not universal Computer assumptions. Concrete local
registration, capabilities, storage and credential delivery belong to P6/P7's
shared/local implementation mapping. Do not silently translate cloud Pod
stop/deletion into host OS shutdown or unrelated local data deletion.

This clarifies the previously open local scope without authorizing Mac UI work,
new harness functionality or a second domain implementation.

## D18. Connector-Connection-Owned Reconciler Recreated from Desired State

- Authority: REQ-4/5/7/8/12/13/17/18 and D5-D9.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

When a Computer Connector is accepted by a Control, that Control owns the
Connector's reconciler. The reconciler continuously observes current desired
state and relevant facts and converges the Computer accordingly. It belongs to
the active connection ownership, not to a permanent server or a transferable
process-local work history.

When the connection is lost or replaced, the previous reconciler loses that
connection's authority. The Connector reconnects to an available Control, which
creates its reconciler from the durable desired state and current observations.
Recheck exact current identity before applying results so a delayed old loop
cannot overwrite current authority. No increasing generation is introduced.

The command transport keeps the agreed lossy/timeout policy. Do not add a
durable command ledger, guaranteed broker replay or an old-loop checkpoint
transfer as a prerequisite for reconciliation continuity. P2's broad policy
question is closed; concrete state fields and mechanics are Design work.

Provider and Connector may still be attached to different Control instances.
Provider-directed work uses the selected owner-targeted delivery path. Initial
provisioning and supported power/deletion management cannot require an already
healthy Connector, so the provider-management paths must remain usable when no
Connector exists or it has disconnected. This preserves REQ-1/5/8 rather than
creating a bootstrap or stopped-Computer deadlock.

Reference evidence: the inspected Azents Control owns the periodic lifecycle
reconciler and handles Runner registration/stream lifetime separately. These
confirm durable desired-state/Control/reconnection foundations. The precise
Connector-scoped ownership contract above is the requester's explicit Chapchoo
choice; it is not a claim that the inspected reference creates one identically
named reconciler object in every Runner registration handler.

## D19. Operator-Owned Kubernetes Policy and Agent-Owned Helm Defaults

- Authority: REQ-8/14/16 and revision 2 fixed constraints.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

Cluster administrators decide operational Kubernetes policy, including storage,
resource/scheduling choices, network/TLS deployment and permissions suited to
their environment. The implementation supplies sound configurable Helm defaults
and validates compatibility with confirmed product/trust requirements.

Do not ask the requester to separately approve each ordinary chart value,
capability, role verb or resource setting as a new product decision. P5 is closed
as that broad decision lane; chart/configuration and verification work continues
under P7 and the eventual implementation plan.

This delegation does not relax the accepted internal-network restriction,
component authorization, operator-owned storage-reclamation or data-preservation
boundaries. Clearly unsupported configurations must not be represented as safe
defaults. A genuine new product behavior or relaxation of confirmed constraints
still requires a material decision; routine Helm policy/default selection does not.

## D20. Permanent Cloud Dedication and Shared Local Computer Association

- Authority: REQ-3/6/8/14, revision 3.
- Decision owner: requester.
- Confirmed clarification of the established relationship: 2026-11-02 KST.

Cloud Agent/Computer association is always dedicated 1:1. This is not a temporary
restriction on the first Kubernetes implementation. A cloud Computer is not a
future shared pool for several Agents merely because the common model can
represent local sharing.

One local Mac Computer can serve multiple Agents of the same owner. Each Agent
has one current Computer. The common domain model must allow that local
relationship without registering duplicate Computers or cloning Connector/
reconciler ownership for each local Agent.

Connection and reconciliation ownership belongs to the Computer's Connector
under D18, not independently to each Agent using that Computer. Agent-level
authorization and any separately defined data boundaries are not erased by
sharing a Computer. This does not add a new sandbox/virtualization mechanism.

The cloud creation path allocates the dedicated Computer. The local association
path uses the existing Mac Computer; D12's Agent-creation deduplication remains
applicable without requiring another Computer allocation. Removing a local
Agent does not delete the shared Mac or another Agent's association. These are
consequences of the confirmed relationship and existing non-target protection.

Earlier summary wording that limited dedication to the initial cloud milestone
was too narrow. The clarification above governs current design and closes the
relationship question; it must not be presented for approval again.

## D21. Per-Request gRPC Between Control Peers

- Authority: REQ-15/17/18, D4/D6/D8.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

Use a separate gRPC invocation for each server-to-server forwarded request.
Ordinary request/response work uses unary RPC; work needing continuous output or
bidirectional interaction uses an appropriate streaming RPC. Reuse Channels and
HTTP/2 connections rather than opening a TCP connection for every request.

Do not introduce one custom permanently open peer-to-peer multiplexing stream
as the universal container for all forwarded calls. Standard per-call response,
status, cancellation and deadline boundaries reduce the separate multiplexing
protocol the application must maintain.

This decision concerns server-to-server forwarding. Provider/Connector-initiated
long-lived bidirectional Control streams remain under D14. The owner Control
still correlates a forwarded call with the target component's request/response
messages and propagates cancellation/deadlines where supported. Native gRPC
between peers does not automatically supply reverse RPC on a component's stream.

The D8 pre-forwarding reroute and post-forwarding uncertainty rules remain.
No persistent command history, broker or automatic replay is introduced.

## D22. Domain Service and Group Map

- Authority: REQ-10/15/17, D2/D3/D14/D21.
- Decision owner: requester for material boundary changes.
- Accepted: 2026-11-02 KST. Service names are illustrative.

### Current Evidence

`proto/chapchoo/accounts.proto` currently puts authentication and personal
presentation reads/writes in Auth, signup/recovery and teams/invitations in
General, and initialization/default settings/account grants in Admin.
`rust/crates/rpc/src/adapter.rs` implements these traits on one RpcService
wrapping AccountsService. File splitting alone would not establish domain-level
wire contracts or narrower handler responsibilities.

### Accepted Boundary

Use domain services as contracts; use Client/Admin/Control as composition groups,
not replacement names for another catch-all service. Common services needed by
multiple groups are registered once per listener. Reuse underlying authorization
and domain operations; different privileged surfaces do not imply separate
copies of business rules.

| Group | Domain service boundary | Existing or required scope |
| --- | --- | --- |
| Shared Client/Admin dependency | Authentication | Login methods, password/code login, session renewal, identity and logout |
| Shared Client/Admin dependency | PersonalPreferences | Read public display defaults and update the current user's display settings |
| Client | AccountLifecycle | Existing signup and password-reset preview/redemption flows |
| Client | Teams | Existing team and membership management, ownership transfer, leave/delete |
| Client | Invitations | Existing send/list/accept/decline invitation flows |
| Client | Agents | Approved personal Agent creation, list/detail and removal |
| Client | Computers | Approved Computer view, capability and manual lifecycle requests |
| Client | ProviderCatalog | Available authorized provider choices for cloud creation, public-safe projection |
| Admin | Installation | Existing setup status and initial administrator setup |
| Admin | AccountGrants | Existing signup grants/password-reset grants, issue/list/revoke only |
| Admin | DisplayDefaults | Existing administrator update of installation display defaults |
| Admin | ProviderInventory | Approved read-only provider status and managed-Computer inventory |
| Control | ProviderAuthentication | Approved Provider issuance/rotation/recovery authority boundary |
| Control | ConnectorAuthentication | Approved execution-bound session create/exchange/refresh boundary |
| Control | ProviderConnection | Authenticated Provider management stream and reports |
| Control | ConnectorConnection | Authenticated Connector stream, current ownership and reports |
| Internal listener only | PeerRelay | D21 per-request forwarding to an exact local component connection |

Authentication plus preferences remain common dependencies where currently
needed. This does not imply that selecting Control exposes every human login or
Admin API. Provider/Connector credentials retain their distinct authority checks.
Actual dependency validation belongs in composition rather than address inference.

### Refactoring and Scope Guard

Move existing methods to the appropriate generated domain service and update
server adapters, generated clients, call sites and affected tests together.
Preserve existing authorization, response semantics and product flows. Do not
add broad user administration merely because an Admin group exists, or add
provider configuration/cleanup controls to read-only inventory.

Give adapters appropriately scoped domain dependencies; do not retain one
ever-growing transport handler solely for convenience. Share persistence and
domain rules instead of copying them for each group. Exact crate/file/helper
layout does not require a separate product choice.

This decision selects neither additional deployable processes nor extra ports.
It adds no envelope-based universal business dispatch service in place of the
domain APIs. PeerRelay's component delivery envelope remains an internal
transport concern, with detailed typing and streaming mechanics in P4.

## D23. Current Connection Admission and Control Directory Lifetime

- Authority: REQ-7/12/17/18, D6-D8/D14/D16/D18/D21.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

### Fixed Foundation

The product DB owns the current target-to-Control/Connection ID directory.
Computer Provider and Computer Connector connections are independent targets.
Exact opaque identities, Connector-connection-owned reconciliation, component
heartbeat/validity defaults, and safe pre-dispatch rerouting are already accepted.
Directory expiry is loss of routing authority, not proof of execution absence.

### Accepted Choice and Rationale

Two admission policies remain materially different: allow an authenticated new
connection to conditionally replace the exact current connection immediately,
or reject it until the former connection explicitly releases ownership or
expires. Select immediate conditional replacement so a component that has
lost its connection can reconnect without waiting for the old Control to detect
the loss. This permits an authorized competing connection to displace a still
live connection; authorization is necessary but does not imply peaceful clients.

After authenticating the target/session, capture the exact current ownership
tuple for that admission attempt. Prepare the local stream without authorizing
work. Revalidate credential/session authority and conditionally install the new
opaque Connection ID in a short DB-only transaction, only if the observed tuple
still matches and the receiving Control registration remains eligible. An absent
tuple is an explicit conditional insertion case.

A losing admission attempt is rejected; it must not repeatedly reread and steal
ownership back inside the same attempt. A fresh reconnect is a new admission.
Only the committed current connection may be acknowledged for active work and
start its Connector-scoped reconciler. Partial activation failure conditionally
releases only its own tuple; it cannot restore the displaced connection.

Renewal, reporting and close use the exact current tuple. Old heartbeats,
late close callbacks and delayed reconciler results cannot renew, clear or
overwrite a replacement. Best-effort notification may close an old stream sooner,
but notification delivery is not the source of authority.

### Directory Registration and Failure Boundary

Bind the internal listener and complete DB readiness before publishing a routable
Control registration. A new process uses a new opaque instance identity, even
when its endpoint is reused. Renew only the exact live registration. Draining
stops new admissions and new dispatch before retiring owned registrations;
already forwarded work retains the existing uncertainty rules.

Use configurable Control-directory renewal/validity values with defaults
of 20/60 seconds, selected explicitly here rather than inferred from D16's
component settings. A current route requires both eligible Control registration
and valid component connection. Expiry makes a route unavailable, never evidence
that its Computer has stopped or that a command did not execute.

Revalidate current DB authority at new dispatch and when accepting authoritative
reports/results; local route caches alone do not grant dispatch authority.
If DB authority cannot be established, pause new authoritative work instead of
continuing indefinitely from cached ownership. Recovery revalidates exact
identity before resuming. DB operations remain short and DB-only; network
forwarding and external effects occur after the transaction.

Ownership transfer cannot retract an already queued/sent external command.
A successful DB check followed by network work is not an atomic exactly-once
boundary. D8 uncertainty and existing execution-identity checks still apply;
neither replacement nor expiry authorizes replay or proves prior retirement.
Detailed dispatch/cancellation envelopes and deterministic race verification
remain P4/Design work.

## D24. Typed Per-Call Delivery, Outcomes and Cancellation

- Authority: REQ-4/7/12/17/18, D7-D9/D16/D21-D23.
- Decision owner: requester.
- Accepted: 2026-11-02 KST.

### Wire Contract Choice

Use operation-specific Protobuf payloads and explicit outcome variants
inside the internal delivery envelope, rather than opaque payload bytes or JSON
whose meaning is selected by an arbitrary operation string. Public domain
services remain D22's contracts; this envelope is only the internal peer-to-
component delivery boundary. Exact message names and file layout are local
implementation details.

Carry the logical Request ID, an opaque per-invocation Call ID, exact expected
target/owner/Connection ID, execution identity when applicable, remaining timeout
budget, and the typed operation payload. A permitted D8 reroute retains Request ID
and the original deadline, but gets a new Call ID for the distinct attempt.
Call ID correlates transient response/cancellation work; it is neither a numeric
generation nor a durable operation-history record.

The owner maps the peer invocation to component-stream request/response messages.
Responses and cancellation target that exact call and connection. A late reply
to a retired call cannot complete another attempt with the same Request ID.
Request-scoped response state is bounded and released at termination. Independently
authorized current observations may still update current state through their
normal report path; they do not reopen a cancelled response waiter.

### Deadline and Cancellation

Forward the remaining call budget, accounting for local waiting and forwarding;
never restart the timeout at each hop or permitted reroute. Use monotonic local
elapsed-time accounting rather than requiring synchronized host clocks for RPC
budgets. The existing ten-second Provider command response deadline caps that
command exchange, not the entire Computer provisioning lifecycle.

Cancel only the invocation's waiter/stream and propagate a request-scoped
best-effort cancellation to component work that supports it. Do not tear down
the shared Provider/Connector stream, another invocation, or the reconciler
because a caller closes its response stream. A cancellation frame is not evidence
that an already submitted external effect was undone.

Durable desired state and accepted lifecycle progress under D9/REQ-12 are
independent of a browser/RPC lifetime. Closing a response or timing out does not
change running/stopped/deleting intent. This is a consequence of the accepted
intent model, not a new cancel-operation product capability. Lifecycle acceptance,
command response and observed convergence remain different results under REQ-4.

### Explicit Outcomes and Bounds

Distinguish known pre-forwarding rejection, target-reported success/failure, and
possible-dispatch uncertainty. A plain gRPC timeout/unavailable/cancelled status
does not prove non-dispatch. Only D8's authoritative non-owner rejection enables
automatic rerouting; other errors retain their own explicit policy.

Use bounded pending-call counts, frames and queues. Reject overload before
dispatch where possible; after possible dispatch preserve uncertainty rather
than falsely reporting safe rejection. Keep component heartbeat processing
independent of command-response pressure as required by D16.

For streaming calls, distinguish payload chunks from terminal outcome and retain
bounded backpressure. Disconnect without the required terminal evidence is not
success and does not trigger automatic replay/resumption. This adds no durable
stream archive or new user-facing streaming feature.

Reference: the [gRPC cancellation guide](https://grpc.io/docs/guides/cancellation/)
defines cancellation as loss of interest and recommends stopping request work
and propagating cancellation. Mapping it onto a shared component stream requires
the explicit per-call boundary above; it does not supply business rollback.

## Scope Interpretation After Requirements Revision 4

The accepted Mac-as-Computer and same-owner sharing boundaries constrain the
common system model for future integration. Concrete Mac enrollment,
authentication workflow and runtime implementation are deferred. Earlier
references to local mapping or Mac runtime verification as current P6/P7 work
are narrowed by Requirements revision 4; they are not prerequisites for this
Design. Common standalone composition, SQLite and memory/network transport
remain in scope. This records the existing delivery boundary, not a new
technical decision or an additional requester interview topic.

## D25. Bounded Browser Projection Reads

- Authority: REQ-2/4/7/10/11/12; D22; current account-web Spec.
- Decision owner: requester.
- Accepted: 2026-11-02 KST through the direct implementation request following complete Design review.

Use the existing app-owned POST tRPC/TanStack Query and generated unary gRPC
path for Computer detail/list and read-only Provider inventory observation.
Read on entry and after accepted mutation; while visible, refresh every five
seconds during pending lifecycle work and every fifteen seconds when stable.
Pause hidden views, read on return, avoid overlap, scope caches by identity,
and consolidate list reads rather than polling every row separately.

Preserve existing account identity-query behavior. A failed read leaves an
explicit stale/unavailable view; a later scheduled read is new observation,
not replay of a mutation or automatic request-level retry. No browser command
queue or durable event history is introduced.

The alternative is a new push/subscription path with its own authentication,
backpressure, subscriber lifetime and distributed notification mechanism.
Current browser transport rejects streaming formats and no requirement mandates
instant push, so bounded unary observation is recommended. D24 internal streams
do not by themselves authorize browser push or event replay.

## D26. Deployment-Provisioned System-Owner Bearer Adapter

- Authority: REQ-1/10; D13/D15; current cryptographic primitives.
- Decision owner: requester.
- Accepted: 2026-11-02 KST through the direct implementation request following complete Design review.

D13 fixes owner-authorized Provider key management and deployment-configured
system authentication. The current source contains no implemented system-owner
authenticator. Recommend a deployment-provisioned opaque bearer, verified using
the existing digest/constant-time primitive, for this distinct owner capability.

Control configuration binds its verifier to a stable system-owner identity and
permitted Provider identities. The deployed Provider receives the corresponding
owner credential through an operator-managed Secret. Authenticate and check
that scope for issuance, rotation and recovery; a Provider communication key
cannot substitute for owner authority. Self-asserted names/owner IDs do not
grant registration or key-management permission.

Generate/provision outside normal startup. Do not reuse the installation
encryption root, setup proof, human login session or Provider communication key.
Do not create a persisted system-user account or a web registration flow.
The credential is authorized while its configured binding is enabled, not by
renewing it with a Provider key. Configuration replacement requires consistent
issuer rollout/withdrawal; static configuration does not promise instantaneous
revocation across unmodified server instances. Component key/session revocation
remains DB-authoritative under accepted decisions.

An external identity-provider or Kubernetes workload-token verifier is a viable
alternative, but introduces issuer/audience/subject, provisioning and refresh
contracts not selected by the current foundation. Recommend the configured
opaque-credential adapter rather than silently introducing such a trust path.
