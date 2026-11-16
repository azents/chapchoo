---
title: "Agent Session Configuration Refresh Decisions"
created: 2026-11-15
document_role: primary
document_type: adr
snapshot_id: prompt-261115
tags: [agent, conversation, architecture, security, mcp]
---

# Agent Session Configuration Refresh Decisions

- Snapshot: `prompt-261115`
- Requirements: [prompt-261115/REQ](../requirements/prompt-261115-system-prompt-management.md). The original prompt-management scope was confirmed at 16:22 KST on 2026-11-15; the complete common-refresh Requirements expansion was confirmed at 21:03 KST on the same day.
- Mode: Autonomous for remaining technical decisions, delegated on 2026-11-16 at 01:39 KST.
- Decision owner: the requester for product outcomes; delegated technical interviewee for remaining mechanisms.
- Accepted decisions: `prompt-261115/ADR-D1` selects proactive refresh, `prompt-261115/ADR-D5` retains requests for automatic continuation after safe preparation, `prompt-261115/ADR-D2` selects direct overwrite for saved edits, and `prompt-261115/ADR-D3` confirms server-owned original evidence with exact supply/result binding. D4 remains pending; these decisions do not authorize implementation.

## Fixed and Derived Constraints

- Preserve the confirmed divider-row entry, desktop modal, mobile full-screen editor and Save-only submission flow (`REQ-3`).
- Preserve existing conversation context and already running work. Do not silently use superseded content for a new request or add an old-prompt permission control (`REQ-4`).
- Distinguish settings persistence, preparation, actual native delivery and model compliance (`REQ-4`, `REQ-5`).
- Retain exact product-supplied originals and sources beyond setting changes and transient input cleanup. Do not use current settings or incoming event capture to reconstruct missing historical supply (`REQ-6`, `REQ-7`).
- Reuse the current private-Agent/current-Team-membership authorization and uncertain-operation fences (`REQ-8` and current [ownership](../spec/domain/agent-ownership.md), [conversation storage](../spec/domain/conversation-storage.md) and [native-owner](../spec/domain/native-harness.md) contracts).
- Interpretation of prompt meaning is Agent behavior, not product conflict policy (`REQ-2`). No harness-wide prompt visibility or model-compliance guarantee is created.
- Use the supported unmodified Codex native implementation. Native-engine source changes, patches and forks are excluded by the Requirements; a limitation of one investigated path does not authorize expanding this boundary.

## Current Decision Map

- [x] D1 — Refresh trigger: proactively prepare changed configuration at a safe boundary, including when no new request is pending. Accepted on 2026-11-15 at 21:32 KST; user responsiveness takes priority over minimizing safe reexecution count.
- [x] D2 — Concurrent settings saves: valid authorized saves directly overwrite the setting; the last successfully committed save wins. Accepted on 2026-11-15 at 22:46 KST. No stale-edit rejection or save-conflict resolution flow.
- [x] D3 — Supplied-configuration authority and evidence: the server preserves exact product prompt originals/sources before dispatch and retains them independently from consumable input rows; the Connector verifies supply and reports exact execution-bound observations. Accepted on 2026-11-15 at 23:10 KST. Reuse the server authority without a separate Connector-authoritative journal.
- [ ] D4 — Native transition, restoration and cleanup: select the supported same-context Resume/reload or full-scope reexecution boundary, preserve capabilities/effect ownership, and resolve initially empty, unrestorable or uncertain contexts without blind replacement. Pending. This is separate from D1's accepted trigger and D5's accepted request-waiting outcome.
- [x] D5 — New-request admission before refresh is ready: retain the request and continue it once automatically after safe preparation. Accepted on 2026-11-15 at 21:56 KST. Actual preparation failure or target/authority loss remains a failure, not uncertain native replay. Exact observation bound and coordination mechanism are not selected.

Product-visible gaps are returned to Requirements before the affected mechanism is accepted. Treatment of new requests during refresh, outstanding work, unrecoverable native contexts and unavailable new tools must not be invented as local implementation details. Native compatibility evidence may also expose an unavailable path rather than a viable alternative.

