---
title: "Server-Provided Tools Architecture Decisions"
created: 2026-11-12
document_role: primary
document_type: adr
snapshot_id: tools-261112
tags: [tools, mcp, architecture, connector]
---

# Server-Provided Tools Architecture Decisions

- Requirements: [tools-261112/REQ](../requirements/tools-261112-server-provided-tools.md).
- Mode: collaborative. The requester owns unresolved material decisions.
- Accepted technical decisions: `tools-261112/ADR-D1` (Connector-side MCP relay).
  `tools-261112/ADR-D2` selects Connector-managed local HTTP MCP.
  `tools-261112/ADR-D3` selects the existing component RPC boundary remotely.
  `tools-261112/ADR-D4` selects Connector-owned local access capabilities.
  `tools-261112/ADR-D5` selects a bounded process-owned invocation waiting queue.
  `tools-261112/ADR-D6` selects Agent-partitioned waiting limits and fair dispatch.
  `tools-261112/ADR-D7` selects bounded invocation response streaming.
  `tools-261112/ADR-D8` records the initial exclusion of session-based chat
  reconstruction, preserving exact native resume and existing capture copying.
  `tools-261112/ADR-D9` selects shared temporary delivery data with independently
  managed accepted session data and chat data.
  `tools-261112/ADR-D10` selects user chat receipt at server input acceptance,
  independently of native delivery.
  `tools-261112/ADR-D11` selects initial MCP `2025-06-18` interoperability
  through the inspected native default initialization mode.
  `tools-261112/ADR-D12` selects stable bidirectional chat order by completed
  server chat receipt.
  `tools-261112/ADR-D13` selects transfer of retained original non-deleted user
  inputs into chat data when applying the feature.
  `tools-261112/ADR-D14` records the final chat/full-record item mapping.
  Remaining implementation verification does not create new operating modes.
  Whole-Design approval remains separate from these accepted decisions.

## Fixed Outcomes

- Generalized Chapchoo-owned tools, initially consumed by Codex through the
  existing ACP v2 setup/restoration boundary.
- The harness owns session history and the server copies captured records.
  Persistent domain state and separate query tools belong to their owning
  service. Tool transport does not become a second session-history author.
- No common mandatory domain-state archive, durable invocation journal, uniform
  idempotency/recovery policy, file transfer, or additional harness scope.
- Discovery is not invocation authorization. Native or model-supplied IDs are
  not proof of Agent/conversation authority.
- Response loss is not known non-execution; common transport cannot silently
  replay potentially effective calls.
- REQ-8/REQ-9 select explicit text publication as the first real tool and keep
  default native text recorded without automatic publication or fallback.
  Exact full-record UI presentation is deferred to final agreement.
- Initial product scope is text publication only. Progress/input-wait/finish/
  quiet-completion declarations and continuation are deferred, not additional
  modes to design into this tool.

## System Framing

Current evidence is based on
`83a41877867f3602f741da0a9de819fa52f76929`.

| Boundary | Current owner or fact | Design gap |
| --- | --- | --- |
| Native setup | Harness actor initializes strict ACP v2 and creates/resumes the exact native session | No Chapchoo MCP declaration or endpoint is installed |
| Connector authority | Current execution credential, route, Control and Computer checked by completed server operations | Computer-level component authority must not become arbitrary conversation authority |
| MCP invocation | No common Chapchoo server-tool contract | Provision, caller binding, catalog, dispatch and response semantics |
| Conversation history | Harness-owned session; local captured ACP source replicated with raw bytes and normalized projections including tool input/output | Verify native MCP result reporting through the existing path; no independent tool-response writer |
| Domain state | Respective business services | Remains service-owned; common recording is not a replacement store |
| Process/network lifetime | Connector and native actor retain work owners separately from transient callers | New MCP resources need explicit ownership, admission and teardown |

The following are affected seams to analyze, not approved replacements:
native setup declarations and launch configuration, component-authenticated
server entrypoints, session record ingress/reads, bounded request handling,
composition and shutdown, generated contracts and fixtures. Existing ACP capture,
text delivery, steering, permissions and current native ownership remain unless
a later decision explicitly authorizes a necessary change.

## Complete Decision and Verification Map

| Topic | Status | Consequence |
| --- | --- | --- |
| P1 Provision and authority boundary | Accepted as ADR-D1 | Connector-side relay; domain logic stays server-owned |
| P2a Harness-facing local transport | Accepted as ADR-D2 | Connector-managed local HTTP MCP, no stdio bridge |
| P2b Server-facing transport | Accepted as ADR-D3 | Common server-tool service over the existing component RPC boundary |
| Caller admission | Accepted as ADR-D4 | Connector-owned local capability; server invocation authorization remains independent |
| Initial production tools/output | Confirmed as REQ-8/REQ-9 | Explicit bound-conversation text publication; native text is retained, not automatically published or used as fallback |
| Initial completion/input-wait scope | Publication-only scope confirmed | No progress/wait/finish/quiet-completion controls, work-state tracking or continuation in the initial tool |
| Bidirectional data ownership | Accepted as ADR-D9 | Share temporary delivery data; manage accepted session data and chat data independently in both directions. Receipt and publication-success boundaries remain pending |
| User-input chat receipt time | Accepted as REQ-10 / ADR-D10 | Accepted server input is an accepted user chat message; failed, pending or uncertain native delivery does not erase it |
| Session-based chat reconstruction | Excluded initially; recorded as ADR-D8 | No native-history-to-chat reconstruction/import; exact native resume and existing capture copying remain unchanged. Does not select the chat source |
| Full-record UI | Accepted as REQ-13 / ADR-D14 | Message-only chat; expandable actual native items, including publication as one tool record with sent text/result details and no invented assistant row |
| P2c Initial MCP compatibility profile | Accepted as ADR-D11 | Initially target MCP 2025-06-18 with the inspected native default initialization mode; no explicit modern feature activation or added fallback |
| Compatibility and credential lifecycle | Fixed ownership; verification pending | Native-owner-bound local capability and independent current server authority; effective new/resume and credential/connection replacement need tests, not a new readiness grant |
| P3 Invocation and session compatibility | Fixed ownership; verification pending | Harness authors session records; existing capture/replication copies them. Verify tool result reporting without a second writer or record-before-reply barrier |
| P4 Response representation and bounds | Selected profile and bounded framing; draft values need validation | Preserve 2025-06-18 text/structured-object/error meanings through typed ingress; finite encoded payload budgets do not reduce existing user-input semantics |
| P5 Catalog lifecycle | Initial no-hot-reload scope framed; verification and preserving rollout pending | Do not add live native catalog refresh; verify effective names/schema/endpoint and reject unavailable or incompatible calls under current authority |
| Bidirectional chat order | Accepted as REQ-11 / ADR-D12 | One stable completed server chat receipt order for user and Agent messages; delayed capture and sender clocks do not reorder chat |
| Existing retained user-input handoff | Accepted as REQ-12 / ADR-D13 | Move retained original non-deleted pending/failed server inputs into chat data when applying the feature; no native replay or session-history reconstruction |
| P6a Capacity admission | Accepted as ADR-D5 | Finite process-owned waiting queue when execution slots are full; queue count/bytes/time are bounded |
| P6b Shared-service queue isolation | Accepted as ADR-D6 | Agent-partitioned waiting count/byte limits and fair dispatch under shared total bounds |
| P6c Invocation response lifecycle | Accepted as ADR-D7 | One invocation response stream reports actual waiting admission, dispatch start and final outcome |
| P6 Operational and rollout boundary | Ownership/cleanup framed; complete Design and validation pending | Apply selected queue/cancellation outcomes under existing finite cleanup; expose draft limits and preserving activation/downgrade guards without implicit native restart |

