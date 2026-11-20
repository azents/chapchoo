---
title: "Conversation Model Selection Design"
created: 2026-11-20
updated: 2026-11-21
implemented: 2026-11-21
tags: [agent, conversation, harness]
document_role: primary
document_type: design
snapshot_id: model-261120
---

# Conversation Model Selection Design

- Snapshot: `model-261120`
- Document: `model-261120/DESIGN`
- Requirements: [model-261120/REQ](../requirements/model-261120-agent-model-preference.md)
- Accepted decisions: [model-261120/ADR](../adr/model-261120-agent-model-preference.md)

Chapchoo requests the selected model for new turns of ordinary conversation
and work continuation. The harness owns actual turn execution; native
commands keep their existing behavior.

## Current behavior and gap

Connection-time preparation can create an empty conversation and prepare its
native session without starting a turn. A queued input does not prove readiness
or forwarding. An already-live owner can reuse its session; a replacement can
resume the exact retained source. [Conversation storage](../spec/domain/conversation-storage.md)
owns those guarantees and [native harness](../spec/domain/native-harness.md)
owns the release and operation boundaries. Neither currently persists an owner
model choice or orders its save against a new native start. The existing web
projection distinguishes accepted input, native work and delivery uncertainty;
it does not identify a model-setting failure.

## Architecture and state ownership

### M1. Prepared session and durable preference

The server is the sole preference authority. Extend conversation persistence
with a source-scoped optional explicit choice, containing the exact harness
model ID and a bounded display name for the saved choice. Absence is not an
explicit setting and generates no harness configuration call. Do not seed rows
from existing sessions, capture, or startup observations.

Identify the scope by private Agent/conversation, retained native target
(capture snapshot and native session), and continuous applied harness/account
binding. The physical launch, Connector route and Computer execution qualify
observations and operations, not preference lifetime. A retained source is not
proof of the applied binding. Compare authenticated, non-secret application
provenance from actual start with the source and harness identity. A saved
authentication choice waiting to take effect, an alias change, a token refresh
for the verified same account, or transport reconnect alone does not change
the live binding. A replacement API key can retain an authentication record ID
without verified account continuity: require proven reuse of the connection or
credential-source continuity for that case. Reuse existing authority checks
without transferring key material or provider account identifiers to the
model feature or web client.

When the exact retained target and binding are proved again, the next new
execution reuses the choice. An actual new target or an applied different
binding does not inherit it or revive it on a later switch back. Until identity
is resolved, do not apply a detached choice, show it as a current verified
model, treat uncertainty as a fresh session, or create a replacement session.
If an already modified live owner cannot prove safe continuity or a fresh
harness-chosen context, fail closed for model-governed start rather than
dispatching a stale override. This feature does not add a session-creation,
restart, or authentication operation.

### M2. Supported observation and owner operations

The native owner observes the initial concrete model and selectable options
from strict-v2 setup and subsequent session config notifications, applying
whole-set updates and any config responses in receive order. Keep the
configuration option identifier, model identifier and label distinct. Qualify
option snapshots by exact source, launch and applied binding; discard older or
mismatched observations. Do not interpret a missing, stale, unreadable or
unsupported observation as an empty list or authoritative unavailability.
Export only the model projection, not raw config options, auth material,
reasoning settings, or provider metadata. Capture and transcript behavior
remain owned by the existing native capture path.

Expose a typed model projection through the owner Conversations contract and
an explicit save and exact save-result read under the existing owner/Team
boundary. The read is effect-free; its view distinguishes not prepared,
known model, supported choices, empty choices, and unavailable observation.
When the current scope has a saved choice it is the configured value shown
to the owner, not a claim about the running turn. Without a saved choice show
the observed model when known. A read failure never overwrites a known saved
choice or manufactures a model name.

Save carries the exact prepared scope, a concrete advertised ID, and the
existing request-identity pattern for exact result recovery. Under the owner
transaction, validate current membership, target and applied binding, verified
model capability, and current admissible choice. Compare the prior accepted
save identity (or absence) so a stale picker cannot overwrite a later save.
An already accepted identical request returns its receipt without writing;
changed payload conflicts. A historical result is not the current preference.
Save admission is an atomic persistence operation and does not mutate native
configuration. On failure before commit leave state unchanged. An uncertain
response can recover the precise result by read; a missing result is not proof
the original save stopped. Reuse the existing receipt pattern as operation
identity, not a model revision history, public version field, or restore mode.