Agent-owned local details include identifiers, file layout, equivalent helper boundaries and fixture naming. They do not include new state authorities, persistence modes, prompt precedence policy, application timing, replay permission or externally visible failure behavior.

## Current-System Framing

### Settings and authority

The current [Agent profile](../spec/domain/agent-profile.md) is saved presentation, not a runtime persona. Agent RPC/profile values contain name, image and palette only. Owner operations already authenticate and revalidate private ownership and current Team membership inside the Store operation.

References: [`computers.proto`](../../../proto/chapchoo/computers.proto), [`persistence/computers.rs`](../../../rust/crates/persistence/src/computers.rs).

### Delivery and historical lifetime

The current native input operation contains an operation ID, exact snapshot/native-session target and submitted user text. Known native acceptance consumes successful input/delivery rows. Captured history retains incoming native updates and responder envelopes; it is explicitly not an outgoing command journal. Computer execution, product conversation, captured native context, input operation and foreground work have distinct identities and lifetimes.

References: [`runtime/inputs.rs`](../../../rust/crates/harness/src/runtime/inputs.rs), [`delivery.rs`](../../../rust/crates/persistence/src/conversations/delivery.rs), [`capture.rs`](../../../rust/crates/harness/src/capture.rs), [`runtime.rs`](../../../rust/crates/harness/src/runtime.rs).

### Ownership constraints for the future Design

The accepted D1/D5 outcomes require separate lifetimes without selecting a new state-storage mechanism:

- The saved setting remains a confirmed durable setting even if later refresh or reading fails (`REQ-5`).
- Proactive refresh is product-owned, not owned by whichever browser or request first observes the change (`ADR-D1`).
- A waiting product request retains its exact admitted identity/body and current authorization; its continuation must revalidate readiness and the existing possible-forwarding fence (`ADR-D5`, current input contract).
- Existing native work and started tool effects retain their physical owners. An HTTP/read waiter ending or a foreground Idle observation cannot authorize forced cancellation or successor admission (`REQ-9`, current server-tools/native-owner contracts).
- Only authoritative successful configuration preparation can release the corresponding still-eligible request. A deleted/failed request, another conversation's request or a possibly dispatched operation is not a wake-up target (`ADR-D5`).

The current input consumer renews its lease while its processor remains active, but completes remaining PENDING input as ERROR when processing ends. Existing receipt recovery does not launch processing again. Reusing these contracts requires an explicit refresh-wait integration; the current queue is not already an idle-work backlog.

References: [`input_consumer.rs`](../../../rust/crates/service/src/conversation/input_consumer.rs), [`input_owner.rs`](../../../rust/crates/service/src/conversation/input_owner.rs), [`server effect owner`](../../../rust/crates/service/src/server_tools/owner.rs).

### Native protocol capacity and limits

Chapchoo uses strict ACP v2. Its current New/Resume requests provide cwd and MCP declarations, and its input path uses the steering extension followed by ordinary prompt only after explicit non-consumption. It does not currently transmit a system prompt.

