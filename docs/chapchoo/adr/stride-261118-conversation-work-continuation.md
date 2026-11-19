---
title: "Conversation Work and Goal Continuation Decisions"
created: 2026-11-18
document_role: primary
document_type: adr
snapshot_id: stride-261118
tags: [backend, frontend, engine]
---

# Conversation Work and Goal Continuation Decisions

- Snapshot: `stride-261118`
- Requirements: [stride-261118/REQ](../requirements/stride-261118-conversation-work-continuation.md)
- Decision mode: collaborative.
- Requirements confirmed on 2026-11-18 (KST), including the explicitly added
  Connector hook reporting scope.

## Decision Map

1. **Accepted — Connector hook reporting and first event** (`REQ-2`, `REQ-5`,
   `REQ-8`): report qualified normal endings over a reusable Connector-to-server
   hook event boundary. The server owns admission of any continuation.
2. **Accepted — Acknowledge the committed hook outcome** (`REQ-2`, `REQ-5`,
   `REQ-8`): commit the admission or no-action decision before responding,
   without a separate hook inbox or a wait for native dispatch.
3. **Accepted — Ordinary input and continuation contention** (`REQ-2`, `REQ-5`,
   `REQ-6`): decide order at hook admission without a later user-input filter.
   Preserve ordinary input delivery and replay rules.
4. **Accepted — Shared hook call contract** (`REQ-4`, `REQ-8`): await the
   handler's response within a deadline without waiting for downstream work.
   The shared hook system does not depend on continuation logic.
5. **Accepted — Goal authority and restore ordering** (`REQ-1`, `REQ-3`,
   `REQ-4`): keep one authoritative server goal and allow a lookup at the
   compaction boundary. Proving the native bridge and deciding the failure
   policy remain open; do not introduce another goal authority or auto-runner.
6. **Accepted — Commit `send` message and work effects together** (`REQ-3`,
   `REQ-6`): commit the effects requested by one call in one server
   transaction. Preserve the existing uncertain-result boundary.

### Fixed or derived constraints

- The five common goal responsibilities and their exclusions are unchanged.
  An identity used to qualify an ending must not associate a goal with an
  execution. Neither work completion nor the current goal comes from model
  final text, goal text in an old transcript, or presentation state.
- Stop does not change work state; only its execution's termination is excluded
  from continuation. Exact-target Stop authority and a fresh next execution
  remain distinct from the Computer's physical lifecycle.
- Ordinary input, conversation access, and no-blind-replay guarantees remain
  intact. Acceptance of an input does not establish completion of execution.
- The approved summary design and shared material/interaction remain fixed.
  Component structure and equivalent test fixture layout are local decisions.
- An arbitrary restored or replayed Idle, a timeout, a Stop acknowledgement,
  or old-owner observation without current authority is not positive evidence
  of a normal ending. Physical owner retirement has separate safety conditions.

## ADR-D1. Report lifecycle hook events through the Connector

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Use a reusable Connector-to-server reporting boundary for
supported lifecycle hook events. The first event is a positively qualified
normal execution ending from the live native owner. The Connector reports the
event; the server checks source authority and evaluates any continuation using
the current goal. The event is evidence, not a request from the producer to
run an arbitrary effect. Stop, failure, or a restored Idle cannot manufacture
this event. This is a post-ending hook, not a native `Stop` hook that requests
another inference in the same turn.

**Rationale.** A shared reporting boundary can carry future supported event
types without adding a Goal-only Connector command or promoting foreground
snapshots into execution-completion authority. Existing authenticated
Connector reports already identify the conversation, native source, and
captured occurrence. The native observation report is a position-ordered
foreground snapshot: it drops the reason and work target on Idle. It remains
distinct from the new occurrence-based hook event.

**Not decided by this ADR.** The wire service or method, report delivery and
recovery guarantee, storage model, and atomic admission details remain open.
Do not add general event history, progress checkpoints, execution-to-goal
links, or user-defined event handlers. A direct server handoff with one
committed outcome remains a candidate; retain a separate pre-admission fact
only if its recovery gap justifies it. Runtime evidence for the supported
native integration remains pending.

**Discriminating checks:** a live normal end; stopped execution including a
Stop race; subsequent normal execution; a duplicate signal; reconnect before
and after committing a normal ending; restored-only Idle; a stale or superseded
source; interruption after possible input forwarding. For the same boundary,
verify no more than one follow-up and no user-authored receipt for internal input.