### M3. One server reservation from model selection to accepted submission

Serialize saves and starts using the existing owning Computer transaction
boundary in server persistence. For each model-capable scope, admit at most
one unsettled governed start using a short transaction that also reads the
latest committed choice, including absence before a first-ever save. Bind the
reservation to exact operation, native target, applied binding, launch and
authorized route. The server-issued grant adds no permission to forward an
input or continuation that failed existing admission. It cannot be borrowed
by a different operation or replayed after settlement. The native actor
retains one publication attempt for that authorization under its existing
bounded operation ownership. A local timeout does not reset that record.

| Relative order | Consequence |
| --- | --- |
| Save commits before admission | The new start receives the newest saved choice. |
| Start reserves first, not yet settled | A competing save does not commit. A confirmed non-save preserves the draft. The awaiting start still uses the choice it reserved. |
| Exact prompt insertion is acknowledged | A save can complete without waiting for the active execution to finish. No change is sent to that execution. |
| Exact pre-publication abort or proof of no future start | Settle the reservation without replay. A later explicit save may proceed. |
| Start/publication unresolved | Keep the durable reservation, prevent overlapping start and save confirmation for that scope, and retain truthful uncertainty. |

No transaction or synchronous route/observation lock spans native I/O. The
transaction orders acknowledgements against native outcomes, not against a wall
clock, optimistic readiness, general foreground state or request enqueue.
Known non-save describes the save transaction, not whether the competing
native request started.

### M4. Apply on the two native new-start paths

Keep both governed paths in the existing single native owner. The owner
applies the reserved model through the supported session config operation,
with an exact matching response for a selected ID. Apply only on the new-start
path while the exact session is idle; never on browser save, Setup, a live
steer, or an active execution restored to the same owner. An explicit choice is
reapplied after a physical owner change before its first eligible new prompt.
Without a choice, do not override the harness. Never substitute another model
on rejection or uncertain configuration. Revalidate the route, authorization,
source, binding and path-specific local predicate after awaited operations and
immediately before native enqueue. No other writer may change session model
configuration between the matching acknowledgement and that publication.

1. **Ordinary input.** Preserve `_session/steering` with
   `idleBehavior=promptRequired`. An injected input joins existing work with
   no model configuration or start reservation. Only
   `promptRequired/noRunningTurn` permits the new-start branch. If a qualified
   ending is still undecided, preserve the current deferral and grant
   revalidation without reserving against that pending ending. If it becomes
   ineligible after reservation, retire the unused publication locally before
   releasing it. Unknown steering or prompt delivery never enters this path
   again as a retry. On the idle fallback, distinguish a verified
   harness-native command from ordinary input using the vetted adapter's
   command contract. For Codex, match the supported built-in command names
   and token boundary; do not treat all leading `/` text as a command.
   Verify whitespace, case, arguments and ordinary path text against the
   released adapter. A stale suggestion list or a partial list does not
   qualify an unverified release. Forward a recognized command verbatim
   through the existing path, without applying the preference on that
   command. A command receipt cannot settle a different conversation start
   or bypass a current forwarding fence.
2. **Continuation.** Keep the one-use, system-originated handoff from the
   exact ordinary ending with the existing goal and input ordering checks.
   The owner qualifies the exact live Idle ending and route before obtaining
   the new-start reservation, then rechecks both at enqueue. The existing
   combined enqueue and ending-consumption boundary consumes once only;
   configuring the model is not consumption. Close is an order release,
   not a native result or a replacement grant.

Both branches settle the reservation on the exact insertion acknowledged by
their `session/prompt` request on a qualified release. The vetted Codex
plain-prompt path sends `turn/start` with a concrete configured model and
acknowledges insertion with the native message ID. The continuation marker
uses that same path without a user Chat receipt. The acknowledgement may
refer to a prompt adopted by an existing turn; it asserts neither a new
turn nor a model used. It does settle the one-use submission ordering
boundary. Configuration acknowledgement, SDK enqueue, generic foreground,
and ambiguous prompt errors do not settle it. Qualification tests inspect
the actual outgoing model on an idle prompt and confirm exact insertion
without claiming turn identity or model use.

### M5. Settlement, errors and recovery

