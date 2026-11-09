---
title: "Codex Conversation Phase 2: Native Preparation and Lifetime"
created: 2026-11-09
tags: [agent, conversation, codex, implementation, runtime]
---

# Phase Execution Plan

- Phase: 2/6, native preparation and lifetime.
- Branch/base: `feature/chat-text-2-native` → `feature/chat-text-1-foundation` (PR 97), including its Control-lock correction `1be9ad9`.
- PR boundary: a reusable native owner/preparation library, execution-local Connector lifetime composition, restricted runtime prerequisites and deterministic/model-free runtime verification. Public conversation wire/service/UI activation belongs to later phases.
- Inputs: [Requirements r24](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md), [approved Design r9](../design/chat-261108-text-conversation.md), [implementation plan](chat-261108-implementation.md), [execution grounding](../notes/chat-261108-execution-lifetime.md).
- Approved Design mechanisms: M02–M06, M17, M19, M24–M25 (native/preparation portions; credential bytes/publication are phase 4).
- Authority references: REQ-1/4/5/11/12/14/15/18, ADR-D1–D15/25–27/31–32, existing Docker/Kubernetes saved-storage and Connector authority contracts.
- Design delta: None.
- Owner: primary agent `/root`; exact independent reviewer: `/root/chat-implementation-reviewer`.

## Deliverables and Interfaces

1. A reusable harness library owns preparation, a coherent applied launch descriptor,
   bounded native command/observation state, supported ACP v2 connection and physical
   process cleanup. The foreground Connector owns this instance outside the
   replaceable server `connected()` loop; there is no additional daemon/adoption
   broker and normal task completion does not retire it.
2. Actual input-triggered start receives the server-selected supply/context rather
   than inferring an executable or account. Absent settings were already resolved
   to managed by phase 1. A changed setting does not mutate a live applied owner.
3. Managed acquisition resolves the newest complete eligible combination within
   explicit compatibility constraints, using official HTTPS metadata/digests.
   The managed Node distribution's standard package tooling may resolve/extract
   its declared dependency closure with lifecycle scripts disabled, a fixed
   official registry, isolated configuration and verified complete publication.
   No ambient npm configuration, arbitrary source or shell command is admitted.
   Existing CLI remains an explicit absolute Computer-local path and is never
   overwritten or silently replaced with managed supply.
4. Preparation stages and publishes only complete combinations, preserves eligible
   complete cache for transient freshness/download failure, and distinguishes
   integrity/layout/compatibility failure. Cache collection protects every active,
   preparing and latest complete eligible component, including shared components,
   and deletes only inactive superseded owned cache after complete replacement.
5. The public `agent-client-protocol` SDK has a concrete draft-v2 API. Use strict
   `Client.v2()` with `unstable_protocol_v2`, not a v1 protocol-router/fallback or
   copied private SDK implementation. Prompt acknowledgement is not work completion.
   Original incoming session-event bytes must remain available for phase 3 capture;
   typed SDK reserialization must not erase unknown source fields. Credential
   messages are excluded from this event interface. Interactive responder ownership
   and observations remain bounded and bound to their actual native session/work.
6. The launch descriptor explicitly supplies cwd, saved root, native HOME/state/TMP,
   selected executables and a cleared child environment. Bootstrap/operational
   platform credentials and ambient model credentials do not enter children.
   Providers pass the configured actual saved mount through the strict bootstrap
   profile; this adds no new mount or server-HOME-derived workspace authority.
7. Before production native admission, establish the actual supported UID/PID1,
   non-shared namespace and allowed writable-storage conditions. Preserve
   non-root/read-only-rootfs/dropped-capability/no-new-privileges constraints.
   Normal shutdown provides bounded cooperation/drain then whole-scope escalation
   and quiescence/reaping evidence. Owner death uses verified namespace-init
   teardown; direct-child wait/process-group signalling alone is insufficient.
   Unknown retirement blocks a successor, without silently power-cycling Computer.

Names, module decomposition, equivalent concrete SDK/transport helpers and exact
resource budgets are local details within these contracts. They do not add a mode,
policy, source of truth, new credential bridge or protocol compatibility fallback.

## Owned Workstreams

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Runtime/preparation domain | `/root` | `rust/crates/harness/**`, workspace Cargo manifest/lock and explicit dependency-boundary table | phase 1 domain/approved native contracts | typed applied descriptor, safe storage/preparation, cache leases and native owner | owned unit/integration tests, Clippy, docs and dependency gate |
| Connector composition | `/root` | `rust/apps/chapchoo/src/{connector,main,config}*` and owning tests | library interfaces | lifetime outside stream refresh and explicit saved-root input | stream/key replacement, cancellation and shutdown fixtures |
| Substrate/profile prerequisites | `/root` | `rust/crates/{docker,kubernetes}/src/**` owning profile areas, `images/chapchoo/Dockerfile` | explicit launch descriptor and current strict profiles | saved-root bootstrap data, private PID namespace admission, ABI libraries | both adapter mapping tests and model-free actual Docker probe |
| Evidence/current contracts | `/root` | affected semantic Spec pages and this phase plan | integrated implementation | current-only native boundary and exact conditional acceptance | semantic lint, actual evidence and same-reviewer stable-diff review |

