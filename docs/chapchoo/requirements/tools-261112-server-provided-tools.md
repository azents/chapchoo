---
title: "Server-Provided Tools Requirements"
created: 2026-11-12
updated: 2026-11-12
document_role: primary
document_type: requirements
snapshot_id: tools-261112
tags: [tools, mcp, connector, harness]
---

# Server-Provided Tools Requirements

- Snapshot: `tools-261112`
- Document reference: `tools-261112/REQ`
- Confirmation: Confirmed on 2026-11-12 after the persistence-responsibility
  correction; clarified on the same date that the harness owns session records
  and the server copies them. The first text-publication tool and default-output
  policy were also confirmed on that date. The subsequent receipt, ordering,
  retained-input preservation and final record-presentation agreements are
  captured in REQ-10 through REQ-13. Complete Design approval and implementation
  remain separate.

## Problem

Chapchoo has authenticated Connector communication and native conversation
observations, but no common capability through which a harness can discover and
invoke Chapchoo-provided tools. The harness already owns session history; the
server copies captured session records for storage and reads. Tool integration
must preserve that ownership. Session history and the persistent state managed
by a tool are different responsibilities; one common persistence or recovery
policy cannot stand in for every tool's domain contract.
The current main conversation also displays native assistant text directly.
The first real tool must enable intentional text publication without mixing
that publication with automatic native output.

## Primary Context

### Primary System Outcome

A harness associated with an authorized Chapchoo Agent and conversation can
discover and invoke Chapchoo-provided tools and receive truthful outcomes that
participate in the harness's own session history and its existing server-copy
model. A stateful tool's owning service remains responsible for its persistent
state and any separate query tools. The first real consumer is explicit text
publication to the originating product conversation; default native assistant
text remains recorded but is not automatically published as Agent speech.
Additional server tools share the provision and authority boundary without
being forced into a common domain-state store or recovery protocol.

## Supporting Scenarios or Effects

- New and exactly resumed native sessions can use the provided tools.
- Temporary disconnection and response loss do not become permission to repeat
  a potentially completed mutation.
- Session response records remain distinguishable from current domain state
  and model-facing previews.
- A tool that needs persistent state, queries, or operation recovery uses the
  contract of the service that owns that state.
- A later feature can consume this foundation without changing its ownership
  boundaries.
- A user receives intentionally published Agent text without automatic native
  utterances or a duplicate fallback becoming another user-facing message.
- Native output remains available for inspection even when not published.

## Goals

- Provide a common MCP-based integration for Chapchoo-owned server tools.
- Preserve current Agent, conversation, execution, and ownership boundaries.
- Distinguish execution outcomes from transport and presentation observations.
- Preserve harness-owned tool-response history and server copying without a
  second session author or takeover of domain persistence/query responsibilities.
- Bound resource use and expose unsupported or unavailable behavior honestly.
- Provide explicit text publication as the first real tool outcome while
  preserving inspectable native output separately from user-facing publication.

## Non-Goals

- File upload, download, binary exchange, attachment publication, filesystem
  materialization, object storage, or resumable file-transfer design.
- Arbitrary cross-Agent/cross-channel messaging, Channel Work, routines, or a
  generic messaging/integration platform.
- Product progress/completion/input-wait declarations, quiet-completion controls,
  automatic continuation or native restart behavior. The initial delivery is
  text publication plus the confirmed default-output policy only.
- Selecting exact full-history record presentation before the final UI agreement.
- Reconstructing or importing chat messages from native session history in the
  initial delivery. Exact native-session resumption and existing captured-history
  copying remain supported under their established contracts.
- A third-party MCP marketplace, arbitrary remote-server management, or a
  dynamic plugin platform.
- Adding another harness, ACP v1 fallback, or support for every MCP revision.
- Universal exactly-once execution of external effects, automatic rollback of
  completed work, unlimited result size, or indefinite retention.
- Automatically adding an application-level versioning system.
- A mandatory common result archive, result-query tool, durable task journal,
  idempotency key, or restart-recovery policy for every server tool.