## ADR-D2. Acknowledge the committed hook outcome

**Status: option A accepted by the requester on 2026-11-18 (KST).**

**Decision.** Validate the event and commit one continuation admission or a
no-action outcome before acknowledging. The acknowledgement is not evidence
of native input delivery. Native dispatch remains independently owned. An
unacknowledged report may recover the same committed outcome, not create more
work. Possible native forwarding never grants replay.

**Rationale and rejected alternative.** The normal-ending handler can be
designed around a bounded admission decision. Acknowledging durable receipt
first would require a separate hook inbox and its recovery, stale-event,
consumption, and backpressure rules. Do not add that inbox for this snapshot.
The exact rule for concurrent ordinary input is separate.

**Accepted responsibility clarification (2026-11-18 KST).** The shared hook
boundary authenticates the caller, validates the event envelope, dispatches
to the registered handler, and returns that handler's outcome. It does not
look up goals, check the input queue, enqueue continuation, or define a
continuation-specific result or waiting mode. The normal-ending handler is an
ordinary consumer; it owns goal eligibility, the committed admission or
no-action decision, and safe recovery of that outcome. This commit requirement
is specific to the handler's effect, not a requirement that all hook handlers
write to storage. The harness, not the shared hook boundary, governs when an
outcome must be applied to its next inference.

## ADR-D3. Fix input order at hook admission

**Status: option A accepted by the requester on 2026-11-18 (KST).**

**Decision.** A user input already eligible to lead the next execution at
hook admission takes precedence. After the hook admits a continuation, retain
that order: a later user message follows the existing prompt or steering
path. Do not recheck for new user input to cancel, filter, or reorder the
admitted continuation at dispatch. This does not remove the ordinary
authorization, current-source, or no-replay checks.

**Rationale and rejected alternative.** Giving a later user message
precedence up to the forwarding grant would require an extra atomic
contention check and cancellation branch for a narrow interval. The
requester chose the simpler stable order, accepting that the already-admitted
continuation can start first. This is not permission to start two automatic
executions for one ending.

Preserve the current goal and work state, user-message content and ordering,
and existing Chat receipt rules. No timer or implicit merge is added.

## ADR-D4. Await the shared hook handler's response

**Status: accepted by the requester on 2026-11-18 (KST).**

`ADR-D2` fixes when the server may acknowledge the normal-ending handler's
decision. It does not require the harness to wait for that response. Choose
the caller contract independently of any registered consumer's behavior.

- **Fire and forget.** The harness returns after local handoff, without a
  server processing result. An observation that does not depend on the
  result stays off the harness's critical path, even when the network or
  handler is slow. A background owner must account for bounded buffering,
  overflow, shutdown, and unsuccessful delivery. Local handoff alone cannot
  promise durable handling. A caller that must use the result before its
  next step needs a separate synchronization mechanism.
- **Fire and await response.** The harness waits for the registered handler's
  result or an explicit failure, within a deadline. This establishes ordering
  at the invocation site and lets that site act on the result; I/O can remain
  asynchronous. Network and handler latency are now on that call site's
  critical path, and it must handle timeout or cancellation. A response lost
  after server commit does not prove that the handler did nothing.

**Decision and rationale.** Use one bounded request-response contract. Await
the handler's outcome, not execution of later work it admitted. This gives
callers the result without a separate synchronization mechanism while keeping
business rules in event handlers. The wait applies at the invocation site,
not to all harness work or to the user input queue. It does not by itself
order work outside that site. Timeout or cancellation does not establish that
the handler did not commit its effect. Do not add a second mode, a
continuation-specific exception, or a durable event outbox without a separate
delivery need. The native restore call site's availability and failure policy
are not decided here.

## ADR-D5. Keep one current goal for continuation and restoration

**Status: option A accepted by the requester on 2026-11-18 (KST). Native
restore feasibility and failure policy remain open.**

The server admits continuation from a qualified ending, while a compaction
restore must supply context before the next inference that needs the goal.
Those are different boundaries. On the examined Codex release, a native active
goal can already start a separate continuation turn, but is not automatically
added to the next in-turn inference after compaction. A captured
`compaction_update` has no next-inference barrier and is not a restoration
mechanism. Ordinary user prompt or steering is not a silent, synchronous
replacement for that boundary.

