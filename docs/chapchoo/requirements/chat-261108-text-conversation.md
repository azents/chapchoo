---
title: "Codex Text Conversation Requirements"
created: 2026-11-08
tags: [agent, conversation, codex, acp, frontend, backend]
document_role: primary
document_type: requirements
snapshot_id: chat-261108
---

# Codex Text Conversation Requirements

- Snapshot: `chat-261108`
- Document reference: `chat-261108/REQ`
- Review revision: 24

## Problem

The existing Agent Chat is a local presentation surface. Entered messages and
activity are not backed by a real Codex conversation, retained conversation
history, or actual execution control. A connected Computer and a saved
backend-authentication selection do not establish harness readiness or successful
message delivery. Users need one coherent conversation experience across live
output, intervention, requested decisions, reentry, and failure.

## Primary Context

### Primary Actor

An authenticated user who owns a private Agent and is currently authorized in
that Agent's Team context.

### Primary Scenario

The user selects an Agent, continues its conversation with ordinary text, reads
streaming assistant Markdown, supplies requested information or execution
permission in the timeline, inspects work when needed, and returns to the same
conversation after leaving the screen or recovering from failure.

## Supporting Scenarios or Effects

- Intervene during current work using steering or explicitly stop current work.
- Read retained conversation and execution records while the Computer is off.
- Recover without confusing confirmed non-delivery with uncertain delivery.
- Preserve private ownership and selected-authentication boundaries throughout
  conversation reads, input, decisions, and execution control.
- Use a compatible existing Codex installation available in the executing
  Computer as an explicitly selected alternative to managed preparation.

## Goals

- Deliver real Codex text interaction through the approved ACP v2 scope.
- Preserve distinct assistant messages and truthful delivery/execution states.
- Present structured questions and permission choices directly in the timeline.
- Retain conversation continuity and explicit, non-duplicating recovery.
- Preserve the agreed main-conversation, activity-summary, and history roles.
- Make harness preparation practical for cloud Computers, keep compatible
  existing installations viable, and define a maintenance policy.

## Non-Goals

- ACP v1 compatibility or another initial harness.
- Multi-conversation creation, selection, branching, or routing UI; a permanent
  Agent-to-conversation one-to-one product constraint is also not introduced.
- ACP URL-mode external-interaction consent, browser handoff, or completion UI.
  Ordinary Markdown links and existing backend authentication are distinct.
- Product-defined response deadlines, countdowns, automatic request expiry, or
  time-elapsed automatic execution termination. No ten- or thirty-minute policy
  is implied. Actual request cancellation or closure remains relevant to validity.
- Proactive delivery, notifications, separate result inboxes, or channel-tool
  message injection.
- Dedicated rich cards or automatic conversion of assistant prose into them.
- Attachment/file-management features, model selection, or redesign of existing
  account authentication, Agent creation, and Computer lifecycle management.
- Arbitrary JSON Schema builders, nested-object forms, unrestricted arrays,
  uploads, or conditional form editors.
- Automatic replay of uncertain work or rollback of completed external effects.
- Detection or recovery of native events missed before capture, and coverage of
  activity from independently launched external harnesses.
- A product-level native-history reload action.
- Dedicated storage-health monitoring/recovery or comprehensive handling of
  every local recording-failure scenario.

## Requirements

Record-retention outcomes below apply within the available source snapshot.
REQ-16 explicitly permits complete replacement of the session's server
conversation data when its previous source snapshot is unavailable.

### REQ-1. Authorized ongoing Agent conversation

The private Agent's authorized user can continue one ongoing conversation with
that Agent using its selected, eligible backend authentication.

**Acceptance criteria**

- Selecting an Agent enters that Agent's conversation; the sidebar lists Agents,
  not conversations.
- Ordinary use does not require starting a new conversation for each task.
- Existing Agents without an eligible authentication or available harness show a
  truthful inability to execute rather than fabricated readiness or a silent
  fallback to a different credential.
- Reads and mutations preserve individual ownership and current Team admission;
  another private Agent is not exposed merely through a known identifier.
- A saved authentication, Computer power, Connector connectivity, and harness
  readiness are not represented as interchangeable facts.
