---
title: "Conversation Work and Goal Continuation Requirements"
created: 2026-11-18
tags: [backend, frontend, engine]
document_role: primary
document_type: requirements
snapshot_id: stride-261118
---

# Conversation Work and Goal Continuation Requirements

- Snapshot: `stride-261118`
- Document reference: `stride-261118/REQ`

## Problem

A user can ask an Agent to do work that lasts beyond a single execution. A
normal execution ending does not tell the user whether the requested work is
finished, still in progress, or waiting for input. The user should not need to
send repeated reminders, nor should stopping an execution restart it unaided.

## Primary Context

### Primary Actor

A signed-in user who hands work to an Agent in an existing Chapchoo conversation.

### Primary Scenario

The user asks for a multi-step result in Chat. The Agent sets the goal and can
show a compact work summary. A normal execution ends before the work is done;
the Agent receives the unfinished goal and continues in the same conversation.
If the Agent asks for input, it waits until a new user message in that
conversation is accepted. The Agent then explicitly concludes the work. Only
intentionally published messages appear in Chat, and the summary is removed.

## Supporting Scenarios or Effects

- The user sends additional input through the existing composer while work is active.
- The user stops a current execution without ending or pausing the work itself.
- Native context compacts while an unfinished goal still matters.

## Goals

- Give the common goal foundation exactly five responsibilities: goal setting,
  unfinished-goal continuation after normal execution, explicit completion,
  restoration after compaction, and minimum continuation safety.
- Make Conversation Work the first consumer, with explicit messaging and work
  actions and a compact progress view.
- Allow the Connector to report different supported lifecycle hook events to
  the server through a shared boundary. Normal execution ending is the first
  event, for goal continuation.

## Non-Goals

- No per-execution goal binding, continuous checkpoints, common goal query or
  tracing, separate goal management UI, planner, subgoal tree, or success judge.
- No Routine, Scheduled Task, cron, or external chat provider integration.
- No separate work-action tool alongside the Chat message tool and no new input
  composer or message-merging workflow.
- No change to native authentication, Computer power management, or existing
  conversation access rules.
- No additional hook event types or consumers are required for the first
  delivery. No user-configurable hooks or general event automation platform.

## Requirements

### REQ-1. Set goals independently of execution

The common foundation lets internal consumers set and amend goals without
attaching them to particular executions. More than one distinct goal can
remain unfinished in the same conversation. Conversation Work is the first
consumer: the Agent sets and updates its associated goal through `send`.

**Acceptance criteria**

- A goal set or amended during one execution remains in effect afterward
  unless explicitly changed or ended. Adding a distinct goal does not implicitly
  replace an existing goal.
- Starting or stopping an execution alone does not create, replace, or settle a goal.
- Explicitly completed goals remain recorded outside the current goal set.
  An update exceeding the supported full-context limit
  is rejected without changing existing goals; settlement remains available.

### REQ-2. Continue unfinished work after normal execution

After a normal execution ends, eligible unfinished work continues with
authoritative goal context in the same conversation without another user
message, subject to the continuation safety limit.

**Acceptance criteria**

- An eligible normal ending results in one follow-up execution that receives
  the conversation's full current goal set with goal states. Inclusion in
  context does not make a goal eligible for automatic work.
- The Conversation Work goal context includes the complete current ordered
  task list and each task's status, not just the Work title. A task list update
  is reflected in subsequent continuation or compaction recovery. The list
  does not change the rule that settlement must be explicit.
- A live execution ended by a compaction hook is eligible for the same
  follow-up if the native runtime delivers an otherwise qualifying normal
  ending. It does not need to have made another inference after compaction.
- A follow-up requires at least one goal eligible to continue. A completed
  or input-waiting goal cannot cause a follow-up on its own. Waiting on
  one goal does not block an otherwise eligible goal.
- Internal continuation does not appear as a user Chat message.

### REQ-3. Record explicit work completion

Completion is the single terminal disposition for Work and its Goal,
independent of the current native execution.

**Acceptance criteria**

- Explicit completion removes the Work summary and prevents automatic
  continuation of that work.
- Ordinary model output, a normal native turn ending, or all visible to-dos
  being checked off does not by itself mark the work complete.
- The `send` tool description directs the Agent to complete Work explicitly
  when it decides to end the work because it cannot be completed or has been
  cancelled. Use `finish` with an intentional message or `ignore` without one.

