---
title: "Native preparation and execution-local ownership"
---

# Native preparation and execution-local ownership

`chapchoo-harness` is a reusable native foundation. The foreground
[Connector](connector-process.md) owns an idle instance through stream/key
replacement and explicitly finishes it on process-loop exit. Source-qualified
conversation commands and actual-start authentication use separate admission
boundaries; implemented paths do not certify successful live model execution.

## Storage, preparation and applied supply

The caller supplies the actual existing saved root. Storage admission rejects
relative/noncanonical roots and redirected product-owned directories. Private
installation cache, HOME, native state, temporary files, workspace and captured
source classes are separate under that root. A write probe checks actual storage;
server HOME and ambient account configuration provide no fallback.

The single execution actor serializes preparation, applied leases and collection.
Managed acquisition selects the latest stable Node 24 Linux x86-64/ARM64 release,
verifies its official SHA-256 archive digest, rejects escaping archive paths/links
and bounds download/extracted size. Managed npm resolves the adapter `^2.5.0` and
Codex `^0.165.0` dependency closure with scripts disabled, a fixed official
registry, separate empty user/global configuration and an isolated cache. Complete
publication validates the lock's registry/integrity records, selected package
versions, layout and actual Codex version. Native optional-package aliases are
resolved by npm, not parsed as stable CLI versions.

Only a complete compatible installation can be reused after transient acquisition
failure, with a distinct freshness result. Integrity/layout/compatibility failure
does not select cache. Cache reuse rechecks payload hashes and supported layout;
the mutable local manifest is not a same-UID tamper boundary. Active leases and
the latest complete combination protect their complete self-contained closure.
Collection removes only inactive superseded owned installation directories;
native context, credentials, captures and workspace are outside deletion.

The applied descriptor retains exact Node, adapter and Codex versions. Existing
mode records the actual probed CLI version rather than the managed CLI's version.
A read-only owner preparation query exposes these nonsecret identities and the
freshness outcome from the physically started combination; it never acquires,
starts or refreshes anything. An owner without a live native process reports no
applied preparation. Foreground Idle is not physical process retirement.

The live observation reads foreground, exact source, preparation and applied
authentication identity from the same actor-owned physical lifetime. Connector
publication verifies that source even for Idle, where no work target exists.
These nonsecret facts accompany ordered captured state; historical reports that
did not contain them remain unknown. Owner conversation reads also read the
current saved authentication selection without acquiring credential material.
Saved selection and a retained applied identity have independent meanings:
selection changes do not retarget the live process, and a deleted applied record
does not prove offline native revocation. File publication of newer material from
the same record does not change that record identity.

Explicit existing CLI mode checks its absolute local path and compatible actual
version. It uses the managed adapter/runtime but never overwrites or silently
replaces the owner's CLI. Applied supply stays fixed through work completion and
server-link loss. The live owner also retains the exact requested managed/existing
choice from actual start. Returning an already-live session does not replace that
choice with a newly saved selection. Source-qualified observation reports it
separately from current foreground work; storage/UI do not infer it from the
latest saved choice or native connectivity.

## Protocol and physical ownership

### Execution-only credential file boundary

The library validates API-key material or a server-converted external-token
file. External-token ingress requires the admitted account, matching access-JWT
identity fields, an empty refresh token and a valid RFC 3339 `last_refresh`.
The native token reader requires that timestamp even for externally managed
access tokens. The server derives it from the retained application admission
time, so all bounded reads for one application/source are byte-identical. It is
native application metadata, not evidence of an OAuth renewal or immediate
adoption. Mixed API-key material and unknown fields are rejected.
This is schema validation, not JWT signature verification
or execution authorization. The server retains refresh authority.

The finite blocking publication operation writes only `state/auth.json` beneath
the explicitly retained saved-root descriptor. Private directory/file descriptors,
exclusive temporary creation, mode 0600, file sync, handoff identity checks,
atomic rename and directory sync form its publication boundary. It rejects
redirected or non-private classes and symlink/special/hard-linked targets rather
than mutating their referents. Rejection cleans the temporary file. Native
sessions, workspace and captured events remain separate. Same-UID isolation is
not claimed.

