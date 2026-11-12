---
title: "Native-owner server tools and explicit publication"
---

# Native-owner server tools and explicit publication

Server tools connect one actual native owner's local HTTP capability to current
server authority. They are independent of Control commands, native readiness and
captured model output. The sole production tool, `publish_text`, appends an
intentional message to its fixed product conversation.

## Authority and publication

The Connector seals the actual Agent and conversation before a native ID exists.
A local capability grants access only to that recipient under the current
operational Connector session, execution and Control route. Discovery, dispatch,
the publication transaction and protected final-result disclosure independently
check current authority. Private Agent ownership and current Team membership
remain server facts. Native IDs, startup readiness and the UI's selected
conversation are not publication permission.

`publish_text` accepts only a nonempty, bounded UTF-8 `text` field. Unknown
fields, recipient changes, retry keys and work-state arguments are rejected.
Accepted bytes are preserved. The completed transaction allocates the immutable
[chat receipt](conversation-storage.md#independent-chat-receipts); the tool returns
an acceptance text result and structured `message_id`, not the message body.
There is no recall promise after dispatch. Ordinary commentary/final model output
stays in captured records and does not invoke this operation.

## Bounded invocation owner

The server process owns started effects independently of stream/HTTP waiters.
Its common owner admits at most 16 executing and 128 waiting calls. Waiting
payloads are bounded by 8 MiB globally and eight calls/512 KiB per trusted Agent.
Each Agent partition is FIFO; eligible Agents dispatch round-robin. A separate
144-slot ingress/retained-response bound covers preauthorization and response
lifetime without another waiting scheduler.

Calls have a ten-second queue deadline and fifty-second total observation
deadline. Observations are only Waiting, Started and Final, with bounded
buffering. Explicit cancellation arbitrates with dispatch under the same state
lock: a waiting cancellation winner is known nonexecution; a dispatch winner
retains physical ownership and cannot promise rollback. Losing an observer or
timing out the response is not cancellation. Started effects retain capacity
until actual completion even after their response is no longer observable.

Known nonexecution, a complete tool result, uncertainty and uncertain result
representation are distinct outcomes. Oversized/unsupported results after a
possible effect are never reported as success or known nonexecution. No durable
invocation ID, result query, replay, resend or shared cancellation API exists.

Control alone registers the dedicated component `ServerTools` service.
Catalog is unary; Invoke accepts one Submit and optionally one cancellation of
that invocation. Frames are at most 64 KiB; catalog, arguments and results are
at most 48 KiB, with at most 32 catalog definitions. Initial Submit has a
ten-second deadline. Client/Admin groups do not register this service.
Shutdown closes admission and joins forwarding and effect owners; reentrant drain
retains unfinished tasks and collected failures. Unexpected worker unwind closes
admission before queue handoff.

Received RPC opening rejections carry the canonical Rejection name in
`chapchoo-tool-not-executed` metadata only before successful owner submission.
The relay preserves that explicit protocol/admission outcome. An unmarked,
unknown or duplicate marker, transport loss or post-submission stream error
remains uncertain; a gRPC status code alone never establishes nonexecution.

## Local HTTP capability and protocol

The official released MCP SDK serves an ephemeral `127.0.0.1` HTTP `/mcp`
endpoint. A random memory-only bearer appears only in an Authorization header;
it is not a server credential, URL, environment variable or model argument.
The boundary removes that header before SDK contexts/diagnostics and separately
checks Host/Origin. Field-free Debug and sanitized operator errors do not expose
the capability, server credential, arguments or result.
The application independently excludes ACP and MCP SDK diagnostic targets even
when an operator enables TRACE; those dependencies can otherwise serialize
authentication or tool payloads.

Before HTTP parsing, capacity is reserved for at most 128 connections/requests,
16-KiB headers and 64-KiB bodies under an 8-MiB retained-byte budget. Header and
body reads expire after ten seconds. Overload refuses admission rather than
forming a local waiting queue. Arguments and possible results have additional
physical-work reservations.

Only MCP `2025-06-18` is accepted. Newer initialization/version headers,
continuations and event replay are rejected. Each owner capability has at most
four live protocol sessions; explicit session deletion releases its slot. There
is no event store, completed-result retention, SSE retry priming or keepalive.
Only the SDK's typed cancellation notification sends request-local cancellation;
HTTP disconnect and context-drop tokens do not.

The Connector uses a separate long-lived 128-capacity tool RPC channel rather
than the eight-slot Control/authentication channel. Reconnection/key refresh
replaces current server credentials without changing the endpoint capability or
fixed recipient. Route withdrawal prevents calls during disconnection. Native
retirement closes local access and explicitly joins listener, protocol sessions
and retained call tasks. Same-UID isolation is not claimed.

## Native provision and verification

The [native actor](native-harness.md#actual-start-server-tools) requires a
consumer-owned tools provider alongside authentication. On an actual launch it
acquires one provision and sends its header-only HTTP declaration through the
public strict-v2 ACP New or exact Resume request. Missing advertised HTTP MCP
capability rejects setup; another transport is not substituted.

The source/native identity is retained before effective endpoint verification.
Ready requires that native actually initialized the expected local endpoint and
requested its unchanged catalog within the remaining startup budget. A configured
name collision that causes the adapter to ignore the declaration therefore
cannot silently establish readiness. Verification does not call a tool or require
a model turn. Its failure neither erases restoration evidence nor proves that no
earlier effect occurred. An already-live lookup validates the original product
scope and reuses the existing provision without reapplication.

A native identity does not guarantee that the native engine persisted an empty
context. The supported Codex adapter documents that a never-prompted thread has
no rollout until its first user input, and its empty-thread fallback requires
the same still-live app-server. Physical owner loss may therefore make exact
restoration fail even while the original product capture binding remains.
That rejection preserves the exact source; it cannot authorize fabricated input
or replacement with a fresh context. The model-free acceptance separates real
New and this real rejection from successful controlled-fixture New/Resume.
Successful real persisted-session restoration and model-side tool use require
their own evidence.

## Connections

- [Conversation storage](conversation-storage.md) — Stable completed chat receipts and independent captured history.
- [Private conversation web](conversation-web.md) — Message-only Chat and separate actual invocation/output records.
- [Native preparation and lifetime](native-harness.md) — Exact context, actual-start provision and owned retirement.
- [Connector process](connector-process.md) — Current operational routes across reconnect and key refresh.
- [Control services](computer-control-rpc.md) — Component authorization and separate command/peer boundaries.
- [Foreground ownership](runtime-process.md) — Admission closure and combined fallible cleanup.

## References

- [`proto/chapchoo/server_tools.proto`](../../../../proto/chapchoo/server_tools.proto)
- [`rust/crates/service/src/server_tools.rs`](../../../../rust/crates/service/src/server_tools.rs)
- [`rust/crates/rpc/src/server_tools.rs`](../../../../rust/crates/rpc/src/server_tools.rs)
- [`rust/apps/chapchoo/src/connector/server_tools.rs`](../../../../rust/apps/chapchoo/src/connector/server_tools.rs)
- [Server-tools decisions](../../adr/tools-261112-server-provided-tools.md)
