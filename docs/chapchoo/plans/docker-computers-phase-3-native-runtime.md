---
title: "Docker Computers Phase 3 Native Provider Runtime"
created: 2026-11-03
tags: [computer, docker, implementation, runtime, security]
---

# Phase Execution Plan

- Phase: `3/4 selected Docker connection and native lifecycle`.
- Branch/base: `feat/docker-computers-3-native-runtime` →
  `feat/docker-computers-2-native-authority` at `eef46b1`
  ([PR #50](https://github.com/azents/chapchoo/pull/50)).
- PR boundary: separate DB-free Docker Provider entrypoint, faithful frozen
  connection resolution/public SDK transport, native resource/effect adapter and
  common retained Provider task ownership. Preserve Kubernetes operation.
- Inputs: phase 2 schema/native server authority, M16 exact live manifest,
  generated contracts and native inventory; 142 Rust tests/47 groups and 67 live
  RPC checks per engine; full frontend checks/build and independent review with
  its single lifetime finding corrected and targeted re-review clear. Create
  the complete planned PR stack before checking phase CI results; handle reviews
  during implementation and correct confirmed failures in their owning phase.
- Deliverables: resolve the executing account's Docker CLI environment/context or
  coherent explicit override once; preserve chosen Unix/SSH/TCP/TLS policy; bind
  and recheck actual daemon; reserve/report/ACK original volume and full-ID
  container through Control only; start once and classify actual power; exact
  intentional retirement under `always`; individually scoped cleanup; conservative
  uncertain recovery and retained effects across waiter/stream loss.
- Non-goals: product DB access from Providers; enrollment/adoption/reset APIs;
  generic command ledger/retry queue; endpoint fallback/security-policy changes;
  private-registry credential management; host mounts/socket exposure; broad
  prune/privileged repair; deployment, production effects or merge; claiming real
  Linux/Mac/Desktop/remote-mode acceptance from synthetic tests. Platform packaging
  and required real acceptance are phase 4.
- Interfaces: phase 2 ProviderKind/native wire variants, current-source CheckCommand
  and report ACK boundaries, immutable daemon binding, original saved-volume
  identity, M16 exact command/call correlation and independent native timing.
  Common dispatch uses substrate-native adapter values, never Docker-as-Pod/PVC.
  Authenticated CheckCommand returns the exact execution's existing prior-running
  completion and initial-allocation facts as read-only admission evidence. These
  derive from the current execution row, not Provider assertions, new stored
  state or replay grants. Native same-process unknown Start/stage correlation
  remains a veto; bare running observation cannot settle a possibly pending Start.
  Docker Retire carries its original saved-volume reservation/identity as a
  read-only verification reference, so native mount checks do not invent that
  authority from labels, names or process-local cache after reconnect.
- Approved Design mechanisms: M2, M3, M5, M6, M7, M8, M9, M13, M15, M16;
  consumes M1/M4/M10/M11 and prepares M12/M14 validation without new authority.
- Authority references: confirmed [Requirements](../requirements/docker-261103-server-computers.md)
  REQ-1–REQ-6; accepted [ADR](../adr/docker-261103-server-computers.md) D1–D6;
  approved [Design](../design/docker-261103-server-computers.md) revision 6,
  connection fidelity, preparation/security, intentional retirement, M16 and timing
  sections; current computer-storage/directory/control-rpc/kubernetes Specs and
  supported-SDK, typed-ingress, test-owned-contracts conventions.
- Design delta: None.
- Removal obligations: move shared effect ownership out of the Kubernetes-only
  module and remove shared `Arc<Kubernetes>` dispatch ownership; replace fixed
  Pod/Secret/PVC preparation/cleanup assumptions at native activation. Preserve
  actual Kubernetes adapter/types/UID witnesses, timings and existing CLI meaning.
  Remove obsolete ownership exports rather than adding compatibility wrappers.
- Absence verification: compile/search for Kubernetes-only shared dispatch/effect
  ownership; both-substrate supported-client fixtures and child-process DB-free
  entry tests; no handwritten Engine routes/JSON operation codecs, alternate-host
  retry or Docker object encoded as a Kubernetes resource.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Selected connection and SDK transport | `/root` | `rust/crates/docker/`, workspace/app Cargo manifests and lockfile, dependency-boundary gate | Approved D1/D5/D6 and released SDK public APIs | Typed startup-frozen context/env/override, bounded socket/TCP/TLS/OpenSSH transport, response-body-owned tasks and no mutation replay | Isolated CLI-config precedence, TLS/SSH effective policy, remote socket, streamed body lifetime and failed connection fixtures |
| Native resource/effect adapter | `/root` | `rust/crates/docker/src/`, owning adapter fixtures | Selected SDK/daemon identity and phase 2 wire | Original-volume metadata, exact config/full ID, ACK-delimited preparation, running evidence, once-only update/stop/remove, safe individual cleanup | Public SDK synthetic service tests for collisions, missing volume, late/ambiguous effects, policy `always` versus admitted `no`, no force/replay |
| Native issue-report and completion evidence | `/root` | `rust/crates/persistence/src/{computer_executions,delivery}.rs`, `tests/native_execution.rs`, `proto/chapchoo/control.proto`, `rust/crates/rpc/src/component_auth.rs`, affected generated contracts/fixtures and owning Specs | Approved M8/M16 and phase 3 native error/stage integration | Preserve admitted uncertainty through Blocked diagnostics and carry existing exact prior-running completion to the native worker without another stored fact | Both-engine issue/late-report/Delete and authenticated dequeue/completion-evidence regressions; generated-client comparison; no reset or source of truth |
| Common Provider runtime | `/root` | `rust/apps/chapchoo/src/provider.rs`, `src/provider/`, `rust/crates/kubernetes/src/{lib,effects}.rs` and owning tests | Native adapter and unchanged K8s contracts | Substrate-native dispatch and common retained task/permit owner; same source/ACK checks for each stage | Existing K8s Control/Provider regressions and deterministic task/waiter/stream barriers |
| DB-free executable activation | `/root` | `rust/apps/chapchoo/src/{main,config}.rs`, Docker Provider CLI/runtime module and tests | Common runtime and native policy | Separate explicit Docker command, startup daemon binding, owner/key/Hello flow and container-reachable Control override | Child process with absent/poisoned backend DB/root config, invalid native config and endpoint/secret redaction tests |
| Owning contract and integration | `/root` | `docs/chapchoo/spec/domain/computer-{kubernetes,control-rpc,directory}.md`, new owning Docker Spec where needed, `DEVELOPMENT.md`, phase checkpoint | Integrated runtime | Truthful operator prerequisites, timing, native policy and remaining acceptance boundaries | Root full Rust/RPC gates, documentation/diff checks, removed-path search |

- Integration order: public SDK/dependency/source verification → typed connection
  resolver/custom transport and fixtures → native resource validation/observations
  → common retained supervisor/adapter integration → ACK-delimited prepare and
  exact retirement/cleanup → separate DB-free entrypoint → complete root checks
  → stable-diff read-only review → root fixes/affected reruns → phase PR before
  phase 4. Check all phase CI only after the complete planned stack exists;
  handle review feedback during implementation.
- Independent review: `/root/computer-reviewer`, exact single reviewer, read-only
  one-way integrated phase review only after root implementation and validation.
  Root performs all implementation, fixes and integrated reruns directly.
- Final validation: `bash scripts/check-rust.sh`, `bash scripts/check-rpc.sh`,
  focused supported-SDK transport/adapter and child-process tests, catalog/diff
  checks and all affected consumers. Use only owned disposable DB/transport
  fixtures; no production URL/daemon/Kubernetes operations. Real native effects
  require an explicitly selected authorized disposable environment and remain
  separately reported. Hooks are mandatory; inspect every phase's exact-head CI
  after the complete planned stack exists, not as a phase-start gate.
- Scope-drift check: complete approved stages/ACKs and removal obligations; no
  fresh permission from 404, elapsed time, response timeout or changed source;
  no independent server DB authority in Provider; no host/context/security fallback
  or widened workload policy. Material mechanism changes return to Design.
- Context checkpoint: branch created from phase 2 `eef46b1`. Implemented native
  volume/container SDK stages, digest-bound image preparation, shared retained
  effect ownership, exact fresh Control checks/ACKs and separate DB-free Docker
  CLI. Removed Kubernetes-owned effect module/export and shared K8s-only dispatch.
  Added current prior-running-completion admission evidence and Retire's original
  volume reference without a new durable ledger. Preserved existing Docker
  UNCERTAIN through generic lifecycle diagnostics in both-engine regression.
  Only positively fresh volume creation grants initial copy; existing/recovered
  originals always use no-copy. Container creation is pinned to the validated
  actual image digest, not a mutable tag.
  Startup captures TLS material, negotiates API version once unless pinned, and
  owns one authenticated SSH master. Mux channels never reread account config or
  reauthenticate after socket/master loss. Proxy-required TCP routing is an
  explicit preflight compatibility failure; it never silently falls back to
  direct routing. No new proxy transport or policy override is introduced.
  Root's focused Docker suite passes 31 cases (16 unit including actual local
  OpenSSH and exact-client mTLS handshakes, 15 supported-SDK cases), zero failed/
  ignored. Workspace all-feature Clippy passes. Independent review found one P2:
  local correlation capacity was incorrectly reported as uncertain even before
  Create submission. Root separated that exact pre-submit rejection, connected
  it to safe PreparationBlocked only after earlier allocation ACKs, and added
  a deterministic 256-record/no-257th-Create regression. After one positively
  retired slot frees, retry keeps the same original volume and succeeds; an
  unknown Start record remains fenced. Native creation rejections and actually
  uncertain stages retain their distinct classifications. The real-Control/native-SDK
  journey passes normal volume/container ACK → Start → Stop/Start/Restart/Delete
  and unknown Start across authenticated source reconnect without replay. A
  source-replacement-before-volume-ACK case also passes: a Delete supersedes
  the request after native allocation but before report ACK, with no later
  container effect or volume deletion. Final corrected root gates pass: 174 Rust tests /
  50 groups / zero failed or ignored; fmt, workspace Clippy and dependency
  boundaries; generated comparison/client build; 67 live RPC checks per SQLite
  and PostgreSQL; explicit local OpenSSH suite and all-feature Clippy; RPC-client
  format/lint/typecheck; docs catalog and diff checks. The exact independent
  reviewer confirmed the capacity P2 resolved in targeted re-review, with no
  remaining findings. Hook-enabled commit and phase PR are next. No actual Engine,
  native Mac/Desktop or remote Engine security-mode acceptance is established;
  those remain separately required phase 4 evidence, not synthetic-test claims.
  PR #49/#50 remain unmerged. No merge/deploy authorization. Finish the planned
  stack before checking CI results.
