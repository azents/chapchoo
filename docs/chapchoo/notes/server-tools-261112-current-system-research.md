---
title: "Server Tools: Current Session, Capture and Authority Research"
created: 2026-11-12
tags: [tools, mcp, harness, connector, research]
---

# Server Tools: Current-System Research

Presentation clarification: earlier broad assent did not establish detailed visual
authority. The requester subsequently selected the uniform-row option and refined
role icons, current-primary message rails, sender-label removal and normalized
body previews. REQ-13/D14 and Design revision 2 now record that explicit selection.
Static component renders support visual feasibility, not native/product acceptance
or whole-Design approval.

This is an evidence report, not an implementation Design or new authority.
It supports [tools-261112/REQ](../requirements/tools-261112-server-provided-tools.md)
and the [snapshot ADR](../adr/tools-261112-server-provided-tools.md).

Repository baseline: `83a41877867f3602f741da0a9de819fa52f76929`, also verified
as `origin`'s advertised `main` head during this investigation. No application
implementation was changed. No model, production, deployment or credential
operation was performed.

## 1. Ownership Before New Server Tools

The harness/native execution layer manages the actual conversation. Server
product conversation identity, captured history and normalized views do not make
the server a second author of that native session.

| Responsibility | Current owner and evidence |
| --- | --- |
| Product Agent/conversation identity and authorization | Completed server Store operations; selected conversation association is distinct from native session identity |
| Physical adapter process, ACP session handle, exact start/resume and retirement | `chapchoo-harness::NativeOwner` and its actor-owned `State`/`Live` |
| Native persistent context | Native process using the explicit saved `CODEX_HOME`; separate from installations, workspace and captured copy |
| Original incoming ACP observation copy | Local `CaptureLog`, intercepted before typed SDK decoding |
| Copying retained observations | Connector `CaptureSync`, through authenticated component capture RPC |
| Server storage and reader projection | Raw/normalized event pairs plus materialized conversation record fields |
| Persistent state changed by a domain tool | The domain service; session history is not that state's current source of truth |

The server stores a copy of supported ACP observations, not a mirror of every
native state file, and not an independent original MCP network archive.

## 2. Actual Record Path

```mermaid
flowchart LR
    N["Native session / Codex adapter"] -->|"incoming ACP events"| I["Harness ingress before SDK decoding"]
    I --> C["Local CaptureLog: source.jsonl + manifest"]
    C -->|"finite retained pass"| S["Connector CaptureSync"]
    S -->|"authenticated chunks"| R["ConnectorCapture RPC"]
    R --> V["Service normalization"]
    V --> P["Store: raw + normalized + fields + cursor"]
    P --> Q["Owner record/field reads"]
```

### Harness ingress and local copy

[`runtime.rs`](../../../rust/crates/harness/src/runtime.rs) builds the native
command with explicit storage and a `CODEX_PATH` wrapper. `launch` wraps incoming
stdout lines, calls `capture` before passing them to the ACP SDK, and records
supported `session/update`, `session/request_permission` and session-scoped
`elicitation/create` messages. It does not capture all protocol traffic: login
requests/results and outgoing setup configuration are outside this filter.

[`capture.rs`](../../../rust/crates/harness/src/capture.rs) retains the original
UTF-8 event line, explicit ordinal and occurrence identity in `source.jsonl`.
Its manifest holds capture snapshot and optional native session ID. Appending
checks native identity and preserves original unknown fields. `bind_native`
checks events received before the setup result against the actual returned ID.
A `CapturePass` fixes a complete-record end; later appends cannot extend that
transfer indefinitely. The copy is not an outgoing command journal.

### Connector transfer

[`capture_sync.rs`](../../../rust/apps/chapchoo/src/connector/capture_sync.rs)
reads the native owner's existing source. Absence does not start native work.
It obtains current server-copy state, binds the exact local source when needed,
checks native identity, resolves the durable server occurrence and transfers
remaining original events in at most 32-KiB pieces. A captured event is bounded
to 16 MiB. `Complete` names the finite pass boundary, not the end of the session.

A later pass can repeat retained observations after a lost acknowledgement.
This is record-copy recovery, not permission to repeat a native input or tool
invocation. A missing/corrupt local copy is not silently repaired from invented
records. Capture transfer failure does not automatically stop/restart native.

### Server publication and reads

[`conversation_capture.rs`](../../../rust/crates/rpc/src/conversation_capture.rs)
authenticates component requests and assembles complete events. The
[`ConversationService::ingest`](../../../rust/crates/service/src/conversation.rs)
checks the source, normalizes supported ACP semantics, then calls completed
Store publication with another source-authority check.