Tool-specific domain persistence, retries and queries are not reopened as a
mandatory common mechanism. Local helper boundaries and equivalent code layout
are agent-owned details, not decisions in this map.

## P1 — Where Is the Harness-Facing Tool Boundary?

Question: should a harness reach Chapchoo server tools through a Connector-side
MCP relay, or connect directly to an authenticated server MCP endpoint?

Related outcomes: `tools-261112/REQ-1`, `REQ-2`, `REQ-3`, `REQ-4`, `REQ-5`.

### Option A: Connector-side MCP relay

Logical path: harness -> local MCP surface -> Connector-owned forwarding ->
server tool service -> owning domain service.

- The Connector mediates the native caller scope and current server authority;
  the server still authorizes every actual invocation.
- Server-owned code executes server tools. The relay is not a second business
  implementation or a persistent domain-state owner.
- Connector operational credentials stay outside ordinary harness credentials.
  A local endpoint or process relationship alone is not sufficient proof of
  conversation identity; the local admission mechanism remains to be designed.
- The added local surface needs lifecycle, bounded calls and failure reporting.
  Connector disconnection affects new server-tool availability.
- This choice does not yet select stdio versus local HTTP, an IPC mechanism,
  forwarding over existing RPC versus another transport, or a recording commit
  order. Existing short Control frames are not assumed to fit arbitrary calls.

### Option B: Direct authenticated server MCP

Logical path: harness -> authenticated server MCP endpoint -> server tool service
-> owning domain service.

- Avoids a local forwarding hop and provides a direct server-owned endpoint.
- Requires a dedicated, appropriately scoped native-facing authorization
  contract, not inheritance of the Connector operational bearer.
- Scope, credential issuance/expiry/rotation and resumed-session behavior must
  be enforced independently of model arguments and MCP transport-session IDs.
- Adds a server MCP access boundary and operational exposure. It need not be
  public Internet exposure, but must be reachable from the harness environment.
- Current Connector route checks are not automatically a reusable permission
  for direct calls; any dependency on that route requires an explicit decision.

### tools-261112/ADR-D1 — Connector-side MCP relay

Accepted by the requester on 2026-11-12: Option A.

The harness reaches Chapchoo-provided tools through a local MCP surface managed
within the Connector's execution environment. The Connector mediates the exact
native caller scope and server forwarding; the server still authorizes each
invocation and the owning domain service executes business behavior.

This aligns provision with the existing execution owner and leaves Connector
operational credentials outside ordinary native credentials. The relay does not
become a domain-state store, universal recovery journal, or second implementation
of server tools.

Option B is not selected: its direct endpoint and separately managed native
server-authentication lifetime are unnecessary for the current execution-bound
scope. This is not a claim that direct MCP is invalid for other products.

Costs retained by this decision: local-surface ownership, caller-scope binding,
bounded forwarding, and truthful server-disconnection behavior.

Local transport, local authentication mechanism, server-facing RPC, session
recording timing and wire-version compatibility remain pending. In particular,
this decision does not authorize inheriting the Connector bearer, trusting
localhost as authentication, or placing arbitrary work in short Control calls.

Do not select an unsupported MCP-over-ACP transport merely because an SDK
offers an attachment helper. Released Codex ACP evidence supports stdio/HTTP,
while actual version and negotiation remain feasibility work in P2.

## P2a — Harness-Facing Local MCP Transport

Question: within ADR-D1's local relay boundary, should the harness connect to a
Connector-managed local HTTP MCP endpoint or start a stdio MCP bridge?

Related outcomes: `tools-261112/REQ-1`, `REQ-2`, `REQ-3`, `REQ-6`.

### Option H: Connector-managed local HTTP MCP

- A listener owned by the local execution serves MCP and forwards authorized
  calls to the server. The harness receives its endpoint declaration.
- Avoids introducing a stdio bridge subprocess plus a second local bridge-to-
  Connector IPC protocol.
- Requires explicit listener lifetime, address admission, request authentication
  and exact Agent/conversation binding. Loopback is not authentication.
- Any native-facing access material must be narrowly scoped and separate from
  Connector operational credentials. Its concrete format, distribution and
  replacement are pending decisions.
- The listener is local to the execution; this is not ADR-D1's rejected direct
  server MCP endpoint or a newly public network service.

### Option S: Stdio MCP bridge

- The harness starts a small local MCP process and speaks standard MCP over its
  stdin/stdout. That bridge forwards through the Connector to server tools.
- No harness-facing TCP listener, port or HTTP Origin boundary is required.
- Requires a bridge executable, bounded process lifetime, protocol-only stdout,
  body-safe stderr, and an authenticated local bridge-to-Connector channel.
- Process launch and local IPC still need exact conversation binding and
  cleanup. Stdio alone does not provide server authority or safe recovery.
- Recreated MCP subprocesses must not retarget another session or repeat work.

### tools-261112/ADR-D2 — Connector-managed local HTTP MCP

Accepted by the requester on 2026-11-12: Option H, the first proposed local
transport option.

The Connector side owns a local HTTP MCP surface in the harness's Computer
execution environment. Native setup supplies that local endpoint; server tool
business logic remains server-owned under ADR-D1.

This avoids the additional stdio bridge process and bridge-to-Connector IPC
boundary. Option S is not selected, and no automatic stdio fallback is added.
The retained costs are listener lifecycle, bounded HTTP handling, local
authentication and exact caller-scope binding. Loopback is not authorization.

The local endpoint is not a direct central-server MCP endpoint or a public
service. Server-facing transport, the credential scheme and replacement,
session recording order and numeric limits remain pending.

Both local transports are supported by the inspected released Codex ACP
declarations. Protocol negotiation and effective new/resumed native connections
still require validation. Do not infer latest MCP revision support from an HTTP
transport name, and do not introduce stdio/HTTP automatic fallback.

## P2b — Connector-to-Server Tool Contract

Question: should the Connector adapt local MCP to a component-authenticated
server RPC service, or relay the MCP protocol to a separate server MCP endpoint?

Related outcomes: `tools-261112/REQ-1`, `REQ-2`, `REQ-4`, `REQ-5`, `REQ-6`.

### Option R: Reuse the generated component RPC boundary