- With an available authorized Computer environment, a user text submission
  triggers harness preparation/start if no live harness is available and safe
  start conditions are satisfied. An existing live harness is reused.
- Opening the conversation or reading retained records does not itself start the
  harness. This behavior does not introduce automatic Computer power-on.
- Normal task completion does not retire the reusable harness. The initial
  integration has no idle-time automatic harness shutdown; Connector/Computer
  termination, authority withdrawal and failures remain separate lifecycle events.
- After a complete eligible replacement is prepared, automatically remove only
  unused superseded Chapchoo-managed installation cache. Preserve active/preparing
  files and the latest complete eligible combination, including shared files
  needed by protected combinations.
- Installation-cache cleanup does not remove external CLI installations,
  conversation sources, native context, authentication or user workspace.
  A removed installation may need downloading again; this is not a storage quota
  or unlimited-capacity guarantee.
- The initial single-conversation experience does not require a permanent
  product rule forbidding future additional conversations for the same Agent.
- The authorized owner can change the Agent's saved authentication selection
  while its harness is live. Selection save alone does not stop or restart it.
- The running harness retains its previously applied authentication; the changed
  saved selection applies at the next actual harness process start, not the next
  user message or normal task completion.
- Saved selection and running authentication are distinguishable when they
  differ. Save success does not claim native application of the new selection.
- An authentication used only by an already-running harness does not add a new
  deletion block. Existing saved Agent references retain their deletion protection.
- Deleting an otherwise deletable authentication stops new renewal and credential
  delivery for that record; it does not silently switch the running harness to a
  different saved selection. Already-delivered credentials are not immediately
  revoked by removing the record, so later authentication failure is possible.
- This selection-change policy does not authorize continuing use after loss of
  the underlying owner/Team/credential authority, or define same-record key
  replacement/reauthentication behavior.

### REQ-2. Plain-text input and truthful delivery

The user can send ordinary text to real Codex work and observe delivery separately
from execution and output.

**Acceptance criteria**

- User text is delivered as text; no attachment or new channel-tool input is
  required for this primary scenario.
- Accepted input, confirmed non-delivery, uncertain delivery, and resulting work
  are distinguishable where their next actions differ.
- Input survives a recoverable send failure; an uncertain prior send is not
  automatically reissued as new work.
- A local UI append, transport acknowledgement, or connected status alone is not
  presented as evidence that Codex consumed or completed the work.
- After validating current authority/target and retaining the input, the server
  responds with product acceptance and an operation reference without waiting
  for harness preparation, ACP acceptance or work completion.
- Subsequent preparation, harness acceptance and execution/result states are
  delivered separately from that product acceptance using actual evidence.
- Losing the browser connection alone does not cancel accepted work or authorize
  automatic resubmission. Reconnection reads the operation's current known state.
- Early acceptance does not turn running input into a future-task queue or grant
  execution after current authority has become invalid.

### REQ-3. Streaming Markdown with faithful assistant message boundaries

Assistant text appears in the main conversation as it is generated, with Markdown
and the message identity supplied by the harness preserved.

**Acceptance criteria**

- Updates to the same assistant utterance update that message in place rather
  than creating a message per chunk.
- Distinct intermediate and subsequent assistant utterances remain distinct;
  the final answer does not erase earlier separate utterances.
- Harness-provided formal content updates to the same message remain possible.
- Tool activity, plans, and internal execution records are not manufactured into
  assistant chat messages; the UI does not invent assistant text.
- Output arrival is not, by itself, a current-execution completion signal.

### REQ-4. Steering during current execution

The user can send new text while work is running, and it is treated as steering
for that work at the harness's next supported input boundary.

**Acceptance criteria**

- The main composer retains Send while work is running; empty input has no active
  send action.
- Entered text is not silently recast as a separate future task that waits for
  current work to finish, and execution alone does not disable text sending.
- Steering delivery/consumption uncertainty is reported honestly, not as proven
  incorporation into a model invocation.
- If the real ACP/Codex combination cannot provide the approved behavior, the
  user-visible limitation and alternatives are raised before changing it.

### REQ-5. Current-work stop separate from Computer power

The user can request that only current Agent work stop from the activity summary.

**Acceptance criteria**

