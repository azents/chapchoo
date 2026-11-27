---
title: "Remote MCP Tool Compatibility Design"
created: 2026-11-27
document_role: primary
document_type: design
snapshot_id: mcp-261127
tags: [agents, mcp, compatibility, security]
---

# Remote MCP Tool Compatibility Design

## Scope and baseline

This design implements [mcp-261127/REQ](../requirements/mcp-261127-remote-tool-compatibility.md) and the accepted [mcp-261127/ADR-D1](../adr/mcp-261127-remote-tool-compatibility.md#d1-decide-when-to-try-the-earlier-handshake) on top of the external MCP stack. The stack currently reaches only `2026-07-28` (modern lifecycle) on the public side. The Connector-local endpoint, native Codex configuration, persistence and owner settings interfaces do not change. This design supersedes only the external single-version behavior of the [initial design](mcp-261126-agent-external-connections.md). It does not authorize the deprecated separate HTTP+SSE transport, external OAuth, or ACP.

## Outbound negotiation and method boundary

Use one server-owned connection entry point in the existing external MCP service for settings Test connection, preparation, and invocation. Keep the bounded HTTP adapter and supported `rmcp` Streamable HTTP lifecycles. Use a single policy client for the exact saved or tested endpoint and its selected credentials. The destination remains bound at actual dial with public-address validation and platform TLS verification; neither a server reply nor the SDK can substitute another URL, attach authority, add a redirect, or use a proxy. Requests in either lifecycle share the original operation deadline and capacity permit.

1. Attempt modern `server/discover`, preferring `2026-07-28`. A complete correlated response selects that lifecycle only if the protocol is supported. Parse bounded HTTP status plus recognized, correlated protocol results at the boundary where both are available; an invalid response or conflicting evidence is not converted into a successful handshake. A recognized modern protocol error remains in modern negotiation. With only one supported modern revision, no common version is terminal.
2. Only the approved probe rejection permits a fresh earlier Streamable HTTP handshake at the same URL: HTTP 400 without a recognized modern JSON-RPC rejection, or a correlated method-not-found in a normal HTTP 200 discovery reply. Do not accept an unrelated or contradictory JSON-RPC reply as legacy evidence. Request `2025-11-25` once via SDK `Initialize`, and accept a correlated reply negotiating exactly one of `2025-11-25`, `2025-06-18`, and `2025-03-26`. Verify this before forwarding the SDK's `notifications/initialized` or any tool operation; an SDK that accepts a wider version set does not expand ours. Do not use the SDK Auto mode because it can try the earlier lifecycle on timeout.
3. Use a method allowlist by lifecycle. The modern path keeps discover, list and admitted call. The earlier path adds the initialize handshake and its initialized notification, then list and admitted call using the negotiated MCP protocol header and, if assigned by the server, the current session ID. Validate bounded header-safe session data from the successful initialization, never log or persist it, and never forward it to the Connector. A sessionless server is valid. When the SDK requests cleanup for an assigned session, allow a bounded same-endpoint DELETE using the same policy; its outcome does not change a settled tool result. Do not use standalone GET event feeds, stream resumption, automatic session reinitialization, or additional server-request authority. Accept bounded JSON or SSE replies to POST; SSE reply loss is not a request to reopen a connection or replay a call.

Authentication rejection, redirect, timeout, network loss, malformed success, version mismatch outside the approved negotiation signal, or tool-list failure remains terminal for that operation. Require advertised tools and fully validate every page as today. An earlier catalogue without freshness metadata is not reusable across operations; do not invent a default TTL. Preserve exact settings/auth cache identity and modern earliest-page TTL. Do not cache or persist a preference for a particular protocol or handshake. Use typed diagnostics; replace the existing settings text prescribing `2026-07-28` with guidance to use a supported Streamable HTTP version without exposing raw upstream bodies or credentials.

## Invocation, ownership, and recovery

Only existing exact route/operation/native-target and current settings admission gives a saved tool call authority. Reuse the current store serialization point before opening the upstream invocation; protocol selection grants no authority. Prepare a new bounded upstream connection for each admitted invocation, as today. Share one `SingleDispatch` across its setup and call; only the actual authorized `tools/call` transport dispatch consumes it. No alternate protocol after setup, no new session/retry after expiry, and no replay after possible dispatch. Keep text/structured result mapping and definitive, not-executed, unsupported and uncertain outcomes; for example, a lost answer after possible dispatch remains uncertain. Keep the existing check for protocol-owned schema headers when required; never copy a modern header annotation to a legacy request as ambient authority.

Saved deletion ends new admission at commit. It is independent of the lifecycle selected for any previously admitted call, which may complete using the exact admitted settings. No delete drain, cancellation, or rollback is added. A completed catalogue is not itself admission evidence. Preserve required built-in `send` readiness and approval ordering. Select and apply externally discovered tools using the existing same-Chat next-execution machinery. The change does not close the separately tracked gap in task-specific attribution of an integration omitted at discovery.

## Rollout and operability

A focused compatibility change can be based on the open settings stack. There is no migration, new configuration or selector, control-plane startup dependency, generated client change, or a credential handoff to native clients. Rolling back restores the modern-only client; saved old-only connections will fail their next discovery without being deleted or rewritten. Retain existing typed status and limits; no new upstream body or session identifier is written to logs. Native Codex requires neither a patch nor an extension.

## Test Strategy

- **Representative assembled Chat:** In a disposable installation, connect the browser, Control, Connector, and unmodified released Codex. Create or identify the same Chat before the saved change, test discovery and assert it makes no tool call, save, and use a public earlier no-auth server (the candidate has already answered a read-only call directly). Check the same native conversation, real tool approval, one read-only invocation and result. Retain the original live current-revision and Bearer success evidence obligations, plus pagination and both output forms. Do not claim a direct upstream request, live model smoke alone, or a fake app-server journey as passing this lane. Keep the end-to-end suite representative; exercise the compatibility matrix in focused tests, not one assembled journey per version.
- **Deterministic HTTPS matrix:** Add chapter-level Rust fixtures alongside `external_mcp` tests for the four revisions, no-auth and synthetic Bearer, earlier with/without a session, negotiated protocol header and notification ordering, complete pagination with JSON/SSE, and supported text/structured results. Count exact handshakes and zero `tools/call` for settings tests; test confirmed legacy signal vs modern rejection/401/403/redirect/timeout/invalid success and unsupported startup version. Simulate lost replies and session expiry at call time and verify possible dispatch remains one call. The test-local TLS fixture does not change or prove the production public resolver path. Verify unchanged exact settings, current and built-in tool behavior using the existing focused suites.
- **Split backend acceptance:** On SQLite and PostgreSQL start separate real Client and Control processes, use a controlled public HTTPS fixture for invocation, hold the response after admission, delete via the owner settings boundary, confirm a stale new attempt is denied without upstream dispatch, and release the prior response. Record process/DB mode, barrier order, exact call counts and typed outcomes. No public testing service or private policy exception is implied by this design. The TLS fixture must be provided at an approved route reachable by production policy to count this lane as passed.
- **CI and evidence:** Run deterministic fixture and affected checks in ordinary CI. Live model or external endpoint checks are explicit opt-in; fail a requested live run on missing prerequisites and record it as not run or blocked for the release brief, never as a CI pass. Use disposable seeded accounts through supported product APIs and a permitted model credential, without use of retained product credentials in a fixture image. Redact endpoints with private data, secrets, and session IDs from logs. Keep pass/fail, tool counts, same-target evidence, and release provenance, with separate reporting of third-party reachability and end-to-end results.

## Feasibility and authority audit

- **REQ-1 → M1, M2, M4:** The pinned `rmcp` exposes `Discover` and `Initialize`; its legacy init path sends the initialized notification without an application allowlist, so the adapter must check the negotiated reply version before allowing that notification. The current Streamable HTTP wrapper already intercepts typed responses and bounds SSE bodies. The older public peer negotiated `2025-11-25` and returned a read-only result, while the stock released Codex independently completed a live-model test. Production Chat integration remains conditional on the supported product model-login flow and an assembled run; direct success is not a product result.
- **REQ-2 → M2, M3, M4:** The existing exact-admission, one-shot transport and store ordering have owners independent of the protocol handshake; retained behavior is implementable without changing their interfaces. A controlled publicly reachable HTTPS fixture remains a prerequisite for the held-call test under production policy. SQLite and PostgreSQL process startup has been verified, not the split deletion scenario.
- **Reverse trace:** M1 changes only the diagnosed outbound compatibility boundary; M2 and M3 preserve confirmed operation and authority guarantees; M4 makes the supported range accurately observable. Test isolation and public endpoint availability are feasibility constraints, not permission to alter production policy.

## Design Authority

- Design revision: `1`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Shared, modern-first Streamable HTTP startup with one response-authorized earlier handshake at the same URL | `mcp-261127/REQ-1`, `mcp-261127/ADR-D1` | decided |
| M2 | Bound negotiated protocol and earlier session data to the current operation; preserve the tools-only projection, destination policy, complete discovery and result limits | `mcp-261127/REQ-1`, `mcp-261127/REQ-2`, `mcp-261127/ADR-D1`, `mcp-261126/ADR-D3`, `mcp-261126/ADR-D4` | derived |
| M3 | Retain current settings admission, same-Chat application, one dispatch, native approval, built-in availability and independent deletion | `mcp-261127/REQ-2`, `mcp-261126/ADR-D2`, `mcp-261126/ADR-D4`, `mcp-261126/ADR-D5` | required |
| M4 | Keep existing settings interface and typed statuses; update unsupported-version guidance without exposing protocol internals or treating an ambiguous response as known incompatibility | `mcp-261127/REQ-1`, `mcp-261127/ADR-D1` | required |

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Always modern-only external handshake | `mcp-261127/ADR-D1` | M1; no generic retry or separate transport | Shared service startup for test, preparation and invocation | Focused transport tests pass for all four revisions; no direct discovery-only entry remains for those product callers |
| External adapter's modern-only method envelope | `mcp-261127/REQ-1`, `mcp-261127/ADR-D1` | M2 preserves modern checks and adds a bounded earlier-verification path | Wire adapter only; no broadened component or native authority | Assert same endpoint and legal requests per lifecycle, no GET resume or automatic reinitialization |
| Settings error instructs owners to use only `2026-07-28` | `mcp-261127/REQ-1` | M4 names the compatible connection kind without raw upstream text | Localized MCP diagnostic only | Verify no single-version recommendation in any supported locale |

Retain modern-only rejection cases as modern contract tests where applicable; update tests whose expected earlier rejection is being superseded. No authoritative persistence or generated interface is removed. Keep the prior snapshots as decision history. On verified implementation, update the owning Living Spec sections for external tool delivery and owner settings using the actual dependency diff; do not promote design claims ahead of tests.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Approved on: `2026-11-27`
- Approved Design revision: `1`
- Approved authority IDs: `M1`, `M2`, `M3`, `M4`
- Approved scope: response-based Streamable HTTP negotiation, bounded earlier lifecycle, preserved execution authority, and compatible settings diagnostics. The requester authorized implementation.
