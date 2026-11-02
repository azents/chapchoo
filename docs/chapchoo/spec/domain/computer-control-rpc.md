---
title: "Computer Control RPC Boundaries"
created: 2026-11-02
tags: [computer, control, grpc, security]
spec_type: domain
domain: computer
code_paths:
  - proto/chapchoo/control.proto
  - rust/crates/protocol/src/lib.rs
  - rust/crates/rpc/src/component_auth.rs
  - rust/crates/rpc/src/control_calls.rs
  - rust/crates/rpc/src/control_streams.rs
  - rust/crates/rpc/src/control_streams/tests.rs
  - rust/crates/rpc/src/control_streams/reconciliation.rs
  - rust/crates/rpc/src/control_streams/reconciliation/cleanup.rs
  - rust/apps/chapchoo/src/provider/tests.rs
  - rust/crates/rpc/src/delivery.rs
  - rust/crates/rpc/src/groups.rs
  - rust/crates/rpc/src/memory.rs
  - rust/crates/rpc/src/peer.rs
  - rust/crates/rpc/src/reports.rs
  - rust/crates/persistence/src/delivery.rs
  - rust/apps/chapchoo/src/config.rs
  - rust/apps/chapchoo/src/connector.rs
  - rust/apps/chapchoo/src/provider.rs
  - rust/apps/chapchoo/src/control_plane.rs
  - rust/apps/chapchoo/src/runtime.rs
  - rust/apps/chapchoo/src/health.rs
last_verified_at: 2026-11-02
spec_version: 1
---

# Computer Control RPC Boundaries

## Component and peer authority

The reusable RPC crate provides distinct Computer Provider and Computer Connector
stream/authentication factories and a separate trusted-network peer factory.
`ServiceGroups` composes Client, Admin and Control handlers into one public service
union. Authentication and PersonalPreferences are registered once for either
Client/Admin group, including duplicate group selection. The public union never
registers PeerRelay. The foreground application selects this union with
`serve --groups client,admin,control` (the default) on one `--listen` address.
`--groups` without values selects no backend for explicit web-only operation.
The old backend Role and separate General/Admin port configuration are removed.

A network Control requires both `--peer-listen` and `--peer-advertise`; these
settings are rejected when Control is not selected or the public/peer addresses
are equal. Peer advertisement accepts only supported plaintext HTTP; HTTPS is
rejected rather than advertising a TLS endpoint that this listener cannot serve. The peer endpoint is published only after selected sockets are bound.
A local-only Control owns a memory peer server without opening a peer socket.
Control renewal runs every twenty seconds with the existing sixty-second lease.
A failed renewal/worker health check closes admission and fails the owned runtime.
Shutdown drains the exact Control, closes stream waiters and conditionally
withdraws only its connections, under the foreground runtime's cleanup budget.
Readiness recognizes only registered business service names.

Bundled General/Admin web clients require their independent
`CHAPCHOO_GENERAL_BACKEND_URL` / `CHAPCHOO_ADMIN_BACKEND_URL`; neither target is
inferred from a selected group or another client's endpoint. Operators can
explicitly configure equal URLs for a combined deployment.

System-owner configuration is optional but complete when present:
`CHAPCHOO_SYSTEM_OWNER_ID`, comma-separated `CHAPCHOO_SYSTEM_OWNER_PROVIDER_IDS`
and `CHAPCHOO_SYSTEM_OWNER_TOKEN`. The configured bearer is verified against its
exact provider scope. ConfigureSystemProvider updates only owner-authorized
Provider metadata/capabilities; it allocates no Computer or Kubernetes resource.

Provider credentials cannot act as owner credentials. Connector bootstrap keys
can only exchange for operational keys. Operational Connector authority is bound
to one execution and remains independent of Provider credential revocation.
Every control message and oneof has field-free Debug output, including trusted
session digests and bootstrap payloads.

Peer requests carry either the authenticated session digest for fresh owner
permission checks, or the exact current reconciliation context and live owner.
An asserted Computer ID alone is insufficient. A live Connector owner takes
precedence over Provider-owner management authority. The Provider owner is admitted
only while no eligible Connector route exists. Current Request ID, applicable
execution identity and current reserved resources are read again before forwarding.
Prepare/retire/cleanup commands must match the current durable lifecycle intent;
resource kind, namespace, name, execution and optional UID must match reservations.
Prepare also verifies the execution-bound bootstrap session and key digest.
Provider-originated bootstrap session issuance/revocation verifies the exact live
Provider source inside the same Provider/Computer-locked transaction as credential
mutation. A separate successful connection precheck is not a mutation permit.

