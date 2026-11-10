---
title: "Codex Conversation Phase 4: Central Refresh and Execution Credential Files"
created: 2026-11-09
tags: [agent, conversation, codex, authentication, implementation]
---

# Codex Conversation Phase 4: Central Refresh and Execution Credential Files

## Phase Execution Plan

- Phase: 4 of 6, central credential renewal and execution-local file publication.
- Branch/base: `feature/chat-text-4-auth` → `feature/chat-text-3-records`.
- PR boundary: an eligible actual harness start applies the selected admitted authentication; current executions using that applied record can receive same-record credential updates without live harness intervention. ChatGPT refresh remains server-owned and coordinated across eligible consumers.
- Inputs: phase 3 PR #99 at `50f07e3`, the existing protected authentication catalogue/device acquisition and Agent selections, phase 2 native storage/lifetime owner and phase 3 exact execution/Connector/Control admission. Phase 3 full local Rust/RPC gates and mandatory hooks passed; remote checks/reviews remain deferred.
- Deliverables: supported public-SDK refresh, durable source-bound refresh admission/results, protected latest credential publication, saved/applied identity separation, current execution-bound secret delivery and private native file publication with truthful observations.
- Non-goals: production Chat UI replacement, a public credential endpoint, model proxy, new encrypted transport or credential mount, runtime-only catalogue deletion blocks, automatic uncertain refresh retry, adapter/native forks, callback bridges, forced reload/login/restart, general credential versioning, a new outgoing native journal, model entitlement or seamless native adoption claims.
- Approved Design mechanisms: M14–17, M21 and M26, integrated with existing M02/M05/M11/M12/M19 execution and operation boundaries.
- Authority references: [Requirements r24](../requirements/chat-261108-text-conversation.md), especially REQ-1/11/12/17/18; [accepted ADR](../adr/chat-261108-text-conversation.md) D24–28 and D33; [approved Design r9](../design/chat-261108-text-conversation.md), Authentication and Secret Boundaries/Central Refresh Owner; current [backend authentication](../spec/domain/backend-authentication.md), [Connector](../spec/domain/connector-process.md) and [native owner](../spec/domain/native-harness.md); [source-backed native file feasibility](../notes/chat-261108-native-auth-feasibility.md).
- Design delta: None.
- Removal obligations: replace the conversation-start gap where native setup has no explicitly admitted model authentication; keep existing catalogue/device and saved-reference behavior. Exclude server refresh material from execution delivery, native files, captured session events and ordinary diagnostics. Do not retain ambient-file adoption or introduce a runtime-only `InUse` exception.
- Absence verification: exact applied/next-start selection behavior; no refresh token in synthetic execution payloads/files/logs/history; no public credential DTO/route; no native callback/re-login/restart on same-record publication; no late result reviving deleted/replaced records; migrations 1–13 and historical identity/fingerprint semantics remain unchanged.

## Fixed Interfaces and Workstreams

The existing catalogue is metadata/selection authority, not proof of validity,
application or model success. Refresh attempt admission and result publication
are completed Store operations. Provider I/O is a typed external call owned
beyond a browser/consumer waiter. An execution payload identifies the actual
applied record and current admitted credential source rather than borrowing a
new saved selection. Local source/request/attempt identities implement these
approved fences; no general revision counter or credential-version product is
introduced.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Supported refresh adapter | `/root` | `rust/crates/openai-auth/**`, workspace/lock manifests, dependency boundary allowlist | fixed issuer/client/device-acquired bundle | public `oauth2` refresh-grant operation and bounded, verified-TLS, no-redirect/no-retry transport; body-safe failures | deterministic HTTP fixtures, SDK grant/response compatibility, redaction/unknown-outcome tests |
| Source/attempt/applied authority | `/root` | `rust/crates/persistence/src/external_auth*`, preserving paired migration 14, schema/preparation and owning tests | existing protected catalogue, Agent/Team and execution admission | one admitted refresh per current record source, exact result publication, current applied-record binding and eligible payload reads | SQLite/PostgreSQL races, manual replacement/deletion/scope withdrawal, shared consumers, preserving migrations/downgrade |
| Server refresh owner and delivery | `/root` | `rust/crates/service/src/authentication*`, `proto/chapchoo/**`, `rust/crates/{protocol,rpc}/**`, source-generated Node output, app composition | typed adapter and completed Store boundaries | bounded coalesced process-owned refresh, secret-safe execution-only transfer and metadata observations | cancellation/uncertainty/drain tests, exact current session/execution and old-stream fencing, live generated RPC both engines |
| Native application/publication | `/root` | `rust/crates/harness/src/{runtime,storage}*`, `rust/apps/chapchoo/src/connector/**`, owning protocol/process fixtures | current applied descriptor and saved-root boundaries | initial explicit API-key application or eligible external-token file startup; serialized same-record file replacement without native intervention | synthetic file/schema/permission/symlink/stale-publication tests, restricted profile tests, no secret capture or callback installation |
| Integration and current contracts | `/root` | this phase plan, multi-phase plan, affected semantic Spec pages and current authentication/runtime tests | preceding workstreams | connected production lifetime, truthful saved/applied/refresh/publication semantics and evidence | full Rust/RPC/docs/hooks; actual runtime/live-auth gates reported separately |