Before enqueue, only an exact owner decision that makes later publication
impossible can settle non-publication. Serialize that decision with the
operation's enqueue guard; retire the local permission even if the settlement
acknowledgement is lost. A reservation response lost before local acceptance
does not grant a second send. Recover the exact recorded outcome or retire the
exact unused authority through evidence. After possible enqueue, a released
typed terminal or exact physical retirement can rule out a later start.
Retirement must prove the exact owner and pending prompt cannot send later;
it says nothing about whether work ran earlier. Link loss, lease expiry,
source replacement or a new owner by itself is not that proof.

Settle exact reports once. A current authenticated route can forward an
already retained exact result under existing operation/source fences; it
cannot reconstruct a lost result or grant another effect. A stale route or
different launch cannot close a reservation by assertion. Retain unsettled
state across restarts, source-local operation cleanup and deletion requests
until appropriate retirement proof or existing terminal ownership cleanup.
Neither save rejection, settlement nor protecting the scope causes a
restart, resend, or authorization to start a different session.

Keep two independent facts for dedicated recovery: exact model-specific
unavailability and affected input or continuation confirmed not started. A
supported producer can supply both; the first tested Codex integration does
not promise either from a missing option or generic config rejection. A proven
local pre-publication failure can use existing failure handling without
implying model removal. A provider `model_not_found` after prompt dispatch is
not a non-start fact. If evidence is incomplete, use existing truthful failure
or uncertainty and preserve the saved choice. If both facts qualify, expose
same-conversation failure tagged with the exact affected model and operation,
separate from a Chat receipt or synthesized history. An older report cannot
overwrite a newer preference or replace another conversation's feedback.

### M6. Web controls and shared ownership

Use the private conversation header in
[Chat](../../../typescript/apps/web/src/features/computers/chat) for the
desktop `…` and mobile character/status entry into one conversation detail
panel. Preserve the Agent detail link. **Model** opens a concrete single-select
picker there only when the model and scope are known with supported options.
Give loading/error and successful empty their distinct states. The shared
[UI package](../../../typescript/packages/ui) owns the divided full-row
single-select list, keyboard operation, accessibility semantics and single
trailing square check. Chat owns names, saved/current display, next-start copy,
draft, save and recovery. The existing localized private web surface owns
display strings and existing authorization/POST handling.

Opening the ordinary picker marks the current choice if offered. Click or
keyboard selects a draft, not effective state; cancel/close returns to detail
and discards it. Prevent duplicate submission/dismissal while a save is in
flight. Confirmed save updates the row using the server result and announces
that the setting is for new conversation turns. The picker does not expose
diagnostic notices or turn model history. A confirmed non-save keeps prior
choice and draft; an unknown response remains unknown until observational
recovery.

An actionable confirmed recovery opens the picker directly with no alternative
preselected, even if only one is offered. Saving changes only the setting;
cancel preserves the prior choice. Keep this feature out of composer, Agent
creation, per-answer UI, and Admin, which has no private conversation write
authority for this feature.

## Migration, rollout and operations

Add optional source facts, one explicit choice and the exact save/settlement
records in the existing conversation schema, with the repository's SQLite and
PostgreSQL transaction coverage. Preserve previous conversation bytes, queues
and authentication data. Do not derive choices from defaults or previous
metadata. Retire an old choice when its scope is proved replaced; retain only
approved exact request-result history under the existing ownership boundary.
Refuse destructive downgrade that would remove a saved choice or unresolved
ordering obligation instead of losing it silently.

Use additive typed Protobuf fields/operations and the standard generated RPC
clients. A scope can accept a first save only once its current native owner
supports model observation, setting and start settlement and both publication
paths participate. Mixed deployment must drain or observationally settle
legacy pending delivery before enabling model saves for that scope. A legacy
owner or unsupported harness may retain existing ungoverned sends on a scope
with no setting or outstanding model obligation; it cannot publish through a
bypass where a governed scope could be affected. Do not start unsupported
work under a saved choice, clear that choice for rollback, coerce versions,
or change the loader policy to simulate feature support. Do not introduce a
new operator mode. Promote a concrete capability only after the matching
release's behavior is proved; retain current first-send behavior when there
is no selection and no outstanding newer-path obligation.

Upgrade the schema and Control before replacing Connector binaries. An
upgraded Connector cannot forward governed work through a Control without
the model admission RPCs. Keep upgraded Control and schema for rollback
while saved preferences or unresolved reservations remain. An unsupported
native harness still reports actual connection provenance without advertising
choices, so a proved replacement can retire the obsolete choice.

