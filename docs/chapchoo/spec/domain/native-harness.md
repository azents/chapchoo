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
its response receiver. Prompt acceptance preserves its exact native message ID;
it is not completion. Normal task completion does not shut down the owner.
After a possibly published prompt loses its acknowledgement, later prompt
admission is fenced until physical retirement; timeout does not grant replay.

Session resume requests retained history from `start` and fails without creating
a fresh context if restoration fails. Original incoming session update, permission
and elicitation envelopes reach the capture boundary before typed SDK decoding;
unknown fields survive. Authentication requests/results are excluded. Pending
permission/form responders are bounded, cancellation-aware and single-use. The
client advertises forms only and rejects unsupported elicitation modes.

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
- [`rust/crates/harness/examples/scope_probe.rs`](../../../../rust/crates/harness/examples/scope_probe.rs)
