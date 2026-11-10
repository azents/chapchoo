---
title: "Backend authentication catalogue and saved Agent references"
---

# Backend authentication catalogue and saved Agent references

Client callers can register backend-owned OpenAI API keys or acquire ChatGPT
credentials through device authorization. Agents save references to these records;
selection performs no Computer delivery, native credential-file write, refresh,
Codex launch or model call. Stored, selected, valid, entitled and funded are
independent facts. The [General authentication surface](authentication-web.md)
uses these contracts without widening their authority or adding execution.

## Scope, authority and catalogue

`chapchoo.authentication.Authentications` is registered only on the Client group.
Every operation uses the current account session. Personal scope means the caller's
User, never a caller-supplied User ID. Team scope requires current membership;
Owner/Manager manage Team records, while all current members can read eligible
records and select one for an Agent they independently may manage. Installation
administrator status and Team management do not grant access to another User's
private Agent. Catalogue responses contain metadata, not stored secret bytes,
other owners' Agent names or association aggregates.

List/Get return ID, owner scope, kind, alias, known account/email/plan/expiry,
current accepted mutation Request ID and creation/update instants. Missing aliases
are represented by an empty string; registration and rename accept empty aliases
while retaining the 256-character/1024-byte and control-character bounds.
Missing provider metadata stays absent. Provider expiry outside the supported Protobuf Timestamp
range is rejected before credential publication, so accepted records remain
serializable through list/detail. Lists default to 50, allow 1–100 and use an opaque stable
ID cursor; the last full-page ID is the next cursor, so the next page may be empty.
The service bounds request decoding to 256 KiB and response encoding to 2 MiB.

RegisterKey accepts a nonempty protected API key up to 65,536 ASCII-graphic bytes,
without a validation/model/entitlement call. ReplaceKey retains the record ID and
Agent references. Rename changes the alias. Mutations compare the expected
accepted Request ID where applicable. Delete locks the same record as selection
admission; a live reference returns a typed in-use result without an Agent list.
Cancellation/failure never replaces an old key or ChatGPT credential.

Every accepted mutation binds initiating User and Request ID to a canonical typed
payload using the [protected keyed fingerprint](credential-primitives.md).
Changed input under an accepted Request ID conflicts; exact completed replay
returns its original non-secret identity. RequestResult recovers operation,
accepted identity and time without resending the key, even after record/Agent
removal. It rechecks current original-scope authority. Absence is an observation,
not evidence that another uncertain call stopped.

## Explicit device acquisition

StartDevice admits a bounded attempt before issuing a code; an optional exact
target starts same-record ChatGPT reauthentication. HTTP occurs outside database
transactions and acceptance rechecks current session, role, attempt, step and
target mutation. Code disclosure is limited to the initiating current manager.
The active pending projection includes the code, fixed verification destination
and next permitted check time. DeviceStatus observes retained progress and lazily
terminalizes expired active attempts, clearing temporary device material under
current authority without dispatching provider I/O. Exact start-result recovery
performs the same expiry cleanup. PollDevice
performs at most one due provider check, then at most one authorization-code
exchange; there is no autonomous poll worker or client-authorized retry loop.
Successful exchange directly registers and returns the record ID, with no second
confirmation/registration. Same-account reauth preserves the ID and references;
wrong or unavailable account identity cannot replace the old record. Identity
comes from the explicit ChatGPT account claim, not an email/subject fallback.

Progress distinguishes starting, pending, checking, exchanging, registered,
cancelled, expired, failed, superseded and uncertain. Provider rejection,
unavailability, invalid response and uncertain transport are typed independently
of the retained progress. In-flight state without this process's work owner is
projected uncertain. Neither time passing, status reads, duplicate calls nor a
lost browser waiter permits an uncertain exchange to dispatch again. Explicit
cancellation persists its fence independently of external-work permits, JoinSet
bookkeeping contention and unrelated retained work failures. It fences later
publication without claiming provider revocation or physical cancellation; the
work owner still retains failures for ordinary reaping and explicit shutdown.

The foreground Client composition owns capacity 16, within the Service's allowed
1–64 range. Admission rejects contention/overflow before Store admission with a
typed resource-exhausted reason. Started tasks and permits outlive abandoned RPC
waiters through completion/reaping. Shutdown closes admission and explicitly drains
this owner, including partial startup failure; primary and cleanup failures are
retained rather than reported as successful shutdown.

The adapter permits only fixed `https://auth.openai.com` device-issue/check/token
endpoints under the approved direct-HTTP exception. It uses platform-verified TLS,
no redirects or automatic retries, 3-second connect and 10-second call timeouts,
and a 128-KiB bounded successful response body, including chunked responses. Poll
403/404 means pending. Token metadata is optional; absent expiry is not invented.
The API accepts no arbitrary issuer, transport override or certificate bypass.
Deterministic provider substitution belongs to trusted fixture composition only.

## Saved Agent selection, creation and cleanup