Use existing safe structured logs and failure/uncertainty categories to
diagnose accepted save, known non-save, start admission, exact settlement and
unresolved start. Retain coarse harness capability and bounded correlation
in diagnostics. Do not log user input, key material, provider account
identifiers or raw model configuration. A long unresolved start is an
operational risk with an existing stop/retirement path, not an expiry rule
or an automatic repair.

## Test Strategy

Primary browser matrix: one core success journey through the real private web,
Client/Control, Docker Provider, Connector and official adapter. Prepare
before first message, open conversation detail, explicitly save B, send one
input, and assert that the actual provider-facing fixture sees B with the
same native session. Assert no provider request before Send. Use the
[existing conversation acceptance harness](../../../typescript/apps/e2e/README.md)
and testenv only for assembly, public-RPC seeds, owned disposable Docker,
stable native model fixture and ordered evidence. No credential snapshots
containing secret bytes: record release/build identities, capability, backend
and fixture type, and use synthetic credentials kept in memory. Do not
replicate the entire matrix in the browser or add an independent expensive CI
lane.

| Lower layer | Deterministic assertions |
| --- | --- |
| Shared UI and private Chat component | Desktop/mobile entries, Agent link, keyboard/list/check semantics, empty/error, exact save outcomes, failed draft retained, direct recovery, one available alternative not preselected, cancel, and no implicit retry. |
| Store and Control, SQLite and PostgreSQL | First save vs no-choice start; two saves and stale draft; exact-result recovery vs changed request; owner/Team withdrawal; unchanged vs moved target/binding; API-key source continuity; start admitted first vs save admitted first; unrelated session unaffected; stale lease, exact settlement and duplicate suppression. |
| Native actor and Connector | Gates before and after enqueue but before actual start; held turn joins with original model; fresh queued input and continuation use latest; native command pass-through and following ordinary input; current route and exact ending at publication; a lost config reply, start reply, close and settlement ACK; old successor, same owner reconnect, verified physical retirement, plain timeout and no duplicate prompt. |
| Released-program fixture | Same-process actual A→B; fresh-process exact resume with and without reapply; outgoing requested model observed at provider; exact insertion receipt; held active turn steering; native command pass-through; generic rejection during catalog outage; provider failure after dispatch. |

Use explicit synchronization and authoritative snapshots for race tests. Report
sanitized expected/actual choices, same-target assertions, save outcome,
prompt/start counts, and evidence ordering; identify substituted app-server
fixture separately from unmodified released-program evidence. Existing
required Docker/browser CI fails on missing prerequisites or test failures,
including incompatible fixture behavior. Optional live checks require explicit
authorization and an isolated disposable account; mark not run when it is
absent and never use them in place of deterministic acceptance. No feature
test or live-account run is claimed by this document.

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Direct new-prompt publication at the ordinary idle fallback and internal continuation boundary, for governed work | `model-261120/REQ-4`, `model-261120/ADR-D3` | M3/M4 governed publication; native commands, active steering and qualified no-feature paths remain | Native publication call-sites, Connector and exact grants | Call graph and fixture assert neither path can enqueue an unreserved governed start, including legacy or failed configuration cases. |
| No shipped model-setting contract, reset policy, component, or stored preference | No removal | New choice is additive; the draft reset proposal never became implementation authority | Repository baseline inspection; no reset migration, routes, config, UI or tests are introduced | Inspect generated/native surfaces and schema at implementation review; retain requirements coverage, not obsolete draft fixtures. |

## Design Authority