The actual-start actor now invokes this helper for external-token startup. For
API keys it first removes only residual owned `auth.json`, then uses explicit ACP
login rather than initial file seeding. API-key file serialization remains for
replacement, whose live caller uses the same admitted record. File publication alone
proves no cache reload, immediate adoption, model entitlement or external revoke.

### Actual-start authentication

An already-live exact resume returns the existing session before any new
authentication provider call or file operation. A real launch requires a supplied
`NativeAuthenticationProvider`. After compatible artifact preparation, the
Connector provider recovers the previous application, admits the saved selection,
explicitly acquires current material, assembles bounded pieces with an unchanged
descriptor and performs the final source check. Missing/failed authorization
refuses launch rather than adopting an old file. Pending/uncertain refresh is not
an interactive login or a native replay grant.

The actor retains the admitted operation/execution/record/source descriptor for
the live lifetime. It rechecks that descriptor before and after initial file
preparation. The finite blocking file job is stored in actor state, awaited by
reference and cleared only after physical completion. Cancelling a launch or
partial retirement waiter cannot detach it; retirement drains it before scope
teardown or successor admission. Saved context and workspace remain untouched.

The launch wrapper explicitly selects native file credential storage. Following
strict v2 initialization, an API key uses the public SDK `auth/login` operation
with method `api-key` and explicit `_meta["api-key"].apiKey`; credentials never
enter argv or the child environment. Only successful login proceeds to New or
Resume. Rejection reports a body-safe Authentication protocol stage and does not
fall back to session creation. External-token startup uses the seeded file and
does not call the unsupported native token-login/callback path.

The foreground diagnostic layer excludes ACP SDK diagnostics independently of
operator log-filter specificity: the inspected SDK TRACE redacts env/headers,
not arbitrary login metadata. Product diagnostics retain body-safe categories;
raw native stderr is drained without publication. Authentication requests and
responses remain outside session JSONL. These are controlled protocol tests and
source-backed startup wiring, not authenticated runtime/model acceptance.

### Actual-start server tools

Every production setup also supplies a narrow tools provider and seals the
Agent/conversation in its immutable setup fingerprint. An already-live exact
lookup validates that original product scope before returning the existing
session; it acquires neither authentication nor another tools provision.

An actual launch checks scope against its admitted authentication binding and
retains a tool provision before starting the native process. Strict-v2
initialization requires advertised HTTP MCP capability. Both New and exact
Resume carry the same consumer-owned HTTP declaration through public SDK
builders. Effective initialization/catalog verification is part of the
remaining thirty-second native startup budget, after the exact native identity
and capture binding are retained. Failed verification cannot erase that source
or permit a fresh context. It neither invokes a tool nor asserts no possible
prior effect.

Retirement first withdraws tool access, then drains native and tool resources
through explicit fallible completion. Pending provision and cleanup state stay
with the actor when a waiter is cancelled; independently authorized stages
continue and combined failures preserve their causes. The
[server-tools contract](server-tools.md) owns HTTP capability, invocation and
publication authority; this library owns declaration translation and physical
native lifetime, not server credentials.

### Same-record live publication

The Connector reads the existing live binding without starting native work.
Its separate coalesced worker explicitly renews/reads only that application;
it never calls saved-selection admission for an update. An unchanged source
requires no payload transfer or file write. Changed sources are assembled
outside the native actor and submitted with the exact preceding binding.

The actor rejects a changed Agent, conversation, start operation, execution,
record or preceding source. It performs one bounded final server check, then
serializes the private file replacement through its retained blocking owner.
It updates its source echo only after completed file publication. An old-route
result cannot pass the final server check, and an old preceding source cannot
overwrite a locally published successor. Native protocol termination denies
publication. Network acquisition/refresh is not serialized inside the actor;
the final authority check is bounded by the ordinary component call budget.