SelectAgent changes only the saved authentication reference for an existing Agent.
Personal records must belong to its User; Team records must belong to the Agent's
Team. Current private ownership and membership are checked separately from record
use. Expected absence or the pair of authentication ID and accepted selection
Request ID fences stale intent, including A→B→A. Completed replay recovers its
original accepted result even after later selection changes or catalogue removal.
The Agent projection optionally contains record ID, alias, kind, selection Request
ID and time. Existing/pre-feature Agents remain unselected.

New Agents require an explicit authentication ID in the complete creation payload.
After current owner/Team admission and accepted-receipt recovery, Store locks and
validates that credential before Provider/Computer admission. Personal records
must belong to the Agent owner; Team records must belong to its current Team.
Agent, Computer, creation receipt and saved selection are inserted atomically,
and the creation Request ID is also the initial selection Request ID. The
authentication ID participates in the typed creation fingerprint; changing it
under an accepted creation Request ID conflicts. Authentication deletion and
creation reference admission share the credential lock. Exact completed replay
precedes live credential eligibility, so later catalogue removal or changed
selection cannot erase an accepted creation result.

Historical creation fingerprints remain untouched. Owner/request-only
Agents.CreationResult recovers pre-feature receipts independently of the new
payload representation; it does not run fresh creation admission or backfill a
selection.

