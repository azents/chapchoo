---
title: "Native preparation and execution-local ownership"
---

# Native preparation and execution-local ownership

`chapchoo-harness` is a reusable native foundation. The foreground
[Connector](connector-process.md) owns an idle instance through stream/key
replacement and explicitly finishes it on process-loop exit. Public conversation
command routing and authenticated text execution are not activated by this
foundation.

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
starts or refreshes anything. An idle owner reports no applied preparation.

Explicit existing CLI mode checks its absolute local path and compatible actual
version. It uses the managed adapter/runtime but never overwrites or silently
replaces the owner's CLI. Applied supply stays fixed through work completion and
server-link loss.

## Protocol and physical ownership

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
retained. The server's settled checks fence later dispatch, while the lease and
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
- [Docker profile](computer-docker.md) — Exact saved root, Connector init and private namespace profile.
- [Kubernetes profile](computer-kubernetes.md) — Saved PVC and rejection of shared process namespaces.
- [Linux images](account-delivery.md) — The restricted image supplies runtime ABI libraries.

## References

- [`rust/crates/harness/src/lib.rs`](../../../../rust/crates/harness/src/lib.rs)
- [`rust/crates/harness/src/runtime.rs`](../../../../rust/crates/harness/src/runtime.rs)
- [`rust/crates/harness/src/preparation.rs`](../../../../rust/crates/harness/src/preparation.rs)
- [`rust/crates/harness/src/capture.rs`](../../../../rust/crates/harness/src/capture.rs)
- [`rust/crates/harness/examples/scope_probe.rs`](../../../../rust/crates/harness/examples/scope_probe.rs)