- **A — One server-owned goal, with a read at the restoration boundary.** Goal
  commands and server admission use the authoritative conversation goal.
  A trusted, source-bound harness restore obtains the current eligible goal
  when it can actually provide context to the next inference. No independent
  native goal, continuous progress checkpoint, or general model-facing query
  is introduced. This requires the source and authoritative state to be
  available at the restore boundary.
- **B — A local goal projection for restoration.** Keep the server
  authoritative but also maintain a projection at the native owner. This can
  avoid a round trip when the projection is known current, but requires a
  coherence protocol for amendments, settlement, source replacement and
  disconnection. Without freshness evidence the projection cannot restore
  a goal safely; this is not unrestricted offline operation.

**Decision.** The server is the only authoritative goal store. A server round
trip before the next inference is allowed for compaction recovery; offline
restoration is not required. Do not maintain a second goal or a synchronized
local copy. The restore path obtains the current context (or confirmed absence),
and the native call site applies the result before the next model request.
Returning context is not proof it has been applied. The normal-ending
acknowledgement still confirms the admission decision, not native input
delivery (`ADR-D2`). A local projection was not selected because avoiding the
round trip was not required and a projection would need freshness coordination.

**Feasibility check:** Test the existing `SessionStart(source=compact)`
additional-context path first. The examined Codex source awaits that hook
before continuing the ordinary in-turn rollover, but that neither proves
Chapchoo has a usable trusted bridge nor covers all enabled context-replacement
paths. Validate the supported combinations against the actual next model
input. Include goal amendment, goal termination, successful and ineffective
replacement, replay, and loss of fresh authority. The acceptance condition is
current goal context without a new user message, second turn or restarted
goal. An unproven path may not silently substitute stale text or prompt the
model again to approximate restoration. Report missing coverage before
accepting any fallback behavior.

**Application to the shared hook boundary (proposed).** A restore handler
would return additional context or a confirmed absence of eligible goal
context. The native call site applies that result before its next inference.
This requires neither a continuation-specific response nor a waiting mode in
the shared hook API. Whether restoration can use the same Connector transport
is contingent on proving a usable native restore bridge and is not decided here.

**Timing check.** Asynchronous I/O is compatible with restoration only if the
native inference path awaits the hook and applies the returned context before
building the next request. A fire-and-forget compaction notification is not
sufficient. Distinguish a successful read with no eligible goal from a
lookup failure or timeout. The native hook returning without additional
context is not by itself proof that no goal exists. Validate delayed,
failed, and cancelled hook calls at the actual request boundary; do not
apply a late result to a stopped or superseded turn. The fault policy for an
unavailable authoritative state is not accepted by this proposal. This is
separate from the normal-ending event, whose acknowledgement waits for the
server admission decision but not native execution (`ADR-D2`).

## ADR-D6. Settle `send` message and work effects together

**Status: option A accepted by the requester on 2026-11-18 (KST).**

Today `send_message` publishes a scoped conversation receipt without modifying
work. The unified `send` can request a message and a work transition in one
call, such as publishing a result while ending work, or publishing a question
while beginning input wait. It can also request just a message or just a work
effect. The decision is the commit boundary when one call requests both.

- **A — One server transaction.** Validate the scopes and transition, and
  commit both requested effects together. On a known pre-commit failure,
  neither effect becomes visible. Other readers cannot see the intermediate
  state of that write. The message-only and silent operations commit only
  their requested effects.
- **B — Stage the effects separately.** Commit the receipt and work effect
  through independent settlement boundaries, with explicit recovery for one
  succeeding before the other. Independent settlement is useful when effects
  belong to different owners, but introduces partial states and recovery
  that must not duplicate messages or reopen work.

**Decision and rationale.** Commit the requested message and work effect in
one server transaction, or only the requested effect when the call supplies
one. The effects have the same authoritative owner; separate settlement would
add a partial-state recovery path without a distinct owner to justify it.
An atomic write does not guarantee that the caller receives the result:
preserve the existing distinction between known nonexecution and uncertain
outcome, and do not treat response loss as permission to send again. Catalog
cutover and the exact call and error shape are not settled by this decision.

