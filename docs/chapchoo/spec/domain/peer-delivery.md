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

## Connections

- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Owner-scoped convergence and restoration](reconciliation.md) — Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.

## References

- [`rust/crates/rpc/src/memory.rs`](../../../../rust/crates/rpc/src/memory.rs)
- [`rust/crates/rpc/src/peer.rs`](../../../../rust/crates/rpc/src/peer.rs)
- [`rust/crates/rpc/src/delivery.rs`](../../../../rust/crates/rpc/src/delivery.rs)