[`records.rs`](../../../rust/crates/persistence/src/conversations/records.rs)
commits original raw bytes, normalized bytes, projected fields and source cursor
in one completed operation. Identical occurrences do not append twice; changed
bytes/order conflict. This atomicity is about copying one observed event, not
about committing a domain tool's external effect with session history.

[`normalize.rs`](../../../rust/crates/service/src/conversation/normalize.rs)
already maps `ToolCallUpdate.raw_input` and `raw_output` into independent
`TOOL_INPUT` and `TOOL_OUTPUT` fields. Omission preserves a field; explicit null
clears it. An update containing only status does not erase tool output.

[`conversations/records.rs`](../../../rust/crates/rpc/src/conversations/records.rs)
exposes bounded source-qualified owner record/field reads. The web's
[`ToolRecordDetails`](../../../typescript/apps/web/src/features/computers/chat/components/ToolRecordDetails.tsx)
already renders retained structured tool input/output. No new history UI or
record writer is necessary merely to make a Chapchoo-owned tool an MCP tool.

## 3. MCP Result Reporting in the Inspected Adapter

Reference: Codex ACP **2.6.0**, commit
[`f3208913693bd4d5171d6c6b9be5ec1ff96c6431`](https://github.com/agentclientprotocol/codex-acp/tree/f3208913693bd4d5171d6c6b9be5ec1ff96c6431).
Seven relevant local reference files were byte-compared with official raw files
at that exact commit; all matched. This verifies source provenance, not the
package currently applied by a running native process.

The source path is:

1. [`McpToolReporter`](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/tool-calls/reporters/McpToolReporter.ts)
   emits `{server, tool, arguments}` as input and `{result, error}` as opaque
   output for a Codex MCP tool-call item.
2. [`AcpToolCallRenderer`](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/tool-calls/AcpToolCallRenderer.ts)
   maps those to `rawInput` and `rawOutput` independently of readable content.
3. [`AcpV2SessionUpdate`](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/AcpV2SessionUpdate.ts)
   preserves those fields when emitting `tool_call_update` in ACP v2.
4. Chapchoo's existing local capture, server normalization and owner reads carry
   that native event/output through the path above.

Consequently, lack of a dedicated MCP transport recorder does not establish a
missing session-result path. The supported contract is harness session history,
not byte-for-byte preservation of every MCP HTTP envelope or a server-produced
response that native never received. If a response is lost before native obtains
it, history can reflect the native error/observation; it must not be backfilled
with an invented successful result.

## 4. Start, Resume and Reconnection

[`storage.rs`](../../../rust/crates/harness/src/storage.rs) provides an explicit
saved root with separate `state`, `workspace`, `capture`, `home`, `tmp` and
installation classes. Native launch uses cleared ambient environment and an
explicit `CODEX_HOME` pointing to its retained state.

[`setup.rs`](../../../rust/crates/harness/src/runtime/setup.rs) selects new or
exact retained context. Missing expected source does not fall through to a new
conversation. [`protocol.rs`](../../../rust/crates/harness/src/runtime/protocol.rs)
uses strict ACP v2 and exact new/resume SDK operations; already captured history
is not requested again. A failed resume does not create a fresh session.

`State::start` can return an already-live exact session without launching,
authenticating again or resending setup configuration. Thus simply editing a
future MCP declaration cannot update an already-live native session. Effective
provisioning and a permitted rollout boundary must be verified explicitly;
no implicit restart is authorized.

[`connector.rs`](../../../rust/apps/chapchoo/src/connector.rs) composes the native
owner outside operational reconnect/key refresh and finishes it when the
Connector loop exits. Changing an operational stream is not changing the native
session. The local MCP capability chosen in ADR-D4 is a future mechanism, not
something already implemented by current Connector authentication.

The inspected adapter accepts MCP URL/header declarations for new and resumed
session requests. Its config normalization can skip requested names already
provided by existing config; declared success is not sufficient to establish
that the intended tool endpoint won. It also registers resumed-thread event
tracking before the resume response because native may produce events during
resume. Neither fact supports a blanket native-ready authorization gate.

## 5. Identity and Authority

These identities serve different roles:

- **Agent ID and product conversation ID:** authorized product target, already
  present in sealed `SetupCommand` before a new native ID exists.
- **Connector session/execution, Control and connection IDs:** current component
  authority and route. The component credential is not an arbitrary Agent grant.
- **Native session ID:** exact native restoration and event-source identity.
- **Capture snapshot/occurrence/ordinal:** copy identity, deduplication and order,
  not permission to invoke another tool or restore missing context.
- **MCP capability/session/request IDs:** the local capability is a selected
  future authority mechanism; protocol session/request IDs alone are not grants.

The current component query in
[`authority.rs`](../../../rust/crates/persistence/src/conversations/authority.rs)
checks current Computer execution, unrevoked Connector session, key/overlap,
current route, live non-draining Control and expiries. `source_conversation!`
additionally checks the conversation's Agent belongs to that Computer and the
owner still has Team membership. Owner record reads separately authenticate
private Agent ownership; administration is not access to another owner's records.

At Connector setup dequeue, sealed target/supply/context are checked before
native admission. The narrow
[`Authentication`](../../../rust/apps/chapchoo/src/connector/authentication.rs)
provider retains the command's Agent/conversation/operation scope. This model
shows where trusted product identity exists before native setup completes; it
is not an implemented MCP permission endpoint.

For ADR-D4, the intended capability must retain the validated product target.
Do not substitute model arguments or dynamically select whichever conversation
is current at invocation time. Per-call current server/domain authorization
still applies. No second permission grant is derived from native readiness.

## 6. What Is Already Implemented and What Is Not

| Area | Evidence state |
| --- | --- |
| Harness-owned session lifecycle and exact restore | Implemented; protocol/setup fixtures executed |
| Local captured source and ordered copy | Implemented; capture fixtures executed |
| Server tool input/output projection | Implemented; normalization fixtures executed, Store/RPC/read path inspected |
| Codex MCP result-to-ACP output conversion | Source verified at the pinned external reference, not live acceptance |
| Chapchoo local HTTP MCP endpoint and local capability | Accepted ADR direction, not implemented |
| Common server-tool discovery/invocation RPC | Accepted ADR direction, not implemented |
| Effective new/resumed native MCP provision | Still conditional; no authenticated model/native acceptance performed |
| Whole Connector-to-DB-to-browser round trip in this investigation | Source traced; not newly executed end-to-end |

Searches of the affected Harness/Connector/RPC/Protobuf implementation found no
Chapchoo MCP listener, tool catalog or `tools/call` forwarding implementation.
Current `protocol.rs` does not supply Chapchoo MCP declarations. Reference support
for HTTP transport is therefore not evidence that the feature already works.

## 7. Executed Verification

Existing model-free tests were run with the locked Rust dependencies and an
isolated temporary build target. No tests or application code were added.

```sh
# From rust/; use a disposable CARGO_TARGET_DIR if needed.
cargo test --locked -p chapchoo-harness --lib capture::tests
cargo test --locked -p chapchoo-harness --lib runtime::protocol::tests
cargo test --locked -p chapchoo-harness --lib runtime::setup::tests
cargo test --locked -p chapchoo-service --lib conversation::normalize::tests
```

Results: **9 + 11 + 4 + 9 = 33 passed**, zero failed or ignored in these groups.
These cover retained original capture bytes, finite boundaries, occurrence
identity, pre-setup event binding, exact restoration without new-session
fallback, protocol/authentication/steering ownership and raw tool-field
normalization. They do not prove actual MCP HTTP negotiation, model execution,
server-tool forwarding or current deployed native versions. No database
integration or browser suite was newly executed in this research pass.

## 8. Consequences for the Unimplemented Design

- Keep ADR-D1 through ADR-D4 as the approved provision/transport/local-auth
  direction, while retaining their unverified implementation boundaries.
- Remove the unaccepted native-readiness permission choice and P3a alternate
  response-writer comparison. The latter conflicts with session ownership.
- Use the existing harness-event capture and server-copy path. Do not introduce
  a second response history, new generic result-read API, mandatory original-wire
  archive, session-record commit barrier, new retention policy or synthetic ACP
  events merely to support server tools.
- Preserve independent domain-state persistence and tool-specific recovery.
- Remaining design work concerns supported RPC/MCP input/result/error mapping,
  effective catalog provision and collisions/stale definitions, bounded call
  ownership/cancellation/shutdown and a preserving rollout boundary. Admit only
  genuine unresolved alternatives; existing ownership is not another decision.

## 9. Follow-up Protocol and Admission Evidence

The official MCP
[2025-11-25 tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
and [2026-07-28 tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
contracts distinguish malformed/unknown-tool protocol failures from recognized
tool validation/execution errors (`isError`). Structured-output schema promises
must be honored. The revisions differ in allowed structured values and response
envelopes; an internal lossless representation must not silently narrow them.
This evidence does not select support for every MCP revision or binary content.

The [2025-11-25 HTTP transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
does not equate a lost connection with request cancellation.
The [2026-07-28 HTTP transport](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)
removes protocol sessions/GET resumption and treats closing a request's SSE stream
as its cancellation signal. In either case, a signal is not proof that a started
domain effect stopped or rolled back.

The native `rust-v0.166.0` tag resolves to commit
[`6ff1e1ca3b30fc8b914720681fcf5fe62e95f856`](https://github.com/openai/codex/tree/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856).
Its [`protocol_mode.rs`](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/rmcp-client/src/protocol_mode.rs)
defaults to initialization-based `Legacy` with preferred protocol `2025-06-18`,
while modern mode uses a different lifecycle. Its MCP client and Cargo sources
were inspected at that commit, separately from the shared research clone.
Availability of modern types does not mean the current native session enabled
modern mode. No new native mode or fallback was applied in this investigation.

For overload, the existing
[`InputWork::run`](../../../rust/crates/service/src/conversation/input_work.rs)
reserves slots and input bytes with non-waiting acquisition before Store
acceptance, then retains work/permits after caller loss. This is a reusable
ownership pattern, not authority to copy its durable input protocol. Immediate
capacity rejection and a bounded process-owned waiting queue remain distinct
viable common-tool admission contracts; P6a records the pending comparison.

This follow-up used source/specification inspection only. The 33 tests in
section 7 belong to the earlier session-path research and were not rerun as
evidence of a new tool service or protocol implementation.

### Queue and catalog follow-up after ADR-D5

The backend composition in
[`runtime.rs`](../../../rust/apps/chapchoo/src/runtime.rs) builds shared
conversation/input/answer/Stop service owners, not a new service instance for
each incoming Agent request. That is evidence of the shared-service environment,
not an existing per-Agent tool scheduler. ADR-D5 adds only bounded invocation
waiting; Agent waiting budgets and fair dispatch remain P6b alternatives.

The exact native 0.166.0
[`LoggingClientHandler::on_tool_list_changed`](https://github.com/openai/codex/blob/6ff1e1ca3b30fc8b914720681fcf5fe62e95f856/codex-rs/rmcp-client/src/logging_client_handler.rs)
logs the notification. That callback by itself does not establish model tool
schema refresh or safe replacement of a queued call's definition. It is not
enough evidence to promise live catalog refresh. No automatic refresh mechanism,
catalog version, native restart or schema-replacement policy is approved here.

### Component response lifecycle after ADR-D6

ADR-D6 accepts per-Agent waiting isolation under shared total bounds; it does not
choose a transient RPC progress protocol. Existing Protobuf Control services
have generated bidirectional streams, so component streaming is an existing
transport capability. The common tool contract remains new, and no server-
streaming tool round trip was executed in this investigation.

`connector.rs::request` attaches a ten-second unary timeout. The Control stream
open path deliberately uses a separate lifetime. A future tool invocation must
likewise distinguish finite queue/execution/response budgets from that helper's
short operation budget. P6c compares a pending final-only unary call with an
internal bounded queue/start/final response stream. Neither introduces a durable
task journal, repeated tool invocation or a second author of session history.

### Exact 0.167.2 profile and cleanup constraints

The official native `rust-v0.167.2` tag resolves to
`ce21072c90d32c40af7a55b60b2fb35c84a43a6a`.
The local copies of `protocol_mode.rs`, `core/src/config/mod.rs` and
`features/src/lib.rs` were byte-compared with the official raw files at that
commit; all matched.

[`Config::mcp_protocol_mode`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/core/src/config/mod.rs)
selects modern mode only when `Feature::Mcp20260728` is enabled.
[`features/src/lib.rs`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/features/src/lib.rs)
marks both `mcp_2026_07_28` and the independent `mcp_tool_list_changed` as
under development and disabled by default. Thus available enum variants do not
prove modern mode or live list refresh in a running session. P2c records the
initial profile comparison. The requester subsequently selected the default
2025-06-18 profile as ADR-D11; no feature was enabled in this investigation.

Current Chapchoo backend `runtime.rs` withdraws RPC admission and schedules
Control shutdown on loop exit. Cleanup has a 20-second graceful join followed
by forced cleanup with a five-second budget; forced cleanup is reported as
failure. This is evidence of existing finite shutdown, not a selected new tool
timeout. Future queue dispatch must not bypass withdrawn authority and cannot
assume a process survives indefinitely to drain effects. A new generic durable
task journal is not implied by that limitation.

## 10. Bidirectional Messages and Session-Based Reconstruction

This follow-up is source inspection, not a new runtime or E2E test. Both
`CodexAcpServer.ts` and `McpToolReporter.ts` were byte-compared again with
official raw files at pinned adapter commit
`f3208913693bd4d5171d6c6b9be5ec1ff96c6431`; both matched.
Subsequent choices select no initial session-based chat reconstruction (ADR-D8)
and independent accepted session/chat data with shared temporary delivery data
(ADR-D9). These are unimplemented decisions, not current runtime evidence.

### User input acceptance is not permanent chat storage

[`inputs.rs`](../../../rust/crates/persistence/src/conversations/inputs.rs)
retains exact submitted text, request, operation, target and acceptance time.
`admit_conversation_input` explicitly establishes product acceptance only.
Receipt recovery compares text and target while the pending input exists.

[`delivery.rs`](../../../rust/crates/persistence/src/conversations/delivery.rs)
records `POSSIBLE` before dispatch. `Inserted { native_message_id }` and
`Steered` both consume the pending input by deleting `conversation_inputs`.
The delivery/preparation references cascade with it; see migration
`00000000000013_conversation_projection` and the current
[storage contract](../spec/domain/conversation-storage.md#conversation-identity-and-product-input).
Successful report ACK after removal closes the operation, not a permanent
historical receipt. Failed/unresolved inputs are distinct pending facts, not
proof of native user-message existence.

Consequently the pending-input store is not an existing full user-chat archive.
If B retains all accepted user messages, it needs an explicitly selected
message-domain lifetime independent of this transient handoff lifetime.
Keeping that message cannot itself authorize another native dispatch.

### Steering has a separate observation boundary

[`steering.rs`](../../../rust/crates/harness/src/runtime/steering.rs) returns
`Steered` for `_session/steering` injection, without a native message ID.
Only explicit `promptRequired/noRunningTurn` transitions to ordinary prompt
insertion, whose response supplies a message ID.

The pinned adapter's
[`trackSteerLanding` and `emitLiveSteerUserMessage`](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/CodexAcpServer.ts#L1821-L1864)
match an actual Codex user-message landing before emitting the ACP user message.
Pending unmatched landings are removed on turn completion. Injection response
and native message observation are therefore different evidence. The adapter
mints a fresh client user-message ID for steering; it does not return that ID
in the steering response or accept the Chapchoo input operation as that ID.
Text equality cannot safely recover the missing correlation.

For ordinary prompts, `onInserted` emits the user message and resolves its ID.
Historical `createUserMessageUpdates` prefers persisted `clientId`, otherwise
native item ID in v2. Chapchoo normalization preserves user messages by their
observed message ID. These paths support observed user history, not complete
recovery of every server-accepted, pending, failed or dropped input.

### Resume, recopy and reconstruction differ

- `setup.rs::retained_context` selects exact retained native context with
  `CaptureReplay::AlreadyCaptured`. `protocol.rs` omits `replayFrom` in that
  case. Failed restoration does not create fresh context.
- The pinned adapter's `resumeSessionV2` uses ordinary resume without
  `replayFrom`; explicit `replayFrom: start` instead calls historical load/replay.
  The existence of the latter is not a current Chapchoo repair policy.
- Existing finite capture passes can copy retained observations again.
  Raw/normalized pairs and projected fields can support derived reconstruction
  only to the extent their required records are retained and complete.
- `records.rs::bind_conversation_source` clears copied records/events, native
  state, requests and pending inputs on source replacement. A future separate
  message source cannot inherit that deletion/rebuild policy by accident;
  its lifecycle and source linkage remain material pending decisions.

### Agent publication under candidate B

The server could create a published message before its MCP response reaches
native or its ACP record reaches capture. In that loss window the message exists
but native history may have only the call or an error, not a successful result.
Historical tool arguments alone cannot prove publication. The adapter can replay
retained MCP items with input and any retained result/error through
`McpToolReporter.started`; this proves a representation path, not universal
retention or recovery completeness. Default native assistant text must never
be imported as intentionally published text under REQ-9.

### Consequence-level limits

Session-only reconstruction cannot promise the original full bidirectional chat:
server-only accepted inputs and published messages may be missing. Source order,
native IDs and capture occurrences are not the original server operation/order
by themselves. A bounded import needs selected evidence, stable correlation,
duplicate/deletion policy and an explicit treatment of missing records. Restoring
derived data is different from restoring an authoritative message source.
The requester subsequently selected no session-based chat reconstruction for
the initial delivery; ADR-D8 records that scope boundary. Bounded import and
session-to-chat rebuild are not initial mechanisms. ADR-D9 subsequently selects
independent accepted session/chat data with shared temporary delivery data.
Receipt transaction boundaries and temporary-data release remain pending;
current input removal after native acceptance cannot be assumed sufficient for
both responsibilities without checking their actual acceptance evidence.

## 11. Receipt and Temporary-Data Release Boundaries

This follow-up traces production code only. No native/model call, database
mutation, application implementation or new runtime test was performed.

### Current user-input boundaries

- [`conversations/input.rs`](../../../rust/crates/rpc/src/conversations/input.rs)
  assembles complete UTF-8 before calling product admission. Incomplete upload
  bytes are not a received product message. Upload permits are retained through
  the input owner's handoff, then released; this is not native consumption.
- [`input_owner.rs`](../../../rust/crates/service/src/conversation/input_owner.rs)
  calls completed Store admission before returning acceptance. Only newly
  accepted work invokes the processor. Existing acceptance lookup does not
  dispatch again. The acceptance response is sent before processing finishes.
- [`input_work.rs`](../../../rust/crates/service/src/conversation/input_work.rs)
  retains the actual work and slot/byte permits beyond the original browser
  waiter's lifetime. Response loss does not free accepted work.
- [`operations.rs`](../../../rust/apps/chapchoo/src/connector/operations.rs)
  obtains the sealed input body, checks its finite boundary and asks the native
  owner to process that operation. It reports a known Inserted/Steered/Failed
  result; a generic error does not become proof of native non-delivery.
- `delivery.rs` deletes the input body and dependent handoff state on a known
  Inserted or Steered result, while failed/unresolved work retains its current
  semantics. It does not wait for replicated user-message history.
- Connector
  [`operations/reported.rs`](../../../rust/apps/chapchoo/src/connector/operations/reported.rs)
  and harness
  [`runtime/reported.rs`](../../../rust/crates/harness/src/runtime/reported.rs)
  release known native facts only after successful server result publication,
  retaining the exclusive operation lease through release. Re-reporting the
  same known result after a lost ACK is not a new native input or tool call.

Partial-upload memory, accepted input delivery data, native result facts and
captured history therefore have different existing release boundaries. ADR-D9
shares temporary delivery data between session/chat responsibilities; it does
not make every existing buffer or observation one lifetime.

### Consequences fixed by existing authority

- Accepted session data and chat data remain independently managed (ADR-D9).
  Their independent lifetime does not require waiting for all history copies.
- Tool execution cannot require a separate session-record commit before its
  response (REQ-5). Native captures the actual response it receives. Waiting for
  that response's captured copy before returning it would create a cycle.
- Queue admission, chat receipt, native input acceptance and browser rendering
  are different facts. Shared temporary data must not be reported as permanent
  chat receipt merely because it is waiting in a queue.
- Releasing a delivery payload must follow its actual consumers' completed
  handoff obligations. It cannot erase a not-yet-accepted required record.
  Current native-only input deletion is a seam to adapt, not an automatic
  deletion rule for independently accepted chat messages.
- No Agent re-invocation, new publication retry key or new retry protocol is
  assumed. Existing result-report ACK handling stays distinct from execution.

### Input chat receipt comparison and accepted outcome

The selected data split alone does not determine when a submitted user message
becomes accepted chat data.

1. **Server input acceptance:** establish chat receipt with the initial accepted
   input. Native delivery proceeds through the shared temporary data. Failed
   or pending native delivery does not erase the accepted chat message.
2. **Known native input acceptance:** keep the submission in temporary input
   state until native accepts insertion/injection, then establish chat receipt
   before releasing the shared payload. Inputs never accepted by native remain
   pending/failed submissions rather than accepted chat messages.

Both preserve separate session/chat records and require no Agent re-execution.
Recommend the first if chat should retain what the user successfully submitted
to Chapchoo even when native delivery fails. This is a product receipt/lifetime
choice, not a table name or chat row-layout decision. The requester subsequently
selected server input acceptance, recorded as REQ-10/ADR-D10. It is confirmed
intent, not already implemented runtime behavior.

## 12. Derived Bidirectional Receipt and Release Framing

Authority: REQ-5/REQ-8/REQ-10, ADR-D5 through ADR-D10 and the repository
[`completed-persistence-operations`](../../../.claude/conventions/rust/completed-persistence-operations.md),
[`bounded-admission`](../../../.claude/conventions/rust/bounded-admission.md)
and [`async-effect-ownership`](../../../.claude/conventions/rust/async-effect-ownership.md)
rules. These are constraints for the future Design, not implemented mechanisms
or a new ADR choice.

### User-to-Agent

Server input acceptance must establish accepted user chat data and the existing
input-delivery responsibility before its successful acceptance response. A
completed database-only repository operation is the existing composition
boundary for those related writes; native/network effects remain outside it.
An accepted-input response without its required chat receipt would violate
REQ-10. Partial upload completion alone remains insufficient.

Known native Inserted/Steered reporting can then end temporary input delivery
under its existing fencing contract without removing the accepted chat message.
Native failure/uncertainty and the existing failed-input removal interaction
concern temporary delivery, not implicit deletion of accepted chat. The exact
independent chat deletion and source-replacement policy remains to design.
Native session events are captured/copied independently.

### Agent-to-user

Common waiting admission is not the publication effect. Actual publication is
the owning chat service's completed receipt of the intentional text in the
bound conversation. A final successful publication result must follow that
effect, rather than merely acknowledging a transient queue entry.

The server does not author native tool history. It returns the actual result;
the harness records its observation and existing capture copying follows.
Native response arrival, copied history and browser rendering are not required
second commits in the chat service's receipt operation. Adding browser delivery
or read confirmation would be new product scope, not an implicit success gate.

The invocation owner retains work/input capacity through actual processing.
After its required handoffs and owned processing settle, delivery buffers can
be released independently of the accepted chat record. A lost response cannot
reclassify a completed receipt as known non-publication, remove the accepted
message, or authorize common-layer replay. Started work is not abandoned just
because its waiting connection disappears.

### Shared data without a shared archive

Sharing temporary delivery data does not create another native-session writer,
force session/chat lifetime equality, or require a universal durable tool queue.
The selected process-owned common waiting queue remains ADR-D5/ADR-D6; the
publication domain manages accepted chat. Existing input handoff evidence,
Connector result-report ACKs and capture occurrence identities retain their
different purposes. No Agent resend procedure or publication retry key is added.

ADR-D11 now selects the initial MCP interoperability profile; actual new/resume
acceptance remains unverified. Remaining material work concerns catalog
continuity, concrete domain failure/cleanup and source/deletion policy,
finite limits, cancellation/shutdown and preserving rollout. The full-record UI
agreement remains last, before complete Design approval.

## 13. Initial Catalog Continuity After ADR-D11

At pinned native commit `ce21072c90d32c40af7a55b60b2fb35c84a43a6a`,
[`connection_manager.rs`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/codex-mcp/src/connection_manager.rs)
gates `refresh_notified_catalog` on `refresh_tool_list`. Core configuration
sets it from the independent `McpToolListChanged` feature. The transport's
[`tool_list_changes.rs`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/rmcp-client/src/tool_list_changes.rs)
tracks notifications in transport order; the logging callback alone is not the
complete native refresh implementation. Both source files were byte-compared
with official raw files at that commit and matched.

The feature is under development and disabled by default in the inspected
reference. This is not proof of an installed native's effective feature state.
ADR-D11 selects the initial protocol profile, not automatic catalog refresh.
Requirements explicitly do not assume live catalog hot reload. Consequently
initial provision must not add that feature or advertise a live-change guarantee.

Current Chapchoo `State::start` returns an exact already-live owner without
resending setup configuration. A real new native-process opening, including
exact restoration of its retained session, is different from a live-owner lookup
or Connector operational reconnect. No automatic native restart is authorized.
The inspected ACP adapter also detects sanitized name collisions and may preserve
independently configured MCP entries. Verify effective endpoint, names and schemas
rather than treating declaration acceptance as provision success.

Initial catalog content and declared contracts must remain stable for a live
owner; per-invocation current authority still applies. An unavailable or
incompatible definition cannot silently route an old call to different behavior.
Concrete preserving rollout must enforce that boundary; no catalog version,
fingerprint, retained old-handler platform or dynamic replacement mechanism is
selected here. Actual authenticated new/resume verification remains outstanding.

### Bidirectional chat order comparison and accepted outcome

Server receipt is now independently authoritative for accepted chat. Its order
need not match a caller's creation clock, native tool-start order or the later
capture-copy order. Using capture ordinal would also omit user messages that
never reach native, contrary to REQ-10.

Recommend one stable conversation order based on completed server chat receipt
for both user and Agent messages. Late native capture would not move an accepted
message, and concurrent Agent publications would follow actual chat acceptance,
not the order their native calls started. Sender/source creation time is the
alternative presentation policy, with out-of-order arrivals and clock
differences affecting order. The requester selected stable completed server
chat receipt order, recorded as REQ-11/ADR-D12. No physical sequence schema or
new clock-trust contract was selected.

## 14. Lifetime, Existing Deletion and Original-Input Cutover

Current source inspection confirms four different deletion boundaries:

- `delivery.rs` ends accepted input delivery on known Inserted/Steered results.
  Migration 22 adds SQLite/PostgreSQL terminal guards so even old connected
  result writers consume that input. Its dependent preparation/delivery state
  is removed with the input; this is not accepted chat storage.
- `input_reads.rs::delete_failed_conversation_input` clears the submitted body
  and records a deletion tombstone only for confirmed setup/delivery failure.
  It does not delete native transcript records.
- `records.rs::bind_conversation_source` replaces source-local captured records,
  events, native/request state and inputs within the existing conversation.
  It is not deletion of that product conversation.
- `computers.rs::force_remove_agent` and completed Computer cleanup in
  `computer_executions.rs` delete Agent rows under their existing authority.
  Conversations reference their Agent with `ON DELETE CASCADE` in both
  supported databases.

### Consequences under accepted authority

REQ-10 and ADR-D9 prevent delivery consumption, failed-input body cleanup or
capture source replacement from implicitly deleting accepted chat. Chat data
must belong to the product conversation's ownership/lifetime boundary rather
than to an ephemeral input or capture-source child row. The existing authorized
Agent/conversation deletion boundary must still clean its private child data;
an orphan chat archive is not in scope.

This does not introduce a retention timer, new chat-deletion UI, extra account
archive or change to native restoration. Physical schema, preserving migrations
and any further explicit deletion behavior still belong in the complete Design.
Migration 22 also means migration safety must consider connected old writers,
not merely copy one moment's input rows and assume that all subsequent writers
already implement the new receipt contract. Activation must not permit accepted
inputs to bypass required chat receipt.

### Cutover choice distinct from session reconstruction

There is no existing independent complete chat archive. Successfully consumed
inputs have already been deleted; full historical native messages are excluded
as a reconstruction source by ADR-D8.

Non-deleted pending/failed inputs may still retain the exact originally accepted
server text and identity. They are original server admission data, not imported
native history. Two viable initial migration boundaries remain:

1. Hand those retained original inputs to the new chat data at controlled
   cutover, preserving their content while leaving native delivery facts and
   dispatch authority unchanged.
2. Keep them only in the existing temporary-input path and begin chat receipt
   solely for submissions accepted under the new behavior.

Recommend the first for preservation of already received text. Exclude deleted
bodies and absent historical inputs; do not manufacture messages or native
effects. If selected, establish deterministic chat receipt order for the
cutover batch before new admissions, rather than claiming an unrecorded exact
historical commit order from sender clocks or native events. No application
versioning or new retry key is implied. The requester selected transfer of the
retained originals, recorded as REQ-12/ADR-D13. This is not an executed migration
or a change to the existing native delivery state.

## 15. Actual Native Cancellation and Finite Cleanup Evidence

The pinned native's legacy-profile `RmcpClient::call_tool` sends through
`RequestCancellation::OnDrop`. Dropping its pending response future can send
`notifications/cancelled` for that actual MCP request ID. This is an existing
producer path, not an assumed Agent re-invocation or a new product Stop action.
The auth-challenge branch explicitly preserves authentication-required output
without replaying the rejected tool call. The transport's own credential handling
must remain distinct from replaying an effective tool invocation.

The relevant
[`rmcp_client.rs`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/rmcp-client/src/rmcp_client.rs)
and [`request_cancellation.rs`](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/rmcp-client/src/request_cancellation.rs)
were byte-compared with the official raw files at the pinned commit and matched.
No actual running native cancellation was exercised.

Under REQ-4 and ADR-D5/ADR-D7, an actual cancellation signal must be arbitrated
against dispatch by the queue owner. Confirmed cancellation before dispatch
prevents later execution. If execution won, retain its owner and report only
known outcomes; cancellation cannot remove an already accepted chat message.
HTTP disconnection in the selected 2025-06-18 protocol alone is not an explicit
cancellation notification. Do not collapse those observations.

Existing backend `runtime.rs` withdraws new admission, closes owners and signals
shutdown. Cleanup permits a 20-second graceful join followed by a five-second
forced-failure boundary. New tool work must join that actual owner tree and keep
its capacity through processing. A forced stop is not successful completion or
proof that an in-flight chat write rolled back.

Existing RPC adapters and Connector clients explicitly cap encoded messages at
64 KiB. Native test-server source demonstrates the supported SDK's Streamable
HTTP service/session-manager shape; it is feasibility reference, not proof that
the new Chapchoo relay or its bounded session/cancellation behavior works.
Actual SDK integration and native new/resume remain implementation verification.

## 16. Current Full-Record Presentation Seams

`ConversationBody` currently filters transcript kinds after general record
reads, while its history surface uses `ConversationRecordDisclosure` for the
retained user/assistant, tool, thought and other observations. History requests
are displayed read-only; live permission/form interaction is a separate surface.
`ToolRecordDetails` retains original structured input/output, with input JSON in
a disclosure. `ConversationRecord` supplies the existing summary/preview/status
and expansion interaction. No new visual renderer was executed in this pass.

Final presentation is still a requester choice. A source-preserving proposal is:
main chat reads server user/Agent messages only; full records show actual native
observations. Intentional publication appears as one tool record with readable
text/result details and retained raw fields, not an invented assistant utterance.
Default/final native text remains its own inspectable output record. A user input
never observed by native stays in chat but does not gain a fabricated native row.
The requester confirmed this mapping as REQ-13/ADR-D14. It is design authority,
not evidence of a newly rendered UI or permission to implement.