- A separate server/relay-authored tool-response history, or a server session
  write as a prerequisite for returning a tool response.

## Requirements

### REQ-1. Common provision of server tools

Provide Chapchoo-owned tools through a common harness-facing contract rather
than coupling the integration to one messaging operation.

**Acceptance criteria**

- At least two distinct representative tool behaviors can be provided through
  the same integration in verification.
- Each available tool has an unambiguous name, input contract, and result/error
  meaning.
- An additional tool does not require changing ordinary conversation-text
  delivery or inventing a separate harness control mechanism for that tool,
  beyond the first publication behavior explicitly required by REQ-8 and REQ-9.

### REQ-2. Current authority and isolation

Tool discovery, invocation, and session-record access respect current
authorization for the originating Agent and conversation. A domain query tool
additionally enforces the authority of its owning service.

**Acceptance criteria**

- Changing model-supplied identity fields cannot grant another Agent's or
  conversation's access.
- Revoked, expired, superseded, or mismatched authority cannot authorize new
  work or disclose protected records or domain state.
- Listing a tool does not replace invocation-time authorization.
- Server/component operational credentials are not exposed as tool output,
  ordinary transcript content, or general native environment credentials.

### REQ-3. Reliable availability across native lifecycle

The initial supported harness can use tools in both a new native session and an
exactly resumed one. Connection replacement must not silently change the
conversation or tool target.

**Acceptance criteria**

- New and resumed session verification confirms the effective Chapchoo tools,
  not merely successful submission of configuration.
- Unsupported protocol or failed provision remains distinguishable from an
  empty authorized catalog or a successfully available tool.
- A stale definition, name collision, or changed authorization cannot silently
  execute a different operation or target.
- Failed restoration does not create a replacement native conversation.

### REQ-4. Honest invocation outcomes and tool-specific recovery

Keep non-admission, accepted work, known completion or failure, cancellation
observation, and uncertain effects distinguishable as applicable.

**Acceptance criteria**

- Lost response, timeout, caller cancellation, or connection replacement alone
  is not reported as proof that an effect did not happen.
- The common layer does not automatically replay a potentially effective
  invocation merely because its response was lost.
- Retry, deduplication, cancellation, and recovery guarantees follow the
  individual tool's declared behavior and owning service. The common layer
  does not invent these guarantees or impose one durable mechanism on all tools.
- Reading a session record does not execute the tool. A separate domain query
  is an invocation with its own contract, not replay of the original mutation.
- Protocol/admission errors and a tool's own execution errors remain
  distinguishable.
- Cancellation does not promise rollback or termination without evidence.

### REQ-5. Harness-owned session history and domain-state ownership

The harness manages its session records, including tool responses. The server
copies those records under the existing session-storage ownership model.
Determine additional persistence and query behavior according to the individual
tool's purpose.

**Acceptance criteria**

- A response-only tool can rely on its MCP response in session history without
  a separate common result entity or query operation.
- The common server-tool service and local relay return tool responses; they
  do not independently author a second session history or require a separate
  session-record commit before delivering a response.
- For tools that manage persistent state, the owning domain service stores and
  manages that state and provides separate query tools where needed.
- Session response history is not presented as the current authoritative
  domain state. A domain-state change does not rewrite an earlier MCP response.
- Supported text, structured data, errors and tool identity follow the harness's
  session representation. Server copying preserves the supplied session records.
  Presentation previews and captured source records remain distinguishable;
  byte-for-byte MCP wire archival is not required.
- Oversize, unsupported, expired, or unavailable results are explicit rather
  than fabricated as a complete successful result.
- Session-record lifetime and domain-state lifetime are separate. Any
  additional result lookup, durable operation recovery, or longer retention
  requires the relevant tool's contract, not a universal foundation promise.
- Existing session copying, source ordering, restoration and record lifecycle
  remain intact. Missing copied history is not proof of tool non-execution.
- This requirement does not introduce file exchange or binary-result storage.