### REQ-4. Restore current goal context after compaction

Restore current goal context after compaction when it can be recovered. An
unsuccessful restore does not settle a goal or require the execution to stop.

**Acceptance criteria**

- On successful recovery after compaction, the next model step receives the
  same full current goal set and states as continuation would, including
  explicit amendments and waiting goals. Inclusion in that set does not make
  a goal eligible for work.
- If goal recovery or application fails or times out, let the current execution
  continue without newly restored goal context when the native runtime keeps it
  running. The restore outcome does not change the authoritative goal or by
  itself initiate a new execution. A qualifying native ending follows the
  ordinary continuation rule, even if a compaction hook ended the execution.
- Work no longer eligible to continue must not become active merely because
  context was compacted or restored.
- Restoration is internal; it does not publish fabricated Chat messages.

### REQ-5. Honor Stop and prevent unsafe continuation

Stop affects the current execution only. A stopped execution is not a normal
ending for continuation evaluation. Maintain existing input and access safety.

**Acceptance criteria**

- Stop alone does not change the work state, title, to-dos, or summary. The
  execution ended by Stop does not introduce a continuation.
- If a subsequent execution ends normally, the ordinary continuation rule
  applies again without a separate resume action.
- Duplicate, stale, restored, or uncertain observations do not create duplicate
  executions. A possibly delivered input is not blindly replayed.
- If a new user message lands near the end of an execution, it retains the
  existing input behavior and appears at most once; that race does not cause a
  redundant follow-up for the same boundary.
- Goals in one conversation share a limit of three automatic follow-up
  admissions between qualifying resets, with no separate cumulative cap
  across resets. A fourth admission is blocked until a reset. This does not
  end an active execution, settle a goal, or block ordinary user input.
- Adding a genuinely new goal resets the shared counter. Amending, retrying an
  addition for, or moving between existing goals does not itself reset the
  counter. A goal addition does not itself start an execution.
- Genuinely new explicit user input accepted in the same conversation resets
  the shared counter whether or not the limit was reached. A duplicate,
  rejected submission, recovered receipt, or internal input is not a new
  user input and does not reset it.
- An accepted change to the Conversation Work task list through `send`
  resets the shared counter. This includes additions, removals, content
  changes, reordering, and status changes in either direction, including
  return to a previously held status. An accompanying message or title
  change with an unchanged task list does not reset the counter.

### REQ-6. Give Conversation Work and messaging one `send` tool

One Agent-facing tool named `send` handles Conversation Work actions and
any accompanying intentional Chat publication within the current authorized
conversation.

**Acceptance criteria**

- The first `continue` when there is no active Conversation Work starts it
  with a title and task list. Subsequent `continue` calls update the same
  work, require a task update, and may include a new title. Either call may
  include a message. A call missing the required information does not change
  work or publish a message. No separate start action is needed.
- A valid task update may repeat the current task list unchanged. An
  accompanying message can still be published; unchanged progress does not
  reset the continuation counter.
- A task update supplies the complete ordered list. An empty list clears the
  tasks without completing the work.
- A message cannot be published by `send` without a valid work action.
- The Agent can publish a result and finish, request user input and wait,
  or explicitly choose `ignore` to finish without publishing a message.
  Silent completion settles only the affected work.
- Without active work, `ignore` has no effects. Completed work is not reopened;
  a later request can start new work with `continue`.
- A simple request with no active Conversation Work can receive a result via
  `finish` without creating a work item, goal, or progress summary.
- `request_input` publishes a question. If there is active Conversation
  Work, that work enters input wait. Otherwise the question is published
  without creating work, a goal, a summary, or a separate waiting state.
- Only a newly accepted user input in the same conversation satisfies the user
  input wait. Duplicate, failed, unconfirmed, or other-conversation inputs do not.
- New user text uses the existing composer and does not itself replace or settle
  work. Native final/model text remains available as a record and is not
  substituted for an intentionally published Chat message.
- `send_message` and a parallel work-action tool do not remain as separate
  Agent-facing choices for the same capability.

### REQ-7. Show progress without disturbing the composer

Display an active work summary above the composer as one floating glass surface.
The user can open it to inspect ordered to-dos and return it to compact form.

**Acceptance criteria**