- The activity summary has a separate Agent-execution row with Stop, distinct
  from the current plan and Computer power controls.
- Stop is not placed in or substituted for the main composer's Send action.
- There is no confirmation dialog for current-work Stop.
- Pending stop, confirmed termination, failure, and unknown outcome are
  distinguishable; click acceptance alone is not confirmed termination.
- Stop does not promise rollback of already-completed external effects, disable
  all future activity, delete conversation history, or automatically restart work.

### REQ-6. Activity summary and navigable execution history

The user can understand current activity through a bounded summary and inspect
exact execution records without mixing those records into assistant text.

**Acceptance criteria**

- The summary presents Computer state, the available current plan, and recent
  records; it does not invent a plan when none exists.
- Selecting a summary record opens full history at the corresponding record.
- History supports bounded retrieval of older records, loading upward while
  preserving reading position, following the bottom, and returning to latest.
- Tool records expose title/status and expandable input/output.
- The exact recent-record count is a reversible presentation detail, not a new
  product capability or storage-lifetime promise.

### REQ-7. Complete bounded ACP form input

The user can review and answer the ACP v2 restricted flat-object form field set.

**Acceptance criteria**

- Supported field families are text; string single choice using enum or titled
  oneOf; number/integer; boolean; and string multiple choice using enum or titled
  anyOf, with applicable length, pattern, format, number, and item-count limits.
- Titles, descriptions, requiredness, and valid defaults are presented for review
  and editing before submission.
- Supported text formats include email, URI, date, and date-time; an unknown
  format remains ordinary text rather than a newly invented input contract.
- Response values retain their types and original choice values. Unanswered
  optional fields are omitted, not converted into null or fabricated defaults.
  False and otherwise valid empty strings are distinct from unanswered fields.
- An unsupported required field or invalid request is a request error, not a
  simulated user refusal. Unsupported optional fields follow the ACP omission
  contract without inventing answers.
- Secret/credential requests are not collected through ordinary form fields.
- Accept, decline, and cancel are distinct; decline/cancel do not carry answers.

### REQ-8. Inline form questions, options, and retained answers

Structured questions and all their choices are visible and answerable in the main
conversation timeline, with complete readable results after submission.

**Acceptance criteria**

- A separate modal, sheet, dropdown, or detail screen is not required to discover
  the question's choices or to read the submitted answer.
- Single choices are expanded radio choices; multiple choices are directly
  selectable. Requiredness, help, and actionable validation errors are adjacent.
- The user reviews answers and explicitly submits with Send answers.
- After submission, all questions, original options, selected answers, and typed
  values remain directly readable in the timeline. They are not replaced with
  only a Submitted status, truncated with ellipsis, or hidden behind disclosure.
- Choice titles are readable while original response values remain accurate;
  selected options are identified by text/state as well as visual emphasis.
- Optional non-answers and valid empty answers are distinguishable.

### REQ-9. Inline explicit execution permissions

The user can inspect and explicitly answer actual Codex execution/file/permission
requests with the exact choices and scopes the harness provides.

**Acceptance criteria**

- Supplied action content, target/work location, reason, choices, and permission
  scope are visible within the timeline without a separate approval screen.
- Long commands or change details remain inspectable; absent details are not
  replaced with unproven safety claims.
- No allow option is preselected; the user selects a scope and explicitly sends
  the selection. Ordinary chat assent and silence are not permission responses.
- The UI does not create scopes or assume every request has the same fixed set
  of options. One-time, current-work, session, or broader scopes remain distinct
  when actually offered.
- After response, request content, original choices, selected choice, and scope
  remain visible. Permission selection is not execution success.
- Rejection/cancellation is delivered as its actual outcome. Subsequent work or
  termination is shown from actual harness evidence rather than invented success
  or automatic continuation.

### REQ-10. Request validity and response integrity

Form and permission responses apply only to the authorized, still-valid request.

**Acceptance criteria**

- Repeated activation or multiple views cannot produce duplicate effective
  responses or additional authorization for one request.
- A cancelled, closed, superseded, or otherwise invalid request cannot accept a
  late answer, and late results do not restart cancelled work.
