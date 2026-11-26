---
title: "Agent Integrations: First MCP Connection Requirements"
created: 2026-11-26
document_role: primary
document_type: requirements
snapshot_id: mcp-261126
tags: [agent, mcp, frontend, backend]
---

# Agent Integrations: First MCP Connection Requirements

- Snapshot: `mcp-261126`
- Document reference: `mcp-261126/REQ`

## Problem

Chapchoo lacks a way to configure external integrations for a specific Agent and use them from that Agent. For the first integration type, a user cannot add an external MCP server and use its tools in Chat without configuring the Computer or harness separately.

## Primary Context

### Primary Actor

A user authorized to manage the target Agent's settings.

### Primary Scenario

The user opens the Agent's detail page, adds an external MCP server URL and authentication settings, optionally tests the connection, and saves it. On the next execution in the same Agent's Chat, the Agent can discover and call compatible tools from that server, subject to the harness's existing approval behavior, and use the result to complete a representative task.

## Supporting Scenarios or Effects

- The user can correct a URL, replace a token, or remove a connection from Agent settings.
- An unavailable external connection does not prevent the Agent from handling unrelated work.

## Goals

- Establish Agent-scoped external integrations, with MCP as the first supported type.
- Define the first-release MCP profile and observable boundary from configuration through actual tool use.
- Leave room to add OAuth within the same Agent connection flow later.

## Non-Goals

- Shipping a second integration type or defining its product behavior in this release.
- User-owned or Team-shared connection catalogs, or a separate per-connection on/off state.
- OAuth login or renewal, private destinations, local stdio, legacy HTTP+SSE transport, or older external MCP revisions.
- Separate MCP Resources or Prompts, image or file results, tool requests for additional input, or live tool-list change subscriptions.
- Guaranteed adoption during an already-running execution or rollback of a request already sent to an external server.
- Changes to the existing built-in `send` contract or to the harness's approval policy.

## Requirements

### REQ-1. Manage connections on the Agent

- The Agent detail page shows an **MCP servers** section between execution configuration and Computer settings. It offers **Add** when the list is empty or populated, and shows each saved connection's name, URL, and **Edit** action.
- Add and edit use the same editor for the display name, server URL, and authentication mode. Selecting Bearer shows its token field. A previously saved token is not displayed and can be replaced in the editor. **Delete** is available from the edit view.
- Saving an addition registers the connection for that Agent without a separate activation action, state, warning, or confirmation. Saving does not require a successful connection test. Another Agent does not inherit the connection, even if it has the same owner or Team. Existing Agent management authority governs settings access.

### REQ-2. Connect from the Chapchoo server using the declared profile

- Discovery and actual external calls originate from the Chapchoo server, using a publicly reachable HTTPS MCP endpoint, Streamable HTTP, and either no authentication or a supplied Bearer token. The external token is not delivered to the Computer, Connector, harness, or Agent as configuration or a tool argument.
- Chapchoo speaks external MCP `2026-07-28` only. It does not fall back to an older revision. A confirmed version mismatch is distinguishable from failed authentication; an unclassified response is not reported as a proven version mismatch.
- The first release does not offer OAuth. Adding it later must not require a separate connection-to-Agent flow.

### REQ-3. Test discovery without executing a tool

- The optional **Test connection** action checks the current editor inputs from the Chapchoo server. It lists tools accessible under the selected authentication, follows pagination, and never sends `tools/call`.
- The editor shows the confirmed tool count, including zero, or a failure with the known cause and applicable recovery action. A partial traversal is not reported as a complete list. A successful test is neither saved registration nor evidence of successful Agent use.

### REQ-4. Discover and use the tools in the same Agent's Chat

- On the same Agent's next execution, saved additions and edits take effect automatically without a separate conversation, manual Apply, or restart. Applying the change does not require interrupting ongoing work. Earlier application is permitted but is not a first-release guarantee.
- When a tool list is expired at execution time, Chapchoo refreshes it. A list discovered with old authentication is not reused under changed authentication. A real-time subscription is not required.
- A compatible tool accepts JSON arguments and supplies text or structured JSON to the Agent. Structured-only results are usable without a text duplicate. Use of an unsupported result or a tool that requests follow-up input reports the limitation without falsely reporting completion or resending the call to repair result handling.
- The harness's existing policy decides whether to prompt for a call or proceed automatically. Chapchoo adds neither a new approval step nor an approval bypass for external tools.
- Acceptance exercises discovery and a representative completed Chat task for both no-auth and Bearer configurations, including a paginated list, a text result, and a structured-only result. A settings save or discovery trace alone does not meet this criterion.

### REQ-5. Keep other work available and preserve truthful outcomes

- A failure in one external connection does not prevent Chat, built-in `send`, or independent external connections from working. A task that needs the failing connection receives its known failure reason and an applicable recovery action, such as replacing an expired token.
- A request known not to have run, a tool-reported failure, and an unconfirmed outcome after possible dispatch are distinguishable. A tool-reported failure is not evidence of rollback. Chapchoo does not automatically resend a call whose outcome is unconfirmed.
- Once deletion is saved, show the connection as deleted without waiting for, cancelling, or checking previously admitted tool calls. Calls not yet admitted are rejected after deletion, even from an execution that still has the old tool list. A previously admitted call can still be sent or complete; neither deletion nor Chat Stop promises to undo it.

### REQ-6. Start with MCP without limiting Agent integrations to MCP

- The first release supports only MCP and retains the Agent detail flow in REQ-1. It does not expose unsupported integration types or add a type-selection step.
- The design explains how a future, non-MCP integration can be managed on an Agent alongside existing MCP connections, without converting those connections or requiring the new integration to use the MCP protocol. A design walkthrough is sufficient for this requirement; shipping or prototyping another integration is not required.

## Fixed Constraints

- Use released external harnesses through their supported integration interfaces. A Chapchoo-maintained harness fork or patch is not part of this feature.
- Preserve the private Agent boundary and the existing built-in MCP behavior while keeping external credentials on the server side.

## Open Assumptions

- The representative endpoints and released harness build matrix for acceptance remain to be selected and validated. Existing built-in MCP discovery establishes neither external protocol interoperability nor support for every result shape.
- The implementation mechanisms for next-execution adoption, connection failure isolation, and rejecting new calls after deletion are not selected by this document.

## Confirmation

REQ-1 through REQ-5 were confirmed by the requester on 2026-11-26 before ADR and design decisions began. On the same date, the requester clarified that the feature establishes Agent integrations generally and that MCP is the first supported type. REQ-6 records that direction without changing the first-release MCP behavior.

On 2026-11-26, the requester directed that a saved deletion be shown immediately without considering ongoing tool calls. REQ-5 records that update: deletion ends new admission, not execution of work already admitted.
