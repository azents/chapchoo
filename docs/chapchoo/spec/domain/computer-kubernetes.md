---
title: "Kubernetes resource and retirement contracts"
---

# Kubernetes resource and retirement contracts

Supported Kubernetes SDK operations validate original resource identities and use current API observations to converge the requested state.

# Computer Kubernetes Adapter Boundaries

The reusable `chapchoo-kubernetes` crate supplies a narrow supported-client
adapter used by the Computer Provider process and owner-scoped Control reconciler.
No live Kubernetes acceptance or cluster credentials are implied by its tests.

## Resource identity and effects

The adapter uses public `kube::Api` methods and typed `k8s-openapi` Pod, PVC and
Secret models. Its production client-config constructor explicitly disables
kube's default server retry layer and sets finite connect/read/write timeouts.
The caller must provide explicit namespace, image/pull policy, resource requests
and limits, saved mount, storage policy and ServiceAccount. It does not load
ambient credentials merely by constructing a test fixture.

Every present resource must match reserved namespace/name, original UID when
allocated and exact Provider/Computer/applicable execution annotations. Annotations
rather than labels carry opaque IDs that can exceed the Kubernetes label-value
limit. A foreign same-name UID/ownership mismatch is not adoption authority.
The annotation values come from the required committed
[allocation witness](computer-storage.md), not current canonical context IDs or a
configured Provider key. Missing, malformed or resource-role-mismatched witnesses
reject before native reads or effects. Correlation IDs remain independently exact.
Managed objects have no controller ownerReferences; the saved PVC is independent
of standalone Pod deletion.

Initial allocation may create an as-yet-unallocated dedicated PVC. A known original
PVC UID can never be recreated, including an initial-execution retry after its
original storage disappeared. Existing immutable bootstrap Secrets must match the
exact execution session/key/Control endpoint content. The adapter never patches
foreign or mismatched bootstrap material.

Standalone Pod creation verifies the original PVC and bootstrap UID first and
submits the Pod last. The Pod uses one Computer Connector container, the preserved
PVC mount and exact Secret-key references. It has an explicit ServiceAccount with
API token automount disabled, non-root identity, dropped capabilities and
read-only root and RuntimeDefault seccomp. Collision checks require that Pod
profile and reject an Unconfined container override. Collision checks verify its supported image, command, mount,
bootstrap references and security constraints rather than treating AlreadyExists
as success. CPU/memory and storage settings remain runtime inputs.

Each create/delete is one SDK submission; unknown native outcomes are not retried.
Deletion requires a known original UID precondition and uses normal graceful
semantics, not force/grace-zero deletion. A successful delete response means
submission, never confirmed retirement or cleanup. For Secret/PVC cleanup the
same task polls exact-identity observations for up to ninety seconds until
absence is confirmed, including asynchronous finalizer completion. It never
resubmits DELETE; timeout, observation failure or replacement UID remains a
failure rather than cleanup success. The lifecycle permit remains held while
this observation task runs. Native create rejection is classified only at the
POST response; a GET/AlreadyExists follow-up error cannot grant preparation retry.
The classified rejection retains its original typed SDK cause for private handling;
ordinary Display and Debug still expose only the adapter's static safe context.

## Evidence and task lifetime

Successful Pod GET absence means that the reserved Pod is absent. It requires
no prior terminal event, process-local witness, or historical watch cursor.
The Provider reports that absence under the current source/request/execution;
Control records stopped power and converges a Running target by preparing a new
execution with the original saved PVC when a previously allocated Pod is gone.
An unfinished UID-less creation keeps its existing execution and native names.
After fresh absence it can ensure those same names with the same bootstrap
session; a late original POST and a recovery POST meet at the same name, and
GET/409 collision validation converges onto one exact object. No historical
terminal event is needed for either path. A failed GET, denied API request or
disconnected Provider is not absence. A present object still requires the exact
reserved ownership and original UID.

`retire_pod` starts the supported API watch from the exact Pod GET's opaque
resourceVersion before submitting one UID-preconditioned normal delete. It observes
exact deleted events as API-level absence regardless of the final reported
container phase. An already absent Pod completes without another DELETE.
A delete acknowledgement, watch expiration, stream EOF or native failure alone
does not establish absence; a subsequent successful current GET can do so.
The native watch cursor is call-local observation support, not durable authority.
For a still-present terminal Pod, complete terminated-container status remains
the predicate for automatic terminal-Pod replacement; that predicate does not
qualify or gate a 404.

GET absence of a known original PVC or Secret can describe that object's absence.
An unallocated storage/Secret candidate remains unknown unless current
terminal-intent command authority carries committed non-allocation proof.
Unexpected present identity or ownership is rejected, not adopted or deleted.

The common Provider runtime's `Effects` owns started SDK tasks independently of
call waiters and stream lifetime. It is no longer a Kubernetes-crate export;
Kubernetes native identities and SDK task lifetime remain adapter-owned.
One lifecycle permit remains held until the actual task ends, including when its
response receiver is dropped. A Provider process must retain this owner across
reconnects and supervise its drain; replacing a connection is not cancellation
of a possibly accepted substrate effect. Drain joins every retained task before
returning its first task failure, including work retained after an earlier failed
task released its permit.

## Verification

Synthetic services beneath the real supported client verify original PVC
preservation, initial allocation, foreign collision rejection, Pod-last creation,
Secret/PVC references, exact UID delete preconditions, no ambiguous-create replay,
and successful Pod absence versus lookup failure. An explicit task barrier checks
that losing a waiter cannot release a still-running SDK effect permit.
These are adapter contract tests, not live retirement/network-policy/storage
release-gate evidence.

## Connections

- [Provider process and allocation acknowledgements](provider-process.md) — External trusted Providers own native IO, while authenticated server operations own durable mutations and evidence acknowledgements.
- [Execution reservation, evidence and cleanup](execution-evidence.md) — Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.
- [Owner-scoped convergence and restoration](reconciliation.md) — Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.
- [Helm and Argo deployment boundaries](deployment.md) — The public chart composes workloads and explicit prerequisites; offline rendering does not prove a live rollout or substrate cleanup.

## References

- [`rust/crates/kubernetes/src/lib.rs`](../../../../rust/crates/kubernetes/src/lib.rs)
- [`rust/crates/kubernetes/tests/adapter.rs`](../../../../rust/crates/kubernetes/tests/adapter.rs)