- A response is not applied to another Agent, conversation, request, or execution.
- Delivery uncertainty is distinguishable from confirmed response delivery; a
  browser-local response fence is not the sole proof of integrity.
- The actual harness's terminal/cancellation events are handled without adding
  a product-defined response deadline or automatic time-elapsed termination.
- Unsupported URL interaction, including an equivalent permission fallback, is
  not presented as a supported external-consent flow or automatically allowed.

### REQ-11. Reentry, retained history, and context continuity

Leaving and returning preserves the conversation experience and reconnects to
existing live work where it remains available.

**Acceptance criteria**

- Browser reload or navigation away/back does not itself reset accepted
  conversation records or replace them with sample content. Source-snapshot
  replacement follows REQ-16.
- Conversation and execution records remain readable while the Computer is off.
- Reopening the UI does not stop work, start a new task, or replay a dead task.
- A still-live execution is reconnected to rather than duplicated.
- Supported native-context restoration and its limits are verified; the UI does
  not silently discard context or claim an unsupported exact execution resume.
- This requirement does not promise unlimited permanent retention, a new archive
  product, or restoration of an Agent after authorized deletion.
- Ordinary UI reentry reads retained product records without requiring repeated
  native-history replay. Native-history replay is limited by default to the
  session's first load.
- First entry reads recent conversation records; older records remain available
  through incremental history loading rather than requiring the entire history
  before displaying the conversation.
- While connected, the screen receives real-time normalized message, activity
  and request-state updates.
- After reconnection, the currently read conversation region is reconstructed
  from the server's latest normalized state. Recovery does not require replaying
  every transport event missed by the browser.
- Re-reading screen data does not delete server conversation records, trigger
  native replay or re-execute user operations.

### REQ-12. Recovery with preserved input and partial output

Recoverable failures preserve user work and explain the safe next action within
the same conversation.

**Acceptance criteria**

- Confirmed unsent input, uncertain delivery, and interrupted output are distinct.
- Within the retained source snapshot, original input, already-received assistant
  messages, partial answers, and relevant execution evidence remain available
  with truthful status. Complete snapshot replacement follows REQ-16, including
  removal of product facts that cannot be reconstructed from the new snapshot.
- Existing execution/request state is checked before offering a replay-sensitive
  next action; uncertain input is not automatically sent as fresh work.
- The user can give a follow-up instruction or use a supported explicit recovery
  path without a silent conversation reset or fabricated successful completion.
- User Stop is not automatically reversed, and no external-effect rollback is
  implied by failure, interruption, or recovery.
- Preservation covers captured input/output and retained product evidence. The
  initial integration does not detect or recover native events missed before
  capture, including activity from independently launched external harnesses;
  continuity is not a claim of complete native history.
- A surviving Connector preserves command-processing continuity across ordinary
  server-link replacement. After Connector replacement, only already-retained
  or currently provable evidence can settle a previous command; otherwise its
  outcome remains uncertain.
- This recovery scope does not promise reconstruction of every prior dispatch
  fact after Connector failure. Missing processing state is not confirmed
  non-delivery and does not authorize automatic command replay.
- A known local recording failure is not represented as successful persistence.
  Special input-blocking/automatic-shutdown behavior for this failure is
  conditional on small implementation cost; it may be omitted from the initial
  integration if it requires substantial additional work.

### REQ-13. Responsive accessible and truthful presentation

The agreed interaction model works on desktop and mobile while preserving the
existing Agent navigation, main conversation, composer, and activity roles.

**Acceptance criteria**

- Questions, option scopes, long content, answers, and actionable errors remain
  readable without document-level horizontal overflow.
- Keyboard operation, named controls, associated labels/errors, and visible focus
  support the same operations available by pointer.
- Actual current state determines empty, unavailable, pending, completed, failed,
  or uncertain presentation; fixture activity is not production evidence.
- Changing viewport does not introduce a second request or discard an answer.

### REQ-14. Existing Codex installation support in the initial integration

The initial integration supports using an explicitly selected compatible existing
Codex installation available inside the executing Computer, in addition to
managed preparation.

**Acceptance criteria**

- Existing-installation use is available in the initial integration, not merely
  an architectural extension point reserved for future work.
