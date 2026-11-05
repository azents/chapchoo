---
title: "Execution reservation, evidence and cleanup"
---

# Execution reservation, evidence and cleanup

Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.

## Current-Connection Execution and Resource Evidence

Managed operations authenticate the Computer Provider key and exact current
Provider/Control/Connection ID plus the route's credential digest. Provider,
route and Computer mutations use short DB-only locks; PostgreSQL shares the
Control lock to fence draining without serializing unrelated Providers.
Directory/key expiry uses DB time after locks, never a component-supplied
timestamp. Expired/draining Control, expired route, key revocation, a superseded
route or a different credential cannot authorize a report. After the Computer
lock, exact current Request ID and current authority are rechecked.

`reserve_execution` commits the candidate execution and substrate-native resource
names before external creation: Pod/Secret/PVC for Kubernetes, container/original
saved volume for Docker. Repository issues the execution identity inside that
reservation; the Service prepares independent native names, not proposed IDs.
A recovered current candidate is returned even if the Service prepared different
names. Uncertain/blocked preparation does not replace that candidate.
Each reservation also commits its exact native allocation witness. Existing
saved-resource witnesses are never overwritten from current DTO identities;
[native storage](computer-storage.md) owns that distinction.
Resource reservation collisions roll back the entire operation. A successor
requires prior-execution retirement and current-connection evidence of the
original saved resource with its accepted native identity. First allocation may resume only with
definitive never-allocated evidence; it cannot replace previously allocated
missing storage. Initial creation uses create capability; saved-environment
starts/restoration use start capability.
Reservation recovery returns the same initial-allocation classification. Dispatch
checks this execution fact even after the initial PVC has been confirmed, so a
create-capable/start-incapable Provider can finish its initial preparation.

`prepare_admitted_at` is a durable once-only admission fence, separate from
`initial_start_dispatch_at`, which records actual component dequeue. Its
Computer-locked CAS revalidates the exact current source, request and execution.
A reconnect cannot reset an admitted uncertain Prepare. The Control can release
an exact current claim only when it proves rejection before queue acceptance;
possible forwarding, cancellation and lost results retain the fence. A current
Provider can report PreparationBlocked only for the current starting execution
with an unallocated Pod identity, after explicit native creation rejection or a
pre-Pod authority failure following acknowledged resource allocation. That report
sets Blocked and clears the prepare fence, but never resumes effects itself.
Owner RetryPreparation compares the exact predecessor request under the usual
User/Provider/Computer locks, requires this safe blocked state and live management,
and assigns a fresh Request ID while retaining the same execution and resources.
It resets the dispatch interval for this explicit new request. Exact result retry
uses the same request/fingerprint and cannot create another attempt. An authorized successor
reservation clears both fields only after retirement/original-PVC gates; an
existing candidate retry preserves them. Both engines add this nullable field in
`00000000000004_prepare_admission`, backfilling known actual-dispatch timestamps.
The down migration refuses to remove the fence.

Resource reports identify the exact reserved object and execution. Owned-present
evidence binds the original UID; a different same-name UID is rejected. Unknown
presence does not clear identity. Definitive absence must match the stored UID,
or explicitly concern a never-observed candidate after all possibly accepted
creates are quiescent. A timeout or disconnection is unknown. Kubernetes Pod
GET 404 is current absence, without historical terminal-event qualification;
Docker absence retains its own native adapter contract. Confirmed absence of an allocated object is sticky even after an
unknown report: later present reports cannot adopt/recreate it. The external
Computer Provider is responsible for verifying ownership/content and the
supported substrate retirement boundary; this module performs no Kubernetes IO.

Running completion requires a separately reported actual running execution plus
current-connection present/UID evidence for its Pod/bootstrap Secret and original
PVC. Pod existence alone is not running. Stop/restart cannot complete while the
old execution remains current, even if a blocked/uncertain phase obscures the
retiring display phase. During terminal deletion, an admitted uncertain cleanup
keeps its uncertainty fence across Pod retirement and later blocked issue reports;
those facts cannot reset the lifecycle to a fresh deleting admission. `retire_execution` requires definitive current-source
Pod absence, retires the exact execution, revokes and wipes its Connector session
authority and withdraws its route before clearing the current pointer.
If that exact retirement is replayed while no successor exists, fresh definitive
Pod absence from the current connection can reconfirm stopped power after
reconnect. Replay preserves the lifecycle phase and dispatch interval. An older
connection's absence alone cannot refresh current power; original saved-PVC
evidence must independently be refreshed before Start admission.
Restart advances to ordinary start under the same Request ID; late old-execution
reports cannot stop or complete a successor. Automatic restoration uses the same
old-absence, intact-storage, current-capability and conflicting-intent gates.
Original dispatch time is set once per start operation; reconnect/retry does not
reset it. Unexpected disappearance after completed running starts a fresh
restoration dispatch interval.
The reset uses the retired execution's confirmed-running fact rather than its
mutable current issue phase; reporting uncertainty before definitive retirement
cannot make recovery inherit a previous execution's dispatch timer.

Managed inventory is Provider-scoped and independent of an Agent association.
Terminal Computers accept exact safe cleanup reports without resurrecting an
Agent. Normal cleanup confirmation requires no current/live execution and
definitive absence of every reserved resource from the current source; it then
removes the Agent and records confirmed completion once. Force removal does not
imply any of those facts. Completed cleanup inventory rejects every further
resource report, including unknown/absence evidence, preserving its confirmed
terminal facts until collection. A completed stop with no current execution also
rejects lifecycle issue reports; late blocked/uncertain reports cannot reopen that
operation and strand the next start. Running executions can still report issues
without claiming definitive power-off.
Explicit collection removes only confirmed completed
Computers at least 24 hours after completion, with no live execution or unresolved
resource. Unresolved cleanup has no TTL. Creation receipts survive collection.
These are internal lifecycle operations, not public adoption/cleanup controls.

## Connections

- [Computer schema and native identities](computer-storage.md) — Durable Computer state records authority, exact resource identity and cleanup obligations; schema records do not themselves perform external effects.
- [Computer lifecycle admission and Provider keys](computer-lifecycle.md) — Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.
- [Execution-bound Connector credentials](connector-credentials.md) — A Connector session belongs to one execution. Bootstrap recovery, operational refresh and revocation have different permissions and retained results.
- [Owner-scoped convergence and restoration](reconciliation.md) — Current Connector owners take precedence; reconciliation observes exact resources and advances only authorized, known-unsent work.
- [Docker connection, volume and container contracts](computer-docker.md) — The Docker adapter owns one selected source and exact native resources, with uncertainty distinct from absence or confirmed cleanup.
- [Kubernetes resource and retirement contracts](computer-kubernetes.md) — Current API observations distinguish Pod absence from failed lookup and preserve exact present-object identity.

## References

- [`rust/crates/persistence/src/computer_executions.rs`](../../../../rust/crates/persistence/src/computer_executions.rs)
- [`rust/crates/persistence/src/native_resources.rs`](../../../../rust/crates/persistence/src/native_resources.rs)
- [`rust/crates/persistence/src/delivery.rs`](../../../../rust/crates/persistence/src/delivery.rs)