Provider command dequeue checks authenticate the exact current Provider source
and Computer request together in one short transaction, then validate the same
command contract used by the peer adapter. A valid Prepare dequeue records the
existing first-start-dispatch marker with fresh source/current-intent checks.
Connector command dequeue verifies the current request, execution, session and
exact connection. Checks do not renew connection leases. A check is not an atomic
Kubernetes effect or a reusable permission to keep acting after authority changes.

## Stream and call lifetime

Streams require an authenticated Hello before admission. The service generates
opaque connection identities and uses conditional captured-owner admission only
after the bounded local stream is prepared. Durable admission and local
activation are serialized, so an admitted but delayed predecessor cannot replace
or close a later committed winner. Replacement closes only the old registry entry. Delayed old frames and cleanup cannot remove a successor.
Heartbeats renew exact current authority; ordinary frames and periodic checks do
not extend TTL. Incoming report processing is separate from heartbeat processing.

Resource and execution observations go through current-source storage checks.
Computer Connector readiness asserts connectivity, never substrate retirement,
power-off or storage cleanup. Current reports may update evidence without
reopening a retired transient call.

Each connection has bounded command and priority channels, local pending permits
and one lifecycle dispatch permit. Pending capacity is also shared across all
connections in the Control. Current defaults are 64 streams, 256 total pending
calls, 32 pending calls per connection, eight queued commands and four priority
frames. Frames are limited to 64 KiB; resource batches are limited to 32.

Cleanup retains at most 32 connection-local correlations, each describing only
one potentially accepted batch of at most 32 exact resource identities. A timed-out
response does not discard that manifest or enqueue the batch again. A fresh sweep
observes every resource in the exact batch, including historical bootstrap
Secrets, and requires committed current-source absence for the complete ordered
prefix before dispatching only its never-submitted remainder. A correlated
NotStarted or proven pre-queue refusal can retry an unsent batch. Completed or
superseded correlations are pruned before retained cleanup inventory is collected.
This is bounded transient call correlation, not a durable command ledger. Losing
it on connection/process replacement never authorizes reconstructing/replaying
possibly accepted deletes; complete durable absence can finish cleanup, while
unknown earlier work stays uncertain.

A call matches Request/Call/Connection/Computer/execution identity. Queue time
consumes its original deadline, capped at ten seconds. Expired queued work is not
dispatched. Timeout or connection loss after acceptance is uncertain. Cancellation
releases only that waiter and sends a best-effort exact-call cancel; it does not
close a shared stream or prove that an external effect stopped.

## Computer Provider process and acknowledged evidence

`chapchoo computer-provider` selects explicit in-cluster credentials or an
explicit kubeconfig path, not ambient cluster discovery. It accepts runtime
namespace/image/storage/resources/ServiceAccount policy and a scoped owner token
through `CHAPCHOO_PROVIDER_OWNER_TOKEN`, never a credential CLI argument.
It owner-authenticates stable metadata configuration and communication-key
issuance, opens its own authenticated stream and maintains heartbeats separately
from lifecycle work. Proactive owner-authorized key renewal preserves its
absolute schedule across reconnects. Provider keys cannot self-rotate.

One `Effects` owner survives stream replacement and caller cancellation. A
started SDK task retains the lifecycle permit until it actually ends, including
when its result waiter is gone. Each task captures its original connection and
request identities; a later connection does not adopt its calls or reports.
Shutdown drains started SDK work under the component command's finite budget.

ProviderAuthentication.Report returns a unary ACK only after its source-bound
evidence operations complete. It shares resource/execution mapping with stream
reports. Every operation independently revalidates current Provider source and
Computer request; a batch can have earlier persisted evidence when a later item
fails, so failure is not rollback proof. The ACK is not a command replay receipt.
The process reports actual PVC UID and waits for its ACK before creating the
bootstrap Secret, then reports/ACKs that UID before creating the standalone Pod.
It rechecks the updated command's current source/request/resource authority
between stages. A lost ACK stops later effects; it is never treated as success.

Observe commands report only current source observations. Retirement uses the
SDK's exact ordered terminal/deletion proof, never a successful delete response.
Cleanup cannot destroy the saved PVC while Pod retirement remains unknown.
Completed cleanup requires the existing storage-side confirmation predicates.
Unknown native outcomes report uncertainty instead of automatically repeating
the effect. A same-name foreign UID/ownership or missing original environment
blocks the request. Heartbeat and exact late-call identity remain independent.