- The selected installation must be usable in that Computer and compatible with
  the initial Codex/ACP v2 integration; a file's existence alone is not readiness.
- A missing or ineligible selected installation produces truthful unavailability
  rather than a silent switch to another installation or managed download.
- Managed preparation does not overwrite or update externally installed files as
  a side effect of selecting or using them.
- This support does not introduce remote execution against a user's PC or the
  Provider host, another initial harness, or implicit use of ambient credentials.

### REQ-15. Owner-controlled saved harness supply selection

The private Agent's currently authorized owner can select managed preparation or
existing Codex use and retain the applicable executable selection for that Agent.

**Acceptance criteria**

- Selection is an initial product capability, not solely a deployment-operator
  startup setting. It remains available across UI reentry and Connector replacement.
- An Agent without a saved supply selection uses managed preparation by default.
  No explicit supply choice is required before its first actual input; preparation
  still follows the input-triggered start boundary and does not power on a
  Computer or start a harness merely because the UI is read.
- Reads and changes require individual Agent ownership and current membership in
  its Team. Team management or installation administrator status alone does not
  authorize access to another user's private supply settings.
- Existing-installation selection retains its explicit executable choice; managed
  preparation does not expose arbitrary download-source or release-selection
  controls through this capability.
- Harness-authored local file changes are not treated as an authorized change to
  the owner's saved product selection.
- An authorized owner can save a changed selection while a harness is running.
  The running harness continues with its existing configuration; saving alone
  does not start, stop or restart a harness or Computer.
- When saved selection differs from the running configuration, that difference
  is visible. Save success is not reported as applied configuration or execution
  readiness.
- The changed selection applies when a genuinely new harness process starts,
  not on the next user message, model turn or task-completion event. A failed
  start does not silently revert the selection or reset conversation context.
- This requirement does not select a screen location or a restart/apply-now
  control.

### REQ-16. Captured-event catch-up and rebuildable server history

The server recovers captured conversation events missed during a Connector-link
interruption and can reconstruct its conversation data from the currently
available source snapshot.

**Acceptance criteria**

- Reconnecting resumes after the server's latest retained progress within the
  same source snapshot rather than requesting native-history replay.
- A request against a source snapshot that no longer exists fails explicitly;
  it is not treated as a valid empty result or silently continued in another one.
- When the previous snapshot is unavailable, the server rebuilds its
  entire session conversation data from the available snapshot instead of
  continuing the old progress as if the source had not changed.
- Clear the previous session conversation data before filling its replacement;
  the initial integration does not retain a second dataset until an atomic swap.
- While the replacement is incomplete, use ordinary conversation loading/error
  presentation rather than a completed empty history or internal processing
  labels. New effect-bearing inputs/responses are not accepted against that
  incomplete source; exact known current-work Stop/shutdown remains distinct.
- A failed transfer remains incomplete and may continue from retained captured
  progress. This is data recovery, not resubmission of user work.
- Rebuild replaces previous captured history, rendering state and independently
  admitted product inputs, form/permission responses and delivery evidence.
  Facts absent from the available snapshot are not separately retained or
  reconstructed by inference.
- This replacement is confined to the affected session's conversation data; it
  does not delete the Agent, account, authentication selection or Computer.
- Reconstructing records does not re-execute work or resend historical inputs
  or permission responses. Reconstructed history alone does not establish
  present request validity.
- Subsequent captured events continue to update server history and rendering.
- Retained source data supports debugging and renewed normalization; rendering
  uses the service-normalized representation of those same events.
- The current captured snapshot retains all its captured events until actual
  source-snapshot replacement or authorized storage deletion. Server storage
  acknowledgement does not shorten that retained source.
- The initial integration has no automatic acknowledged-prefix pruning, log
  rotation, compression or age-based expiry for that current snapshot. Retention
  remains subject to existing storage capacity and REQ-12's recording-failure
  limits, not an unlimited-capacity or lossless-crash guarantee.
- This does not recover events the Connector never captured or add a
  user-facing native-history reload feature.

### REQ-17. Centrally maintained selected ChatGPT authentication

Agents use their selected ChatGPT authentication with centrally coordinated
renewal rather than independent refresh-token copies in each Computer.

