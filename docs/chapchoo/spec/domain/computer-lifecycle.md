---
title: "Computer lifecycle admission and Provider keys"
---

# Computer lifecycle admission and Provider keys

Owner requests commit durable intent under current authority. Component evidence and external completion determine what may happen next.

## Current Lifecycle Admission and Provider Credentials

Power requests compare the expected current Request ID under User, Team, Provider
and Computer locks. An exact current request/fingerprint returns current progress;
changed payload or a stale predecessor rejects. Running/stopped evidence must
come from the current eligible Provider connection and match the current execution.
An older connection's facts remain last-known information but yield unknown
current power. Ordinary Start additionally requires verified saved environment
from the current Provider connection and no current execution; stop/restart
require the confirmed current execution. Pending or uncertain operations block
competing physical work. Acceptance records intent/phase, not external success.

Start against the exact current Stop may instead update only its desired state
to Running while retirement is Retiring or Uncertain. Start capability, current
management connectivity and current-source original saved-storage verification
remain required. The Stop's request ID, fingerprint, phase, expected execution
and native command correlation remain unchanged. This is an idempotent target
update, not a separate queued Start or a newly stored Start receipt. Repeating
the Running target against that same Stop returns its current progress; original
Stop receipt replay remains intact, and a later operation/Delete invalidates the
captured predecessor. If Stop completed first, ordinary Start admission applies.

Old-execution Running observations cannot fulfill this target or reclaim its
preparation. Current Kubernetes API absence clears the old execution and selects
Starting for the Running target without a historical terminal witness;
existing original-storage and successor gates still apply. Public progress then describes Start while stored Stop provenance
continues to qualify target-update recovery. The desired state survives page
closure and backend restart without a browser timer or another intent store.

Delete can supersede preparation and persists terminal intent. In the same
transaction it revokes that Computer's Connector sessions, discards encrypted
bootstrap/operational recovery material and removes Connector routes. Before
replacing the intent it can record non-allocation only for the current candidate
and unallocated saved environment when the captured Prepare fence proves no possible dispatch
(or an acknowledged safe preparation failure). Older execution resources and
allocated native identities are unchanged. Every native identity column
participates in the never-allocated proof. These absence facts have no Provider source until
fresh observation agrees; a possibly forwarded Prepare never grants this proof. Force
removal requires the current deletion request and removes only the Agent; residual
Computer/cleanup authority and creation receipt survive. An owned removed Agent
can be recognized while residual inventory remains, but only with that Computer's
exact current deletion Request ID. Wrong IDs reject, including retries racing
normal or forced removal. After inventory collection, removal retry returns
not-found rather than treating a creation receipt as proof of a deletion request.
Creation replay still returns removed without recreating it.

For Docker, direct Delete and context-driven leave/kick/Team deletion preserve
prior uncertain retirement or possibly admitted preparation uncertainty. A new
deleting Request ID cannot erase a possibly accepted native effect and create
fresh dispatch permission. Connector revocation and terminal cleanup intent
still commit; force removal does not clear that uncertainty.

Provider key issuance requires verified bootstrap evidence, a currently permitted
scoped registration credential, or the existing owning User session management
authority, never the Provider communication key. Each credential mutation
has a target-scoped Request ID/fingerprint receipt. Exact current-result retry
recovers the same encrypted key and original expiry; superseded/revoked/expired
results cannot be recovered. Fresh rotation retains only the immediately previous
digest for at most five minutes, capped by its original expiry. Communication
admission checks current/previous digest, key expiry and revocation each time.

Explicit revocation has no overlap, discards protected recovery and withdraws the
Provider route. Its receipt makes retry safe; replay of an old revocation after a
new owner-authorized issuance cannot revoke the new key. Owner-authorized fresh
issuance can restore communication authority after revocation, but a new component
connection is still needed. Provider keys have 15-minute validity and proactive
reauthentication from five minutes, with at most five minutes of prior-key overlap.
The separate Connector policy retains 24h/12h defaults. No root or old-secret
archive is exposed. [Provider authentication](provider-authentication.md) owns the
evidence/source fingerprint and bootstrap withdrawal contract.

## Docker retirement authority

The current-source/current-request transaction claims eligible fresh
Stop/Restart `RETIRING` or terminal Delete `DELETING` to `UNCERTAIN` once.
Qualified terminal retirement under Running intent additionally requires the
exact completed current execution with admitted preparation; its expected old
execution remains recorded. An already uncertain request yields no new admission.
Late Running evidence cannot clear this Docker retirement fence. A Blocked
issue report also preserves an already uncertain Docker lifecycle: a diagnostic
is not proof that its native stage never started. Neither a late Running report
nor a subsequent direct/context-driven Delete can turn that diagnostic into a
fresh retirement claim. Definitive preparation failure and exact NotStarted
release retain their separate qualified admission paths.

The returned opaque admission records original phase/source/request/execution.
Only Control's exact live manifest and proven prequeue/dequeue NotStarted result
may release that admission. Timeout, cancellation, connection loss and native
rejection after any prior effect do not qualify. Durable state alone cannot prove
a native call was never submitted. Current-source absence can retire an exact
execution observationally without replay permission.

Saved-volume cleanup requires terminal deleting intent, no current or live
execution, current-source qualified absence of every reserved Docker container,
and Control's live known-never-submitted suffix authority. Complete absence of
every reserved resource can independently finish cleanup after correlation loss;
absence of a subset cannot authorize a new volume mutation.

## Connections

- [Private Agent ownership and Team deletion](agent-ownership.md) — Team membership selects context while individual ownership remains the private Agent boundary. Context deletion accepts cleanup intent, not completed native deletion.
- [Execution reservation, evidence and cleanup](execution-evidence.md) — Reservations precede effects; current-source observations qualify power, retirement and resource absence without granting ambiguous replay.
- [Docker retirement and exact-call correlation](native-retirement.md) — A live manifest qualifies continuation of one possibly accepted native call; lost correlation never becomes replay authority.
- [Control directory authority](computer-directory.md) — The durable directory chooses current component owners and excludes expired, revoked or draining authority. Listener and native effect lifetime are separate owners.
- [Computer owner and installation inventory RPC](computer-client-rpc.md) — Owner operations and redacted installation inventory are separate capabilities over current account authority.

## References

- [`rust/crates/domain/src/computer.rs`](../../../../rust/crates/domain/src/computer.rs)
- [`rust/crates/persistence/src/computers.rs`](../../../../rust/crates/persistence/src/computers.rs)
- [`rust/crates/persistence/src/component_keys.rs`](../../../../rust/crates/persistence/src/component_keys.rs)