## ADR-D7. Preserve the current execution when compaction restoration fails

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Keep the native execution running when the bounded
post-compaction restore cannot supply or apply current goal context. On success,
retain `ADR-D5` ordering: read current server state, apply the result, then
proceed. Distinguish an authoritative absence from an unsuccessful restore;
both allow the same turn to proceed without new goal context. Neither the
failure nor its timeout changes the authoritative goal, emits a normal-ending
report, or starts another turn. A later, independently qualified normal end
of that same execution is eligible under `ADR-D1` through `ADR-D3`; a user
Stop or native execution failure is still excluded.

**Rationale and rejected alternative.** The released `SessionStart(source=compact)`
hook supplies the approved ordering on success and leaves the turn alive on a
bare hook error, missing executable, or native hook timeout. Requiring an abort
on restore failure would need an additional native barrier, and
`SessionStart` can label an explicit `continue: false` abort as an ordinary
`end_turn`. The first delivery retains native continuation on restore failure
instead of returning that abort, racing it with cancellation, replaying an input,
or recovering from a local goal copy. The accepted consequence is that the
ongoing turn can reason without newly restored goal context until it ends.
This resolves the failure-policy question left open by `ADR-D5`; authenticated
binding and supported compaction-path coverage still need verification.

## ADR-D8. Use the ordinary ending rule after an explicit compaction-hook stop

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** If the current live execution actually ends and the supported
adapter reports an otherwise qualifying `end_turn`, evaluate it under
`ADR-D1` through `ADR-D3`, including when a compaction hook returned
`continue: false`. The report needs positive live ending and source evidence,
not successful restoration or a final assistant message. The hook return
alone still does not start a subsequent execution. If admitted, the next
execution gets the current server goal through the ordinary handoff.

**Rationale.** Released `SessionStart` can report `end_turn` without any
model request after the hook stop. Treat that actual ending consistently
with other `end_turn` outcomes instead of adding a separate restore-aborted
reason or a retry path. `ADR-D7` remains the behavior when the native
execution stays running; `ADR-D8` specifies the case when it ends. User
Stop, native turn error, uncertain or restored Idle, stale source, and
duplicate effects remain excluded. This does not prescribe an explicit abort
on restore failure or reclassify `cancelled` as `end_turn`.

## ADR-D9. Share one continuation counter across conversation goals

**Status: shared count and reset on new goal addition accepted by the
requester on 2026-11-18 (KST).**

**Decision.** Maintain one server-owned automatic-admission counter per
conversation, not one budget or runner per goal. A genuinely new,
authorized goal addition resets that counter. Commit the addition and reset
together so replay or response loss cannot replenish the counter twice.
Amendment, settlement or moving between existing goals does not itself
reset it. A reset makes future admission possible; it is not an ending
event, a pending admission or an instruction to start a turn.

Charge once in the normal-ending admission transaction when it admits one
automatic follow-up (`ADR-D2`), independently of the number of eligible
goals. Ordinary input, no-action decisions and in-turn compaction
restoration do not consume that counter. Do not release a spent admission
on delivery uncertainty or replay an outcome after a reset.

**Trade-off.** If A and B are already registered, B inherits any usage
incurred after the last addition even if A used the whole allowance. Adding
B restores shared headroom at registration, not at a later switch to B.
This avoids per-goal allowance state, selecting a goal for accounting, and
multiplying starts when several goals are unfinished.

**Still open.** The exact limit, who may establish a reset-eligible new
goal and how to prevent repeated model-driven creation from defeating the
guard, and the proof required for tool-specific use of an internal reset
operation. An independent lifetime ceiling has not been selected. This ADR
does not specify the composition of multi-goal context or authorize a goal
selector or scheduler.

## ADR-D10. Deliver the full conversation goal set without eligibility filtering

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Use one server-owned goal-context projection for both automatic
follow-up and successful compaction restore. A projection contains all goals
in the conversation's current authoritative set with their states, including
goals waiting for input and any terminal goals retained in that set. It is
not a read of historical or removed goals, and does not define a new retention
policy. The same projection rule applies at both boundaries; each reads
current state at its own authorized point.