The local MCP surface adapts discovery and invocation into a common server-tool
RPC service. Reuse the existing component transport/authentication facilities
with current Connector authority, rather than giving native code that credential.

- Current code already has Connector-originated request/response RPCs for
  capture, credential delivery and observations; the architecture is not
  limited to the server-to-Connector Control command stream.
- The new tool service needs its own domain contract and current Agent/
  conversation admission. A valid Computer credential alone is insufficient.
- This does not mean adding every tool as a separate Protobuf method, copying
  native MCP requests into arbitrary existing messages, or reusing the current
  short Control command queue and deadline as the tool execution lifetime.
- The local MCP adapter must correctly map schemas, results and protocol/tool/
  transport failures. Common envelopes and supported payload representation
  remain later interface decisions.
- No separate central MCP transport endpoint, initialization/session lifecycle
  or MCP HTTP authentication surface is required for this path.

### Option M: Server MCP endpoint behind the local relay

The server exposes an authenticated MCP endpoint; the local Connector surface
proxies or forwards MCP requests to it while maintaining exact caller scope.

- Can reuse MCP server/client infrastructure and retain protocol-level
  discovery/invocation semantics across the remote hop.
- Adds an MCP transport/access boundary alongside the existing component RPC
  service. Proxying must preserve cancellation, streaming, protocol errors and
  caller isolation rather than blindly forwarding local credentials.
- Whether the local side is a transparent proxy or a terminating adapter is a
  material interface/lifecycle choice, not an automatic consequence of HTTP.
- It can be private to components. This option does not authorize direct
  native access or third-party clients and does not reverse ADR-D1.

### tools-261112/ADR-D3 — Existing component RPC for server tools

Accepted by the requester on 2026-11-12: Option R, presented as option A in the
decision brief.

The harness-facing MCP protocol terminates at the Connector-side local HTTP
surface. That adapter uses a common server-tool service over the existing
generated component RPC transport/authentication boundary. Server tool services
perform invocation authorization and delegate business behavior to their owning
domain services.

This reuses the existing authenticated component relationship without adding a
central MCP transport/access lifecycle. Option M is not selected; no remote MCP
fallback or direct native access is added.

The new service must preserve discovery, schema, result and error meaning across
the protocol mapping. This decision does not require one Protobuf method per
tool, prescribe payload schemas, or place tool work in the current short
server-to-Connector Control command stream.

Exact local authorization, common RPC/payload shapes, bounds, cancellation,
session recording and actual harness compatibility remain separate decisions.
There is no common business-state store or universal retry/recovery guarantee.

## Caller Admission — Who Grants Local MCP Access?

Related outcomes: `tools-261112/REQ-2`, `REQ-3`, `REQ-4`; ADR-D1 through ADR-D3.

Both alternatives use current server-side Connector authentication and recheck
Agent/conversation/execution authority per invocation. Neither accepts a model's
Agent/conversation argument, a URL alone, localhost, or a protocol session ID as
permission. Native-facing access material is separate from the Connector bearer.

### Option L: Connector-owned opaque local access capability

- The trusted local execution owner creates a random local MCP access secret
  and retains its permission binding in process state.
- The local binding selects the admitted Agent, product conversation and exact
  execution/native-owner lifetime; the native session's final identity can be
  associated when setup establishes it. The model cannot retarget that binding.
- The harness receives only this local MCP capability in its endpoint
  configuration. It is not accepted as a server RPC credential.
- The Connector maps the capability to its bound scope and uses current
  component authority for RPC. The server independently checks the actual
  Agent/conversation/execution relation and applicable domain permissions.
- No separate server-issued local token or durable auth record is required by
  this option. Losing the local owner loses the local grant; a later exact
  restore must provision access for its current owner, not recover authority
  from stale native configuration.
- This trusts the execution component to bind its native caller correctly,
  within the existing server-validated Computer/execution authority.

### Option C: Server-issued scoped local access grant

- The server admits an access grant for the exact Agent/conversation/execution,
  and the Connector validates or resolves that grant at its local HTTP boundary.
- Provides a central issuance/control point rather than permitting the
  Connector to create a local grant solely from its trusted binding.
- Requires issuance, validation, revocation/expiry and replacement behavior,
  including availability during setup and exact restoration.
- Whether the grant is opaque or signed, and whether it is persisted, would
  require further decisions. This alternative is not automatically a new
  database table or a JWT scheme.
- It is still not the Connector operational credential and does not replace
  server-side invocation authorization.

### Recommendation and Scope of the Choice

Recommend Option L for the current trusted execution-component model. Current
server RPC authorization remains the final access boundary, so central issuance
of an additional local-only credential is not required by the confirmed scope.
Option C is viable when centrally controlled issuance of native local access is
itself a required security boundary.

A local bearer grants its scope to whoever possesses it. Neither option claims
to distinguish the model from arbitrary code running under the same native
identity, or to establish same-UID isolation. Secrets must not enter model
messages, ordinary capture records, or diagnostics.

This choice does not settle exact token encoding/header, listener topology,
setup-readiness admission, TTL, rotation/teardown mechanics or recording order.
Those must be consistent with the selected issuer and current native ownership.

### tools-261112/ADR-D4 — Connector-owned local access capability

Accepted by the requester on 2026-11-12: Option L, presented as option A in the
decision brief.

The trusted Connector execution owner creates and owns a random, local-only
access capability. Its in-memory binding selects the authorized Agent, product
conversation and exact execution/native owner. Native setup receives only this
local access material, never the Connector operational credential. The capability
is not a server RPC credential; the server independently checks current component
authentication, Agent/conversation/execution relations and domain permissions.

This places correct local caller binding with the trusted Connector and avoids a
separate central grant issuance/validation lifecycle. Option C is not selected.
Losing the local owner does not preserve a reusable local grant; exact restoration
needs access provisioned for its current owner, not authority inferred from stale
native configuration.

Possession permits use within the bound scope. This is not same-UID isolation or
a promise to distinguish the model from arbitrary native code. Access secrets
must remain outside model messages, ordinary transcript records and logs.

The decision selects issuance and ownership only. Setup admission before native
identity is known, exact replacement/revocation rules, token delivery details,
TTL, listener topology and recording order remain unresolved.

## Authority Binding and Native Identity

Clarification of ADR-D4 and `tools-261112/REQ-2`, not a new technical decision.
The proposed setup-time A/B comparison was unaccepted and is withdrawn: it
introduced an additional native-readiness gate without an authorization basis.
The earlier accepted ADR-D4 remains unchanged.

The trusted Connector establishes the Agent, product conversation and execution
binding before supplying the local MCP capability. Authenticating a request
resolves that binding; it does not wait for a native session ID to determine
which Agent or product conversation is authorized. Neither native readiness nor
the model's identity arguments replace this binding.

The server still checks current component authentication, execution/conversation
relations and tool-specific domain permissions at invocation. Authentication at
connection establishment fixes the caller scope; it does not freeze permissions
against subsequent revocation or grant every tool's domain access.