### REQ-6. Bounded operation and lifecycle ownership

Bound concurrent work, input/result size, memory, execution wait, and cleanup.
Availability problems must not silently discard accepted work or affect
unrelated authorized calls.

**Acceptance criteria**

- Overload has a defined admission outcome; accepted work is not silently lost
  merely to free capacity.
- A caller disappearing does not erase the distinction between pending,
  completed, and uncertain work.
- Shutdown, connection replacement, and credential changes have defined
  effects on new admission and existing work.
- Limits are explicit and verifiable; no particular numeric value is approved
  by these Requirements.

### REQ-7. Verifiable common contract and initial publication

Verify the common provision/result contract independently and exercise the first
real publication behavior without implicitly adding unrelated domain tools.

**Acceptance criteria**

- Representative response-only and persistent-state tool behavior cover
  success, error, isolation, response recording, response loss, and reconnect.
- Stateful fixtures verify the domain service's storage/query contract;
  retry/deduplication recovery is tested only against guarantees that the tool
  actually declares.
- Model-free boundary fixtures provide deterministic failure evidence.
- The selected actual harness combination is checked for effective provision
  and supported result behavior before compatibility is claimed.
- Existing text delivery, steering, permissions, and exact restoration keep
  their established contracts.
- Publication and default-output verification covers REQ-8 and REQ-9; user
  chat receipt and ordering verification covers REQ-10/REQ-11, including native
  delivery failure and delayed capture. The representative tool-behavior matrix
  does not require publishing artificial
  echo/time/self-inspection tools as a production catalog.

### REQ-8. Explicit text publication to the originating conversation

The first real server-provided tool lets the Agent intentionally publish text
to the product conversation from which its authorized request originates.

**Acceptance criteria**

- An authorized Agent can deliberately publish user-facing text to its bound
  product conversation; model-supplied target identities cannot redirect it to
  another Agent or channel.
- Publication acceptance, known rejection/failure and uncertain outcome remain
  distinguishable. A lost response alone cannot trigger automatic republication
  or be presented as proof that nothing was sent.
- The actual tool invocation remains in harness-owned history and its existing
  server copy. Inspecting that record does not execute the tool again.
  Independently managed accepted chat data does not replace harness history.
- Accepted chat data and session data are independently managed for both
  message directions. Managing chat messages does not make tool execution
  history the chat's message source or change the harness's session ownership.
- The initial content is text; attachments and binary/file exchange stay excluded.
- Publication alone does not implicitly establish product-work completion,
  user-input waiting or permission to start another native turn.
- The initial tool publishes text only. It has no progress, wait, finish or
  quiet-completion action and does not introduce product-work state tracking.

### REQ-9. Preserve default native text without automatic publication

Intentional publication, rather than native assistant text output, is the
source of Agent speech on the main product conversation.

**Acceptance criteria**

- Default assistant text, including final native text, remains in harness-owned
  history and the existing captured-copy path. Suppressing automatic publication
  does not delete, rewrite or stop recording that text.
- Default native text is not automatically published to the main conversation
  or as a user message/notification. Explicitly published text is the Agent's
  user-facing message, not an additional duplicate native-text message.
- The chat surface presents conversation messages, not tool-call records.
  Preserving detailed tool history does not require exposing it in the chat;
  its item mapping follows REQ-13.
- If no publication tool was called, or publication failed, native text is not
  promoted as a fallback message.
- No publication by itself does not establish quiet successful completion.
  Native idle/turn end does not establish product-work completion or input wait.
- User input retains its established native delivery contract; chat receipt
  follows REQ-10. Current permission/form interaction and execution/failure
  status retain their established behavior. This requirement does not hide those
  controls or failures merely because they are not published Agent text.
- Native output stays inspectable. Native output, publication-tool items and
  other full records follow the confirmed mapping in REQ-13.

### REQ-10. User chat receipt at server input acceptance

A user message becomes accepted chat data when Chapchoo accepts the submitted
input, independently of subsequent native delivery.

**Acceptance criteria**