**Admission is independent.** Admit at most one follow-up under the existing
normal-ending and shared-counter rules, and only if at least one goal is
eligible. A pending user reply excludes the waiting goal from eligibility,
not from context. It does not exclude another eligible goal. Terminal or
waiting state stays explicit and is neither cleared by injection nor interpreted
as permission to work on that goal. Compaction restore applies the projection
inside the existing turn without admitting or charging a follow-up, including
when nothing is eligible to start another turn.

**Rationale and rejected alternative.** Filtering each payload by execution
eligibility would hide state that the Agent may need to understand the
conversation and risks different context after a continuation versus a
compaction. A full stateful projection avoids that discrepancy without adding
goal selection, scheduling, per-goal turns or per-execution goal binding.

## ADR-D11. Bound automatic follow-ups between resets and renew on user input

**Status: the interval bound and the explicit user-input reset were accepted
by the requester on 2026-11-18 (KST).**

**Decision.** The shared counter bounds automatic follow-up admissions since
the last valid reset. There is no second cumulative limit. Keep the new-goal
reset from `ADR-D9`. Additionally, acceptance of a genuinely new, explicit
user input for the same conversation resets the counter, even when it has
remaining capacity. Apply the reset once at the same authority boundary as
new input acceptance. Partial or rejected input, recovery of existing
acceptance, and internal continuation are not qualifying input. Later native
delivery or failure does not itself apply a reset.

**Ordering.** Serialize acceptance and reset with automatic admission. If
an input is already eligible to lead, it still wins under `ADR-D3`. If
automatic admission has already committed, the later input may reset the
counter but does not cancel or reorder that admission or reopen its ending.
Reset grants capacity for future qualifying endings; existing user input
handling remains responsible for delivering the user's own turn.

**Rationale.** A fixed allowance without recognized progress would block
long-running work that is still advancing. A cumulative ceiling, separate
from the shared counter, was not selected. Genuine input demonstrates
renewed user participation without a separate resume action. The resulting
guarantee depends on rejecting duplicate or unqualified reset events.
The threshold and exact eligibility of tool-owned resets and new goal
registration remain open.

## ADR-D12. Admit at most three automatic follow-ups per reset interval

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Use three as the first-delivery limit for the shared
conversation counter (`ADR-D9`, `ADR-D11`). With all other admission
conditions met, counts zero through two allow one new automatic admission
and increment the count; at three, record a no-action outcome. The limit
does not count a direct user execution or restore within the same execution.
A valid reset grants a new allowance of three, not a retry of any prior
ending.

**Rationale.** Three admissions allow short multi-turn work while bounding
an unattended loop that does not produce a qualifying reset. This is a
product default, not a measured workload limit or a second cumulative cap.
Tool-owned reset qualification remains a separate decision.

## ADR-D13. Reset the shared counter on an accepted `send` progress change

**Status: progress-change reset accepted by the requester on 2026-11-18
(KST). The exact qualifying transitions remain open.**

**Decision.** An authorized `send` call that commits a qualifying change to
Conversation Work progress resets the conversation's shared counter through
the same internal reset boundary used by server domain code. Apply the state
change and reset in the same transaction, alongside a message receipt if
requested (`ADR-D6`). No model-facing reset command or flag is added. Other
domains define their own qualifying changes before calling the internal
operation. A message-only call, title-only edit, identical progress state,
validation error or rolled-back write does not reset the counter.

**Rationale.** Progress changes can support long-lived work without a user
round trip. The bound from `ADR-D11` and `ADR-D12` applies between resets,
not across accepted progress changes. This decision does not determine
whether reverse transitions, repeated back-and-forth changes, or changes to
the to-do list structure qualify. A lost response is not authorization to
replay a `send` call.

## ADR-D14. Require a work action for `send` and a task update for continuation

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Every `send` request specifies a Conversation Work action.
Messages are allowed only as part of a valid work action. Continuing work
requires a task update; a title or message cannot substitute for it. If a
required action or update is absent, reject the entire request without
changing work, publishing a message, or resetting the counter. Work actions
that do not require a user-visible message can still be silent.

**Rationale.** The work action states the effect the Agent intends. Requiring
a task update makes continuation carry progress information instead of
serving as a standalone publisher. The earlier message-only option does not
apply to this tool. The shared transaction and uncertain-result rules from
`ADR-D6` are unchanged. The task update representation, validity of an
unchanged supplied update, and exact progress-reset qualification under
`ADR-D13` are separate decisions.

