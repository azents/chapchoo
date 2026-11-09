---
title: "Codex Conversation Phase 5: Actual Conversation UI"
created: 2026-11-09
tags: [agent, conversation, frontend, implementation]
---

# Phase Execution Plan

- Phase: 5/6, actual conversation UI and minimal assembled acceptance.
- Branch/base: `feature/chat-text-5-ui` → `feature/chat-text-4-auth` (`1efa5a8eca7181bffc89518fac629c2e6efddbb1`).
- PR boundary: private owner SSR/hydrated conversation reads, generated-client adapters, exact text/answer/Stop mutations and real activity/request presentation, replacing production samples.
- Inputs: phases 1–4 (#97–100); generated Conversations owner service, protected request sessions, existing Agent layout and Computer controls.
- Deliverables: complete normalized text and activity, recent/older navigation, coalesced visible-region rereads and reconnect reread, exact operation result handling, inline native permission/form answers and retained original answers.
- Non-goals: native replay, automatic effect retries, durable browser feed/cursor, raw capture exposure, new model credential surfaces, attachment/rich-card capabilities, Team Agents, authentication-dependent model acceptance, merge or deployment.
- Interfaces: generated `Conversations` RPC messages remain wire authority; request-owned session/cookie response remains web authority; bounded piece reads/uploads preserve complete values and original source/operation target.
- Approved Design mechanisms: M01, M05, M10–12, M15–16, M18–19, M23–24, M26.
- Authority references: [Requirements r24/REQ-1–18](../requirements/chat-261108-text-conversation.md), [ADR D1–D33](../adr/chat-261108-text-conversation.md), [Design r9/M01–M26](../design/chat-261108-text-conversation.md), existing web rendering and private Agent contracts.
- Design delta: None.
- Removal obligations: local-only message append, simulated work/Stop, production sample conversation/plan/tools, obsolete sample-only Chat types and assertions, and Agent-only Chat initial loading.
- Absence verification: production render/container searches and executable component assertions; samples remain only typed static story/test fixtures. Preserve Computer power actions and their confirmation dialog.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Typed boundary | Primary agent `/root` | `typescript/packages/web-core/src/conversation*`, package export and tests | generated client | complete source-bound reads and exact effect adapters | executable Node tests, both-engine RPC evidence |
| Request ownership and SSR | Primary agent `/root` | web shared server conversation router, Chat server loader and route | typed boundary, existing session owner | server-rendered initial region, short request-owned continuation | server/route tests and build |
| Live conversation | Primary agent `/root` | web `features/computers/chat/**` | SSR, typed owner operations | readable utterances/activity/requests, source-replacement clearing, read-only recovery | component tests, typed stories, app tests |
| Assembled acceptance | Primary agent `/root` | E2E scenario and `testenv/chapchoo` protocol-fixture support | real backend/browser and execution owner | short text/streaming/request/navigation journey | minimal E2E plus lower-layer cases |
| Current contracts | Primary agent `/root` | relevant current Spec pages and this plan | stable implementation | accurate UI and read ownership claims | documentation validation, link and drift checks |

- Integration order: typed generated-client adapter → SSR/router → client read owner and effect state → renderer/requests → component and minimal assembled checks → Spec/checkpoint → commit hooks and PR.
- Independent review: `/root/chat-implementation-reviewer`, read-only; root requests only after all six phase PRs exist under the requester's explicit timing instruction. No new reviewer or implementation delegation.
- Final validation: root runs affected format/lint/typecheck, package/app tests, executable components, build, minimal E2E and mandatory hooks. Report actual commands/results and missing prerequisites; fixtures do not establish real model success.
- Scope-drift check: preserve exact input bytes, complete original choices/answers, single responder authority, separate current-work cancellation, source-bound uncertainty and explicit Computer power. No local UI success claims, hidden replays or new design mechanisms.
- Context checkpoint: phase 4 PR #100 submitted with full Rust/RPC/hooks passing; phase 5 begins from its clean committed tip. Real credentials/model calls remain unused. All remote CI/review inspection waits for the complete stack.

## Integration Checkpoint

The generated-client adapter, actual SSR loader, request-owned web router, live
read owner and production renderer/mutation wiring are integrated work in
progress. Production sample data and local-only success actions have been
removed. Historical answer discovery now binds the retained question occurrence
to the exact source without granting live response authority.

Retained input discovery and complete byte reads now bind the current source,
including the initial not-started boundary. Result rereads preserve uncertainty
without resubmission. The latest plan has its own bounded read even when its
record falls outside the recent page. Stop results retain the original source
and operation identity. Historical answer lookup is a read-only discovery path.

Server admission and native response validation share offline JSON-Schema string
constraints with bounded regular-expression compilation/backtracking. Known
email, URI, date and date-time formats are checked; unknown formats remain text.
Ordinary password-format collection is rejected. Browser feedback uses standard
format validators, while server/native validation remains authoritative. Exact
i64 values and arbitrary field names preserve their original typed meaning.

Completed local evidence:

- Full `scripts/check-rust.sh`: format, all-target Clippy, documentation, mandatory
  SQLite/PostgreSQL tests and dependency boundaries; 392 passed, zero failed or
  ignored. Later owner-read wire assertions passed in the affected component suite.
- `scripts/check-rpc.sh`: generated synchronization and actual Node/Rust RPC,
  129 checks on each SQLite and PostgreSQL.
- Web-core unit tests: 131 passed; General-web unit tests: 27 passed.
- Affected TypeScript lint/typecheck and General-web production build passed.
- Latest Storybook build and Chat executable component checks passed (24 tests),
  including expanded responses, original choices, exact integers, read-only
  history, mobile overflow and existing Computer confirmation/focus behavior.

The Runtime required its supported SWC native-cache override under a private
user-owned cache directory; no cache validation or filesystem protection was
relaxed. These results do not establish authenticated model success.

The deterministic existing-CLI app-server fixture and explicit Docker-browser
Assembly are integrated. Real Docker prerequisite creation, Connector admission
and individual product cleanup passed. Official-adapter protocol probing passed
initialize, synthetic login, session, text insertion, inline form/typed answer and
two assistant identities. The complete minimal native browser journey then passed,
including same-message updates, distinct assistant identities, typed native
answers, online reread, reentry and owned cleanup. The [acceptance note](../notes/chat-261108-browser-acceptance.md)
records its exact evidence and limits. Decimal request hints corrected bigint
query-key hashing without reducing integer precision.

Schema/choice descriptions and adjacent constraint help, exact-native-ID duplicate
echo association, obsolete sample CSS removal, current Wiki updates and
General/Admin shared-consumer type/build checks have been addressed. The latest
web-core suite has 131 passing tests; the Chat/ConversationBody/Supply subset has 24.
Conversation invalid-session responses now join the shared identity-withdrawal
boundary. The Docker browser lane is separate from ordinary browser scenarios
and from browser-free native lifecycle integration.

The actual backend/browser execution-owned fixture journey, hydration/reconnect
observations and individual cleanup passed, including browser-selected supply.
REQ-15 owner supply selection is integrated: the native owner freezes the choice
at physical start and reports it through the exact-source observation. Additive
paired migration 15 retains the explicit observed fact in both engines without
inferring historical values. A retained capture may span physical process starts;
later ordered native facts can describe the next start. Exact existing paths,
managed default, running-selection immutability, source replacement and owner
admission are covered by the owning tests and full Rust/RPC gates.

Owner setup separates the saved choice from the observed actual-applied choice,
fences saves by retained request identity and keeps uncertain saves read-only.
Reading or saving does not start, stop, refresh credentials or restart anything.
Unobserved applied supply stays unknown. A failed follow-up read cannot withdraw
an acknowledged supply save, input, native answer or Stop result. Unknown effect
outcomes remain uncertain without automatic retry or renewed responder authority.

Before submission, finish final affected frontend gates and mandatory hooks.
General provider image-label handling remains in Phase 6 runtime acceptance,
without relaxing ownership or security checks. No production test mode or
security-profile relaxation is authorized. These changes complete M05/REQ-15
within approved authority; Design delta remains None.

## Complete-stack correction scope

All six PRs exist; remote checks and the assigned complete-stack review are now
active. Phases 2–4 corrections are inherited. The primary agent directly owns
these approved-contract corrections:

- Reconcile matching local uncertain input with recovered retained input. Clear
  its uncertainty guard so an intentional later operation may submit identical
  text; never replay the recovered operation.
- Keep Stop readiness independent of text upload/sending and unrelated reader
  activity. Preserve exact observed work/source identity, current owner admission
  and final native-owner validation.
- Delegate native regular-expression pattern validation to the existing bounded
  server/native authority rather than running an unbounded browser RegExp. A
  known definitive validation rejection permits manual correction, not automatic
  mutation replay or reuse of an uncertain responder.
- Prefer retained answer facts over obsolete local uncertain warnings/recovery
  controls, and render text plans from their typed text field.
- Decode and render independent typed tool input/output, exact preparation
  freshness/versions and current saved-versus-physically-applied authentication
  facts. Unknown historical observations stay unknown; no secret bytes or
  credential-source fingerprint enters UI.
- Add focused web-core/component regressions and static meaningful stories.
  Run affected package/app checks, full frontend gates and mandatory hooks,
  then propagate the correction to phase 6.

Continuous append-between-chunks liveness remains an independent deterministic
investigation. Preserve complete-content and source fences; no new durable feed,
versioning or storage authority is approved. Reuse the same sole reviewer only
after integration stabilizes. Real authenticated model acceptance, merge and
deployment remain outside this correction. Design delta: None.

### Correction checkpoint

The corrected UI and source-qualified observation decoder are integrated.
Recovered retained input removes only matching local Request identities; a
different unresolved operation with identical text remains uncertain. Stop
retains its exact observed work target independently of sending and general
reader errors. Definite validation errors remain manually editable, while
unknown response delivery never reopens a responder. Tool I/O, text plans and
retained answers use their own typed fields. Setup presents exact preparation
and saved/applied authentication comparison without credential material.

Verified integrated evidence:

- Full Rust quality gate: 393 passed, zero failed/ignored across 66 result blocks.
- Actual Node/Rust RPC: 129 checks on each SQLite and PostgreSQL.
- Web-core: 135 tests; General-web: 27; Admin-web: 20, all passing.
- All-package typecheck: 14 successful tasks. General/Admin production builds
  passed after the final Stop rendering correction (unchanged tasks used their
  content-addressed cache).
- Storybook built and the Chat/ConversationBody/Supply component subset passed
  32 tests, including independent Stop during sending/read failure, editable
  definite validation rejection, retained-answer recovery, tool-only exact
  input/output with mobile overflow checks, preparation and authentication.
- The existing single Docker/browser journey uses the newly scoped applied
  observation region, not an ambiguous count of all unknown diagnostic fields.
  Latest assembled execution remains phase-6 validation, not an implied pass.

The first component invocation used the wrong existing textbox accessible name;
the locator was corrected and the complete affected subset rerun. The first unit
invocation omitted this Runtime's supported private SWC cache override; supplying
that environment fixed the prerequisite without relaxing cache validation.
Mandatory commit hooks, latest-SHA remote CI and targeted single-reviewer
re-review remain delivery gates.

The independent large-body probe ran the actual web-core consumer against a
deterministic exact-ordinal storage port: twenty forced appends between its first
and second chunks prevented complete reads of a 103,800-byte body; a quiescent
read then completed. This is a reproduced scheduling/liveness boundary, not a
native-frequency measurement or an actual-database stress run. Existing DB
tests separately verify the changing-field fence. No new read/storage authority
or safety relaxation was introduced as a workaround.
