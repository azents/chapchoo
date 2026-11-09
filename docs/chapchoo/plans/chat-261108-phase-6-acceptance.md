---
title: "Codex Conversation Phase 6: Runtime and Acceptance"
created: 2026-11-10
tags: [agent, conversation, runtime, verification, implementation]
---

# Phase Execution Plan

- Phase: 6/6, supported-profile integration and final acceptance evidence.
- Branch/base: `feature/chat-text-6-acceptance` → `feature/chat-text-5-ui` (`076e8ac8c11e3485a4d854157396affbce23983a`).
- PR boundary: correct the runtime integration finding, exercise the unchanged restricted profile with image metadata, consolidate implemented contracts and honest acceptance limits.
- Inputs: submitted phases 1–5 (#97–101), full Rust/RPC/frontend gates, model-free actual Docker/browser conversation evidence.
- Deliverables: benign image metadata does not impersonate ownership or prevent valid native starts; exact reserved ownership labels and all security checks remain enforced; representative Docker lifecycle and conversation checks; requirement/removal evidence and connected Specs.
- Non-goals: merge/deployment, production resources, credential extraction, actual model calls, ARM64/release ABI certification, new adoption paths, security relaxation or automatic operation replay.
- Interfaces: existing Docker/Bollard profile validation, reserved `chapchoo.io/` ownership labels, original resource identities and source/operation receipts remain authoritative. No new wire/schema or product setting is introduced.
- Approved Design mechanisms: M01–M26, especially M02–06, M10, M18–19 and M24–26 for integration and verification.
- Authority references: [Requirements r24/REQ-1–18](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md), [Design r9/M01–M26](../design/chat-261108-text-conversation.md), existing Docker ownership and distribution contracts.
- Design delta: None.
- Removal obligations: verify every Design removal row against its owning phase. Preserve plans and unimplemented snapshot markers while authenticated/native support acceptance remains incomplete; do not erase pending gates as cleanup.
- Absence verification: targeted production sample/simulated-success searches, old runtime/credential path inspection, source-fenced tests and final acceptance report.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Docker metadata | Primary `/root` | `rust/crates/docker/src/containers/profile.rs`, owning native fixture tests | exact provider labels/profile | harmless image labels coexist with strict reserved ownership | regression before/after, full Rust gate |
| Restricted execution | Primary `/root` | acceptance Dockerfile, `testenv/chapchoo` and existing integration/E2E lanes as needed | corrected native validation | actual metadata-bearing image starts without security changes | real Docker lifecycle and minimum browser journey; exact owned cleanup |
| Final contracts | Primary `/root` | relevant current Specs, acceptance note and phase plans | stable evidence and approved authority | bounded requirement/removal coverage and clear missing prerequisites | semantic/link lint, docs validation, mandatory hooks |
| Complete-stack follow-up | Primary `/root` | responsible phase branches only after PR 6 exists | all six PRs | current-SHA CI and one-reviewer findings resolved | remote CI, assigned review and affected reruns |

- Integration order: profile regression and correction → built metadata-bearing image → actual Docker lifecycle/browser checks → acceptance/Spec evidence → hooks/PR → complete-stack CI and review.
- Independent review: `/root/chat-implementation-reviewer`, read-only; request integrated phase reviews only after all six PRs exist, then apply valid findings directly.
- Final validation: primary owns focused and full Rust gates, generated RPC regression as affected, fixture static checks, actual Docker checks and document validation. Keep deterministic adapter success separate from actual authenticated Codex support.
- Scope-drift check: enforce exact reserved label values, original full IDs, PID1/UID/private namespace/read-only root/dropped caps/no-new-privileges/seccomp and original saved-volume retention. No metadata may authorize adoption, forwarding or replay.
- Context checkpoint: phase 5 submitted with all mandatory hooks passing. Real credentials/model calls remain zero. Public acceptance cannot be marked complete merely because local deterministic gates pass. Snapshot `implemented` and plan removal wait for their actual prerequisites.

## Integrated checkpoint

The ordinary-label regression failed before correction. The Docker suite now
passes 22 tests, including benign metadata acceptance and missing/changed/extra
reserved-label rejection before start. Exact volume labels and security checks
are unchanged. The first real lifecycle attempt failed the obsolete whole-volume
emptiness assertion; it is recorded as failed, with its exact leftover resources
separately retired after owner shutdown. The fixture now preserves the native
storage class while checking empty user storage and exact no-copy.

The corrected metadata-bearing actual Docker lifecycle passed in 96.5 seconds:
original data retention, successor/restart no-copy, normal deletion, original
volume identity and unrelated-canary protection. The same image passed the
native/browser journey, browser-selected supply, original typed answer retention,
reentry and individual cleanup. No Provider-owned container remained.
Testenv's six unit tests and affected static checks passed. The connected Spec
review distinguishes ordinary image metadata from reserved ownership, and user
storage from Connector-native storage. Document metadata and local links passed.

The [acceptance boundaries](../notes/chat-261108-implementation-acceptance.md)
retain requirement/removal evidence and explicit real-authentication, actual
Codex, architecture/ABI and complete-stack CI/review limits. Final commit hooks
and PR submission precede any remote CI or independent-review inspection.