- A complete, authorized input accepted by the server is retained as a user
  chat message without waiting for native insertion/injection or history copying.
- Pending, failed or uncertain native delivery does not erase that accepted
  chat message. Temporary delivery state and accepted chat data keep their
  distinct responsibilities.
- Rejected or incomplete uploads are not reported as accepted chat messages.
- Chat receipt does not claim native consumption, model reading or completed
  work, and retaining the message does not dispatch the input again.
- This establishes receipt and delivery independence, not indefinite retention,
  a new deletion UI, reconstruction or a new native delivery/retry policy.

### REQ-11. Stable chat order by completed server receipt

User and Agent messages share a stable conversation order based on completed
server chat receipt.

**Acceptance criteria**

- Both message directions use the same server chat receipt order.
- Delayed native delivery, captured-history copying, sender clocks and native
  invocation-start order do not move an already accepted chat message.
- Concurrent Agent publications follow their actual completed chat receipt,
  not an inferred order of when the native tool calls started.
- Session records retain their own source ordering. Chat ordering does not
  rewrite those records or imply that the two histories have identical order.
- This selects a data-order contract, not exact full-record UI presentation
  or a new application versioning mechanism.

### REQ-12. Preserve retained original user inputs when applying the feature

When the new chat behavior is applied, carry existing non-deleted user inputs
whose original text is still retained by the server into chat data.

**Acceptance criteria**

- Retained original pending/failed user input text becomes chat data rather than
  being omitted solely because it was accepted before the new behavior.
- Existing native delivery state and authority remain unchanged. Moving stored
  content does not submit the input again or execute new native work.
- Deleted bodies and already absent inputs are not reconstructed. Native session
  history is not imported to manufacture a complete historical chat.
- The transferred messages have stable chat receipt order before newly admitted
  messages; the system does not invent an unrecorded historical commit order.
- Existing session records and privacy/ownership boundaries remain intact.

### REQ-13. Separate chat messages from inspectable full records

Confirmation scope: message-only chat and expandable actual history are agreed.
The requester subsequently selected the uniform-row visual treatment and refined
its role distinction and normalized message preview. This establishes the
presentation outcomes below, not approval of the complete Design or implementation.

The chat presents conversation messages. The full-record surface presents actual
harness observations with expandable item details.

**Acceptance criteria**

- Chat displays user messages and intentionally published Agent messages, not
  default native output or general tool history.
- Full records retain actual native user input, default/final output, thoughts,
  other tools and request observations as inspectable items. A chat message
  never observed by native does not acquire a fabricated native-input record.
- Publication is one tool-call item with readable sent text and the actual
  observed result in its expanded details; original input/output remain
  inspectable. Publication does not create an extra native assistant bubble.
- Default/final native output is separately identifiable and expandable. Real
  native output is preserved even when its text resembles a published message.
- Within full records, user messages, intentional Agent messages and model output
  are distinguishable without visible message sender-name labels. Model output has no Agent-name
  prefix. Message emphasis follows the current primary color.
- Collapsed user and intentional Agent message items preview the normalized
  message-body beginning. Agent publication does not substitute a generic action
  title for the body preview. Expanded text and original evidence remain available.
- Existing permission/form interaction remains available where authorized;
  the full-record copy remains a record view rather than a second live responder.
- Missing responses do not fabricate publication success or known non-delivery.

## Fixed Constraints

- File transfer is deferred to a later design.
- Persistence is tool-dependent: session MCP response records may suffice;
  persistent domain state and associated query tools belong to its owning service.
- The harness owns session history. The server copies captured records; adding
  server tools does not create another session-history author.
- The initial real tool is explicit text publication; default native text is
  preserved without automatic publication or fallback. Full-record item
  presentation follows REQ-13.
- Connector-to-harness communication remains ACP v2; server-to-Connector
  communication is a separate boundary.
- The existing initial harness is Codex. Other harness research is reference,
  not an instruction to add support in this snapshot.
- Agent and conversation remain distinct; one channel does not share its
  session with other channels.
