---
title: "Codex Conversation Implementation Plan"
created: 2026-11-09
tags: [agent, conversation, codex, implementation]
---

# Codex Conversation Implementation Plan

## Approved Baseline

- [Requirements](../requirements/chat-261108-text-conversation.md): revision 24, REQ-1–18.
- [ADR](../adr/chat-261108-text-conversation.md): accepted D1–D33.
- [Design](../design/chat-261108-text-conversation.md): approved revision 9, exact M01–M26.
- Mode: Collaborative; product/material decision owner: requester.
- Implementation owner: primary agent `/root`.
- Exact independent reviewer for every integrated phase: `/root/chat-implementation-reviewer` (read-only, one-way opinions; only root requests reviews and makes corrections).
- Design delta: None.

The requester has authorized implementation within the approved baseline. New
product/security/material authority conflicts return to design; local helpers,
equivalent row/field layouts and fixture organization do not become interviews.
Merge, production deployment and use of real model credentials need their own
applicable authority. No installation secret or saved credential is copied into
fixtures or local commands.

## Delivery Stack

Each phase opens its PR before the next phase begins. Later branches depend on
the prior phase branch, not unmerged changes recreated on main. Do not add a
plan-only PR or activate partial user-visible success paths. The phase count is
an execution decomposition, not new product/design authority.

| Phase | Deliverable and boundary | Approved mechanisms | Interfaces / dependencies |
| --- | --- | --- | --- |
| 1 | Domain and preserving conversation/supply storage with typed source/operation boundary DTOs | M01, M05, M07–12, M18, M20, M22–24, M26 | Current owner/session/Agent authority; no native launch or public conversation service until integrated |
| 2 | Managed/existing supply, native owner, restricted environment and whole-scope retirement | M02–06, M17, M19, M24–25 | Phase 1 contracts; image/runtime prerequisites; deterministic native process fixture |
| 3 | Incoming JSONL capture, raw/normalized pairs, source catch-up/replacement, product acceptance and exact dispatch/control/read service | M01, M07–13, M18, M20, M22–23 | Phase 1 storage/contracts + Phase 2 native owner; no uncertain command replay |
| 4 | Public-SDK central refresh, eligible execution credential publication and file-only updates/deletion fences | M14–17, M21, M26 | Existing protected credential catalogue; Phase 2 file owner and Phase 3 exact execution admission |
| 5 | Actual SSR/hydrated conversation, bounded visible-region reads, composer/Stop/requests, generated adapters and minimal E2E | M01, M05, M10–12, M15–16, M18–19, M23–24, M26 | Phase 3/4 services; remove production samples and simulated success on activation |
| 6 | Supported-profile integration, final owned-contract acceptance, Spec promotion and removal/plan cleanup | M01–M26 | All preceding phases; actual evidence, final-SHA CI; external prerequisites reported separately |

PR titles use `Codex conversation [n/6]: <phase>`. Root owns all implementation,
integration and validation. The one reviewer examines each complete stable phase
diff against its phase contract and the approved baseline, not isolated fragments.

## Data, Interfaces and Removal Obligations

- Preserve existing User/Team/Agent/Computer/authentication identities and accepted
  creation fingerprints in SQLite and PostgreSQL. New-state downgrade refuses
  destructive data loss; schema readiness/preparation covers the new tables.
- Keep opaque product IDs, source snapshot/event occurrence, native entities and
  operation request identities distinct. Proto is the wire authority; regenerate
  Rust/Node output with project tooling.
- The event pair/cursor publication is atomic. Source replacement clears only the
  affected session conversation data, including independent input/answer/delivery
  facts; no hidden old-data archive, native replay repair or dispatch WAL.
- Phase 5 removes local-only Chat append, sample runtime evidence and simulated
  Stop only when their real replacement activates. Retain Computer power and its
  confirmation dialog. Update typed stories and owning component assertions.
- Phase 2/3 owns stale-source/pending-owner boundaries; Phase 4 owns deleted/stale
  credential rejection; Phase 6 verifies every Design removal row has evidence.
- Preserve native context, workspace and credential/source storage classes during
  inactive managed-cache collection. Do not create a general storage manager.

## Validation and Prerequisites

Root runs focused tests first, then the affected integrated gates on the stable
phase diff. Missing prerequisites are failures/unverified conditions, not pass
claims. Existing local tooling provides Rust 1.99.1 and disposable PostgreSQL;
use `scripts/run-rust-tests.py` / `scripts/check-rust.sh` rather than a shared DB.
Install locked pnpm packages when needed, not arbitrary manifests.

- Domain/Store: typed state and validation; both DB dialects; private owner/Team
  admission, initial/source replacement, operation conflict, pair/cursor atomicity,
  coherent reads and preserving migration/downgrade tests.
- Protocol: source-generated Rust/Node, bounded transfer and exact target/outcome
  mappings; `scripts/generate-rpc-clients.sh`, `scripts/check-rpc.sh`.
- Native: deterministic protocol/process fixtures with explicit synchronization;
  supported ABI/PID1/private namespace, detached descendants and actual retirement.
- Auth: synthetic secret markers and fixture SDK transport before live tokens;
  shared refresh, stale/deleted results, body-safe errors, native file adoption.
- UI: format/lint/typecheck/build and owning tests; one short assembled core flow
  at current E2E scope rather than a long error/restart/permission matrix.
- The requester additionally requires direct tests through the real Docker
  Provider in an isolated owned environment, starting when the runtime path is
  available. Do not modify existing production containers/volumes. Use deterministic
  protocol fixtures for process/reconnect/error combinations; real Codex model
  calls are kept to a short minimum core confirmation to conserve usage. Reuse
  the authorized login rather than repeating login/model calls unnecessarily.
- Supported live gates need explicitly provided test credential/account/model
  entitlement and isolated runtime. The requester has offered human Codex login
  when ready; authentication requests must not pause independent implementation,
  model-free Docker validation, review or CI. Keep only the authentication-dependent
  live checks pending until the requester can sign in. Do not request passwords/tokens
  in chat, extract ambient credentials or weaken security to make a test run.
  No CI fixture pass certifies a real model call.
- Each new PR SHA gets CI and review monitoring; valid findings are corrected by
  root. Resolve addressed automatic review threads, not arbitrary human threads.

## Checkpoints, Rollout and Cleanup

Each phase plan records branch/base, exact scope, changed contracts, tests and
remaining gates. Recheck both missing approved work and unauthorized additions at
phase boundaries. Required contracts/fixtures for the next phase may be prepared
while CI runs, but do not start that phase before the current PR exists.

Current-system Spec pages describe only activated implementation. The final
promotion covers Agent ownership, account/conversation storage, Control,
Connector/runtime, authentication, browser reads and acceptance evidence.
Requirements and Design receive one matching `implemented` date only after all
required implementation/validation is complete. Until then, approval is not
verified delivery. No automatic merge/deploy is implied. After verified promotion,
remove this plan and all feature phase plans; keep approved historical artifacts,
useful evidence and current Specs.
