---
title: "Memory and direct peer delivery"
---

# Memory and direct peer delivery

Memory and network delivery use the same generated handlers. A fresh route is a snapshot, not permission to replay a possibly forwarded call.

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

Resource command admission compares the complete committed native reservation,
identity, execution binding and original allocation witness. Canonical current
context IDs are correlation authority, not substitutes for an existing native
resource's original labels. Missing/malformed witnesses reject at wire ingress.

## Queued-input authority

Internal `QueuedInputAuthority` carries the issued input ID, actual
Agent/conversation and exact processing claim. It is not a credential lookup,
asserted owner or new native grant. The receiving completed Store operation
resolves the input's current submitting session under the User lock and
revalidates private ownership, Team/Computer, exact live claim and database time.
It binds the command to the actual Computer, current lifecycle Request ID,
current non-retired execution, Agent, conversation and input operation.

Only Connector input-owned Setup with matching possible preparation, saved
supply/context and sealed instruction evidence, or Input with matching possible
forwarding and exact native target, is eligible. Known results do not grant
another effect. Provider commands, convergence, Observe, Answer and Stop reject
this context. Startup/project-refresh Setup has a different operation and cannot
borrow an input claim; the existing runtime-owned startup reconciler uses its
separate reconciliation authority and dispatch gates.

Ordinary user calls retain current credential admission. Queued context changes
neither exact-call ownership nor delivery retry limits: an uncertain invocation
is never retransmitted merely because a session rotated or a claim is current.

## Connections

- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Owner-scoped convergence and restoration](reconciliation.md) — Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.
- [Conversation storage](conversation-storage.md) — Durable queued input, exact claims, current submitting sessions and sealed native facts.

## References

- [`rust/crates/rpc/src/memory.rs`](../../../../rust/crates/rpc/src/memory.rs)
- [`rust/crates/rpc/src/peer.rs`](../../../../rust/crates/rpc/src/peer.rs)
- [`rust/crates/rpc/src/delivery.rs`](../../../../rust/crates/rpc/src/delivery.rs)