## Integration Order and Validation

1. Inspect the current protected record representation, native auth schema and
   supported SDK public refresh interface. Retain the fixed issuer/client and
   existing device flow. Missing prerequisites are not evidence of unsupported
   product scope.
2. Implement source-bound admission, account binding and publication fences with
   preserving storage. Manual replacement/reauthentication supersedes an old
   refresh result; deletion/scope loss denies new work and late publication.
   Applied-only references do not add a deletion blocker. Existing saved-reference
   deletion checks and Team terminal cleanup remain authoritative.
3. Add one bounded server work owner per current record source and coalesce
   eligible consumers. A possibly performed exchange without a publishable result
   remains uncertain; owner loss never grants an automatic refresh retry.
   A successful standard response may retain the valid unchanged refresh token
   when the provider omits its replacement. Missing expiry is not invented.
4. Connect exact execution/applied-record admission and the existing trusted
   internal HTTP component route. Deliver only the required short-lived payload;
   bearer checks do not create transport confidentiality. Reads do not launch a
   harness, and a saved future selection does not change the live applied identity.
5. Materialize current admitted credentials under the explicit saved native home
   with restricted access and safe replacement. Initial API-key auth uses the
   supported explicit ACP/native file-store path. External-token startup binds
   the actual admitted account and excludes a usable refresh token. Same-record
   updates replace only that file; do not force cache reload or claim adoption.
   Leftover files do not authorize startup or ambient account adoption.
6. Freeze integration, run affected checks, canonical full Rust/RPC gates,
   documentation validation and mandatory commit hooks, then open phase 4 PR on
   phase 3 before starting phase 5. No remote CI/review inspection beforehand.

Root performs all implementation and integrated validation. The exact sole
read-only independent reviewer remains `/root/chat-implementation-reviewer`;
request that review only after the entire six-PR stack exists, then make all
corrections directly on their responsible phases.

Deterministic coverage must include concurrent shared consumers, refresh
rejection versus unknown transport, omitted refresh replacement, malformed or
unpublishable successful responses, stale manual replacement/reauthentication,
deleted applied-only records, authority withdrawal, operational-key/stream
replacement, duplicate/late file delivery and native saved-context preservation.
Use explicit barriers rather than sleeps for race ordering. Keep combinatorial
cases below browser E2E.

The supported public SDK uses form refresh encoding, while the inspected native
private helper uses JSON. Exact real refresh compatibility and live native
adoption remain acceptance gates, not facts inferred from fixture parsing. Use
only an explicitly authorized isolated login when a usable path is ready, never
request secrets in chat or extract ambient credentials. Report concrete native
limitations instead of introducing a fallback. Authentication-dependent tests
remain pending without blocking independent implementation or PR delivery;
real model use stays at the minimum core confirmation.

## Context Checkpoint