Native session identity supports exact restoration and record attribution, not
an additional authorization factor. Current ACP setup obtains the actual
new/resumed native identity before reporting `Ready`; the MCP declaration cannot
assume a new native ID exists beforehand. Recording and compatibility work must
account for that ordering without turning native setup completion into a
separate permission grant.

No generic `tools/call` prohibition until native readiness is selected. This is
not approval of automatic setup-time tool execution, a new startup hook, queuing,
or replay. If the harness makes an actual tool request, normal current authority
and the individual tool's contract govern it. A later setup failure does not
prove an already-dispatched tool had no effect.

Exact owner retirement, replacement and in-flight handling remain lifecycle
work. Session ownership remains with the harness as clarified below; native
attribution follows the existing source-bound capture path.

## Derived Lifecycle Boundaries

These follow from ADR-D4, `tools-261112/REQ-2` through `REQ-4` and the existing
Connector/native ownership model; they are not new alternatives for approval.

- The native owner is composed outside the operational reconnect/key-refresh
  loop. A changed Control connection or operational key does not itself change
  the bound Agent, product conversation or physical native owner.
- New server calls use current component credentials and current route/scope
  validation. A previous route/key cannot grant new work through a retained
  local capability. A link outage is unavailability, not permission to replay.
- A local capability cannot be retargeted to another Agent/conversation. Actual
  owner loss ends its grant; a replacement owner must obtain access for itself,
  not inherit authority solely from saved native configuration.
- Reading a known observation or session record does not re-invoke a tool.
  Cancellation, response loss or source replacement does not prove an effect
  stopped or never occurred.

This establishes no periodic local-token TTL/rotation policy, new native restart
behavior, domain rollback, or universal restart-recovery mechanism. Exact
in-flight drain/cancellation behavior remains operational design work.

## Session Ownership and Existing Replication

The unaccepted P3a server-writer/Connector-writer comparison is withdrawn.
Neither option follows the existing session ownership model. This clarification
applies `tools-261112/REQ-5`; it does not add ADR-D5 or modify accepted ADR-D1
through ADR-D4.

The harness manages the native session and its records. Chapchoo captures
supported incoming ACP events locally; Connector copies that source to the
server. The server retains original captured bytes and derives query projections.
It does not originate the native session's tool responses.

The inspected Codex ACP 2.6.0 reference reports MCP arguments in `rawInput` and
the Codex result/error in `rawOutput`. Its v2 conversion preserves those fields.
Chapchoo already normalizes them into `TOOL_INPUT` and `TOOL_OUTPUT`, publishes
them with the source event/cursor and exposes bounded owner reads. The missing
server-tool feature is provision and invocation, not a missing independent
response-history writer.

The tool service returns its response through RPC/local MCP to the harness.
The harness's resulting session events follow existing capture/replication.
There is no new server session-write prerequisite for returning that response.
No synthetic ACP events, alternate source ordinals, standalone exchange archive,
new generic result-query tool or session-retention policy is introduced.

Missing copied history is not known non-execution. Existing source replication
may catch up retained observations; it cannot recreate a response the harness
never received or authorize replay. Domain effects and tool-specific queries
remain with the owning service.

Actual chosen adapter/native compatibility remains verification work, not
authority to add a second history. See the
[current-system research](../notes/server-tools-261112-current-system-research.md)
for source paths, reference provenance, executed tests and unverified boundaries.

## Required Common Contract Semantics

These are requirements/protocol/convention consequences, not new ADR choices.
Authority: `tools-261112/REQ-1` through `REQ-7`, ADR-D3, the selected MCP
protocol's tool contract and typed-ingress/effect-ownership conventions.

- The common service preserves tool name, declared input schema and supported
  result/error meaning. Equivalent lossless RPC encodings and helper/file layout
  do not need a new product or architecture decision.
- Validate the common call envelope at ingress, then decode/validate arguments
  into the selected tool's operation-specific type before domain execution.
  Transparent forwarding can retain opaque JSON but cannot inspect untyped
  fields to make domain or authorization decisions.
- Keep malformed protocol requests/unknown tools distinct from a recognized
  tool's validation or execution errors. The latter use the MCP tool-error result
  meaning (`isError`), not a fabricated successful response or arbitrary generic
  transport failure. Actual network/RPC failure remains a separate observation.
- Preserve supported text and structured values, omission versus explicit null,
  and numeric meaning through RPC translation. An output schema, when declared,
  must match the tool's supported structured output. Do not silently narrow a
  protocol's structured values or coerce large integers through a floating-only
  container.
- Respect the file-transfer exclusion and explicitly refuse unsupported/oversize
  content rather than truncating it into a complete successful result. A response
  failing validation after an effect does not establish that effect was absent.
- Discovery describes currently available authorized definitions; invocation
  still checks current authority and the tool contract. Duplicate names,
  incompatible stale definitions and removal cannot silently select another
  operation. Live catalog replacement/refresh continuity remains P5 work; no
  catalog revision, fingerprint or hot-reload mechanism is selected here.
- A request's protocol correlation and its process-owned work lifetime are
  distinct. Started work and capacity remain owned beyond a dropped waiter.
  Cancellation signals follow the actually supported protocol; they do not
  prove physical cancellation, rollback or permission to replay.

MCP revisions differ materially. The native 0.166.0 source has a default
initialization-based mode preferring `2025-06-18` and a separate modern mode.
The `2026-07-28` HTTP specification removes protocol sessions/GET resumption and
uses per-request stream closure as cancellation. Do not apply either revision's
wire rules to all clients, enable a new native protocol mode implicitly, or
claim support from the HTTP transport name. The actual supported combination
remains a compatibility gate, with no new application-level versioning system.

## P6a — Admission When Tool Execution Capacity Is Full

Related outcomes: `tools-261112/REQ-4`, `REQ-6`, `REQ-7`; ADR-D3.
The alternatives below led to ADR-D5.

The server tool service must reserve finite execution/memory capacity before
dispatching a domain tool. Relay/HTTP resources also remain independently
bounded. This question selects whether the common service waits for execution
capacity, not a numeric limit or a generic durable operation protocol.

### Option A: Reject before execution; no common waiting queue

When execution or retained-input capacity is exhausted, return an explicit
capacity rejection without dispatching that submission to its tool. Calls that
already obtained capacity keep their owner and permits through actual work.

- No additional queue, waiting-order contract or queued-call lifecycle.
- Short bursts can be rejected even if a slot becomes free shortly afterward.
- Known rejection means that exact submission was not dispatched; losing a reply
  is not knowledge of rejection and never grants automatic replay.
- The caller/tool may decide what to do after a known rejection; the common
  layer does not transparently retry an invocation.

### Option B: A finite in-memory queue before execution

The service can accept a bounded number/byte budget of waiting calls, with a
finite total waiting deadline and process-owned queue entries. Dispatch rechecks
current authority when a slot becomes available.

