---
title: "Conversation Work and Goal Continuation Design"
created: 2026-11-18
implemented: 2026-11-19
document_role: primary
document_type: design
snapshot_id: stride-261118
tags: [backend, frontend, engine]
---

# Conversation Work and Goal Continuation Design

- Snapshot: `stride-261118`
- Requirements: [stride-261118/REQ](../requirements/stride-261118-conversation-work-continuation.md)
- Decisions: [stride-261118/ADR](../adr/stride-261118-conversation-work-continuation.md)
- This is the approved implementation baseline. Product proof is required before release.

## Current system and gap

[Conversation storage](../spec/domain/conversation-storage.md) owns ordered Chat receipts, captured history, and ordinary authenticated input. [Native lifecycle](../spec/domain/native-harness.md) and [Connector operation](../spec/domain/connector-process.md) distinguish forwarding from delivery and live work evidence from restored metadata. Server foreground observation reduces Idle to a snapshot without the terminal reason or work target. [Server Tools](../spec/domain/server-tools.md) publishes Chat text with `send_message` under a fixed current native owner but has no Conversation Work state. Those boundaries remain distinct. None supplies a qualified ending event, an authoritative current goal, or a proven post-compaction restore barrier today.

## Design Authority

- Design revision: `18`

| ID | Material mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Authoritative server goal state in the conversation, with internal addition, amendment and explicit completion; `send` manages its associated goal | `stride-261118/REQ-1`, `REQ-3`, `ADR-D5`, `ADR-D9`, `ADR-D26`, `ADR-D29` | `decided` |
| M2 | Typed, authenticated Connector hook reporting and bounded request-response without domain-specific dispatch logic | `stride-261118/REQ-8`, `ADR-D1`, `ADR-D4`, `ADR-D23` | `decided` |
| M3 | Qualified native normal ending, including an otherwise qualifying `end_turn` after a compaction-hook stop; one committed admission or no-action decision, not a native forwarding acknowledgement | `stride-261118/REQ-2`, `REQ-5`, `REQ-8`, `ADR-D1`, `ADR-D2`, `ADR-D8`, `ADR-D23`, `ADR-D24` | `decided` |
| M4 | Independently deliver internal continuation without a user Chat receipt, preserving admission ordering, current-source/non-replay checks and the full goal-context projection | `stride-261118/REQ-2`, `REQ-5`, `ADR-D2`, `ADR-D3`, `ADR-D10`, `ADR-D24`; [conversation storage](../spec/domain/conversation-storage.md#conversation-identity-and-product-input) | `derived` |
| M5 | Current-source, bounded server goal lookup after compaction; apply the full goal-context projection on success before inference and preserve native progress without new context on failure | `stride-261118/REQ-4`, `ADR-D4`, `ADR-D5`, `ADR-D7`, `ADR-D10`, `ADR-D25`, `ADR-D26` | `decided` |
| M6 | One `send` tool under existing conversation authority; explicit actions, full ordered list replacement, and committed results without replay; commit associated effects together | `stride-261118/REQ-3`, `REQ-5`, `REQ-6`, `ADR-D6`, `ADR-D13`, `ADR-D14`, `ADR-D15`, `ADR-D16`, `ADR-D19`, `ADR-D20`, `ADR-D21`, `ADR-D22`, `ADR-D27`, `ADR-D29`; [Server Tools](../spec/domain/server-tools.md) | `decided` |
| M7 | Conversation Work wait clears on a genuinely new accepted input in that conversation; ordinary text and execution outcome do not settle work | `stride-261118/REQ-3`, `REQ-5`, `REQ-6` | `required` |
| M8 | Conversation Chat projects the approved single-surface summary and its shared motion/press interactions | `stride-261118/REQ-7` | `required` |
| M9 | Existing scoped Stop, ordinary user-input delivery, intentional Chat publication, and uncertainty handling | `stride-261118/REQ-5`, `REQ-6`; [Native lifecycle](../spec/domain/native-harness.md), [Conversation storage](../spec/domain/conversation-storage.md), [Server Tools](../spec/domain/server-tools.md) | `existing` |
| M10 | Conversation-wide limit of three automatic admissions between valid resets; charge once per committed follow-up and reset on new goal addition, freshly accepted explicit user input or a committed `send` task-list change | `stride-261118/REQ-1`, `REQ-5`, `ADR-D2`, `ADR-D3`, `ADR-D9`, `ADR-D11`, `ADR-D12`, `ADR-D13`, `ADR-D17`, `ADR-D18` | `decided` |

## Ownership and state

- **Goal foundation (M1, M3, M4, M5, M10):** The server owns canonical goal state and the shared admission counter in the conversation. Adding a goal does not replace another; amendments and completion affect only the relevant goal. The goal dispositions are active, waiting for input, and completed. A run outcome, todos, or rendered UI cannot complete a goal. No goal belongs to an execution. Native source/occurrence identities fence a transport effect, not the lifetime or membership of a goal.
- **Conversation Work (M6, M7, M8):** The work title, ordered todos and input wait are projected while Work is open. The associated goal's completion removes the summary and eligibility together. Ordinary user text can inform work but does not itself replace it.
- **Harness and Connector (M2, M3, M5, M9):** The live native owner supplies execution evidence. The Connector reports a supported typed occurrence using its current route. Ingress authenticates and validates, dispatches to a registered handler and returns that handler's result under a bounded awaited call. Shared ingress knows neither goals nor continuation policy. The handler does not wait for a later model execution. The harness owns when context must be applied to its next request.
- **Chat (M8, M9):** User and deliberately published Agent messages remain ordered Chat receipts. Native output remains capture history. Render the active summary above the composer as one glass surface with the straight expanded divider, shared spring and whole-surface light/dark tint. Existing summary appears without an entrance on load; subsequent appearance grows in place and exit overshoots outward then contracts. Content does not fade ahead of the surface; reduced motion omits the animations. Stop leaves work and its summary intact. No product UI is added to Admin.

## Normal ending, admission and dispatch

1. **Qualify a live ending (M2, M3, M9).** Require positive normal-ending evidence attributable to the current live native source and ending occurrence. An otherwise qualifying live `end_turn` after a compaction hook returned `continue: false` follows the ordinary rule; it does not require a further model request or a final assistant message. Stop, native execution failure, unknown or restored Idle, stale reports and mere cancellation publication are not such evidence. Do not relax the independent native-owner retirement conditions. Carry only identities and outcome evidence needed by a bounded report, not arbitrary actions or Chat payloads.
2. **Decide once (M3, M4, M9, M10).** At the server, authorize the current route and source, correlate the occurrence, and serialize the domain decision with goal changes, the shared counter and existing eligible user input. Commit one admitted internal input and one charge, or a no-action outcome without a charge. An already eligible user input wins. A subsequently accepted user message does not reorder or cancel an admitted continuation; existing prompt/steering behavior remains. A duplicate may recover the committed outcome but cannot admit again, including after the counter resets. Retain the committed outcome by source and qualified ending even after dispatch or ACK loss. No separate hook inbox or event-history service is selected.
3. **Deliver independently (M4, M9).** Wake the input owner after commit; caller timeout or a retry of the report is not permission for another native effect. Internal work has its own trusted authority and no user impersonation or Chat receipt. Check that authority and the exact source before crossing the forwarding grant. Deliver current authoritative goal context in the same conversation, not a stale transcript value. Use the existing ACP prompt envelope with an explicit system-originated continuation marker. Native records retain its observed user role; product authority and receipts treat it as internal. A possibly forwarded operation is not blindly resent. No later filter based on newly arrived user messages is introduced.
4. **Evaluate the next ending (M1, M3, M9).** Repeat only for a new positively qualified ending. Stop applies to the targeted current execution only, not a persistent pause, work settlement or removal. An ordinary later execution can qualify normally. A cancellation receipt, disconnect, or replay cannot create a successor by itself.

At ordered live ingress, bind the current root Running-to-`end_turn` transition to the retiring work token and captured occurrence before clearing its foreground target. A Stop that has won publication for that work excludes it. Reconnect may recover the same live occurrence; no capture replay synthesizes one. Connector reporting is a bounded typed request on existing Control component authority.

## Shared admission counter

Only an admitted automatic follow-up charges the conversation once, even when
multiple goals are unfinished (M10). Goal addition and reset commit together;
retrying that addition does not reset again. A freshly accepted explicit user
input in that conversation also resets the counter, regardless of remaining
capacity. Apply the reset in the new-input acceptance transaction, not on
partial upload, receipt recovery, native delivery result, or internal
continuation (M9). A qualifying Conversation Work progress change also
resets the counter via the internal server reset operation in the `send`
transaction (M6). Serialize the reset paths with the automatic admission
decision. A user input already eligible to lead wins; a later reset does not
cancel or reorder the earlier committed admission.

Moving from A to an already registered B, editing or settling a goal, Stop,
reconnect and in-turn compaction do not themselves replenish the counter.
Counts zero through two leave room for one new otherwise eligible admission.
At three, commit no action without changing any goal or blocking ordinary
input. Reset alone does not start work or reopen a prior ending. There is no
separate cumulative ceiling across authorized resets. Repeated actual list
changes, including status round trips, can renew the allowance. A distinct authorized domain addition qualifies, not a replay or a model-supplied identity alone.

## Goal context versus eligibility

For automatic follow-up and compaction restore, project the full current
authoritative conversation goal set and explicit states under the same
projection rule (M1, M4, M5). Include waiting goals; settled goals retain their outcome but leave the current set. The projection does not enumerate removed or historical goals. Bound the serialized full context to 8 KiB UTF-8 at goal writes with room for status changes, not by truncating on read. Read at the applicable authorized boundary; the two reads need not
return identical values when goals change in between.

For Conversation Work, the current goal entry includes its title, the full
ordered task list and each task's current status alongside the disposition.
Read the canonical Work row in the same projection; do not retain a second
task list in the goal record. An empty Work task list is explicit and does
not change the completion rule. Use the existing bound for the entire
serialized projection. Validate it in the same transaction as the Work
write, before any effects commit.

This is the first Work release; there is no deployed title-only Work
format to migrate. If a retained draft database already contains Work
that cannot fit, keep the original list and summary. Record no new
automatic admission when the complete context cannot be delivered.
Withdraw an already admitted but ungranted slot if it cannot produce
complete context, so normal input is not held behind it. Neither case
rewrites data or substitutes a partial or empty context.

Admission tests eligibility separately (M3, M7, M10). At least one goal must be
eligible to open one automatic follow-up. A waiting or terminal goal may be
visible in its context without authorizing work on that goal. If A is waiting
and B is eligible, B can justify the follow-up, and both are visible with their
states. If none is eligible, do not open a new execution; an already-running
execution can still restore the full set after compaction. Context injection
does not change dispositions or clear a wait. There is no goal selector,
per-goal execution or scheduler.

## Compaction restoration

For a source-bound context replacement that needs recovery, the harness makes a bounded request for current authoritative goal context or confirmed absence and applies a successful result before the next inference in the existing execution (M5). If the restore fails or times out and native execution stays active, it proceeds without newly restored context. An unsuccessful restore is distinct from confirmed absence; neither is a goal state transition. A late response must not enter a stopped or superseded invocation. Restoration itself does not create a prompt, Chat message, new turn, local goal copy, or native Goal runner. A follow-up can only come through ordinary admission after a qualified ending (M3).

Released-program probe: Codex `0.165.0` with `codex-acp 2.6.0` in ACP V2 awaited a trusted fixture `SessionStart(source=compact)` hook and included the newly read goal as developer context in the first post-compaction model request of the same turn. A successful lookup returning no goal continued without that context. The local goal endpoint and deterministic model endpoint tested the native ordering, not Chapchoo's trusted Connector route or live model behavior. An ACP compaction notification alone does not pause inference, and replay may have a similar notification shape.

On both releases, a native `SessionStart` timeout allowed that first request without the goal. A failed command, missing handler or missing executable also allowed inference in the tested cases. These outcomes are compatible with `ADR-D7`. An explicit `continue: false` on a lookup error prevented the next request and ACP reported `end_turn`; `ADR-D8` applies the ordinary admission rule to that live native ending, not to the restore result itself. A user cancellation during lookup prevented the request and reported `cancelled`; Stop remains excluded. `PostCompact` did not apply returned goal context, and its explicit stop reported `cancelled`. The trusted bridge and full supported compaction-path coverage are still unverified.

On released Codex `0.166.0` with ACP `2.6.0`, a bounded local probe
observed reasonless Idle on exact Resume and a separate Running-to-Idle
`end_turn` sequence after each fresh prompt. In that probe the latter
notification provided no turn/work ID. An ordinary ACP prompt carrying a
synthetic internal marker reached the model as native `user` input. These
observations support evaluating a live owner and local work occurrence, not
treating the raw terminal reason as an independent idempotency key or
claiming a developer-role follow-up.

The managed `SessionStart(source=compact)` hook on official, unmodified
Codex invokes a bounded, read-only Connector-local restore operation tied to
the applied source (`ADR-D28`). Accept the local invocation without proving
hook provenance. It carries no source selector or platform credentials.
The Connector reads under current component authority and revalidates the
exact live work and route before returning context. A local direct invocation
is permitted only within the endpoint's same applied-owner scope; stale owner
and stopped work are still excluded. No product Goal read API is introduced.

## `send` transitions and results

Use the existing scoped native tool authority and the same server transaction for all effects requested by one call (M6, M7, M9). The current tool scope, not a model-supplied target ID or text, governs the affected conversation. Require a work action on every call. The first `continue` without active Conversation Work requires a title and task list and starts the work. Later `continue` calls update the same work, require a task list, and may carry a new title. Either may carry a message. The initial call establishes tracked work and its common-goal effect coherently; it does not launch an extra native execution. `request_input` publishes a question and, when work is active, puts that work into input wait in the same transaction. With no active work it publishes only the question, without creating work, a goal, a summary, or a separate wait. `finish` publishes a result and completes the affected active work, if any. Neither a question nor a result sent without active work resets the counter. `ignore` completes the affected work without publication. Reject missing required input or a message supplied with `ignore` instead of changing the requested action. Neither completion cancels the current native execution or settles an unrelated goal. Invalid calls produce no effects.

The native `send` description directs the Agent to complete Work even when
it ends the Work because of failure or cancellation: `finish` for an
intentional message, or `ignore` for silence. This is explicit Work completion,
not an effect of Stop or a native execution error.

The `continue` request supplies an entire ordered task list; an empty list
clears the tasks without completing work. An omitted optional title retains
it, and an omitted optional message means no publication. Required content
must not be blank. `ignore` succeeds silently without effects when no work
is active. Do not reopen completed work; a later new request can start new
work through `continue`. Successful publication returns its message ID;
silent success has no message ID. There is no result lookup or replay key.

An unchanged valid task list satisfies `continue` and permits the
accompanying message to be published. Compare the resulting validated
ordered list with the immediately preceding committed list. Any difference
in membership, content, status or order resets the shared counter once
with the work update (M10). The same status or list can qualify again if
another state was committed in between. An unchanged list with a new
title or message, or a rejected call, does not reset. The server derives the
effect from the committed change, not an explicit model-supplied flag or
a high-water mark.

Normal user input uses the same composer and receipt path; a genuinely new successful same-conversation acceptance can satisfy an active input wait. Recovery of an old receipt, uncertain delivery and other conversations cannot. Preserve existing response semantics: a committed transaction is not proof its response arrived, and an uncertain result is not permission to repeat `send`. Do not infer settlement from result text or auto-publish the native turn output. Exact tool argument layout is local unless it changes an approved behavior; any new long-lived invocation result store or public retry contract requires a separate decision.

## Migration, rollout, recovery and operations

Existing conversations start without an inferred goal or work; existing message receipts and captured history remain intact. Component admission must reject unsupported or stale hook events without effects. Retain the current Connector, owner fences and physical retirement checks. Cut over model-facing discovery and dispatch to `send` together; verify the effective tool catalog on the actual native owner rather than assuming a schema edit reaches a resumed session. Keep the established catalog verification and refresh boundaries. The exact deployment order and rollback behavior for retained in-flight work must be verified against the existing owner refresh and trusted component admission; no parallel legacy tool or new compatibility mode is authorized. Verify catalogue, dispatch and instructions switch together after existing owner effect drain and before exact Resume. Enable only qualified runtime combinations. Deployment and merge require separate approval.

Diagnose outcomes using existing scoped, sanitized operational reporting. Evidence should distinguish invalid/current-source rejection, no action, admitted outcome, uncertain dispatch and unavailable restoration without logging raw goal or message text. No general goal tracing/query surface, new event bus, or credentials in fixtures.

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Model-facing `send_message` catalog entry and dispatch | `stride-261118/REQ-6`, `ADR-D6` | Scoped `send`; retained intentional Chat receipt path | Catalog and dispatch cut over together for current authenticated native owner | Verify new and exact resumed native catalog and addressed dispatch; `send` is the only tool for this feature |
| Old catalog-specific readiness expectation, fixtures, and product Spec claims of `send_message` | Same authority | Matching `send` checks; current Spec updated at implementation | When effective native catalog changes | Inspect catalog comparison, relevant fixtures, and connected Living Specs against the new behavior; preserve unrelated tool tests |

The normal foreground snapshot, old Chat receipts, native record projection, ordinary input queue contract, and Stop are retained. No prior persisted Conversation Work data was found to remove at the inspected base.

## Feasibility evidence and implementation gates

| Area | Evidence and status |
| --- | --- |
| M1, M6, M7, M8 | Existing authoritative conversation scope, transactional message store, scoped tools and Chat presentation provide plausible integration. **Conditional:** combined state transaction, wait race, native catalog and render are not implemented here. |
| M2, M3, M4, M9 | Current live termination evidence, current-source Stop and single-use forwarding grant are reusable. **Conditional:** server report of positive ending and atomic cross-input admission are new; test connector loss around commit and native effect. The Control request, source/occurrence deduplication and internal input origin are specified by ADR-D23 and ADR-D24. |
| M5 | **Conditional:** `SessionStart` success, missing handler, process failure, timeout, and explicit stop were verified on released runtimes using local deterministic endpoints. The observed behavior matches the accepted policy. The authenticated product bridge and enabled compaction-path matrix remain to be verified. |
| M10 | **Conditional:** the limit of three and task-list reset scope are fixed. Existing fresh input admission provides an authoritative write boundary distinct from receipt recovery and native delivery. Serialize goal addition, user acceptance, `send` task-list changes and automatic admission; prove atomic charge/reset and duplicate handling. The bound is only between valid resets. |
| Multiple goal context (M1, M4, M5, M7) | **Conditional:** unfiltered current-state projection is decided. Native fixture evidence must distinguish payload completeness from admission, and check transport size against the supported context/request limits. ADR-D26 selects the full write-time bound without truncation. |

Existing baseline evidence at the inspected base: 47 harness runtime and 13 SQLite conversation tests passed. These do not verify new behavior. If source qualification or restore coverage cannot meet the approved outcomes, bring the precise technical/product gap back for decision rather than quietly dropping the requirement.

The local released-runtime probe also observed the replacement fixture
catalogue in actual model requests after exact Resume and New. The fixture
did not execute the Chapchoo `send` tool or exercise its authority or
project-refresh transaction. A separate two-connection SQLite model
exercised outcome recovery, counter rollback, stale owner and input
precedence in ordered commits; it is not product schema, concurrent
PostgreSQL, Connector ACK or forwarding evidence.

## Test Strategy

- **Primary E2E:** extend the existing conversation Docker/browser lane with a deterministic native integration fixture. One representative journey covers the first `continue` creating a work summary, a later update keeping the same work, a qualified normal ending that continues without a user bubble, an explicit question with new same-chat input, and one `send` result that closes the summary. Verify stable composer placement and no initial entrance animation. Use browser-visible receipts plus bounded native observations, not wall-clock guesses.
- **Native boundary acceptance:** use a disposable supported Codex/ACP pair with a deterministic model endpoint that records ordered requests, using existing native/Testenv fixture support where possible. Verify successful restoration applies the latest goal before the first eligible request; confirmed absence and failed restoration are distinct and do not themselves start a turn. Include a late result, missing/failed hook, a native timeout that preserves current execution, and explicit hook stop followed by a qualifying `end_turn`. A synthetic mock can prove host handoff mechanics but cannot certify deployed native compaction behavior. No production keys or private accounts in tests.
- **Lower-layer matrix:** cover Stop/normal races and later normal ending, duplicate/replayed/stale hooks, server ACK loss before/after commit and no-replay after possible forwarding, user input before/after admission, wait clearing only from a new qualifying receipt, two-effect transaction rollback, stale/cross-conversation calls, and approval-consistent motion in reduced motion and light/dark. Use explicit failure points and synchronization. Exercise affected persistence invariants on SQLite and disposable PostgreSQL as supported by the existing matrix.
- **Shared-budget checks:** admit the first through third otherwise eligible distinct endings and record no action for the fourth. With A and B already present, neither finishing A nor following B refills the shared budget. Adding C resets once and does not wake an execution. Exercise third-slot admission racing a goal addition or a genuine user input in both commit orders. Verify that a newly accepted user input resets both a partly spent and an exhausted counter, and that a partial/rejected input, duplicate acceptance recovery after a later charge, replay of a pre-reset ending, and restart do not replenish it. Verify that renewed intervals do not accumulate a separate lifetime limit.
- **Full-context checks:** when A is waiting and B is eligible, admit one follow-up and carry both states. When all goals are waiting or terminal, admit none; compaction in an already-running turn still restores that current set. Verify amendment freshness, absence of settled or removed goals from the current projection or clearing of input wait.
- **`send` validation and reset checks:** reject a standalone message and continuation lacking a task update with no partial publication or state change. Accept an unchanged valid task list and publish an accompanying message without a reset. Verify that adding, removing, editing, reordering or changing task status resets once with the work change and any requested publication, including reversal and return across separate calls. Rejected calls and rollbacks cannot replenish the counter. Exercise both commit orders when the change races a normal-ending decision, and recover any already-decided ending without admitting another.
- **Completion checks:** `finish` publishes and settles active work together. A simple `finish` without active work publishes the result without creating a goal, work item or summary or resetting the allowance. `ignore` settles work without a Chat receipt. Reject `finish` without a result and `ignore` with a message without side effects. Verify neither completion settles another goal or signals native cancellation.
- **Start checks:** reject initial `continue` without a title or task list even if it includes a message. Verify a successful first call exposes one work item, then an update leaves its identity and any input wait intact. Verify an initial failure does not leave related goal or message effects behind.
- **Question checks:** `request_input` publishes and enters wait atomically for active work. Without active work, it publishes the question without creating work, goal, summary or wait, resetting the counter, or suspending unrelated work.
- **Fixture and CI:** use fictional conversation/goal data and disposable current-source authority. Record exact code SHA, applied adapter/native versions, scope and sanitized event/request ordering. Required fixture lanes fail if prerequisites or assertions are missing; separately opted-in credentialed live scenarios skip only with an explicit missing prerequisite and report coverage excluded. Do not report a skipped model case as passing verification. Testenv diagnostics are not a replacement for the core browser/native acceptance flow.

## Design Approval

- Mode: Collaborative.
- Decision owner: requester.
- Approved on: `2026-11-18` (KST).
- Approved Design revision: `18`.
- Approved authority IDs: `M1`, `M2`, `M3`, `M4`, `M5`, `M6`, `M7`, `M8`, `M9`, `M10`.
- Approved scope: the reviewed Revision 16 plus ADR-D23–D27 as Revision 17 and
  the subsequent ordered Work context, managed restore and completion
  clarifications below. The requester authorized implementation, not merge or deployment.
- On `2026-11-19` (KST), the requester confirmed the ordered Work task list
  and states are part of Goal context and directed implementation to
  continue. This clarifies `M1`, `M4`, `M5`, and `M6` under the existing
  full-context bound. The authority IDs, documented completion actions,
  and merge and deployment boundaries are unchanged.
- On `2026-11-19` (KST), the requester accepted `ADR-D28` for `M5`: use
  unmodified Codex, trust the Connector-local call without additional caller
  authentication or isolation, and proceed with implementation. Existing
  source scope and failure semantics still apply.
- On `2026-11-19` (KST), the requester corrected `REQ-3` and directed its
  implementation as `ADR-D29` in Revision 18 (`M1`, `M6`, `M7`); all other
  authority IDs and the no-merge/no-deployment boundary are unchanged.
