---
title: "Remote MCP Tool Compatibility Decisions"
created: 2026-11-27
document_role: primary
document_type: adr
snapshot_id: mcp-261127
tags: [agents, mcp, compatibility, security]
---

# Remote MCP Tool Compatibility Decisions

Authority: [mcp-261127/REQ](../requirements/mcp-261127-remote-tool-compatibility.md).
Baseline for analysis: `6dd42c5`, the stacked settings head. The prior [outbound decision](mcp-261126-agent-external-connections.md#d3-keep-mcp-credentials-on-the-server-and-enforce-public-https-at-the-outbound-client) is retained except for its one-revision/no-fallback boundary, which this snapshot revisits.

## Fixed boundaries

- Support the confirmed Streamable HTTP revisions with no provider-specific configuration, the deprecated separate HTTP+SSE transport, or external MCP OAuth. Streamable HTTP SSE responses remain supported.
- Preserve existing public HTTPS policy, server-owned credentials, Connector-local projection, native approval, built-in tools, and authoritative delete/admission ordering. Do not persist a protocol preference, replay a tool call, or change how users save an integration.

## Material decision map

- `mcp-261127/ADR-D1`: evidence sufficient to try the earlier handshake, and the boundary where negotiation must stop. Accepted.
- Operation-local session handling follows from supporting the earlier standard without altering connection settings or dispatch authority. SDK client and fixture shapes are implementation details, not separate user decisions.

### D1. Decide when to try the earlier handshake

Accepted by the requester on 2026-11-27: use bounded, response-based fallback; never treat timeout as a legacy signal. Addresses [mcp-261127/REQ-1](../requirements/mcp-261127-remote-tool-compatibility.md#req-1-connect-to-current-and-earlier-streamable-http-servers) and [mcp-261127/REQ-2](../requirements/mcp-261127-remote-tool-compatibility.md#req-2-keep-the-existing-tool-and-security-guarantees-on-all-supported-revisions).

The current client performs `server/discover` for `2026-07-28` only. A representative older endpoint accepts `initialize` and negotiates `2025-11-25` but rejects the modern request with HTTP 400. The [Streamable HTTP backward compatibility rules](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http#backward-compatibility) distinguish this response from a recognized modern protocol rejection. The selected SDK provides both lifecycles, but its general auto mode can also try `initialize` after a discovery timeout.

- **Selected: try the modern discovery first and fall back once, before any tool dispatch.** On HTTP 400 without a recognized modern protocol error, or a correlated JSON-RPC method-not-found in a normal HTTP 200 reply to the discovery probe, try `initialize` at the same saved URL under the same destination and credential policy. Start the earlier handshake at `2025-11-25` and accept only the three confirmed earlier versions returned by the peer. A recognized modern version error stays in modern negotiation; without a mutually supported version, reject it. Authentication rejection, failure to reach the server, timeout, higher-level redirect, malformed success, or failure after discovery does not trigger a second lifecycle. The earlier handshake is additional bounded negotiation, not proof of incompatibility from an ambiguous error and not a retry of a tool call.
- **Broader automatic fallback:** also try `initialize` when the modern probe times out. This can reach servers that silently ignore the probe, at the cost of more latency and a second request during an ordinary outage. The representative endpoint does not need this.
- **Manual version selection:** avoids probing when the server is already known but adds configuration and cannot satisfy the confirmed automatic flow. Keeping the latest-only policy would leave earlier servers unusable.

A failed fallback cannot be reported as successful discovery. Status reporting keeps actual authentication or known negotiation failures distinct from network or malformed-response failures. Earlier and modern attempts share the overall operation deadline, route admission, and at-most-once dispatch guard. A settled failure or possible call dispatch never opens a new negotiation path for the same call.