Phase 3 is submitted as [PR #99](https://github.com/azents/chapchoo/pull/99),
with complete local gates/hooks passing and unchanged approved baseline. This
phase starts from a clean dependent branch. Existing backend code owns device
acquisition only; it has no public-SDK refresh dependency or execution credential
publication. Native storage and operation lifetimes are already available.
No real credential or model call has been used. Current task: implement the
adapter and durable authority foundations, then connect delivery/application.
Design delta: None.

### Internal Refresh Boundary

The public `oauth2` 5.0.0 adapter and typed provider boundary are implemented.
The SDK owns standard form grant construction and successful/error response
parsing; its public asynchronous HTTP interface uses the existing bounded
verified-TLS client. Ordinary errors contain no SDK response bodies or vendor
descriptions. Fixtures cover refresh-token replacement omission, account
binding, expiry, rejection, malformed/oversized response, redirect and uncertain
transport without retries. Real device-acquired compatibility is unverified.

Preserving migration 14 adds source-bound refresh attempts without altering
migrations 1–13 or the catalogue layout. The existing encrypted envelope nonce
is the opaque equality identity of the current protected credential, so alias
changes do not rotate it and credential changes do. Current scope/record locks
admit one attempt, revalidate publication and serialize narrow non-success
settlement. Old-source results, deleted records, account changes and uncertain
attempt recovery do not grant publication or another exchange.

`AuthenticationRefreshService` uses the bounded authentication work owner.
Concurrent consumers observe the same attempt; accepted external work survives
consumer cancellation and explicit shutdown drains it. Source/result metadata
remains separate from the protected bundle. Control's component credential
service and foreground owner now compose this boundary. The actual-start caller
and same-record live publication are connected; final phase gates remain.

The native external-token file representation was rechecked against the exact
inspected Codex commit, not the research clone's moving checkout. The internal
converter uses `chatgptAuthTokens`, an access-JWT-backed identity, the admitted
account and an empty refresh-token field. It rejects absent/incompatible account
metadata. This conversion does not write a file, verify JWT signatures, launch
native code or prove native adoption.

### Native File Boundary

The harness library now validates execution-only material and provides bounded
atomic publication under the retained saved-root descriptor. External-token
ingress rejects a usable refresh token, conflicting account, mixed API-key
material and unsupported fields. API-key file serialization follows the pinned
native schema; it is for same-record replacement, not an initial-login shortcut.
Initial API-key application still requires explicit ACP metadata.

Publication opens private owned classes relative to retained directory handles,
refuses redirected classes and symlink/special/hard-linked targets, writes a
private exclusive temporary file, syncs it, rechecks the retained directory/file
identities, and renames to `auth.json` with mode 0600. Temporary cleanup is explicit
on rejection. Saved native sessions, workspace and captured events are untouched.
These boundaries do not create same-UID isolation. The operation is a finite
blocking boundary whose eventual actor caller must retain physical ownership.

Production setup now uses the file operation for admitted external-token startup
and clears residual owned credentials before explicit API-key login. Same-record
live updates use only file publication. No file helper result establishes native
cache adoption, login validity, model entitlement or a successful model call.

### Applied Store Boundary

`begin_execution_authentication` seals a saved record only for an existing
possible Setup operation under current exact execution authority. It records
the actual-start operation and original selection identity, replays only the
same current binding and compares the preceding application before accepting a
new one. This is applied-configuration provenance, not an outgoing dispatch
journal, a credential version counter or proof that native login succeeded.

Migration 14 retains this provenance independently of input/Setup deletion during
source replacement. Execution/Agent/conversation cleanup cascades; authentication
record deletion does not cascade or block. Late old-operation admission and
material reads cannot recreate a deleted record or borrow the saved future
selection. The current applied record's source is read under repeated
session/route/Control/key/execution and owner/membership/scope admission, with an
optional exact source check before handoff.

Focused SQLite/PostgreSQL tests exercise concurrent same-operation admission,
saved/applied separation, same-record replacement and stale source rejection,
saved-reference deletion protection versus applied-only deletion, superseded
operation fencing, source replacement with removed input history, bad key and
old-route denial, and populated application downgrade refusal. Existing
conversation, external-authentication and profile-preservation checks remain
passing. Control's explicit component methods now invoke these Store operations.
The actual actor-start callback and file-only same-record updates are connected.
Full phase gates and the phase PR remain.

### Execution-Authorized Refresh Boundary

Account and execution callers now use shared database-only record/source refresh
admission and publication primitives. The execution path obtains its locked
record from current Connector/Control/route/key/execution, applied operation and
Agent/Team/record scope rather than creating an account session. Completion
rechecks that authority separately from initial admission. Existing owner
mutation identity, account binding, exact source and one possible attempt remain
required; narrow failure settlement grants no publication or retry.

`AuthenticationRefreshService.current_execution` owns the source/application
data beyond an abandoned consumer waiter and runs through the same bounded work
owner, provider call, coalescing and drain path as account consumers. Store
fixtures cover one shared attempt across the two consumer classes, successful
current-execution publication despite a changed future selection, fresh-source
reads, late old-route failure and retained uncertainty, manual reauthentication
fencing and deletion without revival. No real provider or model is used.

The component delivery service now revalidates the exact applied source and
uses the adapter's pure native encoder before transfer. Raw server bundles never
become wire payloads. Native actual-start application now uses these methods.

Credential-bearing execution operations now separate initial discovery from
final locked Connector admission: Team, authentication record, then Computer and
route, followed by current applied/authority rechecks. This avoids an inverse
Computer → shared record dependency against existing catalogue → Provider →
Computer operations. The PostgreSQL fixture blocks the actual record read,
observes its `pg_blocking_pids` witness and proves that Computer admission is
still available with `NOWAIT`; no sleep establishes the ordering.

### Component Wire and Foreground Owner

The component-only `ConnectorCapture` service now exposes binding recovery,
actual-start admission, explicit acquisition/renewal, 32-KiB native payload reads
and final source checks. Client/Admin group registration excludes them. A
required `ExecutionAuthenticationService` collaborator owns bounded material
reads and the central refresh owner; `NativeTokenEncoder` is the actual adapter/
fixture translation boundary. API-key payloads remain raw for initial explicit
ACP login; ChatGPT payloads are converted native files, not server bundles.
All new message/oneof Debug implementations omit fields.

Control publishes the effect owner before listener startup awaits and retains it
with the Client device owner across partial failure, close and explicit joined
shutdown. Cleanup preserves the new owner's failure alongside backend/web
failures. No native command or file operation is dispatched by these endpoints.

Actual RPC fixtures exercise a full 64-KiB key through bounded frames, changed
source continuation/final-handoff rejection, account-token denial, saved/applied
separation, applied-only deletion without fallback and one execution-triggered
mock refresh followed by refresh-free reads. Group tests prove Client/Admin
exclusion. Protocol redaction, native adapter, foreground failure retention and
the existing model-free Connector CLI path are also checked. This component
fixture uses SQLite; the shared Store authority/refresh contracts already have
paired PostgreSQL coverage, and full generated Node/Rust interoperability for
the completed phase remains a later gate.

Native actual-start credential acquisition, initial API-key ACP metadata/file
store, same-record live publication and blocking-file shutdown ownership are connected.
Authentication-dependent real model acceptance remains
separate.

### Native Actual-Start Boundary

Connector supplies an operation-scoped `NativeAuthenticationProvider` when
dequeueing sealed Setup. The actor calls it only after selecting a real launch
and preparing compatible artifacts. Exact already-live resume returns before
that callback, retaining applied credentials despite a future saved selection.
Provider acquisition binds the previous application, current selected record,
bounded native pieces and final protected source check. The live owner retains
the admitted descriptor.

Before child spawn the actor checks authority, owns a finite blocking credential
job and checks authority again. External tokens seed the private file; initial
API-key application clears only residual `auth.json` and later uses explicit
public-SDK v2 `auth/login` metadata before session New/Resume. The wrapper forces
file-store configuration without credential argv/env. Login rejection creates
no fallback session; token startup installs no unsupported callback/login.
The blocking job remains in actor state across launch/retirement waiter
cancellation and is drained before physical scope retirement.

SDK TRACE inspection found that its redactor covers env/header values but not
arbitrary API-key metadata. The composition-owned diagnostic layer therefore
excludes SDK raw protocol diagnostics independently of log-filter settings.
Body-safe product error stages remain available. Deterministic fixtures cover
explicit login order/rejection, external-file resume without token callback,
already-live preservation without provider calls, residual-file-only clearing,
blocking-owner retention across partial cancellation and TRACE exclusion.
Restricted authenticated native acceptance and model success are unverified.

### Live File Publication and Phase Gate

The Connector's credential worker owns at most one task, with a five-second
cadence separate from capture and native commands. Idle binding absence performs
no credential RPC or native effect. Live acquisition uses the existing applied
operation/record under current route authority; it never admits a saved choice.
Unchanged sources need no byte transfer. Changed material is checked and
assembled outside the actor, then queued with its exact preceding binding.

The actor rejects changed application/record/preceding-source identities,
performs the final bounded authority check and serializes file publication through
the same retained blocking owner. Publication advances only the local source
echo, without login/reload/restart or an adoption claim. Shutdown may abandon a
network waiter while its server work remains owned; it cannot detach physical
file publication. A raw maximum-length key is preserved through an independently
bounded zeroizing JSON buffer, including worst-case quote/backslash expansion.

Focused tests cover file-only same-record publication on a retained SDK session,
stale expected source, different future record and revoked handoff rejection,
no extra protocol request, idle worker coalescing with no server contact, and
complete maximum-length escaped-key publication. Phase implementation is ready
for full Rust/RPC/docs/mandatory-hook validation; real credentials and live
model acceptance remain absent and do not block this phase's PR.

### Integrated Local Validation

- `bash scripts/check-rust.sh`: workspace format, all-target Clippy, tooling
  contracts, documentation build, all workspace tests with disposable PostgreSQL
  and dependency boundaries passed. The combined test result blocks report
  388 passed, zero failed and zero ignored, including compile-fail documentation.
- `bash scripts/check-rpc.sh`: generated client synchronization, Node client
  build and actual Node/Rust RPC checks passed with 129 checks on each SQLite
  and PostgreSQL. New execution-credential positive journeys are covered by
  the Rust component RPC fixture; this does not claim 129 new auth checks.
- Document metadata/tests and whitespace checks passed. Approved Requirements,
  ADR, Design and migrations 1–13 remain unchanged from the predecessor.
- An earlier full-gate invocation was interrupted during compilation and is not
  counted as passing. The complete rerun above finished successfully.

The phase is ready for mandatory commit hooks and PR creation. No real
credentials, real model call, authenticated restricted-runtime acceptance,
remote CI/review inspection, merge or deployment is included in this checkpoint.
The complete six-PR stack precedes remote CI and the assigned independent review.

## Complete-stack correction scope

All six PRs now exist. Correct valid integrated findings within the approved
baseline, with implementation and validation owned by the primary agent:

- Check a refreshed candidate with the actual native encoder before publishing
  its protected source. An opaque or otherwise unpublishable access token cannot
  become an `Updated` source merely because the OAuth SDK accepted its envelope.
- Remove the public unauthenticated start/setup paths that always fail after
  authentication became required. Require the composition-supplied provider at
  the authenticated setup entry; keep idle/read operations effect-free. The
  model-free scope probe covers preparation/retirement, not an invalid native
  session start without an authentication authority.
- Extend the existing exact-source native observation with physically applied
  preparation freshness/exact component versions (M03 integration) and the
  applied authentication record identity (M15). Clone both from the same live
  owner, not from saved settings or a prior credential-delivery attempt. Expose
  nonsecret owner facts and the current saved selection so changes in another
  surface do not leave the comparison tied to stale initial page data.
- Add preserving paired migration 17 for native observation details. Keep
  migrations 1–16 immutable, retain unknown for older rows, reject destructive
  downgrade and clear only the correct source-qualified child facts. An applied
  record may have been deleted; observation storage must not prevent the already
  approved applied-only deletion or imply instant offline revocation.
- Keep wire/client generation source-owned. Exercise stale/current/equal-state
  facts, same-record updates, changed saved selection, deletion, source
  replacement, read-only behavior, both database engines and actual runtime
  interfaces. Owner presentation remains phase 5. No credential bytes or source
  fingerprint is added to owner diagnostics.

The phase-2/3 corrections are inherited. The synthetic authentication fixture now
supplies exact component versions, fixing its rebased all-target compile error.
Source/operation admission, trusted-network delivery, native file-only updates,
whole-scope retirement and no automatic replay remain unchanged. Run focused and
full affected gates, mandatory hooks and a targeted review of the corrected
interfaces with the same reviewer after stable integration. Real credentials and
model calls remain unused. Design delta: None.

### Correction checkpoint

The source-qualified details now traverse the live actor, Connector report,
transactional Store, owner RPC and generated client. Idle observations also
compare the exact native source instead of relying on an absent work target.
The mandatory authentication provider is represented as required throughout
setup and launch, including the private command path.

Focused validation passed all four both-engine conversation/authentication
tests and 56 harness tests. Coverage includes first explicit details, equal-known
conflicts, stale reports, newer physical preparation, saved-selection changes,
same-record replacement, deletion of an unsaved applied record, source
replacement and migration downgrade refusal with byte preservation. Live
Node/Rust RPC checks passed 129 checks per database, and documentation metadata
and its 18 validator tests passed.

The initial full run exposed a historical auth downgrade assertion targeting
the newest migration and a migration-test witness filtered out while the source
was Loading. The test now targets its original migration 14, and the migration
witness uses an explicitly bound source without claiming Ready. Those failed
runs are not counted as complete validation. The final immutable diff still
requires the full mandatory Rust/RPC commit gates; targeted integrated review
remains pending after dependent UI integration.
