---
title: "Computer Kubernetes Adapter Boundaries"
created: 2026-11-02
tags: [computer, kubernetes, security, lifecycle]
spec_type: domain
domain: computer
code_paths:
  - rust/crates/kubernetes/src/lib.rs
  - rust/apps/chapchoo/src/provider/effects.rs
  - rust/crates/kubernetes/tests/adapter.rs
last_verified_at: 2026-11-04
spec_version: 1
---

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

GET absence of a known original PVC or Secret can describe that object's absence.
An unallocated candidate's 404 remains unknown unless current terminal-intent
command authority carries committed proof that it was never allocated. That
proof must agree with a fresh GET; a surprising present object is rejected, not
adopted or deleted. A confirmed never-allocated Pod needs no DELETE or native
terminal inference. Other Pod 404 remains unknown without a matching terminal
witness. A witness requires the same immutable
Computer/execution/namespace/name/UID and terminal phase plus complete current
terminated container states, actual container identities and finish timestamps.
Unknown/container-status-loss reasons or missing status entries do not prove
quiescence. A different UID cannot reuse the witness.

`retire_pod` starts the supported API watch from the exact Pod GET's opaque
resourceVersion before submitting one UID-preconditioned normal delete. It observes
ordered terminal/deleted events and requires the exact terminal container evidence
before returning absence. Node-loss Failed status, missing Pod, watch expiration,
stream EOF or native failure stays unknown; no automatic delete replay occurs.
The native watch cursor is transient SDK state, not an application generation or
durable command log.

The ordered watch is call-local. A bounded 256-entry UID-bound terminal-witness
cache survives same-process reconnects, but not a process crash. The Provider
also accepts exact committed absence only through DB-validated resource command
authority. If necessary evidence was lost or expired, a new source remains
uncertain; API 404 or a process timeout does not prove retirement.

The common Provider runtime's `Effects` owns started SDK tasks independently of
call waiters and stream lifetime. It is no longer a Kubernetes-crate export;
Kubernetes native identities, effect order, timings and witnesses are unchanged.
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
and unknown versus terminal-qualified Pod absence. An explicit task barrier checks
that losing a waiter cannot release a still-running SDK effect permit.
These are adapter contract tests, not live retirement/network-policy/storage
release-gate evidence.
