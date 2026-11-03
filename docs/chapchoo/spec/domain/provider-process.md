---
title: "Provider process and allocation acknowledgements"
---

# Provider process and allocation acknowledgements

External trusted Providers own native IO, while authenticated server operations own durable mutations and evidence acknowledgements.

## Computer Provider process and acknowledged evidence

The Provider is an external process, including when run by a trusted third party.
It does not load backend DB configuration, connect to the product DB or receive
the installation encryption root. Its product operations use authenticated
Control RPC; storage transactions belong to the server. The single executable
also contains backend commands, but their DB initialization is not shared with
the `computer-provider` or `docker-provider` commands.

A child-process boundary test reaches authenticated registration with no DB/root
configuration and with deliberately invalid backend-only settings. A synthetic
Control denies registration before native effects; this test requires neither a
product database nor a live Kubernetes cluster. It is not complete lifecycle or
third-party onboarding acceptance.

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
Every completed Provider loop exit, including a propagated error, awaits the same
SDK task owner's drain. Signal-triggered shutdown retains the command's existing
120-second outer budget; an operational-error drain introduces no new timeout.
Drain joins all retained tasks before returning a task failure. When the loop and
cleanup both fail, both causes remain private and the primary error remains the
source-traversal root.

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

## Connections

- [Control admission, streams and calls](computer-control-rpc.md) — Control separates public component authority from trusted peer delivery. Streams and exact-call correlation do not form a durable transport log.
- [Owner-scoped convergence and restoration](reconciliation.md) — Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.
- [Docker connection, volume and container contracts](computer-docker.md) — The Docker adapter owns one selected source and exact native resources, with uncertainty distinct from absence or confirmed cleanup.
- [Kubernetes resource and retirement contracts](computer-kubernetes.md) — Supported Kubernetes SDK operations validate original resource identities and ordered terminal evidence, without equating a delete response with retirement.
- [Foreground process ownership and health](runtime-process.md) — The composition root owns admission, startup and shutdown of its selected backend and web processes. A successful cleanup submission is not completed shutdown.

## References

- [`rust/apps/chapchoo/src/provider.rs`](../../../../rust/apps/chapchoo/src/provider.rs)
- [`rust/apps/chapchoo/src/provider/effects.rs`](../../../../rust/apps/chapchoo/src/provider/effects.rs)
- [`rust/crates/rpc/src/reports.rs`](../../../../rust/crates/rpc/src/reports.rs)
