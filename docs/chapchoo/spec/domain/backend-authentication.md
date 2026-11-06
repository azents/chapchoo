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
current accepted mutation Request ID and creation/update instants. Missing provider
metadata stays absent. Provider expiry outside the supported Protobuf Timestamp
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

## Connections

- [Account RPC](account-rpc.md) — per-call credentials, generated contracts and group isolation.
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