Both API-key and external-token updates are file-only. No ACP login, cache reload,
session creation, restart or termination is sent. File JSON has an independent
bounded zeroizing buffer so escaping a complete 64-KiB raw key does not truncate
it or fail merely because of JSON overhead. Native adoption still depends on
native behavior and is not asserted by file publication.

### Session protocol and operation ownership

The public Rust ACP SDK uses strict `Client.v2()`; an incompatible initialization
fails without a v1 session. Native stdin/stdout/stderr, protocol driver, pending
responders and applied installation lease belong to the execution actor, not a
server waiter. A bounded command queue retains admitted work after a caller drops
its response receiver. Cloneable operation handles share the same queue and byte
budget; they do not own namespace retirement. The process retains the sole
physical lifetime owner and can finish it even while handles remain. A handle
cannot revive a finished owner. Input has an exact snapshot/native-session target and
application operation ID. The owner records possible forwarding before the SDK
operation, retains bounded fingerprints/results through stream replacement and
never evicts uncertainty to admit another operation. Lookup does not dispatch.
Absent execution-local evidence is not non-delivery. Identical input under a
different operation remains independent; changing a retained operation conflicts.
Queued input bytes have a separate retained memory budget.
Prompt acceptance preserves its exact native message ID, not completion.
Losing an acknowledgement leaves that operation's possible-forwarding fence
intact; timeout never grants replay. Normal completion does not stop the owner.

Input uses the producer's `_session/steering` extension with
`idleBehavior: promptRequired`. Only explicit `promptRequired/noRunningTurn`
permits an ordinary prompt. Failed/malformed/unsupported responses, cancelled
waiters and response loss never permit that transition. Steering acceptance does
not invent a native user-message ID. Ordinary prompt acceptance preserves its
exact native insertion ID; neither is completion. Normal task completion does
not shut down the owner.

Native setup requires the owned captured source. An initial source restore can
request retained history from `start`; an existing captured copy resumes without
requesting history again. Restore failure does not create a fresh context.
Original incoming session update, permission
and elicitation envelopes reach the capture boundary before typed SDK decoding;
unknown fields survive. Authentication requests/results are excluded. Pending
permission/form responders are bounded, cancellation-aware and single-use. The
client advertises supported flat forms only. Unsupported required fields/modes
receive a protocol request error rather than a simulated refusal. Request tokens
are fresh execution-local identities paired with their captured occurrence;
numeric and string JSON-RPC IDs remain distinct and reuse cannot revive an old
response target. Responses recheck cancellation, source-bound native session and
offered permission choices before consuming a single responder. Form responses
also validate declared supported properties, required presence, value types,
string lengths, numeric ranges and single/multi-select choices. False, valid empty
values and optional omission remain intact. Integer-versus-number boundary
comparison does not round large exact integers. Unsupported required constraints
receive a request error; unsupported optional values cannot be submitted as if
validated. An invalid answer leaves the actual responder available for a valid
response. Local SDK responder acceptance does not prove remote consumption.

Operation-owned responses additionally retain a fingerprint of the exact source,
native session, captured occurrence, responder token, response kind and original
typed values. The actor records possible forwarding before checking and consuming
the responder; it distinguishes local SDK acceptance from known pre-dispatch
rejection. A generic protocol failure leaves uncertainty. Repeating that
operation recovers its retained observation, while changing its fingerprint
conflicts. Retained registries are bounded and reject overflow rather than
evicting uncertain operations or granting another effect. The Connector can
release an exact known result only after successful durable result publication,
while retaining its exclusive invocation lease
through actor acknowledgement. Setup, input, answer and Stop share this
known-result release contract; uncertainty and changed-result conflicts remain
retained. The server's settled checks, or denied body/grant reads for a consumed
input, fence later dispatch, while the lease and
single command-checking loop fence already-checked local duplicates.

