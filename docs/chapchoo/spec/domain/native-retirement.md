---
title: "Docker retirement and exact-call correlation"
---

# Docker retirement and exact-call correlation

A live manifest qualifies continuation of one possibly accepted native call; lost correlation never becomes replay authority.

## Docker retirement correlation

The server claims fresh retirement/deletion once through the durable phase CAS,
reserving bounded connection-local manifest capacity before its await. Each
manifest fixes the Computer/request/execution, source connection, immutable
reservation-set fingerprint, exact native command/call and dispatch stage.
Local and peer forwarding share the same gate; CheckCommand validates that same
registry after its fresh server authorization. Unknown or changed calls/targets
cannot create mutation permission. Queue-budget reduction and refreshed absence
flags do not alter the immutable call target.

Potentially forwarded/in-flight/uncertain stages are observation-only on subsequent
convergence. The response waiter's deadline is not the native task's deadline:
its expiry preserves only the exact already-forwarded task's next-stage checks
under the still-current source. It grants no second submission. An explicit
Provider-reported native uncertain outcome, completed result, source replacement
or correlation loss ends that task's continuation authority. An exact late result
can settle its manifest after the original waiter is gone; another call's result
cannot release or advance it. Only proven
NotStarted before acceptance can release the exact initial admission. Once a
prefix completed, a later NotStarted retains the request fence and permits only
its known-unsent suffix. Lost connection/process correlation cannot be rebuilt
from the durable phase.

Normal same-connection Delete retires its current container before the original
saved-volume suffix. Advancement requires a positively settled prior stage and
fresh committed absence for its complete target set, revalidated source/request/
Delete intent and no current/live execution. Already retired history is freshly
observed, not retired again. All reserved resources must be absent to finish
cleanup observationally after correlation loss; subset absence never proves the
volume suffix was not submitted. Stop, Restart and qualified terminal retirement
use the same native gate. Existing Kubernetes dispatch behavior is unchanged.

Synthetic Provider/real Store tests cover normal suffix progression, cancellation
and exact late completion, lost-manifest observation-only reconnect, NotStarted/
late-result ABA and the Stop/Restart/terminal entry points. They do not validate
real Engine lifecycle or Mac Desktop compatibility. The separate native
[Docker adapter](computer-docker.md) has public-SDK/real-Control fixtures for
allocation ACK order, normal retirement/suffix progression, source replacement
before ACK, and unknown Start across stream reconnect without replay.

## Connections

- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Computer lifecycle admission and Provider keys](computer-lifecycle.md) — Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.
- [Execution reservation, evidence and cleanup](execution-evidence.md) — Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.
- [Docker connection, volume and container contracts](computer-docker.md) — The Docker adapter owns one selected source and exact native resources, with uncertainty distinct from absence or confirmed cleanup.

## References

- [`rust/crates/rpc/src/control_streams/native_retirement.rs`](../../../../rust/crates/rpc/src/control_streams/native_retirement.rs)
- [`rust/crates/rpc/src/control_streams/native_retirement/tests.rs`](../../../../rust/crates/rpc/src/control_streams/native_retirement/tests.rs)