## ADR-D15. Name silent completion `ignore`

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Represent silent completion as the `ignore` work action of
`send`. It settles the affected work without publishing a Chat message.
`finish` represents completion with a published result. Omitting a result
does not implicitly select silent completion. The two actions use the same
work completion rules, and neither settles other goals or cancels the current
native execution.

**Rationale.** The Agent explicitly declares whether a result should be
published instead of changing intent by leaving out a message. `ignore`
is a work action, not a separate tool or a failure outcome.

## ADR-D16. Accept an unchanged task list in `send` continuation

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** A supplied, valid task list satisfies the required task update
on `continue` even when it is unchanged. An accompanying message may be
published. Omitting the update remains invalid, and an unchanged list does
not itself mutate task progress or renew the continuation allowance.

**Rationale.** An Agent may report meaningful findings while the task plan
is still accurate. Requiring a state difference would incentivize artificial
edits. The call still explicitly carries the work state; progress qualification
for reset remains independent of request validity.

## ADR-D17. Treat every committed task-status transition as a reset

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** For a task whose status changes in a committed `send` update,
reset the shared continuation counter regardless of transition direction or
whether that status has been held before. Compare the new status with the
immediately preceding committed status. For example, completed to in
progress, followed by in progress to completed in another accepted call,
resets on both changes. An identical current status does not reset; multiple
changes in one accepted call still produce one reset. The reset is atomic
with the accepted work update and any publication under `ADR-D13`.

**Rationale.** The requester treats the Agent's explicit update as evidence
that it is aware of the work state, not proof of cumulative advancement.
Requiring a never-before-reached status would discard deliberate reopening
of work. Repeated actual transitions can therefore replenish the allowance;
the three-follow-up limit applies between resets, not across them
(`ADR-D11`, `ADR-D12`). Pure list-structure changes remain a separate
specification question.

## ADR-D18. Reset when `send` commits any actual task-list change

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Determine whether to reset by comparing the committed ordered
task list with the validated replacement state that the work action will
commit. A difference in task membership, task content, status, or order
resets the shared counter once. An unchanged list can still accompany a
message under `ADR-D16` without resetting the counter. Changes to the work
title or message alone do not count as task-list differences. Apply reset
only with the successful transaction (`ADR-D13`).

**Rationale.** Adding, removing, editing or reordering work can all reflect
the Agent's awareness of what remains. Treat the ordered task list as one
observable state instead of interpreting individual edits as greater or
lesser progress. This extends `ADR-D17` beyond status changes and retains
the limit of three admissions per interval, without a cumulative cap. It
does not settle the tool's representation of edits or whether empty lists
are valid.

## ADR-D19. Allow `finish` to answer a simple request without active work

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** An authorized `finish` with a result can answer a simple
request that has no active Conversation Work item. Publish one result
message without creating a work item, goal, or summary. Do not reset the
counter or settle unrelated goals. When the call applies to active work,
publish the result and settle that work together as before.

**Rationale.** A short answer does not need an artificial progress lifecycle.
The caller still explicitly chooses the `finish` action and supplies a
result; the actionless publication rejected by `ADR-D14` is not restored.
The uncertain-result and no-automatic-replay rules still apply.

## ADR-D20. Start Conversation Work with the first `continue`

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** When there is no active Conversation Work, `send` with
`continue` starts it using a supplied title and task list. Later
`continue` calls update the same work using the required task list and an
optional title. Either may include a message. The first call makes the
new work observable as one coherent change, using the existing common
goal and message boundaries; it neither creates a second runner nor starts
a second native turn. A later update does not implicitly create a new goal,
replace unrelated work, or clear an input wait.

**Rationale.** Declaring a title and plan is enough to start tracking an
ongoing request. A separate start action would add another step without
clarifying intent. Short requests still have the direct `finish` path
from `ADR-D19`.

## ADR-D21. Allow `request_input` without active work

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** `request_input` publishes a question. With active Conversation
Work, publication and the transition of that work to input wait commit
together. With no active work, publish the question without creating work,
a goal, a summary, or a separate wait. This does not suspend unrelated
goals, reset the counter, or start another execution.

**Rationale.** A clarification can precede any tracked work. It should not
manufacture a progress lifecycle, while an actual in-progress work item
still needs its input wait committed with the user-visible question.

## ADR-D22. Finalize `send` list, absence and result semantics as a bundle