- Absorbs a bounded burst instead of immediately rejecting every full-slot call.
- Adds queue capacity, ordering, waiting deadlines, queued cancellation and
  shutdown/drain obligations; accepted waiting entries cannot simply disappear.
- Queue admission is not execution. Only authoritative removal before dispatch
  can establish non-execution; a cancelled waiter cannot establish that.
- This is process-owned transient waiting, not durable restart recovery. It
  creates no generic result store or permission to restore/replay lost entries.

### Recommendation and Scope

Recommend A for the initial common foundation. It bounds overload without adding
a waiting-work contract that the confirmed scope does not require. The current
input-work owner demonstrates pre-effect slot/byte reservation and explicit
capacity rejection, but its separate durable input protocol is not inherited
by server tools.

This choice does not select slot counts, byte limits, timeout values, per-tool
recovery semantics, new automatic retries or cancellation/rollback guarantees.
Those remain explicit follow-up bounds or existing tool-owned responsibilities.

### tools-261112/ADR-D5 — Bounded In-Memory Invocation Waiting

Accepted by the requester on 2026-11-12: Option B.

The common server-tool service owns a finite in-memory queue for calls waiting
for execution capacity. Full execution slots alone do not immediately reject a
call when waiting capacity remains. Queue count, retained bytes and total waiting
time are bounded independently of execution slots. Exhausted waiting capacity,
closed admission and invalid authority still have explicit rejection outcomes.

The service owns accepted waiting entries beyond the response waiter's lifetime.
Queue admission is not domain execution. Immediately before dispatch, the service
revalidates current component/Agent/conversation/execution authority and the
selected tool contract. A known queued cancellation or expiry that wins before
dispatch must prevent later execution; losing a waiter alone is not proof that
this transition occurred.

Once dispatch may have happened, cancellation, deadline expiry, disconnection
and shutdown do not establish rollback or non-execution. Started work and its
capacity retain a physical owner, with cancellation/recovery guarantees governed
by the individual tool. The common layer does not replay after uncertainty.

This absorbs bounded bursts rather than adding a general restart-resumable task
platform. Option A is not selected. The queue is transient and process-owned:
there is no new durable dispatch journal, independent result archive, generic
result-query API or second session-history author.

Queue scope/scheduling, exact numeric limits, the RPC waiting/response shape and
shutdown coordination remain detailed design topics. This acceptance does not
silently choose a global FIFO, per-Agent fairness policy, recovery handle,
periodic polling, automatic retries or a new native lifecycle.

## P6b — Sharing Waiting Capacity Across Agents

Related outcomes: `tools-261112/REQ-2`, `REQ-6`; ADR-D5.
The alternatives below led to ADR-D6.

The current backend composes shared service owners for requests from multiple
Agents. A future common tool service therefore needs to state whether one
Agent's waiting burst can fill all waiting capacity and precede every other
Agent. This affects admission and latency isolation, not merely helper layout.
The alternatives below concern one service owner's transient queue, not a
cluster-wide scheduler, durable queue, team quota or new management setting.

### Option A: One shared arrival-ordered queue

All authorized Agents share the same bounded waiting capacity and dispatch in
arrival order when execution slots become available.

- A request admitted earlier waits ahead of one admitted later.
- A single Agent can consume all waiting slots/bytes, so later requests from
  other Agents can be rejected or wait behind that Agent's whole burst.
- Only global waiting/execution bounds are needed; no per-Agent admission budget
  or inter-Agent scheduling contract is introduced.
- Finite deadlines prevent indefinite waiting, but do not provide latency
  isolation between callers.

### Option B: Agent-partitioned waiting under shared total bounds

Keep global waiting count/byte and execution limits, additionally bound one
Agent's waiting count/bytes, and select among nonempty Agent queues fairly
rather than draining one Agent's entire backlog first. Use trusted scope from
ADR-D4/current server validation, never an argument naming a different Agent.

- An Agent's waiting burst cannot use all waiting capacity beyond its own cap.
  Reaching that cap explicitly rejects additional waiting requests for that
  Agent even if some total waiting capacity remains.
- Waiting Agents get dispatch opportunities as slots free up. This does not
  preempt already-running tools or guarantee equal completion times.
- Multiple conversations of one Agent remain distinct targets but share that
  Agent's waiting budget; no permanent one-conversation restriction is created.
- Adds per-Agent budget and fair-dispatch rules in addition to ADR-D5's total
  limits. Numeric caps and exact implementation remain subsequent design work.
- Per-Agent waiting isolation is not a per-Agent execution-slot reservation,
  rate limit, billing quota, priority tier or distributed fairness guarantee.

### Recommendation

Recommend B for a shared multi-Agent service: protect unrelated callers from a
single Agent's waiting burst while retaining common capacity bounds. The
additional state is transient queue ownership, not a new persistent authority.
The choice does not guarantee space when many distinct Agents together exhaust
the service's total bounds.

### tools-261112/ADR-D6 — Agent-Partitioned Waiting and Fair Dispatch

Accepted by the requester on 2026-11-12: Option B.

Within one common tool-service owner, retain shared total waiting count/byte and
execution bounds, add waiting count/byte bounds for each Agent, and distribute
dispatch opportunities among nonempty Agent queues rather than draining one
Agent's complete backlog first. Only trusted current scope determines the Agent.

This protects waiting capacity and execution opportunities from one Agent's
burst. Reaching an Agent's waiting budget explicitly refuses additional waiting
requests for that Agent, even if total waiting capacity remains. Multiple
conversations retain their distinct targets while sharing their Agent's budget.

Option A is not selected: global arrival ordering alone does not isolate
unrelated Agents' waiting capacity or opportunities. The added queue state is
transient and process-owned under ADR-D5, not a durable authority or task store.

This is not preemption, equal completion time, reserved execution slots, a
billing/rate quota, priority classes or cluster-wide fairness. Many Agents can
still exhaust total capacity. Exact numeric budgets and the equivalent local
fair-selection algorithm remain Design details; this decision does not grant
catalog replacement, automatic replay, a result lookup API or native restart.

## P6c — Does the Connector Observe Waiting and Dispatch Before the Result?

Related outcomes: `tools-261112/REQ-4`, `REQ-6`; ADR-D3, ADR-D5, ADR-D6.
The alternatives below led to ADR-D7.

The common component service now owns bounded waiting and execution. Either
option retains one original invocation across waiting/execution and does not add
a lookup/recovery endpoint. The current component transport supports generated
streams; the existing short unary helper/Control budgets are not the new
invocation lifetime. This is a service interface/lifecycle decision, not a
method name or helper choice.

### Option A: One unary invocation, final outcome only

Keep the RPC pending through queue waiting and tool execution, then return the
final result or explicit non-execution outcome. The Connector has no authoritative
queue/start observation before that response.

- Minimal message protocol; local MCP returns the final tool outcome normally.
- While waiting, the Connector cannot distinguish server queue waiting from
  execution or an unanswered request. A lost final reply leaves that distinction
  unknown rather than permission to infer non-execution.