**Acceptance criteria**

- The server retains and uses the ChatGPT refresh credential; executing
  Computers receive the short-lived token material needed for authorized use,
  not the refresh token.
- Renewal of a shared saved authentication is coordinated centrally. This
  direction does not introduce a one-active-Agent-per-authentication restriction.
- Credential renewal is distinct from native application, model entitlement,
  quota and successful work; none is reported as proven by another.
- Renewal requiring the server cannot be promised while the Computer cannot
  reach it. This does not itself change the agreed behavior for existing work
  on ordinary server-link loss.
- A failure to apply renewed credentials through the supported native path
  returns as a feasibility constraint, not a silent change of credential,
  harness implementation or authentication scope.

### REQ-18. Publish same-record credential changes without harness intervention

When an applied authentication record's key is replaced or its ChatGPT account
is reauthenticated, updated credential material is published automatically to
the executing Computer without manipulating the live harness.

**Acceptance criteria**

- Publication updates the credential material associated with the harness's
  applied authentication record; it does not switch to a different saved
  selection before the REQ-1 next-start boundary.
- Same-record credential changes do not trigger harness termination, restart,
  re-login or a forced credential reload.
- Successful credential publication is distinguished from actual adoption by a
  live process. The native harness may retain cached credentials until its own
  supported reread behavior occurs.
- Publication failure or a disconnected Computer is not reported as successful
  application. Immediate delivery to an unreachable Computer is not promised.
- ChatGPT refresh credentials remain server-held under REQ-17. No credential
  bytes are exposed in conversation records or ordinary diagnostic output.
- Native credential material may remain in the Computer's existing saved
  storage across power-off or execution-environment replacement. The initial
  integration does not require ephemeral-only native credential storage.
- Native files containing API keys or short-lived ChatGPT tokens are
  access-restricted but may contain plaintext credentials. Server-side encrypted
  storage does not imply encryption of these native files, crash-proof erasure
  or isolation from arbitrary code running under the same execution identity.
- Execution credential delivery uses the existing explicitly trusted internal
  component-network boundary. The initial integration does not add transport
  encryption or claim secrecy from observers of that network.
- This delivery remains limited to currently admitted exact executions. It
  does not add a public credential endpoint, relay or credential-bearing
  browser/conversation payload.

## Fixed Constraints

- Initial Codex integration uses ACP v2 without a silent v1 fallback.
- Existing private Agent, account session, current membership, saved
  authentication scope, and execution authority boundaries remain enforced.
- Secret material is not exposed to chat text, general form answers, unauthorized
  users, or ordinary operational logs.
- Existing data and accepted lifecycle/authentication identities are preserved;
  this feature is not permission to reset installations or replace owner data.
- Cloud Computer setup should not depend solely on a harness already being
  installed. Compatible existing Codex installations are also in initial scope
  under REQ-14, with owner-controlled selection under REQ-15. Eligibility mechanisms
  remain pending.
- No deployment, merge, provider provisioning, external-effect replay, or new
  versioning product is authorized by this Requirements draft.

## Open Assumptions and Validation Limits

- Real Codex support for steering boundaries, message updates, cancellation,
  context restoration, and request choices must be demonstrated. A conflict that
  changes user-visible outcomes returns for a material decision.
- Official SDK smoke checks, static producer analysis, and fixture-based UI
  exercises are not evidence of working end-to-end conversation integration.
- Record count, minor labels, and equivalent reversible presentation details can
  be considered together without reopening confirmed product flows.
- Connector ownership/lifetime, conversation authority, credential delivery,
  interfaces, storage mechanisms, migration, and technical failure settlement
  belong to subsequent ADR/Design decisions, not implied Requirements choices.
- Complete snapshot rebuild can lose previous product facts that are absent
  from the current snapshot. Native replay is not guaranteed to recreate
  independently admitted inputs, form/permission answers or delivery evidence.
- Remaining installation and maintenance choices include saved-context
  compatibility and external-installation eligibility/configuration. Preparation
  scope does not itself authorize interruption of work or a user-facing release
  selector; technical maintenance mechanisms are recorded in the snapshot ADR.

## Confirmation

