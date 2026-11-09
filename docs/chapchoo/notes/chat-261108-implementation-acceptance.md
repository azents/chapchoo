---
title: "Codex Conversation Implementation Acceptance Boundaries"
created: 2026-11-10
tags: [conversation, runtime, verification]
---

# Implementation Acceptance Boundaries

## Authority and delivery state

The implementation follows [Requirements r24/REQ-1–18](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md) and approved [Design r9/M01–M26](../design/chat-261108-text-conversation.md). The six delivery boundaries are foundation, native ownership, captured records/operations, authentication publication, actual web conversation and supported-profile integration. Design delta: None.

Implemented code, deterministic acceptance, authenticated native support, current-SHA remote CI, review, merge and deployment are distinct gates. This report does not mark Requirements or Design implemented. Feature plans remain until required acceptance and promotion are actually complete. No real credentials or model calls were used for the evidence below.

## Owned-contract evidence

These are bounded evidence groups, not claims that one passing test proves every requirement or supported platform.

| Requirements | Implemented contract and owning evidence | Remaining acceptance limit |
| --- | --- | --- |
| REQ-1 | Private owner/Team admission, one continuing conversation and execution/source fences in Store/Service/RPC; both database suites | Authenticated native context continuity must still be demonstrated |
| REQ-2 | Complete bounded text upload, retained operation identity, product acceptance distinct from native delivery; input-read and delivery tests | Deterministic browser insertion is not model acceptance |
| REQ-3 | Same-identity utterance updates, distinct identities and complete Markdown/body reads; projection, wire, component and actual Docker/browser checks | Real producer replacement/chunk behavior remains native acceptance |
| REQ-4 | Native steering attempts have explicit non-consumption-only fallback; cancellation/uncertainty tests forbid replay | Actual Codex steering/consumption boundaries remain unverified |
| REQ-5 | Exact current native work token and source-bound cancellation receipts, independent Computer power and termination evidence | Real model cancellation/retirement is not established by a fixture |
| REQ-6 | Retained activity/latest plan, bounded history and navigation to original records; owner-read and component tests | No load-capacity or performance-target certification |
| REQ-7–10 | Shared offline form validation, exact integers/omission/choice identity, single-use responder and duplicate/stale-answer fences; native/service/store/form/component tests | The fixture exercises one explicit form; real producer permission/form entitlement is separate |
| REQ-11–12 | SSR/reentry, current-source reconnect reread, resume failure without fresh-context fallback, source replacement and no uncertain mutation replay | Actual authenticated native resume and crash recovery remain unverified |
| REQ-13 | Responsive typed presentation, original choices/answers, focus and separate Computer confirmation; owning component checks | Tested desktop/mobile states are not exhaustive assistive-device certification |
| REQ-14 | Exact absolute existing CLI selection, compatibility/admission and no supply fallback; native preparation tests and deterministic existing-CLI execution | The synthetic executable is not actual Codex CLI installation certification |
| REQ-15 | Managed default, expected-selection save fencing and separately observed actual-start supply; both-engine persistence, component and browser checks | Later actual native process start must still be verified with real Codex |
| REQ-16 | Captured-copy catch-up, paired event/cursor publication, rebuilding/clear-refill and source isolation; both-engine projection tests | A rebuild cannot recover independent facts absent from its source |
| REQ-17–18 | Public OAuth SDK refresh, account/current-record fencing, file-only publication and deletion rejection; synthetic auth, native and both-engine tests | Real token issuance/rotation/expiry and live CLI file adoption remain unverified |

Phase 5 local evidence includes the full Rust quality gate (392 tests), generated/live Node–Rust RPC checks (129 per SQLite/PostgreSQL), web-core (131), General-web unit checks (27), affected Chat/ConversationBody/Supply executable component checks (24), General production/Storybook builds and Admin shared-consumer types/build. Mandatory hooks repeat the full Rust, RPC and workspace static checks rather than accepting these counts as a substitute.

The [deterministic browser report](chat-261108-browser-acceptance.md) describes actual Docker Provider, Connector, backend and production web listeners with the official adapter. Its app-server executable and authentication are synthetic. First-output observations include native preparation, not steady-state model latency. Successful individual cleanup is part of that run's result.

## Runtime integration finding