## Owner-scoped convergence and recovery

Control owns a two-second reconciliation worker independently of lease renewal.
Current Connector owners are selected first; they invoke the exact current
Provider owner through typed direct peer delivery. Provider owners handle only
Computers without an eligible Connector. Per-Provider scans are bounded to 20
Computers with a cursor and one concurrent convergence permit. A source/current
request check precedes each operation; these checks are not an atomic substrate
transaction.

A short Computer-locked CAS claims `prepare_admitted_at` once before dispatch.
Reconnect does not replay an accepted/uncertain Prepare. `initial_start_dispatch_at`
is recorded separately at actual component dequeue; delayed-start display uses
that actual event. An authorized new execution clears both markers only after
retirement and original-storage gates; retrying an existing reservation does not.
Migration 4 adds and conservatively backfills the admission field on both DBs.

Each pass observes reserved resources before deciding. Unexpected definitive
terminal evidence under Running intent requests exact retirement with
`require_terminal`; the Provider freshly reobserves and rejects live or unknown
Pods. A successor is reserved only after committed retirement and original-PVC
proof. The SDK retains at most 256 exact UID-bound terminal witnesses across
same-process reconnects. They do not survive a process crash. Bare 404, node loss,
foreign UIDs and missing original storage never authorize recreation. Exact
committed absence remains independently storage-validated at command dequeue.

Delete sorts the complete reserved inventory Pod, bootstrap Secret, then PVC,
and sends sequential frames of at most 32 resources under one original deadline.
Only the final completed batch requests whole-Computer cleanup confirmation.
Any uncertain/failed frame stops later dispatch; no implicit effect replay occurs.
Already committed current-source absence can finish confirmation independently
of a lost final response.

Real local Control streams and supported-client synthetic HTTP exercise automatic
create/stop/start/restart, missing-Pod uncertainty, terminal restoration with one
original PVC, and deletion across more than 32 historical resources, including
an eleven-second nonfinal Secret finalizer that outlives the ten-second caller.
Fresh convergence completes only the never-submitted remainder and the fixture
asserts exactly one DELETE per object. A separate Connector Control takes precedence and uses network peer forwarding while the
Provider-only fallback remains inactive. These fixtures are not live Kubernetes
or browser product-flow release-gate evidence.

## Computer Connector process

`chapchoo computer-connector` requires an explicit trusted-network Control
endpoint, execution session ID and bootstrap key. The initial key is read only
from `CHAPCHOO_CONNECTOR_INITIAL_KEY`, not a CLI argument. The process has no
product DB, installation root, Provider or owner credential dependency.

It exchanges the bootstrap key, opens its own operational stream, verifies Hello
acceptance, sends heartbeats, requires finite acknowledgements and uses current
command-dequeue checks before reporting execution connectivity. It handles only
ObserveConnector; readiness here is connectivity, not harness or power state.
Proactive key refresh uses an absolute schedule retained across reconnects; a
refreshed key starts a new authenticated stream before the old overlap ends.
Disconnection creates a fresh bounded-admission attempt after a one-second delay,
never replay of previous calls. Revoked/unauthorized authority ends the process.
SIGINT/SIGTERM cancels its owned loop under a finite shutdown budget.

A real child-process test provides only execution bootstrap inputs, waits for the
current Connector route, revokes the Computer Provider key, then obtains an
independent Connector observation and verifies owned process shutdown.

## Memory and direct delivery

`memory_transport` supplies bounded Tokio duplex IO to the same generated gRPC
clients, Protobuf codecs and Tonic server handlers used by TCP. The composition
root owns the server future and shutdown. There is no direct handler bypass or
unowned background server.

`PeerDelivery` reuses HTTP/2 Channels, limits total pending delivery and bounds its
remote channel cache. A separately registered local memory Channel is retained
outside the remote cache. A fresh route lookup occurs before every delivery
attempt. Only an explicit `NOT_OWNER` response allows one re-resolution under the
same Request ID and original deadline, with a new Call ID. Native gRPC status,
timeout and connection failure never trigger automatic invocation replay; possible
forwarding is reported as uncertain.

No stream, transient call or response is stored as a durable transport log. DB
checks followed by network/SDK work remain non-atomic. These local transport tests
are not Kubernetes retirement/quiescence or browser product-flow evidence.