- Design revision: `3`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Server-owned explicit choice limited to the exact prepared native session and continuous applied binding; harness owns the initial model | `model-261120/REQ-1`, `REQ-3`; `model-261120/ADR-D1`; existing applied-authentication and retained-source contracts | `decided` |
| M2 | Scoped released-harness observations and private explicit save/result contract; saved choice is not live execution state | `model-261120/REQ-1`–`REQ-3`; `model-261120/ADR-D1`, `ADR-D2`; existing private owner/Team admission | `derived` |
| M3 | Short server transactions serialize scoped saves and the unsettled reservation for submission | `model-261120/REQ-3`, `REQ-4`; `model-261120/ADR-D3`, `ADR-D4` | `decided` |
| M4 | Request the model at the qualified idle target for ordinary conversation and work continuation | `model-261120/REQ-1`, `REQ-4`; `model-261120/ADR-D4`; existing native steering/ending and route authority | `derived` |
| M5 | Exact insertion or terminal settlement without replay; dual-evidence dedicated recovery | `model-261120/REQ-3`–`REQ-5`; `model-261120/ADR-D2`, `ADR-D4`; existing delivery and retirement fences | `derived` |
| M6 | Conversation detail and direct qualified recovery with a feature-owned picker, scope guidance and shared single-select list | `model-261120/REQ-1`, `REQ-2`, `REQ-3`, `REQ-4`, `REQ-5` | `required` |
| M7 | Additive adoption and rollback without bypass, inherited preference, or loss of pending ordering evidence | `model-261120/REQ-1`, `REQ-3`, `REQ-4`; `model-261120/ADR-D1`, `ADR-D3`; existing contract and supply ownership | `derived` |
| M8 | Preserve existing private Agent/Team, native capture, credential authority, and operation and route fences | `model-261120/REQ` fixed constraints; existing conversation, harness and credential Specs | `existing` |
| M9 | Keep released harness-native commands on existing verbatim forwarding, outside the model guarantee | `model-261120/REQ-4`; existing conversation input and native protocol authority | `required` |

## Authority and feasibility audit

| Requirement | Forward trace | Feasibility anchor and remaining evidence |
| --- | --- | --- |
| REQ-1 | M1, M2, M4, M6, M7 | Feasible for the vetted released path: existing preparation before Send and strict-v2 initial config inspection. First message does not acquire a preference. |
| REQ-2 | M2, M6, M8 | Feasible: current Chat and UI ownership plus generated Conversations contract. Validate component interactions and exact observation labels. |
| REQ-3 | M1, M2, M3, M5, M6, M7, M8 | Feasible with scoped server records: owner supply saves already demonstrate authenticated exact results in the Store. Released exact Resume plus reapply works. Verify different/replaced binding, process restart and scope-loss handling. |
| REQ-4 | M3, M4, M5, M6, M7, M8, M9 | Feasible for the approved ordinary and continuation scope on the vetted path. Both payloads reach the same model-bearing plain entrypoint. Exact concurrent ordering, command classification, and settlement remain mandatory implementation acceptance gates. |
| REQ-5 | M2, M5, M6, M8 | Feasible for a producer with both facts; first Codex integration retains ordinary failure/uncertainty when evidence is missing, as accepted in ADR-D2. Fixture must exercise both valid and misleading evidence. |

Reverse audit: the authority table accounts for all material mechanisms.
Store schema names, typed DTO/helper layout, list render implementation,
fixtures, and local diagnostics are implementation details, not separate
authority. No alternate preference store, UI override, catalog provenance
inference, default resolver, synthetic reset, external harness patch,
or new credential path is authorized. The only publication replacement
preserves existing route, ending, stop, receipt and uncertainty fences,
as recorded in Removal and Replacement. Use the owning Living Specs for the
implemented changes only after verification.

### Released evidence and scope boundary

With unmodified Codex 0.165.0 and codex-acp 2.8.0 against loopback Responses,
each isolated command case acknowledged configuration B with no dispatch.
Plain text used B at the provider; inline review, compaction, and goal work
used A. The compaction fixture did not produce a valid completion, but its
outgoing request identified A. Compact and goal acknowledged their own input
before Running, unlike the observed plain-text path. Codex has distinct
command RPCs, and review can have its own configured model. A successful
strict-v2 set or exact input acknowledgement cannot prove that all subsequent
work will use the selected model.

In a separate released loopback test, the exact existing continuation text
wrapper after a normal ending and subsequent slash-prefixed ordinary path
text both sent B after configuration. In both cases the native Running update
preceded the captured user-message update and the matching ACP insertion
acknowledgement. That verifies the payload and released adapter behavior, not
the Chapchoo continuation grant or feature integration.

The accepted `REQ-4` scope leaves native command operation and model authority
with the harness. Their prompt acknowledgement cannot settle a governed start;
no warmup prompt is used to change their model. Integrated save/start ordering
and capability compatibility remain implementation acceptance gates.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Approved on: `2026-11-21` (KST), following revision 2 approval on `2026-11-20`.
- Approved Design revision: `3`
- Approved authority IDs: `M1, M2, M3, M4, M5, M6, M7, M8, M9`
- Approved scope: implement the reviewed revision for ordinary conversation
  and Chapchoo continuation against unmodified Codex. Retain save ordering
  until input insertion without claiming actual turn model use. Keep
  conditional manual recovery, minimal picker copy, and native command behavior.