The complete revision 1 requirements and a Korean review copy were presented to
the requester on 2026-11-08. The requester directed collaborative structural
design to continue against the established product scope, accepted the ordered
five-topic architecture discussion, and explicitly selected the first lifetime
alternative on 2026-11-08. That choice confirms the applicable continuity and
recovery outcomes in REQ-1, REQ-5, REQ-10, REQ-11, and REQ-12; it is not approval
of every subsequent mechanism or a completed implementation Design. The remaining
requirements preserve the previously confirmed product intent. No new product
scope is inferred from the instruction to continue.

For revision 2, the requester directed that compatible existing harness
installations remain viable, favored download-based preparation for cloud
Computers, and requested an update policy on 2026-11-08. These directions extend
the preparation and maintenance scope without selecting discovery, acquisition,
verification, or update-application mechanisms. Other requirements retain their
existing confirmation boundaries.

For revision 3, the requester explicitly selected initial support for an existing
Codex installation available inside the executing Computer on 2026-11-08, rather
than deferring actual selection to a later release. REQ-14 records that scope.
The allowed component composition, discovery, configuration authority/location
and trust mechanisms remain subsequent decisions. This confirmation does not
approve those mechanisms or the complete implementation Design.

For revision 4, the requester selected Agent-owner product configuration rather
than operator-only startup configuration on 2026-11-08. REQ-15 records the actor,
scope and retained-selection outcomes. Server authority and Connector application
are recorded in the corresponding ADR decision. Initial defaults, live-change
application and detailed configuration UX are not approved by this confirmation.

For revision 5, the requester selected saving supply-setting changes during live
execution and applying them only at the next actual harness start on 2026-11-08.
REQ-15 records the retained running configuration, truthful saved-versus-applied
presentation and absence of save-triggered restart. Initial launch triggers,
defaults and a separate apply-now/restart feature remain unselected.

For revision 6, the requester selected on-demand harness startup at user text
submission on 2026-11-08, rather than eager startup with Computer preparation.
REQ-1 records that observable trigger. This confirmation does not select
automatic Computer power-on, idle retirement or an autonomous crash-restart
policy and does not override the existing non-replay/retirement conditions.

For revision 7, the requester selected retaining the normal harness after task
completion and excluding idle-time automatic retirement from the initial
integration on 2026-11-08. REQ-1 records this continuity policy. It does not
override termination/revocation/failure handling or authorize automatic restart
of a failed harness.

For revision 8, the requester limited native-history replay by default to a
session's first load and excluded native-side missed-event detection/recovery
and independently launched external-harness activity. REQ-11 and REQ-12 retain
already-captured work and product-history continuity within that boundary.
Local event-copy storage and identity are recorded in the ADR. The requester
described a hypothetical full replay/replacement reload but explicitly stated
that it is not yet a product feature; this revision adds no reload UI or API.

For revision 9, on 2026-11-09 (KST), the requester selected Connector-to-server
captured-event catch-up, explicit unavailable-snapshot failure and server rebuild
when the old snapshot is gone. The requester also selected paired raw and
service-normalized retention, with normalization used for rendering and source
data used for debugging/re-normalization. REQ-16 records those outcomes.
Snapshot/event cursor and paired-storage mechanisms are recorded in the ADR.
The product reload feature remains excluded. The removal boundary for product
facts not reconstructible from incoming ACP events remains an explicit open
question, not permission inferred from the rebuild selection.

For revision 10, on 2026-11-09 (KST), the requester explicitly corrected the
proposed preservation boundary and selected complete reconstruction of the
session's server conversation data. REQ-16 therefore includes independently
admitted inputs, form/permission responses and delivery evidence in the
replacement boundary; facts absent from the new snapshot need not survive.
This does not authorize deletion of unrelated Agent/account/authentication or
Computer data, or replay of historical effects.

For revision 11, on 2026-11-09 (KST), the requester selected recent initial
history loading, additional older-record retrieval, live normalized screen
updates and current-state re-reading on browser reconnection. REQ-11 records
these outcomes. The corresponding ADR excludes a separate durable browser
change feed from the initial design; Connector-to-server catch-up remains
unchanged.