- No implementation, merge, or deployment is authorized by these Requirements.

## Remaining Scope Boundaries

- Harness/session compatibility is a verification responsibility, not permission
  to redesign the session's author, retention or replication lifecycle.
  Common invocation-response mapping must preserve supported harness semantics.
- Concrete domain tools may have additional recovery requirements when they
  enter scope. No generic restart-recovery promise applies to all tools.
- The first publication tool's exact contract and recovery guarantees remain
  design work, including the precise observable meaning of publication success.
  Accepted chat data and session data are separate managed records for both
  user-to-Agent and Agent-to-user messages. Receiving a user message and native
  consumption remain distinct facts. The initial delivery does not reconstruct chat
  messages from session history. Exact native-session resumption and existing
  captured-history copying are separate and retain their established contracts.
  REQ-10 selects user chat receipt at server input acceptance. Concrete receipt
  transactions, Agent publication success semantics, retention periods and
  deletion behavior remain design work.
  The requester selected publication-only scope: work-state declarations and
  automatic continuation are deferred rather than pending additions to this
  delivery.
- Full-record item presentation is confirmed as REQ-13. The complete Design
  must preserve that distinction without adding new history authors or controls.
- Any new user-visible behavior or expanded scope returns to Requirements
  confirmation before a corresponding technical decision.

## Open Assumptions

No new third-party tool integration, end-user tool management UI, live catalog
hot-reload guarantee, binary media support, or automated task-resumption policy
is assumed. Adding any of these requires scope confirmation.

## Confirmation

Confirmed by the requester on 2026-11-12 after reviewing the corrected scope and
responsibility split. File transfer is deferred. Persistence depends on the tool:
a session MCP response may suffice, whereas a persistent tool's owning service
manages its state and separate query tools. The same-day ownership clarification
retains harness-managed session history and server copying; it does not authorize
separate server/relay-authored response records. Proceed with provision/authority,
invocation-response compatibility, catalog and operational-boundary design; no
common domain-state archive or uniform recovery mechanism is required.

The same-day extension confirms explicit text publication to the originating
conversation as the first real tool and confirms preservation of default native
text without automatic user-facing publication or fallback. Native turn end
remains distinct from product-work completion. Exact full-record UI presentation
is reserved for final agreement. This confirmation does not select tool names,
combined/separate completion controls, automatic continuation, cross-channel
publication, attachments, implementation, merge or deployment.

The requester subsequently selected publication-only initial scope. Explicit
progress/input-wait/completion/quiet-completion declarations and continuation
remain outside this delivery; a future need does not enlarge this snapshot.

The subsequent source discussion covers both message directions. On the same
date the requester selected no session-based chat reconstruction for the initial
delivery. This excludes historical chat import/reconstruction, while preserving
exact native resumption and existing captured-history copying. It does not
independently select server-owned message state, retention behavior, a backup
feature or publication success semantics.

The subsequent clarification confirms independent management of accepted session
data and chat data across both message directions. Shared temporary delivery
data serves delivery rather than becoming the permanent record of either.
Receipt timing, failure, lifetime and publication-success details remain design
work; the existing harness-owned session and server-copy boundary is preserved.

On the same date the requester selected server input acceptance as user chat
receipt (REQ-10). Accepted user messages remain chat data even if native delivery
is pending, fails or has an uncertain outcome. Native delivery and session
ownership retain their established contracts.

The requester also selected completed server chat receipt as the stable order
for both user and Agent messages (REQ-11). Delayed capture and sender clocks do
not reorder accepted chat. The subsequent final presentation agreement is REQ-13.

The requester confirmed moving existing retained, non-deleted original user
inputs into chat data when applying the feature (REQ-12). This is preservation
of server-held original content, not session-based chat reconstruction, renewed
native delivery, or authorization to modify a live database during design.

The final item-presentation agreement confirms REQ-13: message-only chat and
expandable actual harness records, with publication represented as one tool
item. Complete Design approval and implementation remain separate.
