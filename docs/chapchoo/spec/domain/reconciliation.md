---
title: "Owner-scoped convergence and restoration"
---

# Owner-scoped convergence and restoration

Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.

## Owner-scoped convergence and recovery

Control owns a 15-second reconciliation worker independently of lease renewal.
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
proof. Successful Kubernetes Pod GET 404 is sufficient current absence: it
retires the old execution and permits the Running target to create a successor
with the existing PVC. This works after Provider reconnect or restart without
terminal-event history. Kubernetes blocked work is still observed so disappearance can
converge; a present safely failed preparation retains its explicit retry path.
Failed lookup, foreign present UIDs and missing original storage are distinct.
Exact committed absence remains independently storage-validated at command dequeue.

Delete sorts the complete reserved inventory Pod, bootstrap Secret, then PVC,
and sends sequential frames of at most 32 resources under one original deadline.
Only the final completed batch requests whole-Computer cleanup confirmation.
Any uncertain/failed frame stops later dispatch; no implicit effect replay occurs.
Already committed current-source absence can finish confirmation independently
of a lost final response.

Real local Control streams and supported-client synthetic HTTP exercise automatic
create/stop/start/restart, missing-Pod restoration, terminal restoration with one
original PVC, and deletion across more than 32 historical resources, including
an eleven-second nonfinal Secret finalizer that outlives the ten-second caller.
Fresh convergence completes only the never-submitted remainder and the fixture
asserts exactly one DELETE per object. A separate Connector Control takes precedence and uses network peer forwarding while the
Provider-only fallback remains inactive. These fixtures are not live Kubernetes
or browser product-flow release-gate evidence.

## Connections

- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Execution reservation, evidence and cleanup](execution-evidence.md) — Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.
- [Docker retirement and exact-call correlation](native-retirement.md) — A live manifest qualifies continuation of one possibly accepted native call; lost correlation never becomes replay authority.
- [Memory and direct peer delivery](peer-delivery.md) — Memory and network delivery use the same generated handlers. A fresh route is a snapshot, not permission to replay a possibly forwarded call.

## References

- [`rust/crates/rpc/src/control_streams/reconciliation.rs`](../../../../rust/crates/rpc/src/control_streams/reconciliation.rs)
- [`rust/crates/rpc/src/control_streams/reconciliation/cleanup.rs`](../../../../rust/crates/rpc/src/control_streams/reconciliation/cleanup.rs)
- [`rust/apps/chapchoo/src/control_plane.rs`](../../../../rust/apps/chapchoo/src/control_plane.rs)