Foreground observations create an execution-local work target that expires on
idle. Stop verifies its snapshot/session/work identity immediately before native
cancel publication; a delayed Stop cannot silently target a later known work
generation. Native state observations are applied at ordered capture ingress,
not delayed SDK callback completion. Protocol/capture termination reuses physical
retirement rather than leaving unrecorded work alive.

## Retained captured source

The current capture has an explicit snapshot/native-session manifest and ordered
JSONL occurrences, with a single writer held through the source lifetime.
Original UTF-8 envelopes are retained before typed SDK decoding. A supplied actual
occurrence can recover identical source data; otherwise every occurrence receives
an independent identity. Native message IDs and JSON-RPC IDs do not become event
occurrence identities.

Opening a missing, corrupt or partial source fails without truncation, replacement
or native-gap repair. A finite read pass resolves an exact retained occurrence,
captures a complete-record boundary and reads only through that boundary while
append may continue. Its read capacity remains held until the pass is dropped.
The entire current copy remains retained through server acknowledgements; no
prefix pruning, rotation or compression is implemented.

A narrow capture reader can reopen an existing retained copy without preparing
artifacts, starting native work or requiring native PID1 admission. Missing first
copy is absence; an existing malformed copy is an error. The execution actor owns
finite blocking open work beyond a cancelled read waiter, retains the same copy
across native retirement and shares it with later setup. Cheap current boundary
metadata does not create a finite pass; only `begin` fixes that boundary.

## Restricted physical ownership

Child environment is cleared and rebuilt from the applied descriptor, including
explicit native HOME/state/TMP, cwd and selected executables. Platform bootstrap
and operational credentials are not inherited. A transient single-use native
launch grant prevents the adapter's internal recovery from starting another
native generation under the same applied owner. Another generation requires
whole-scope retirement, not direct-child EOF or the adapter's own recovery claim.
This helper is not an adoption broker and creates no outgoing dispatch journal.

Native admission checks actual Linux UID/GID 10001, Connector PID1, namespace
PID evidence, zero effective capabilities, no-new-privileges, seccomp, read-only
root and the explicit writable saved mount. Intended Provider configuration alone
does not satisfy these checks. Controlled retirement closes admission, attempts
bounded cancellation/close/drain, then signals every process in that verified
private namespace, escalates and reaps until no descendant remains. Unknown
retirement fences successors and retains the applied live resources; task
completion or link replacement never releases them. Namespace-init death is the
owner-death teardown boundary. No privilege relaxation, daemon or automatic
Computer restart is added.

## Verification boundary

Deterministic tests cover storage escapes, cache leases/integrity/fallback,
raw-event preservation, strict v2, failed restore without fresh context, prompt
acceptance versus completion and idle-owner cleanup. The explicit `scope_probe`
example supports isolated model-free restricted-runtime acceptance. Probe
success is not a real model call, full Provider conversation or live Kubernetes
acceptance.

## Connections

- [Conversation storage](conversation-storage.md) — Independent server input/source facts do not establish native delivery.
- [Backend authentication](backend-authentication.md) — Protected catalogue sources and server-owned refresh.
- [Docker profile](computer-docker.md) — Exact saved root, Connector init and private namespace profile.
- [Kubernetes profile](computer-kubernetes.md) — Saved PVC and rejection of shared process namespaces.
- [Linux images](account-delivery.md) — The restricted image supplies runtime ABI libraries.

## References

- [`rust/crates/harness/src/lib.rs`](../../../../rust/crates/harness/src/lib.rs)
- [`rust/crates/harness/src/runtime.rs`](../../../../rust/crates/harness/src/runtime.rs)
- [`rust/crates/harness/src/authentication.rs`](../../../../rust/crates/harness/src/authentication.rs)
- [`rust/crates/harness/src/preparation.rs`](../../../../rust/crates/harness/src/preparation.rs)
- [`rust/crates/harness/src/capture.rs`](../../../../rust/crates/harness/src/capture.rs)
- [`rust/crates/harness/examples/scope_probe.rs`](../../../../rust/crates/harness/examples/scope_probe.rs)