The locked ACP SDK exposes an independently gated session system-prompt extension. The investigated [official adapter source](https://github.com/agentclientprotocol/codex-acp/tree/f3208913693bd4d5171d6c6b9be5ec1ff96c6431) supports append-mode text on New/Resume. It rejects changed instructions while work is busy; changed live-thread Resume also checks persistence, background terminals and actual unload before exact-thread resume. Null/omission preserves prior client text; an empty content array is invalid, while a text block containing an empty string passes its text validator. Retained developer-message replacement at compaction is distinct from changing current thread instructions.

These are source-level capabilities and constraints, not proof of the runtime selected by Chapchoo, actual model adoption or compatibility with its allowed Codex range. Static feasibility remains conditional until a specific authorized mechanism has credible compatibility evidence. No live model call is included in this investigation.

References: [`rust/Cargo.toml`](../../../rust/Cargo.toml), [`runtime/protocol.rs`](../../../rust/crates/harness/src/runtime/protocol.rs), [`runtime/steering.rs`](../../../rust/crates/harness/src/runtime/steering.rs), [`preparation.rs`](../../../rust/crates/harness/src/preparation.rs).

## D1 Investigation: Common Refresh, Not a Prompt-Only Exception

At 16:54 KST on 2026-11-15 the requester broadened the objective to a general natural session-reexecution lifecycle because session-supplied MCP tools also need adoption. The previously proposed prompt-only unused-context replacement exception is not the active decision request and remains unaccepted. The empty-context facts below remain feasibility constraints within the common lifecycle, not a selected special case.

The investigation distinguishes:

- the product conversation and retained receipts;
- the native thread/session identity and persisted context;
- the adapter/app-server process and physically owned effects;
- ACP New/Resume declarations and the native MCP client configuration;
- MCP catalogue/schema changes, endpoint/capability changes, and implementation or transport-credential changes behind an unchanged tool contract.

The word "restart" does not select one of those boundaries. Any common transition must preserve the current no-blind-replay, current-authority, complete-retirement and exact-context requirements until an explicit decision authorizes a different boundary. A per-change version/epoch mechanism is not implied by the shared-refresh objective.

### MCP configuration and retirement evidence

The current Connector freezes a structured `ListToolsResult` at actual-start provision; a later catalogue read that differs is rejected rather than adopted. Live exact Setup returns before acquiring another tools provider. Operational route/key refresh preserves the local recipient and provision lifetime. Therefore repeated Setup and ordinary reconnect are not common session-reconfiguration mechanisms.

The investigated supported Codex native line ([0.165.0 source](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/mcp_refresh.rs)) also has `config/mcpServer/reload`, exposed by the adapter's `/mcp reconnect` command. It reloads persisted configuration while preserving existing per-thread override layers; it is not a request that accepts a replacement ACP `mcpServers` list. Native tool-list-change support is experimental and disabled by default in that line; the current Chapchoo MCP server does not advertise listChanged or notify catalogue changes. Automatic tool-schema adoption is therefore not an existing enabled contract. New declarations still enter New/Resume session configuration. Existing loaded native threads may ignore Resume overrides unless actually detached/reloaded. A blanket statement that every MCP change requires a physical process restart is too broad, but a general hot-update path for new product declarations has not been established either.

ACP session close is not a harmless release operation in the investigated adapter: it cancels queued requests, interrupts active work, finishes asynchronous tasks as stopped, stops background terminals and unsubscribes the native thread. It does not archive the thread, but an unmaterialized empty thread may subsequently lack a restorable persisted context. Chapchoo's own physical retirement similarly closes tools and may cancel/kill native descendants. Neither operation is itself the requested graceful work-preserving refresh policy.

The current native launch grant is single-use: adapter-internal recovery cannot launch another native process under the same applied owner. Whole-scope verified retirement is required before another authorized native launch. External adapter auto-restart capacity is not current Chapchoo reconfiguration authority, and an unchanged product conversation does not allow a successor to borrow its predecessor's tool capability, publication tickets or start binding.

Local HTTP/relay completion or response timeout does not establish completion or rollback of a remote started tool effect. Effect ownership and disclosure authority remain distinct from observers. A new configuration cannot inherit old publication tickets or capabilities merely because the product Agent/conversation matches.

References: [`server-tools Spec`](../spec/domain/server-tools.md), [`Connector tools owner`](../../../rust/apps/chapchoo/src/connector/server_tools.rs), [`tool handler`](../../../rust/apps/chapchoo/src/connector/server_tools/handler.rs), [`native retirement`](../../../rust/crates/harness/src/runtime.rs), [official adapter close](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/CodexAcpServer.ts#L2072-L2144).

### First-input persistence through a supported metadata operation

Released-program checks with ACP adapter 2.6.0 and Codex 0.165.0 established a
public model-free preservation path. Changed-systemPrompt Resume initially
rejected an unmaterialized thread. Standard `session/set_title` then persisted
that same thread, after which changed Resume succeeded before the first ordinary
fixture turn or tool effect.

The native metadata-update implementation explicitly materializes a newly named
thread, for both Legacy and Paginated history. Initial native title assignment
can therefore be evaluated as part of preparing a durably resumable context.
This is narrower than replacing an unused native context. The earlier assumption
that no public pre-first-input preservation path existed is superseded.

Chapchoo currently binds capture before endpoint verification, and server capture
binding and setup-result publication are independent. The Design must put the
required metadata persistence and verified readiness in the correct preparation
order, preserving an exact observed source on failure. D5 input waiting and D3
server-owned original evidence remain the owning product contracts.

References: [`runtime/setup.rs`](../../../rust/crates/harness/src/runtime/setup.rs), [`conversations/startup.rs`](../../../rust/crates/persistence/src/conversations/startup.rs), [`conversations/records.rs`](../../../rust/crates/persistence/src/conversations/records.rs), [official adapter instruction update](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/CodexAcpClient.ts#L922-L978).

## D3 Investigation: Original Content and Actual Supply

The existing server owns product tool definitions, including their names,
descriptions and schemas. The Connector obtains and freezes that catalogue for
the actual native owner. Its loopback endpoint and bearer are created only at
actual launch; these operational capabilities are separate from product
originals and must not enter prompt-source evidence (`REQ-8`).

The repository already retains setup intent before dispatch, revalidates it at
native dequeue and binds its result to the exact operation, execution and
source/native target. A new native session identity can be attached to the
original preparation operation's result when it was not available before
dispatch. This supports server-retained originals with exact supply/result
binding as a credible candidate; it does not establish that prompt provenance
is already implemented.

Incoming capture does not record outgoing commands. Successful input rows are
consumed. Neither is sufficient to retain the required execution evidence.
Conversely, these limitations do not require a new native-authoritative command
journal: the existing pre-dispatch retention and exact-result pattern is another
credible boundary.

This investigation supports extending the existing server responsibility:
preserve product originals before dispatch, verify supply at the Connector and
bind actual observations to that retained evidence. The requester's simplicity
direction and existing ownership make a separate Connector-authoritative journal
an unnecessary alternative. The accepted scope is recorded in D3 below.

References: [`server tool definitions`](../../../rust/crates/service/src/server_tools/backend.rs),
[`actual-start relay`](../../../rust/apps/chapchoo/src/connector/server_tools.rs),
[`preparation intent and result`](../../../rust/crates/persistence/src/conversations/preparation.rs),
[`startup intent`](../../../rust/crates/persistence/src/conversations/startup.rs),
[`native declarations`](../../../rust/crates/harness/src/server_tools.rs).

## D4 Investigation: Supported Refresh Behavior

The work-preserving boundary is already constrained by `REQ-4` and `REQ-9`:
keep running work and context, then adopt the changed prompt and tools before
releasing eligible new work. Foreground idle alone cannot establish that
background work has drained. Current Chapchoo observation retains only foreground
state, and local Relay completion cannot prove completion of a remote server
effect whose observer expired.

The existing changed-systemPrompt Resume is a per-thread unload/rebuild path.
The adapter checks the root's prompt state and background terminals and retains
the standard child router across Resume. Root and child foreground states are
separately observable when standard subagent capabilities are negotiated.
Current direct tests cover root busy/background rejection; active-child
preservation remains a specific verification obligation.

Changed-prompt Resume checks persistence and background terminals before an
actual unload/resume. The unchanged-prompt path skips this helper's unload
checks, so an MCP-only change cannot rely on that path to rebuild native
configuration. A full process restart also needs positive work-preservation and
exact-context restoration evidence; namespace retirement is not a substitute.

The native engine already has an exact registered-writer persistence operation
for both Legacy and Paginated history. `persist_thread(Standard)` reaches
`recorder.persist()` in both cases; an existing native test creates and immediately
persists a thread without appending user input, for both modes. This is positive
source-level evidence for model-free preservation of the original empty thread.
It does not require a new native identity or a change of history mode.

Public title assignment supplies the needed invocation path for initial
persistence, as confirmed above. The remaining immediate gap concerns actual
instruction delivery after a used context is resumed.

Native unsubscribe is also not the missing safety barrier: it changes subscriber
membership. Ordinary automatic unload eventually calls shutdown, which aborts
tasks and terminates unified-exec processes. An adapter-side background list
followed by unsubscribe is not atomic with admission or teardown. The engine has
a narrower residency gate for certain child eviction, showing a possible owned
boundary, but it is not a public all-work guarantee for root reconfiguration.
Any work-preserving supplement must establish safety at the actual admission and
teardown boundary rather than infer it from an external empty list.

The investigated unconditional full-retirement/exact-restore path remains
unverified under the current integration. This is a limitation of that path,
not proof that the whole feature is impossible without changing Codex.
Reconsider Chapchoo-owned preparation timing, runtime lifetime and the handling
of existing work within supported interfaces. Preserve the confirmed product
context and work outcomes. Codex source changes, patches and forks are excluded;
the earlier engine-supplement proposal is withdrawn. No direct native bypass,
adapter fork, fresh-context replacement or new runtime mode is accepted.

References: [`foreground observations`](../../../rust/crates/harness/src/runtime/observations.rs),
[`owned native retirement`](../../../rust/crates/harness/src/runtime.rs),
[`server effect owner`](../../../rust/crates/service/src/server_tools/owner.rs),
[official ACP router](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/AcpAgentRouter.ts),
[official adapter capabilities tests](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/__tests__/CodexACPAgent/sdk-capabilities.test.ts),
[native thread read and unsubscribe](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/request_processors/thread_processor.rs),
[registered-writer persistence](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/thread-store/src/local/live_writer.rs#L414-L470),
[both-mode empty-thread persistence test](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/thread-store/src/local/mod.rs#L1802-L1832),
[native unload](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server/src/request_processors/thread_lifecycle.rs),
[native shutdown](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/core/src/session/handlers.rs#L293-L317).

### Released-program evidence: settings, history and tool adoption

An isolated probe used the released ACP adapter 2.6.0 and Codex 0.165.0 with a
loopback fixed-SSE response fixture. It inspected actual subsequent native
Responses requests rather than relying on configuration acknowledgements:

- Initial title assignment enabled same-ID Resume before any fixture turn.
- Healthy and closed same-endpoint `/mcp reconnect` observations created status
  discovery connections, while subsequent model-facing requests retained the
  previous tool declaration.
- A changed-prompt Resume rebuilt the root and the following native request
  contained the updated MCP declaration.
- On that used thread, the new prompt text was absent from the following
  request, while the earlier developer text remained.

The last observation matches native history reconstruction: Resume restores
history and its reference context; the next turn only rebuilds the initial
developer bundle when a new full initial context is required. Current developer
settings and the retained model-visible instruction history are distinct.

A separate test-only protocol experiment appended the latest developer item
through the existing public `thread/inject_items` API after Resume. The next
native request then contained the latest prompt text and updated tool alongside
retained conversation history. This validates a native capability for an
integration-layer correction. The released ACP adapter does not perform that
extra mapping today; adopting it remains part of the pending D4 direction.

The implementation contract must also make declaration changes trigger the
appropriate reconfiguration when prompt text itself is unchanged. Actual
configuration delivery, actor coordination, retained tool ownership and
uncertain-result handling belong in that contract. These root-idle experiments
do not replace active-work or production-Relay verification.

References: [title-update persistence](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/thread-store/src/local/update_thread_metadata.rs#L96-L184),
[history reconstruction and initial-context emission](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/core/src/session/mod.rs),
[public item append contract](https://github.com/openai/codex/blob/360319d8dee62d87e2d59fc0c21805d79519cfed/codex-rs/app-server-protocol/src/protocol/v2/thread.rs#L1745-L1755).

### Pending D4 direction: same-context session reconfiguration

The candidate direction is to retain the physical native owner and its
authentication, scoped MCP endpoint and publication ownership while reconfiguring
the original native thread. Use initial native title persistence to make an
empty context resumable. Once existing work permits refresh, apply the selected
prompt and tool configuration through a session-level transition, synchronize
latest developer instructions into used model-visible history and then release
eligible queued input.

This changes Chapchoo's current exact live Setup early-return behavior and its
frozen-catalogue handling: a real explicit refresh must stage the selected tool
definitions for native reinitialization, while preserving already-admitted call
semantics. MCP-only configuration changes need the same guarded reconfiguration
path even when prompt text is unchanged.

The released ACP adapter currently omits the latest-history update and its
unload guard is conditional on prompt changes. Completing this candidate therefore
requires an ACP integration correction and an explicit supported
dependency/release path. Public native API availability and a successful
research proxy do not select that delivery mechanism. D4 remains pending.

Native append is an effect: preparation must retain exact operation/target
evidence before dispatch, distinguish config success from later append failure,
and preserve uncertainty after acknowledgement loss. Busy leaves existing work
owned and queues new product input. Active-child preservation, in-flight
tool/schema handling, prompt clearing, repeated configuration changes and
lost-result recovery remain focused verification gates for the complete Design.

### Product integration checks

The accepted D5 submission boundary requires keeping product input eligibility
separate from native readiness. The current Chat container uses one `writable`
condition for submission and native responses, including a source-loading check;
the renderer disables the textarea from that value. Refresh must not reuse those
native-unready conditions to disable otherwise admissible input. Native
responders and Stop retain their exact live-target checks.

The existing server queue and short acceptance response are reusable, but its
processor lifetime is not already refresh waiting: processing return fails any
remaining PENDING row, and consumer expiry also records failure. The Design must
retain coherent wait ownership and readiness coordination rather than return on
temporary busy, fabricate pending client bubbles or silently restart uncertain
dispatch. The precise mechanism remains unselected.

Primary product verification should use one minimal fixture-backed browser flow
covering save, existing work preservation, continued input acceptance during
refresh, once-only continuation and historical prompt originals after a later
edit. Native safety, concurrent dispatch, background/child work, persistence,
uncertainty and cleanup require lower-layer executable protocol/owner tests.
Real supported-runtime model-free restoration is separate evidence; a fixture
journey cannot certify native compatibility or model compliance.

References: [`Chat admission`](../../../typescript/apps/web/src/features/computers/chat/containers/useChatContainer.ts),
[`Chat renderer`](../../../typescript/apps/web/src/features/computers/chat/components/Chat.tsx),
[`input acceptance owner`](../../../rust/crates/service/src/conversation/input_owner.rs),
[`input consumer lifetime`](../../../rust/crates/service/src/conversation/input_consumer.rs).

## Accepted Decisions

### D1. Proactive Refresh at a Safe Boundary

- Reference: `prompt-261115/ADR-D1`
- Accepted: 2026-11-15 at 21:32 KST.
- Authority: `prompt-261115/REQ-4`, `prompt-261115/REQ-9`, and the requester's explicit priority of responsiveness over fewer safe reexecutions.

#### Decision

When a relevant session-supplied configuration change becomes known, prepare its adoption proactively at an authorized safe boundary. Do not leave an otherwise refreshable conversation on the old configuration solely to avoid reexecution until another product request arrives. Extra safe reexecutions are acceptable when they reduce user-visible preparation latency. Repeated legitimate changes may require further refreshes; this does not authorize infinite failure retries or repeated execution of the same user effect.

Settings persistence remains independently truthful. Save-only submission, product conversation context, already started work and existing uncertain-operation fences remain unchanged. Proactive scheduling does not authorize forced interruption, a fresh native context, a new retry/retention mode, or a per-change version/epoch mechanism.

#### Alternatives and rationale

- **Selected: proactive refresh.** Product-owned preparation can complete before the next request and avoid making that request pay the full refresh latency. Additional preparation/native transition work is an accepted cost.
- **Not selected: request-bound refresh.** Deferring until another request could reduce unused preparation and combine successive edits, but makes the user experience first-request preparation delay. The requester's stated priority outweighs that optimization. The previous recommendation of this alternative was not accepted and is withdrawn.

#### Consequences and remaining boundaries

Refresh must have product-owned lifetime rather than depending on a future request waiter. This selects timing, not its persistence/event mechanism. The Design still needs a supported transition and credible adoption evidence. Unknown previous effects, foreground idle, process launch or an unchanged Setup result alone cannot establish a safe completed refresh.

No zero-delay guarantee is created. A request may arrive while original work is still running or refresh is incomplete. Its waiting/failure behavior, observation bound, exact configuration selection under concurrent edits, evidence storage, and native-context replacement/retirement authority remain pending. D1 must not be used to infer their acceptance.

### D5. Retain Arriving Requests for Automatic Continuation

- Reference: `prompt-261115/ADR-D5`
- Accepted: 2026-11-15 at 21:56 KST.
- Authority: `prompt-261115/REQ-4`, `prompt-261115/REQ-5`, `prompt-261115/REQ-9`, and the requester's explicit confirmation of accepted-request waiting with automatic continuation.

#### Decision

Retain a newly accepted request while authorized configuration refresh is pending. After the required configuration has been safely prepared under the same valid product target and current authority, forward that request once without requiring the user to submit it again. Temporary pending preparation alone is not a reason to fail the request for manual retry.

An actual preparation failure or loss of the required target/authority is reported with preserved input and without fabricating successful application. A possibly forwarded request is not a waiting unsent request and cannot use this decision as permission to replay. Existing work is not force-stopped to accelerate continuation.

#### Alternatives and rationale

- **Selected: accepted-request waiting and continuation.** The product owns temporary coordination and avoids unnecessary recovery effort for an ordinary refresh transition.
- **Not selected: immediate preparation failure merely because refresh is pending.** This avoids a retained waiting operation but forces the user to retry work that could continue naturally.

#### Consequences and remaining boundaries

The accepted product request, existing native work and proactive refresh need distinct lifetime ownership. Cancelling an HTTP/read waiter does not cancel those effects. A refresh's success may release only the exact still-eligible request; it cannot retarget another conversation, revive a removed input, or replay an operation with possible-forwarding evidence.

The existing consumer heartbeat and request data are reusable evidence, not a ready-made idle-wait implementation: processor return currently fails a remaining pending row, while accepted-request recovery alone does not restart processing. The future Design must make waiting and wake-up ownership explicit and preserve the current native-forwarding grant fence.

This decision does not choose a queue schema, crash-recovery/retry policy, unlimited retention, a new task deadline, configuration versioning, the observation bound, native-context replacement, or a physical restart mechanism.

### D2. Direct Overwrite for Saved Edits

- Reference: `prompt-261115/ADR-D2`
- Accepted: 2026-11-15 at 22:46 KST.
- Authority: `prompt-261115/REQ-1`, `prompt-261115/REQ-3`, and the requester's explicit selection of direct overwrite.

#### Decision

A valid save under current owner authority directly replaces the saved owner-authored system prompt. The last successfully committed save is the setting, including when another editor changed the value after this editor loaded it. An old editing baseline is not a rejection condition.

Keep Save as the only submission action. Do not introduce stale-edit rejection, a save-conflict dialog, automatic merging, version history, or a comparison/revision/epoch precondition to enforce the withdrawn stale-edit policy. Ordinary authorization, input validation and real persistence failures still apply; this decision is not permission to replay an uncertain submission or restore a failed write as if it succeeded.

#### Alternatives and rationale

- **Selected: direct overwrite.** This follows the simple saved-setting interaction and existing profile-update pattern. A later committed save may intentionally replace another editor's content.
- **Not selected: stale-edit rejection with draft preservation.** It can prevent concurrent replacement, but adds conflict handling and recovery burden that the requester explicitly declined. The earlier recommendation is withdrawn and was never accepted.

#### Consequences and remaining boundaries

The setting store determines successful write order; browser timestamps and tab opening order do not create an additional authority. Preserve entered content on genuine failure and report known persistence results accurately.

D1 still requires proactive refresh toward the resulting desired setting. D2 does not authorize an older preparation to claim that a newer saved value is already applied, and does not decide D3's supplied-configuration authority/evidence or D4's native transition mechanism. Those mechanisms must implement direct-overwrite semantics without reintroducing rejected save conflicts.

### D3. Server-Owned Originals with Exact Supply Evidence

- Reference: `prompt-261115/ADR-D3`
- Accepted: 2026-11-15 at 23:10 KST.
- Authority: `prompt-261115/REQ-6`, `prompt-261115/REQ-7`, `prompt-261115/REQ-8`, and the requester's explicit confirmation of server-owned evidence using the simple existing responsibility boundary.

#### Decision

The server owns the canonical historical product-supplied prompt originals and
their sources. Preserve the exact selected originals before dispatch and retain
them independently from successful input/delivery consumption and later setting
changes. Historical reads use current server-enforced private-Agent and
Team-membership authorization.

The Connector verifies the supplied content against that retained selection and
reports the actual non-secret supply/preparation observations bound to the exact
operation, execution and native target. For initial native creation, bind the
resulting native target to the original preparation operation. Retained intent
alone is not actual-supply evidence; a lost result remains unresolved.

Keep product tool definitions separate from actual-launch transport capabilities.
Endpoint credentials and authentication material remain operational data and are
excluded from prompt-source evidence. Verify the applicable native declaration
and catalogue as part of preparation without expanding the prompt evidence view
into a tool-history product.

If required prompt originals and sources cannot be prepared and preserved, fail
before starting new work. A later evidence-read failure does not undo retained
originals or known supply success. Neither preservation nor a native preparation
result proves model interpretation or compliance.

#### Rationale

The server already owns product tool definitions, authorization and historical
reads, and existing preparation retains intent before dispatch and binds exact
native results. Extend that responsibility instead of introducing another
canonical store at the Connector. A separate Connector-authoritative outgoing
journal would add storage, transfer, recovery and retirement obligations without
a required product benefit.

#### Consequences and remaining boundaries

The Design must extend the current contracts: prompt-original preservation and
actual-supply evidence do not exist today. Schema and helper details must follow
this single authority without adding setting versions, revision/epoch controls,
rollback modes or uncertain replay permission. Native transition and restoration
remain D4; this decision does not authorize context replacement or implementation.

## Accepted Clarifications

### D4. Default Project Instructions and Same-Context Refresh

- Reference: `prompt-261115/ADR-D4`
- Accepted: 2026-11-16.
- Authority: `prompt-261115/REQ-4`, `prompt-261115/REQ-9`, `prompt-261115/REQ-10`, the requester's explicit project-file direction and delegated technical decision review.

Use the existing persistent native workspace as the Agent's general-purpose
default project. The common contract supplies a selected instruction bundle and
project path; the Codex backend publishes `AGENTS.md`. Keep ordinary project
files and the existing working directory. Preserve a pre-existing instruction
file once outside active instruction search before first managed publication.

Use the standard native instruction reload behavior through actual process
restart and exact context restoration. Released-program verification observed
native-generated instruction replacement after a file edit and instruction
removal after clearing. This supersedes the prior unresolved direct
systemPrompt/Resume and unused-context replacement proposals.

The server retains selected original sources before dispatch. Publish the
instruction file atomically and mark preparation successful only after native
restoration and scoped MCP verification. Compare actual selected content;
setting revisions or epochs are unnecessary.

Retain the existing native owner until its tracked work is finished. Track
foreground, background and child work through the supported native observations;
unknown state does not become ready. A finite failed preparation reports its
real failure while retaining product input. The accepted server queue owns user
submissions throughout refresh.

This direction preserves current ACP, authentication, capture and tool
publication contracts and avoids replacing the native driver. Future harnesses
translate the common project/instruction contract through their own standard
instruction file behavior. No additional harness is implemented in this snapshot.

### D5 Clarification: Accept User Input While Native Dispatch Waits

- Reference: `prompt-261115/ADR-D5`
- Confirmed: 2026-11-15 at 23:31 KST.
- Authority: `prompt-261115/REQ-9` and the requester's explicit confirmation that waiting means accepting and queuing user input, not blocking its submission.

The refresh boundary gates onward native dispatch, not user submission. Keep
the conversation input and Send action available during refresh, accept otherwise
admissible requests into the product's pending queue, and retain their exact
identity and body. Once safe preparation succeeds, continue each still-eligible
request once under the existing authorization and uncertain-forwarding fences.
An adapter's temporary native-admission fence must be coordinated behind that
product queue rather than surfaced as refresh-based submission rejection.

This confirms the submission/dispatch distinction within D5. It does not select
a new queue schema or unlimited capacity, alter ordinary authorization or
admission rules, authorize uncertain replay, or approve D4's unresolved native
transition and restoration mechanisms. Actual preparation failures retain the
previously accepted truthful failure and preserved-input behavior.