- Proper total deadlines and process-owned work remain required; unary does not
  mean that cancelling its request future may abandon physical effects.

### Option B: A server response stream with bounded lifecycle observations

Send queue admission when the service actually owns the waiting entry, dispatch
start when work crosses that boundary, and a final outcome when known.
Direct-to-execution calls need no fictitious queued event. The Connector receives
these on the original invocation response stream, not by polling.

- Separates waiting from started work before final completion, useful now that
  common waiting and fair Agent dispatch are part of the contract.
- Adds an ordered, bounded internal event protocol and stream/backpressure
  ownership. Losing the response consumer must not detach started work or leave
  unlimited progress messages buffered in memory.
- A received queue acknowledgement proves admission at that moment, not that
  execution could not start later. Absence of a start event is not proof of
  non-execution; cancellation/rejection evidence must come from the work owner.
- The final event carries the same supported tool result/error meaning as A.
  Ending without a final event is incomplete/uncertain, not a completed success.
- This creates no durable stream cursor, event replay, reconnect lookup or
  independent session-history writes. Reconnecting must not resend the tool.

### Recommendation and Scope

Recommend B: preserve explicit queue/start boundaries across the server-to-
Connector hop rather than hiding them behind one long unanswered request.
The stream is an internal component contract. It does not authorize new user
status UI, assistant narration, a new native control mode or custom session
events. Any local MCP progress behavior must obey its actual supported protocol
and negotiated capabilities; it is not implied by choosing internal streaming.

Cancellation and shutdown still retain physical work owners under either option.
Numeric limits, precise event shapes and local MCP progress support remain
separate from the choice of final-only versus lifecycle-aware RPC response.

### tools-261112/ADR-D7 — Bounded Invocation Lifecycle Response Stream

Accepted by the requester on 2026-11-12: Option B.

The common server-tool invocation uses one server response stream to report
actual queue admission, actual dispatch start and a final outcome when known.
An invocation that can execute immediately need not invent a queue transition.
Queue acknowledgement occurs only after the service owns the waiting entry.
The final response preserves the same tool result/error contract as a unary
invocation; there is no polling-based result lookup.

This exposes the waiting/execution distinction introduced by ADR-D5 and ADR-D6
to the Connector without a second request lifecycle. Option A is not selected.
The event count, payload and response buffering remain bounded. Loss of the
response consumer cannot detach physical work, release running capacity early,
or create an unbounded stream of progress.

Observations are not replay authority. A queue acknowledgement does not prove
work remained queued afterward; missing a start observation does not prove
non-execution. Ending before a final outcome is incomplete/uncertain, not
completed success. Reconnection does not submit that invocation again.

This is an internal component contract, not a new user UI, assistant narration,
native control mode, source ordinal or session-history writer. It adds no
durable cursor, stream replay, recovery handle or generic result-query API.
Local MCP progress support is not implied; it must follow the actually supported
protocol/capabilities. Numeric limits, precise wire shapes and shutdown/cancel
coordination remain detailed design work under the accepted ownership rules.

## Derived Response and Shutdown Boundaries

REQ-4/REQ-6, ADR-D5 through ADR-D7 and the existing effect-ownership rules
determine the following without another queue/result store:

- Stream observations describe facts at their actual transition, not intent.
  A bounded receiver buffer cannot be the sole owner of a queued/running call.
  Missing a response cannot itself release execution capacity.
- Explicit cancellation/expiry before dispatch must arbitrate with dispatch
  under the queue owner. Only a known pre-dispatch terminal transition can be
  reported as not executed. If dispatch won, retain the started-work owner and
  apply the tool's own cancellation guarantees.
- Closing new admission is distinct from killing started work. Current
  component authority may be withdrawn during backend shutdown; queued calls
  cannot bypass that withdrawal when revalidated for dispatch.
- Current backend cleanup has a finite graceful period and forced-failure
  boundary. A hard process loss does not preserve the new transient queue or
  supply generic outcome recovery. Missing final observations remain unknown;
  a tool with stronger durability/recovery owns that guarantee separately.

Exact shutdown ordering/budgets and cancellation signalling still need the
complete Design and feasibility audit. These derived boundaries do not permit
unbounded draining or change Computer power/native restoration behavior.

## P2c — Initial Local MCP Protocol Profile

Option A was selected and is recorded as ADR-D11. The comparison below explains
that choice; it does not authorize a second operating mode.
Related outcomes: `tools-261112/REQ-1`, `REQ-3`, `REQ-4`, `REQ-5`, `REQ-7`;
ADR-D1 through ADR-D4. ACP stays v2 under both options.

Official Codex 0.167.2, commit
`ce21072c90d32c40af7a55b60b2fb35c84a43a6a`, selects the initialize-based
`2025-06-18` preferred profile by default. The separate `mcp_2026_07_28` and
`mcp_tool_list_changed` feature entries are marked under development and disabled
by default. The latter is independent of the protocol-mode switch.
The relevant reference files were byte-checked against that official commit.
This is source evidence, not proof of a currently deployed native version.

### Option A: Implement the current native default profile initially

Initially target MCP `2025-06-18` over local Streamable HTTP with the existing
native initialization mode. Use its tool/schema/result and cancellation rules,
while preserving ADR-D7's separate component-RPC response stream.

- Meets the current text/structured tool provision and invocation needs without
  enabling the native's under-development protocol mode.
- Establishes a narrower, explicit interoperability target; it does not promise
  `2026-07-28` result envelopes, discovery/MRTR or modern cancellation semantics.
- The producer calls this mode `Legacy`, but this is not an additional Chapchoo
  legacy fallback. Do not add old HTTP+SSE endpoints, stdio fallback, ACP v1,
  arbitrary protocol downgrade or an application versioning layer.
- Effective negotiation, new/resume provision and result fidelity still need
  actual selected-combination tests. Source support is not final acceptance.

### Option B: Explicitly enable and target the native modern profile initially

Use MCP `2026-07-28` from the start and supply the required native feature
selection through the verified launch/configuration boundary.

- Aligns the foundation with the newer protocol's request metadata, discovery,
  result envelopes and request-stream cancellation semantics.
- Changes the native configuration contract and depends on an explicitly
  under-development capability in the inspected released native.
- Requires additional effective-mode, new/resume, HTTP and result/error
  compatibility evidence. It does not automatically enable live tool-list
  refresh, MRTR consumers, additional domain tools or every optional feature.
- Does not permit silently switching protocols or creating a fresh native
  context after failure.

### Recommendation

Recommend A for the initial foundation: the confirmed common-tool outcomes do
not require the newer mode, and current native behavior already supplies the
needed initialization-based path. Choose B only if adopting the newer protocol
now justifies its extra native configuration and compatibility risk.

This chooses an interoperability profile, not a mandatory permanent package
version pin, a second configurable operating mode, SDK dependency, new session
writer or domain persistence contract. Which exact supported SDK/native
combination passes the profile remains a feasibility/verification obligation.