- Integration order: actual environment/storage/scope values → official preparation/cache → strict public ACP v2 owner/transport → Connector lifetime composition → both Provider profile inputs/ABI → model-free Docker verification → focused/full local gates → phase PR → next phase. Remote CI inspection and all further independent/automated reviews and corrections wait until all six phase PRs exist.
- Removal obligations: replace stream-local-only execution assumptions where the owner is composed; no implemented harness or adoption broker exists to retain. Do not remove Computer power, credentials, native context, JSONL sources or workspace. Sample Chat/local success remains until phase 5 activates its replacement.
- Absence verification: stream refresh cannot construct a second owner or retire the first; no native start on read/connectivity/task completion; no ambient executable/auth fallback, v1 path, uncertain successor, new mount, privilege relaxation, lifecycle script or hidden broker.
- Non-goals: server normalization/projection/catch-up, public conversation RPC, central OAuth refresh, actual credential publication, UI changes, idle shutdown, native-gap repair or new current-work confirmation.

## Verification and Human Authentication

Use deterministic ACP/process fixtures and explicit synchronization for accepted
versus completed work, output ownership, EOF/error, stream replacement, pending
responders, cooperative refusal, detached descendants and shutdown races. Keep
these combinations below browser E2E. Inspect actual supported image ABI and
private namespace conditions; never infer physical retirement from a transport
close. Root runs the full Rust/RPC gates and relevant image/profile checks.

The requester requires direct real Docker Provider tests with isolated owned
resources. Begin model-free physical preparation/lifetime checks as the path
becomes runnable and integrate the complete provider/owner conversation in later
phases. Existing production containers/volumes are outside cleanup authority.

Human Codex authentication is a separate nonblocking lane: leave a usable login
path when ready and continue all independent implementation, model-free Docker,
and PR creation. Review/remote CI work follows only after the stack is complete.
Reuse the authorized login. Actual model calls are restricted
to a short minimum core integration check; fixture success is not model success.
No real credential or model call is required to complete the independent work in
this phase. Unsupported runtime/SDK facts return as concrete findings rather than
new unapproved fallback behavior.

## Checkpoint

Phase 1 is open as PR 97. Its Control-lock correction is included in this
branch. Phase 2 adds the reusable execution actor, explicit storage/environment,
managed/existing preparation and complete-cache leases, strict public SDK v2,
bounded cancellation-aware responders and namespace-scope retirement. The
Connector owns the idle actor outside stream/key refresh. Both Providers pass
and validate their actual saved root; Docker rejects an injected init and
Kubernetes rejects a shared process namespace. The image supplies libstdc++.

The adapter's internal recovery is fenced by a per-applied-owner single-use
native launch grant. This transient child helper cannot authorize a successor;
only the Connector's verified whole-scope retirement can release the owner.
It adds no adoption daemon, persisted outgoing operation journal, privilege or
new storage/credential mount. Public conversation routing remains phase 3.

Root ran deterministic library/protocol/cache tests and the full local Rust gate.
Final commit hooks also run the canonical Rust/RPC and packaging checks. Review
and remote CI inspection are deliberately deferred until all six PRs exist.
The approved Requirements/ADR/Design are unchanged; Design delta remains None.

Actual model-free Linux x86-64 Docker checks use the product runtime-base image,
UID:GID 10001:10001, read-only root, dropped capabilities, no-new-privileges,
private IPC/PID and an isolated saved named volume. Detached descendants were
escalated/reaped to quiescence; a separate namespace-init exit check completed.
Official acquisition verified Node 24.22.2 (archive SHA-256
`361718c444e49af2ca5a110c83ba6a8901143f23ab285ebbef0341f391f482a9`),
adapter 2.6.0 and Codex 0.165.0, including 6,194 recorded payload files.
A separate managed-Node fixture verified that a second same-owner native launch
is rejected before its effect. An offline genuine-start probe reused that complete
cache and completed strict ACP v2 initialization. Session creation returned authentication-required
`-32000`; whole-scope cleanup completed. No model prompt or credential was used.
The local probe binary emitted a weak GLIBC_2.39 compatibility warning on
Bookworm, but scope/preparation probes exited with code 0; release-builder ABI acceptance is
still a separate phase 6 gate.

Remaining gates: authenticated native session/new/resume and short core model
confirmation, complete real Docker Provider conversation, supported ARM64
runtime acceptance and final full-stack validation. Human authentication blocks
only its dependent live checks, never phase progression.