A metadata-bearing image exposed a Docker validation defect: Engine inherits image labels into the container, while validation compared the complete map with the Provider's ownership-only map. The focused regression failed before correction. The corrected boundary requires every exact expected `chapchoo.io/` label and rejects unknown reserved labels; unrelated OCI metadata supplies no ownership authority. Missing/changed/extra reserved-label variants remain rejected before start. Volume label validation remains unchanged.

The first actual lifecycle attempt reached the original and successor connections
but failed an obsolete whole-directory emptiness assertion after Restart. The
Connector legitimately creates its `.chapchoo-native` storage class even without
starting a model. The fixture now checks empty user storage separately, preserves
that native class and retains the exact no-copy assertion. It removes only its
own user marker. That failed run also reported unconfirmed normal product
cleanup; after its owned processes stopped, its exact original container and
volume were verified by full identity and labels and individually retired.
Later successful evidence must not be applied retroactively to this failed run.

The Docker crate suite passes with the new positive and negative cases. The fixture image deliberately carries an OCI title annotation for actual restricted-profile validation. All existing full-ID, bootstrap, original-volume and security checks remain in force; no profile weakening, automatic mutation retry or alternative adoption path was added.

The corrected real lifecycle run passed in 96,456 ms on Docker 29.8.2/API 1.56,
Linux amd64, with metadata-bearing image
`sha256:183df110a1833fd5517d40f0fef8d3dd67f8cd80f64ee83c3e7b20fd6236b1ea`.
It established original/successor/restarted full container identities, the same
original named volume, retained user marker, successor no-copy, empty user storage
after Restart, confirmed normal product cleanup and unchanged unrelated canaries.
Only the explicitly delegated Unix socket and existing bridge were used.

The same image then passed the representative native/browser conversation and
individual cleanup. Initial region reads were zero; input-to-first assistant was
53,928 ms including preparation, answer-to-update 2,360 ms, online reread 248 ms,
and reentry 832 ms. The journey preserved explicit original answers and the
saved-versus-observed-applied supply distinction. No Provider-owned container
remained after these successful runs.

## Removal and replacement evidence

- Production local append, simulated Stop/activity and sample transcript: the Chat route now loads the actual conversation; its container calls owner mutations and its renderer takes typed retained data. Static sample content is limited to stories/tests. Existing Computer power/dialog checks remain.
- Local-only Chat state/types and obsolete sample assertions: replaced with independent read/input/native answer/Stop states and retained record fixtures. A later read failure cannot withdraw an acknowledged effect; missing mutation responses stay uncertain.
- Control waiter as durability: product input/answer/Stop operations and captured event pairs are independently owned. Lost/cancelled waiters never authorize command replay.
- Proposed outgoing WAL and durable browser feed: no such persistence platform was introduced. Connector capture synchronization is incoming read/catch-up work; browser continuation coalesces current-region reads rather than maintaining a durable change cursor.
- Agent-only initial route: server entry passes the authorized initial conversation and hydration reuses it. The representative browser run observed zero duplicate region requests before input.
- Missing conversation wire contracts: source-generated owner/execution messages are exercised through live Node/Rust checks; hand-edited generated variants are not used.
- Superseded managed installation cache: preparation tests cover protected active/preparing/latest/shared artifacts. Cleanup remains inside the managed artifact cache and does not delete user workspace, native context or credential/source storage.

These checks do not authorize deleting existing account/Computer identities, original volumes, workspace or historical migrations as a feature reset.

## Explicitly unverified gates

- Actual Codex 0.165.0/adapter 2.6.0 authenticated new/resume/steering/request/cancellation and same-record token-file adoption with a specifically authorized test account/model.
- ARM64 and release ABI/artifact certification. The observed Linux x86-64 development executable in the Bookworm runtime emitted a weak GLIBC 2.39 warning; successful execution does not erase that limitation.
- Real SSH/Desktop/remote daemon security modes, encrypted component transport, production deployment or broader platform acceptance.
- Current-SHA remote CI and the single independent review until all six PRs exist; their results must be recorded separately before shipping readiness is claimed.

Missing authentication-dependent prerequisites do not block independent implementation, deterministic verification or PR submission. They also do not become implicit passes. No merge or deployment is authorized by this report.

## Connections

- [Conversation web](../spec/domain/conversation-web.md)
- [Captured conversation storage and acceptance](../spec/domain/conversation-storage.md)
- [Native harness lifetime](../spec/domain/native-harness.md)
- [Docker ownership and effects](../spec/domain/computer-docker.md)
- [General verification boundaries](../spec/domain/acceptance.md)