Selection neither requires connectivity/power nor changes Computer lifecycle
requests. Lock order is User, Team, sorted old/new authentication records, Provider,
Computer. Agent removal detaches the reference without deleting reusable
credentials. Team departure/kick detaches the departing User's affected Agent
references and invalidates affected attempts without transferring personal
credentials. Team deletion clears its Agents' references and Team-owned catalogue
material before existing native cleanup admission. Record removal/cancellation
invokes no external revoke endpoint. [Schema constraints](account-storage.md#external-authentication-storage-foundation)
protect relationships and empty-only downgrade, not provider validity.

## Internal central-refresh foundation

The service/adapter libraries provide a source-bound central ChatGPT refresh
operation, composed into Control's execution-only credential service and the
Connector's actual native-start provider. Catalogue selection and ordinary
metadata reads still do not refresh, publish files or start native work.

The adapter uses the public `oauth2` refresh-grant API for request construction
and response parsing, with the existing fixed issuer/client/token route and
verified-TLS transport. Redirects, automatic transport retries and arbitrary
issuers remain disabled. The body is bounded to 128 KiB before SDK parsing;
SDK errors and vendor descriptions never enter returned error source chains.
Definite local non-dispatch, parsed standard OAuth rejection and uncertain
transport/malformed/unpublishable responses are separate results. Successful
responses preserve a valid prior refresh token if its replacement is omitted.
The admitted account remains bound and missing expiry is not invented. Before
publishing a candidate, the adapter checks the complete refreshed bundle against
the same account-bound native external-token encoder used for execution. An
opaque or otherwise ineligible access token remains uncertain even when the
OAuth response envelope succeeded. These
fixture-backed contracts do not certify actual device-acquired refresh
compatibility or native adoption.

Preserving migration 14 adds one durable refresh attempt per authentication record
and exact existing protected source nonce. It does not add a credential revision
counter or change earlier migrations, record IDs, owner mutation identities or
historical fingerprints. Alias-only changes do not create another credential
source; replacement, reauthentication and successful refresh do. Current
owner/scope checks and record locking fence admission and publication. A late
old-source/account result cannot overwrite manual replacement or recreate a
deleted record. Narrow failure settlement shares the record lock with publication
and cannot publish tokens or restore authority. Catalogue deletion retains only
its existing saved-reference checks; the refresh table cascades rather than
adding an in-use blocker. Populated new refresh state blocks downgrade.

The process-owned refresh service admits bounded external work independently of
an abandoned consumer waiter. Concurrent consumers encounter the same durable
attempt instead of exchanging the same source independently. An attempt still
possible without this process's work owner is projected uncertain; server
replacement does not grant another exchange. Completed uncertainty/rejection
remains tied to that source until explicit owner replacement/reauthentication
provides a new source. Fresh or unknown-expiry sources can be read internally
without issuing a refresh. The component service below explicitly composes this
work owner with execution admission; a metadata query does not invoke it.

## Internal applied-execution boundary

Completed Store operations admit a saved authentication only against an existing
possible Setup operation and the current private Agent/Team, Connector session,
Control route, operational key and execution. This admission is intended for
the actual native launch boundary, not every Setup receipt. It records the
selected record and saved selection Request identity. Repeated admission for
that current operation recovers the same record; a new operation compares the
previous current application before superseding it. A superseded operation
cannot reclaim current authority or obtain another record by replay.

Credential-bearing operations lock the eligible authentication record before
final Connector/Computer admission. They then recheck the current route,
execution, Agent authority and applied identity before returning material or
publishing a refresh. Initial execution discovery grants no secret authority.
This preserves the catalogue → Provider → Computer lock order when several
Agents share one personal authentication record.

Applied provenance belongs to the execution/Agent/conversation lifetime. Source
replacement may remove input and Setup history without erasing that identity.
It contains no command-delivery result, native login/adoption assertion, secret
or replay grant. Initial admission still requires the original live Setup
permission; a removed input cannot be used to admit a new start.

Current material reads consult the applied record, never the latest saved
selection. Same-record key replacement changes its protected source while
preserving the applied identity. An optional exact source equality check rejects
an older handoff. Each read rechecks current Connector/route/execution, private
ownership, Team membership and record scope. The server-internal result can still
contain the protected ChatGPT bundle, so a delivery adapter must convert it to
short-lived execution material before any wire transfer.

Applied-only provenance adds no catalogue `InUse` blocker. Deleting the last
unsaved applied record leaves a non-secret identity but denies material reads
and old-operation admission; it never selects a replacement automatically.
Existing saved-reference deletion protection remains unchanged.

Execution consumers can also admit and complete the same central refresh under
current applied-record authority instead of an account session. Account and
execution consumers share the record/source attempt; they do not get independent
refresh grants for the same source. Successful publication separately rechecks
the current execution/route/application, record scope, source, account and
possible attempt. A saved future selection does not retarget this authority.
Route loss, manual replacement, application supersession or deletion fences a
late publication. Narrow non-success settlement cannot restore authority or
grant an automatic retry.

`AuthenticationRefreshService.current_execution` retains an owned authenticated
Connector source and application reference in the same bounded work owner as
account consumers. It does not manufacture an account session. Provider work,
source coalescing, cancellation independence and explicit close/drain use the
shared orchestration. A returned protected bundle is still server-internal and
requires separate execution handoff revalidation/conversion.

## Component credential delivery

Only Control's `ConnectorCapture` service registers the execution credential
methods. Client/Admin account services do not expose a credential-distribution
API. Account or Provider tokens cannot replace the exact operational Connector
source. The existing trusted-network plaintext HTTP boundary is retained; a
bearer is not transport encryption.

- `GetAuthenticationBinding` recovers non-secret current application provenance.
  It neither selects nor refreshes a credential.
- `BeginAuthentication` seals the current saved selection for the exact possible
  Setup operation, comparing the previous application. It performs no native
  effect and is intended only for a real native-start caller.
- `AcquireAuthentication` is the explicit eligible-consumer renewal path.
  API keys return current metadata without provider I/O. ChatGPT uses the shared
  refresh owner and rereads the exact resulting applied source before native
  encoding. Pending/settled observations contain no credential bytes.
- `ReadAuthentication` transfers at most 32 KiB of the current source's raw API
  key or converted native external-token file. It never transfers the server's
  ChatGPT bundle or usable refresh token. The complete supported material is
  bounded to 64 KiB and is not truncated to the RPC frame limit.
- `CheckAuthentication` revalidates the exact application/source at final handoff
  without refresh or native intervention.

Each piece rereads current authority and the requested protected source. Changed
sources reject continuation or handoff with the existing typed conflict reason;
the receiver must discard a partial old transfer rather than combine sources.
Requests/responses retain the 64-KiB frame bound. Sixteen read/materialization
permits bound database/encoding work, and refresh work has its separate bounded
lifetime. Debug output omits every credential message field and nested payload.
API keys are delivered for explicit ACP login; their presence is not a model
validity/entitlement claim. ChatGPT conversion validates the inspected native
shape and admitted account, not JWT signatures or live native adoption.

The foreground Control composition owns this service before listener admission
and closes/drains its refresh work on normal shutdown and partial startup failure.
The native actor invokes these methods only for a real new launch; exact
already-live Setup does not replace its authentication. The live update worker
uses only that applied record and publishes changed-source files without native
login/reload/restart. Authenticated runtime acceptance,
immediate file adoption and successful model calls remain outside this component
transfer's verification boundary.

## Connections

- [Account RPC](account-rpc.md) — per-call credentials, generated contracts and group isolation.
- [Control admission](computer-control-rpc.md) — Separate component authority and trusted transport boundary.
- [Credential primitives](credential-primitives.md) — separate purpose keys, contextual envelopes and zeroizing secrets.
- [Account storage](account-storage.md) — preserving explicit preparation and retained historical identities.
- [Private Agent ownership](agent-ownership.md) — independent Agent authority and Team cleanup.
- [Owner RPC](computer-client-rpc.md) — optional saved-reference projection, separate from Computer observations.
- [Authentication web](authentication-web.md) — scoped catalogue, device/key interaction and explicit Agent selection.
- [Foreground process ownership](runtime-process.md) — partial startup and joined failure completion.

## References

- [`proto/chapchoo/authentication.proto`](../../../../proto/chapchoo/authentication.proto)
- [`rust/crates/service/src/authentication.rs`](../../../../rust/crates/service/src/authentication.rs)
- [`rust/crates/persistence/src/external_auth.rs`](../../../../rust/crates/persistence/src/external_auth.rs)
- [`rust/crates/openai-auth/src/lib.rs`](../../../../rust/crates/openai-auth/src/lib.rs)
- [`rust/crates/rpc/tests/authentication.rs`](../../../../rust/crates/rpc/tests/authentication.rs)