**Status: accepted by the requester on 2026-11-18 (KST).**

**Decision.** Each `continue` supplies the entire ordered task list. An
empty list is an explicit replacement, not a missing field or completion.
Use the actual resulting list for the shared reset rule. Reject blank
required titles, results and questions, and invalid action/field
combinations. An omitted optional title or message means retain the title
or publish nothing, respectively.

An `ignore` without active work succeeds with no effects. A completed work
item is not reopened; a later new request can use `continue` to start new
work. A successful result returns a message identifier only when a message
was committed. Known rejection has no effects. If the result may have been
lost after execution, it is uncertain and not permission to replay the
call. Do not add a durable result query or caller-supplied retry key.

**Rationale.** Full replacement gives the Agent one way to describe current
work. Silent no-action does not create work just to finish it, and the
response tells the Agent what was actually published without turning a
tool result into a delivery acknowledgement.

## ADR-D23. Report exact live root endings over Control

**Status: accepted by the requester on 2026-11-18 (KST).**

Qualify a root Running-to-`end_turn` transition at current live capture
ingress. Bind it to the ending work token and captured occurrence before
foreground clears the token. Exclude prior Stop publication for that work,
errors, child sessions, restored Idle and stale sources. Use typed bounded
request-response reporting on the existing Control component authority.
Reconnection may recover the same live occurrence; restart does not create
new reports from retained history. No separate durable hook outbox.

## ADR-D24. Commit ending decisions and dispatch internal inputs independently

**Status: accepted by the requester on 2026-11-18 (KST).**

Retain the outcome by current source and qualified ending identity. In one
server transaction commit the outcome, shared charge, and internal input
when admitted; acknowledge afterward. Apply the ordinary-input precedence
of ADR-D3, and do not reopen a decided occurrence after a reset. Dispatch
under independent internal authority and the existing no-blind-replay
boundary. Do not wake a different owner or Computer. Use ACP's existing
prompt path with explicitly identified internal context. Native records and
model messages retain the real `user` role; product Chat, input-wait clearing,
user-input resets and permissions do not treat this as user input.

## ADR-D25. Restore through a source-scoped managed native hook

**Status: accepted by the requester on 2026-11-18 (KST).**

Use the validated `SessionStart(source=compact)` path. A managed, bounded
hook invokes a Connector-local, read-only restore boundary scoped to its
applied native owner. The Connector reads the server via current component
authority; callers cannot choose another conversation or source. Do not
expose a model-facing lookup, platform credential or local goal copy.
Prove isolation and reject unqualified or stale callers before enabling
the bridge. Preserve ADR-D7 and ADR-D8 failure and ending semantics.

## ADR-D26. Bound the complete current goal projection at write time

**Status: accepted by the requester on 2026-11-18 (KST).**

Only an authorized distinct domain goal addition renews the allowance;
retrying that addition or inventing an identity in model text does not.
Durably record explicit terminal outcomes and remove settled goals from
the current set. Use the complete remaining set for both projections
without truncation. Bound serialized goal context to 8 KiB UTF-8 at
admission of goal additions and amendments, accounting for later state
labels so state changes and settlement are not refused for size.

## ADR-D27. Replace the tool at the existing owner cutover boundary

**Status: accepted by the requester on 2026-11-18 (KST).**

Retain existing captured history and Chat receipts. Use the existing
exact-context owner refresh and external-effect drain boundary. Replace
catalogue, dispatch and instructions together; verify New and exact Resume
without exposing a legacy tool or falling back to a fresh context. Qualify
only native version/path combinations that pass the release checks. No
deployment or merge is authorized by this decision.

## ADR-D28. Trust the local restore invocation

**Status: accepted by the requester on 2026-11-19 (KST).**

Keep official, unmodified Codex and ACP. The Connector accepts a local restore
invocation without independently proving that Codex started the hook. The
endpoint has no caller-selected identifiers; it reads only the applied native
owner's current conversation under the existing authenticated Control route.
This supersedes ADR-D25's direct-caller qualification gate, not its read-only
scope, stale-owner exclusion, same-turn ordering, or failure semantics. No
new hook credential, process isolation, general Goal endpoint, or alternate
runner is introduced. An ordinary process in the same Computer may invoke this
local read for the current execution; do not claim hook-only isolation.
