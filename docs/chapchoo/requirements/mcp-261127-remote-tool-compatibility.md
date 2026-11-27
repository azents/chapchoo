---
title: "Remote MCP Tool Compatibility Requirements"
created: 2026-11-27
updated: 2026-11-27
document_role: primary
document_type: requirements
snapshot_id: mcp-261127
tags: [agents, mcp, compatibility]
---

# Remote MCP Tool Compatibility Requirements

- Snapshot: `mcp-261127`
- Document reference: `mcp-261127/REQ`

## Problem

The [initial external MCP requirements](mcp-261126-agent-external-connections.md#req-2-connect-from-the-chapchoo-server-using-the-declared-profile) accept one protocol revision. Public HTTPS servers that offer useful tools via an earlier Streamable HTTP revision cannot be used from Chat even when their authentication and tool capabilities fit the existing feature.

## Primary Context

### Primary Actor

An Agent owner who wants to use a compatible remote MCP server in that Agent's Chat.

### Primary Scenario

The owner enters a public HTTPS MCP server URL, optionally tests discovery, and saves the connection. Whether the server speaks the latest supported revision or an earlier supported Streamable HTTP revision, the owner can use an available tool on the next execution in the same Chat under the existing native approval policy. No provider-specific configuration or manual protocol selection is required.

## Supporting Scenarios or Effects

- The same Agent may connect to servers on different supported revisions. One incompatible server does not block the others or the built-in `send` tool.

## Goals

- Make external tool compatibility general across released Streamable HTTP revisions, rather than adding individual provider exceptions.
- Keep the Agent connection setup and tool use consistent across supported revisions.

## Non-Goals

- OAuth for external MCP servers. Existing no-auth and Bearer configurations remain in scope.
- The deprecated separate HTTP+SSE transport. SSE responses within Streamable HTTP remain in scope.
- Reviving ACP or modifying a released native harness.
- Adding MCP resources, prompts, or server-initiated user interaction to the Chat tool feature.
- Changing the existing ownership, destination, transport security, or approval boundaries as a consequence of protocol compatibility.

## Requirements

### REQ-1. Connect to current and earlier Streamable HTTP servers

**Acceptance criteria**

- With the same connection form and no manual choice of protocol revision, a server using `2026-07-28` or a released Streamable HTTP revision (`2025-11-25`, `2025-06-18`, `2025-03-26`) can be used if it meets the existing authentication and tools contract.
- A representative earlier server's tool catalogue is returned by **Test connection**, which executes no tools. After save, one read-only call succeeds through released Codex in the same Chat. An independent fixture covers the earlier revisions, including servers that require an MCP session and those that do not.
- A confirmed incompatibility is distinguishable from rejected credentials and a general network or malformed-response failure. A partial catalogue does not count as a successful test.

### REQ-2. Keep the existing tool and security guarantees on all supported revisions

**Acceptance criteria**

- Public HTTPS destination enforcement, server-owned no-auth/Bearer configuration, complete pagination, and supported text/structured results continue to apply. Credentials do not reach the Computer or model.
- Chat, native approval, failure isolation, and built-in `send` continue to work. No unresolved tool dispatch is replayed as part of compatibility behavior.
- After a saved deletion, stale declarations cannot admit a new call. Already admitted work may finish. The existing split Client/Control verification on SQLite and PostgreSQL remains an acceptance obligation, not replaced by protocol interoperability tests.

## Fixed Constraints

- Do not follow redirects or loosen dial-time destination or certificate checks.
- Preserve the released Codex interface and existing approval policy; compatibility is with the remote server, not a change to the harness implementation.
- The original requirements remain applicable except for the single-revision limitation.

## Open Assumptions

- The representative public server verifies the earlier no-auth path. Bearer, session expiry, pagination, and deletion ordering also require controlled test fixtures.

## Confirmation

On 2026-11-27 the requester confirmed general compatibility with earlier MCP revisions, without an external MCP OAuth flow or the separate deprecated HTTP+SSE transport in this delivery.