For revision 12, on 2026-11-09 (KST), the requester selected short server product
acceptance after input retention, with an operation reference and separately
reported harness preparation/acceptance/execution evidence. REQ-2 records this
response boundary. This does not select automatic command replay, a persistent
Connector dispatch journal or an indefinite work queue.

For revision 13, on 2026-11-09 (KST), the requester selected recovery from
server-held records, captured events and a surviving Connector's command state,
without adding a separate persistent Connector dispatch journal. REQ-12 records
the resulting restart-recovery limit: unproven prior command outcomes remain
uncertain, and absent state never implies permission to replay.

For revision 14, on 2026-11-09 (KST), the requester selected stopping new
effect-bearing input and controlled harness shutdown on local recording
failure, then explicitly limited that selection: skip the specialized handling
if it substantially increases implementation work. REQ-12 records that
qualification; comprehensive storage-failure management is not required.
Known recording errors still cannot be reported as successful persistence.

For revision 15, on 2026-11-09 (KST), the requester selected server-owned
ChatGPT refresh and delivery of required short-lived token material to Computers.
REQ-17 records centralized renewal and the absence of a new single-consumer
restriction. Native token application remains subject to feasibility validation;
this confirmation does not approve an adapter fork or a new native bridge.

For revision 16, on 2026-11-09 (KST), the requester explicitly selected option 1:
save a changed authentication selection while preserving the running harness's
previously applied authentication, and apply the new selection at the next
actual harness start. The requester rejected adding selection-triggered
shutdown logic. REQ-1 records the saved-versus-running distinction and deferred
application. Underlying authority withdrawal and same-record credential changes
remain distinct unresolved cases.

For revision 17, on 2026-11-09 (KST), the requester selected automatic
credential-file replacement for same-record key replacement/reauthentication,
without intervention in the harness. REQ-18 records automatic publication and
the lack of forced live-process adoption. The ADR records the file mechanism.
The deferred different-record selection policy remains unchanged.

For revision 18, on 2026-11-09 (KST), the requester selected option A after
reviewing native credential-store modes: use the Computer's existing saved
volume/PVC for native credential files, without a separate ephemeral mount.
REQ-18 records persistence and plaintext-residue limits. ChatGPT refresh tokens
remain server-only, and file publication still does not prove live adoption.

For revision 19, on 2026-11-09 (KST), the requester selected retaining the
existing trusted-network HTTP component boundary for execution credential
delivery after reviewing its lack of transport confidentiality. REQ-18 records
that trust limit without weakening exact execution/scope admission or permitting
public credential distribution. No new relay or encryption protocol is selected.

For revision 20, on 2026-11-09 (KST), the requester selected retaining the entire
current captured snapshot until replacement/deletion, including events already
stored by the server. REQ-16 excludes automatic prefix pruning/rotation,
compression and expiry in the initial integration while retaining the existing
capacity and recording-failure limits. No reload product feature is added.

For revision 21, on 2026-11-09 (KST), the requester selected A: clear prior
session data before filling the replacement, rather than retain it for a staged
atomic swap. REQ-16 records that loading/failure boundary and the incomplete
source's input-admission limit. The requester also corrected internal processing
terminology in normal UI; show ordinary conversation loading/error affordances,
not a reconstruction status label.

For revision 22, on 2026-11-09 (KST), the requester selected A: managed
preparation is the default for an Agent without saved supply settings. REQ-15
records first-use behavior without adding UI-read startup or Computer power-on.
Explicit existing-installation selection retains REQ-14's truthful failure and
no-silent-managed-fallback boundary. Artifact-cache cleanup remains a separate
policy choice.

For revision 23, on 2026-11-09 (KST), the requester selected A: automatically
clean unused superseded managed installation cache only after a complete
replacement exists. REQ-1 records the protected active/preparing/latest eligible
files and the separation from external installations, conversation sources,
native context, authentication and workspace. No numeric retention policy or
general storage-management subsystem is selected.

For revision 24, on 2026-11-09 (KST), the requester confirmed allowing deletion
when an old authentication remains only in a running harness after selection
changes. REQ-1 preserves the existing saved-reference protection without adding
runtime-only deletion blocking. Deleted records receive no new renewal/delivery;
already-delivered credentials are not promised immediate provider revocation.