Requester choice: Option A, recorded as ADR-D11.

## D8 — No Session-Based Chat Reconstruction in the Initial Delivery

Reference: `tools-261112/ADR-D8`.
Decision date: 2026-11-12.
Authority: requester-confirmed initial scope in
[tools-261112/REQ](../requirements/tools-261112-server-provided-tools.md#non-goals);
related preservation constraints: `tools-261112/REQ-3`, `REQ-5`, `REQ-8`, `REQ-9`.

The initial delivery does not reconstruct or import chat messages from native
session history. Exact native-session resumption and existing captured-history
copying remain unchanged. This records a confirmed product recovery boundary,
not a new technical recovery mechanism.

### Alternatives and rationale

- No session-based reconstruction: keep initial text publication bounded and
  avoid presenting partial native evidence as a complete recovered chat.
- Bounded historical import: could retain evidenced user/publication entries,
  but requires additional identity, ordering, deduplication, deletion and missing
  evidence contracts. It is outside the initial delivery.

Server-accepted inputs may not enter native history, and server publication
could precede a lost native tool response. Existing session records therefore
cannot universally reproduce the original bidirectional chat.

### Consequences and limits

- No automatic or manual session-to-chat import/rebuild feature is added.
- Reading or recopying existing history does not invoke publication again or
  promote default native assistant text into intentionally published messages.
- This decision does not authorize deleting current history, changing exact
  restoration, or removing the existing capture-copy path.
- Bidirectional chat source B remains under consideration; this scoped choice
  alone does not accept its storage, retention or commit-success contract.
- No backup feature, universal recovery promise, implementation or deployment
  is authorized. Existing ADR-D1 through ADR-D7 remain unchanged.

## D9 — Shared Temporary Delivery Data, Independent Session and Chat Data

Reference: `tools-261112/ADR-D9`.
Decision date: 2026-11-12.
Related outcomes: `tools-261112/REQ-5`, `REQ-8`, `REQ-9`.

Use temporary delivery data jointly for the session/chat delivery path. Once
accepted, session data and chat data are managed independently, for both user
input and Agent publication. The separation is between data responsibilities,
not a split in which user messages belong only to the harness and Agent
messages belong only to server chat.

### Alternatives and rationale

- A chat projection whose authoritative content is derived from captured tool
  execution history couples chat availability and lifetime to those records.
- Independently managed accepted chat data, alongside harness-owned session
  data, separates conversation messages from execution history while sharing
  temporary delivery data. This is the selected boundary.

The shared temporary data is delivery state, not the permanent source of either
accepted record. Session data continues to follow harness ownership and existing
server copying. Chat includes both user and Agent messages; it does not need
tool-record payloads to be useful.

### Consequences and remaining decisions

- Do not introduce independent duplicate delivery mechanisms merely because
  accepted session data and chat data have different owners and lifetimes.
- Shared delivery does not make session and chat commits atomic, impose equal
  retention, or authorize changing one record when the other is removed.
- Exact acceptance boundaries, temporary-data release conditions, identifiers,
  correlation, ordering, duplicate handling, failure and publication-success
  semantics remain pending. No shared physical table, new durable journal,
  generic all-tool queue or automatic retry is selected by this decision.
- ADR-D8's initial reconstruction exclusion and the final full-record UI
  agreement remain in force. No implementation is authorized.

## D10 — User Chat Receipt at Server Input Acceptance

Reference: `tools-261112/ADR-D10`.
Decision date: 2026-11-12.
Authority: [tools-261112/REQ-10](../requirements/tools-261112-server-provided-tools.md#req-10-user-chat-receipt-at-server-input-acceptance).
Related decisions: `tools-261112/ADR-D8`, `ADR-D9`.

Establish accepted user chat data when the server accepts a complete authorized
input. Do not wait for native insertion/injection or captured-history copying.
Native delivery continues through the shared temporary delivery data.

### Alternatives and rationale

- Server input acceptance: retain what the user successfully submitted to
  Chapchoo even when native delivery is pending, fails or remains uncertain.
- Native-first chat receipt: leave the submission outside accepted chat until
  a known native acceptance result. This would omit never-delivered submissions
  from accepted chat.

The requester selected server input acceptance. This extends chat receipt
without changing the established native forwarding/steering authority.

### Consequences and limits

- Native delivery failure and temporary-data cleanup cannot implicitly remove
  an already accepted user chat message.
- Partial upload progress is not accepted chat. Server acceptance must truthfully
  establish both its accepted-input responsibility and the accepted chat record;
  exact database transaction mechanisms remain Design work.
- Chat receipt, native consumption, session observations and browser rendering
  remain distinct facts. Accepted chat does not authorize redispatch.
- There is no indefinite retention, new retry key/protocol, historical import,
  work-state declaration or implementation authority.
- Agent publication receipt/success, remaining temporary-data release,
  ordering, source replacement and deletion remain to design.

## D11 — Initial MCP 2025-06-18 Interoperability

Reference: `tools-261112/ADR-D11`.
Decision date: 2026-11-12.
Related outcomes: `tools-261112/REQ-1`, `REQ-3`, `REQ-4`, `REQ-5`, `REQ-7`.
Related decisions: `tools-261112/ADR-D1` through `ADR-D4`, `ADR-D7`.

Initially target MCP `2025-06-18` over the selected local Streamable HTTP boundary,
using the inspected native default initialization mode. Do not explicitly enable
the native `mcp_2026_07_28` feature for this delivery.

### Alternatives and rationale

- Native default profile: meets the initial tool discovery, invocation and
  text/structured-result needs without a new under-development native feature.
- Explicit modern profile: changes native configuration and HTTP/result/
  cancellation semantics, requiring additional effective-mode verification.

The requester selected the default profile. The initial publication consumer
does not require the modern profile's added mechanisms.

### Consequences and limits

- Apply the selected revision's actual initialization, result/error and transport
  cancellation semantics. ADR-D7's component response stream is a separate
  internal contract, not a change to the local MCP protocol.
- Do not add old HTTP+SSE transport, stdio fallback, ACP v1, implicit protocol
  downgrade, an application versioning layer or a second configurable mode.
- Modern discovery/MRTR, modern result envelopes and live tool-list refresh
  are not promised. Catalog continuity remains a separate pending decision.
- Verify effective protocol, endpoint, authorization, tool names/schema and
  result semantics in both new and exactly resumed native sessions. Source
  inspection is not authenticated MCP acceptance or deployed-version evidence.
- This does not pin the native package forever or authorize a native restart,
  implementation, provisioning, model call or deployment.

## D12 — Stable Bidirectional Chat Order by Completed Server Receipt

Reference: `tools-261112/ADR-D12`.
Decision date: 2026-11-12.
Authority: [tools-261112/REQ-11](../requirements/tools-261112-server-provided-tools.md#req-11-stable-chat-order-by-completed-server-receipt).
Related decisions: `tools-261112/ADR-D9`, `ADR-D10`.

Use one stable conversation order based on completed server chat receipt for
both user and Agent messages. Delayed native capture does not reorder accepted
chat; concurrent publications follow actual chat receipt rather than native
invocation-start order.

### Alternatives and rationale

- Server chat receipt order: provides a stable common ordering authority for
  both message directions, including user messages never delivered to native.
- Sender/source creation time: permits late arrival and different clocks to
  change the conversation order.

The requester selected server chat receipt order. Native capture ordinals
cannot represent every accepted user message and remain session-copy identities.

### Consequences and limits

- Establish order at the completed chat receipt boundary, not browser response
  arrival, local clocks or later history ingestion.
- Session data retains its independent source order and ownership.
- The Design must support concurrent acceptance and stable paged reads without
  late insertion ahead of already accepted messages.
- Physical sequence storage, local identifiers and equivalent transaction
  helpers remain implementation details under this selected contract.
- No application versioning, historical reconstruction, native resequencing or
  full-record UI arrangement is authorized by this decision.

## D13 — Preserve Retained Original User Inputs in Chat at Application

Reference: `tools-261112/ADR-D13`.
Decision date: 2026-11-12.
Authority: [tools-261112/REQ-12](../requirements/tools-261112-server-provided-tools.md#req-12-preserve-retained-original-user-inputs-when-applying-the-feature).
Related decisions: `tools-261112/ADR-D8`, `ADR-D9`, `ADR-D10`, `ADR-D12`.

Transfer existing non-deleted pending/failed inputs whose original user text
remains on the server into chat data when the feature is applied. Preserve
their existing native delivery facts and authority without dispatching again.

### Alternatives and rationale

- Transfer retained original inputs: preserve already received user content
  under the new independent chat-data contract.
- Begin chat only with later submissions: leave existing retained input solely
  in the old temporary delivery path and omit it from the new chat data.

The requester selected transfer. It is a data-preserving application step,
distinct from the excluded reconstruction of messages from native history.

### Consequences and limits

- Move actual retained original content only. Do not resurrect deleted bodies,
  infer absent messages, import native assistant text or fabricate publication.
- Establish a deterministic ordered chat prefix before new admissions; retain
  any available original acceptance metadata without claiming an unknown exact
  historical chat commit sequence.
- Preserve ownership, delivery uncertainty, consumed-input guards and all
  remaining delivery/cleanup facts. Migration does not authorize input replay.
- The application boundary must prevent old/new writers from accepting input
  without the required chat receipt. Concrete preserving migration/activation
  belongs in the complete Design for both supported databases.
- No backup feature, application versioning, live DB mutation, implementation
  or deployment is authorized by this design decision.

## D14 — Message-Only Chat and Expandable Actual Full Records

Reference: `tools-261112/ADR-D14`.
Decision date: 2026-11-12.
Authority: [tools-261112/REQ-13](../requirements/tools-261112-server-provided-tools.md#req-13-separate-chat-messages-from-inspectable-full-records).
Related decisions: `tools-261112/ADR-D9`, `ADR-D10`, `ADR-D12`.

Keep chat messages in the chat surface and actual harness observations in the
full-record surface, using expandable record items. Publication appears as one
tool-call item with readable text/result details and retained original fields.
Do not add a second native assistant utterance merely because chat accepted a
publication.

### Alternatives and rationale

- Reuse inspectable actual record items: keeps the chat readable while preserving
  native output and tool evidence without a second history author.
- Synthesize another conversation utterance into full records for publication:
  duplicates presentation and risks confusing chat receipt with native history.

The requester confirmed the first mapping as the final presentation agreement.
Actual native user/default/final/thought/tool/request records remain available;
undelivered user chat does not create a fabricated native-input item.

### Consequences and limits

- Label default output distinctly from intentionally published chat.
- Preserve existing live permission/form controls and read-only history records.
- Tool details show what the harness actually observed, including lost/error
  results; they do not invent physical publication outcomes.
- Existing components are presentation seams, not permission to change unrelated
  layout, native ownership or product-work state.
- Overall Design approval and implementation remain separate.

## Evidence Entry Points

### D14 confirmation-scope clarification

The requester clarified that the detailed visual distinction between intentional
messages and native/model text remains undecided and requested rendered variants.
The earlier assent covers the broad message-only chat and expandable-history
direction. D14's detailed presentation wording is a proposal within that direction,
not a selected visual design. D1–D13 are unchanged. Whole-Design approval is on
hold pending the visual decision and a refreshed authority/feasibility audit.
This clarification preserves the earlier entry as discussion history.

### D14 subsequent rendered selection

After comparing component renders, the requester selected option 1: uniform
expandable record rows. User input uses the user icon; intentional Agent
publication uses a paper-plane icon; native/model text uses the ordinary text
icon. Only message rows carry the current primary-color left rail. They have no
visible sender-name/message labels or label-background highlight. Model output
has an output label without the Agent name. Collapsed Agent publication previews
the normalized message-body beginning, as user messages do, rather than its tool
action title. Expanded publication retains the readable sent body and original
input/result. Collapsed and expanded views were requested for review.

This records the later explicit presentation choice without altering the earlier
agreement history. D1–D13, harness authorship and the one-publication-tool-item
mapping remain unchanged. Complete Design approval and implementation remain
separate. Fine icon paths and equivalent component adapters are local details.

- [Native setup](../../../rust/crates/harness/src/runtime/protocol.rs)
- [Setup readiness and captured source](../../../rust/crates/harness/src/runtime/setup.rs)
- [Connector/source authority](../../../rust/crates/persistence/src/conversations/authority.rs)
- [Current Control contract](../../../proto/chapchoo/control.proto)
- [Connector-originated RPC contract](../../../proto/chapchoo/conversations.proto)
- [Component capture adapter](../../../rust/crates/rpc/src/conversation_capture.rs)
- [Native ACP normalization](../../../rust/crates/service/src/conversation/normalize.rs)
- [Connector reconnect and owner lifetime](../../../rust/apps/chapchoo/src/connector.rs)
- [Existing bounded input-work ownership](../../../rust/crates/service/src/conversation/input_work.rs)
- [Shared backend service composition](../../../rust/apps/chapchoo/src/runtime.rs)
- [MCP tool semantics, 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [MCP tool semantics, 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
- [MCP HTTP lifecycle, 2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)
- [Codex 0.167.2 protocol modes](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/rmcp-client/src/protocol_mode.rs)
- [Codex 0.167.2 feature defaults](https://github.com/openai/codex/blob/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/features/src/lib.rs)
- [Native ownership](../spec/domain/native-harness.md)
- [Control lifetime](../spec/domain/computer-control-rpc.md)
- [Session storage](../spec/domain/conversation-storage.md)
- [Released Codex MCP transport normalization](https://github.com/agentclientprotocol/codex-acp/blob/f3208913693bd4d5171d6c6b9be5ec1ff96c6431/src/McpServerConfig.ts)