- The compact summary shows a title and completion count. Expanded details use
  the same outer surface and a straight divider under the heading. The composer
  remains in the same position across resize, appearance, and dismissal.
- Press feedback tints the entire surface with the existing theme, in light and
  dark modes, rather than drawing a native dark highlight over the trigger.
- The summary changes size with the reviewed spring motion. An existing summary
  renders without entrance motion on first load or refresh. If a summary appears
  after being absent on the same screen, it grows into place.
- At work end, the visible surface expands slightly outward and then shrinks
  away, without independently fading the title or to-dos first. Reduced-motion
  settings omit the animated transitions.

### REQ-8. Reuse Connector hook reporting across supported event types

The Connector can report supported lifecycle events to the server using one
reusable reporting boundary. The first supported event is a qualified normal
execution ending; the server uses it to evaluate goal continuation.

**Acceptance criteria**

- The first event reaches the server under the current Connector and
  native-source authority. Stop, failure, and a restored Idle observation
  are not reported as normal endings.
- Each accepted normal ending is eligible for at most one continuation
  handoff. Duplicate or stale reports do not create additional executions.
- Another supported event type can be added without replacing the Connector
  reporting boundary or changing the first event's meaning. Unsupported
  reports do not trigger product effects.
- Reporting an event does not grant a Connector or its payload authority to
  choose arbitrary server actions or publish a Chat message.

## Fixed Constraints

- Goal state and current execution state are distinct. Stop leaves Work and its
  Goal unchanged, and neither view state nor adapter recovery is evidence of
  work completion.
- Current conversation permissions and ordinary user-input delivery guarantees
  remain in force. Background continuation cannot impersonate a user.
- Work summary behavior applies to the conversation Chat surface; it does not
  introduce Conversation Work to installation administration.

## Open Assumptions

- The supported native integration must expose sufficient information to
  distinguish attributable normal endings from stops, failures, and restored
  snapshots. Runtime proof is pending.
- The supported model integration must admit a current-goal restoration at a
  usable point after compaction, without duplicating or diverting execution.
  Runtime proof is pending; the Requirements do not prescribe a mechanism.

## Confirmation

Confirmed by the requester on 2026-11-18 (KST) before ADR and Design decisions.
On the same date the requester explicitly added reusable Connector hook event
reporting with normal execution ending as the first case (`REQ-8`). This
confirmation covers requirements, not implementation authorization.

On the same date the requester confirmed the failure behavior in `REQ-4`:
preserve same-execution progress on post-compaction restore failure, and use the
ordinary continuation rule (`REQ-2`) if a compaction hook ends the execution
and the resulting native ending otherwise qualifies.

On the same date the requester confirmed a shared conversation counter and
reset on addition of a new goal (`REQ-1`, `REQ-5`). The requester also confirmed
that the limit applies between valid resets, with no separate cumulative cap,
and that new explicit same-conversation user input resets the counter
(`REQ-5`). The requester confirmed three automatic admissions per reset
interval for the first delivery and reset on a `send` progress-state change.
The requester confirmed that any actual task-list change qualifies, including
structural edits and status changes in either direction.
The requester also confirmed that both
continuation and compaction restore pass the full current goal set without
filtering by eligibility (`REQ-2`, `REQ-4`).
The requester also made a task update mandatory when asking `send` to
continue Conversation Work and excluded standalone message publication
through `send` (`REQ-6`).
The requester chose the distinct `ignore` work action for silent completion
and confirmed that an unchanged task list can satisfy the `continue` input
requirement (`REQ-6`).
The requester confirmed that `finish` can answer a simple request without an
active Conversation Work item or creating one (`REQ-6`).
The requester confirmed that the first `continue` starts Conversation Work
with a title and task list, and later calls update the same work (`REQ-6`).
The requester confirmed that `request_input` can ask a question without
creating work or a separate waiting state when no work is active (`REQ-6`).
The requester accepted the consolidated tool details for list replacement,
silent no-action when there is no work, and the completion/result boundary
(`REQ-6`).

On 2026-11-19 (KST), the requester clarified that the Conversation Work goal
context carries its current ordered task list and individual statuses (`REQ-2`,
`REQ-4`), without introducing Goal management as a product feature.

On 2026-11-19 (KST), the requester clarified that completion is the only Work
settlement and directed the `send` tool description to use that settlement
when the Agent ends Work because of failure or cancellation (`REQ-3`).
