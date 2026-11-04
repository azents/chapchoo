---
title: "Control admission, streams and calls"
---

# Control admission, streams and calls

Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.

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
The stream worker drain joins remaining retained workers after a worker failure,
within its existing twenty-second drain budget. A failure is not successful drain;
deadline expiry still aborts and joins the remaining workers.
Readiness recognizes only registered business service names.

Bundled General/Admin web clients require their independent
`CHAPCHOO_GENERAL_BACKEND_URL` / `CHAPCHOO_ADMIN_BACKEND_URL`; neither target is
inferred from a selected group or another client's endpoint. Operators can
explicitly configure equal URLs for a combined deployment.

`ProviderAuthentication.Exchange` validates the explicitly selected evidence
against captured bootstrap policy or a database-owned registration credential.
It resolves canonical Provider/owner scope and atomically creates missing permitted
bootstrap metadata with scoped operational-key issuance. Existing metadata and
native binding cannot be overwritten by reauthentication. SA verification is
external to DB transactions; current stored random-token status is checked inside
the owning exchange transaction. The old shared system-owner bearer and
ConfigureSystemProvider RPC are removed. Existing RotateKey/RevokeKey operations
are user-account-authorized management, not a SYSTEM bearer path.

Kubernetes and Docker Providers may run externally. Their authenticated Control
boundary, not product database access, performs server-owned mutations. Standalone
has a distinct private Provider-bound authentication adapter, not Trust on the
public component or existing peer listener. See
[Provider authentication](provider-authentication.md). Neither executable Provider
receives database/root-key configuration.

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
resource kind, native scope, name, execution and optional native identity must
match reservations. Kubernetes namespace/UID and Docker daemon/full container ID/
original-volume metadata are separate typed payloads; mixed or unknown input is
rejected at ingress, not mapped to another substrate or Secret.
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
native effect or a reusable permission to keep acting after authority changes.
Docker mutation dequeue additionally checks the exact live native manifest in
the owning connection registry; a valid DB snapshot alone is insufficient.
Provider CheckCommand returns `prior_running_completion` only from the exact
execution's existing `running_confirmed_at` fact read under that current-source/
request snapshot. This is read-only prior-completion evidence, not a new stored
state, native quiescence attestation or dispatch/replay grant. A Provider's
same-process unknown Start or later native stage remains a veto; bare running
state cannot prove a possibly pending Start settled. Existing Kubernetes callers
may ignore this evidence without changing their native behavior.

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

## Connections

- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Docker retirement and exact-call correlation](native-retirement.md) — A live manifest qualifies continuation of one possibly accepted native call; lost correlation never becomes replay authority.
- [Provider process and allocation acknowledgements](provider-process.md) — External trusted Providers own native IO, while authenticated server operations own durable mutations and evidence acknowledgements.
- [Connector process lifecycle](connector-process.md) — The execution-local Connector exchanges bootstrap authority, maintains its own stream and reports connectivity, not substrate power or harness readiness.
- [Memory and direct peer delivery](peer-delivery.md) — Memory and network delivery use the same generated handlers. A fresh route is a snapshot, not permission to replay a possibly forwarded call.
- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.

## References

- [`proto/chapchoo/control.proto`](../../../../proto/chapchoo/control.proto)
- [`rust/crates/rpc/src/component_auth.rs`](../../../../rust/crates/rpc/src/component_auth.rs)
- [`rust/crates/rpc/src/control_calls.rs`](../../../../rust/crates/rpc/src/control_calls.rs)
- [`rust/crates/rpc/src/control_streams.rs`](../../../../rust/crates/rpc/src/control_streams.rs)
