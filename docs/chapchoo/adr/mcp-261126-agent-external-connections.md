---
title: "Agent Integrations: First MCP Connection Decisions"
created: 2026-11-26
document_role: primary
document_type: adr
snapshot_id: mcp-261126
tags: [agent, mcp, architecture, security]
---

# Agent Integrations: First MCP Connection Decisions

Authority: [mcp-261126/REQ](../requirements/mcp-261126-agent-external-connections.md).
Baseline: `875f69c`.

## Fixed boundaries

- MCP is the first integration type. The first-release UI remains MCP-specific; future integrations need not speak MCP upstream or expose tools.
- For MCP, the Chapchoo server owns external discovery and invocation and keeps the external credential out of the Computer. The supported upstream MCP profile is `2026-07-28` only. The harness-facing protocol is a distinct boundary; this requirement does not upgrade the built-in tool endpoint.
- Released harnesses receive connections through their supported interfaces. External declarations do not inherit any built-in `send` approval override. A connection being saved, a tool being visible, and a call being admitted are distinct facts.
- One failed external connection must not block Chat or independent tools. Saved deletion ends new admission but is independent of already admitted calls. Possible dispatch is not permission to replay.

## Material decision map

1. **Shared Agent integration boundary — accepted in D1.** Affects REQ-1 and REQ-6.
2. **Harness-facing topology for MCP tools — accepted in D2.** Reuse the Connector path and register each external integration separately from the unchanged built-in declaration. Affects REQ-2, REQ-4, and REQ-5.
3. **MCP credentials and outbound transport — accepted in D3.** Affects REQ-1 through REQ-3 and REQ-5.
4. **Discovery and applied configuration — accepted in D4.** Affects REQ-3 through REQ-5.
5. **Deletion and admission ordering — accepted in D5.** Affects REQ-4 and REQ-5.

Schema field names, local helpers, and fixture construction are implementation details. Validation must nevertheless prove the chosen public and internal boundaries on the released harnesses and applicable deployment shapes.

## Evidence: harness-facing topology for MCP tools

The current [native tool provision](../spec/domain/native-harness.md#actual-start-server-tools) passes a launch-local HTTP declaration through Codex App Server and strict ACP v2 New/Resume. The [built-in tool contract](../spec/domain/server-tools.md) expects the `send`-only catalogue and uses required readiness and publication checks. The existing typed component RPC is scoped to a current component route and Agent/conversation. Its current semantics are not a generic external tool API.

- **Connector-local MCP projection:** reuse the supported local HTTP declaration and existing authenticated component connectivity. Keep optional external declarations logically distinct from the built-in `chapchoo` declaration. The Connector projects metadata and forwards calls; the Chapchoo server still owns upstream destinations, credentials, discovery, dispatch, and external effects. This avoids a new Internet-reachable inbound MCP route or separately deployed relay and reuses the reachable network path across Standalone and Hosted. It requires extending launch provision and a narrowly scoped internal contract for external calls. Listener sharing and catalogue ownership are not decided by this proposal.
- **Chapchoo-server MCP route addressed directly by the harness:** the native HTTP interface could address a Chapchoo-issued MCP URL instead of a Connector-local endpoint. This removes the Connector from individual tool payload forwarding but adds a new harness-to-backend HTTP reachability and authentication surface. It must work for Computers outside the server network as well as Standalone and keep capabilities execution-bound. No matching existing inbound product MCP route was found.

Pointing a harness directly at the upstream server does not implement REQ-2. Folding external tools into the current required `chapchoo` declaration as-is would change the built-in catalogue and couple failure and approval; it is not a drop-in reuse option.

The current Codex approval event adapter recognizes the `chapchoo` namespace; simply adding a declaration is not proof that a normal external-tool approval reaches Chat. Test the released native approval path without applying a new approval policy. ACP advertises HTTP capability and accepts the full client-supplied declaration list on Resume; failure isolation, usable structured results, and exact resumption still require verification against actual released builds. No live upstream or released-harness end-to-end proof is claimed here.

## Evidence for D2 refinement: per-integration registrations or one external registration

Both options retain the built-in `chapchoo` declaration at `/mcp`, reuse the Connector path, and keep outbound connections and credentials on the Chapchoo server. They change how the harness sees tools, not how users manage integrations. Both must authorize calls against the current Agent, execution route, and target integration; neither can route on an unqualified tool name or use a caller-supplied upstream URL. Neither requires a process or port per integration.

- **One optional registration per external integration.** Use a stable, integration-derived harness name and route on the shared loopback listener, independent of the display name. The native client gets separate tool lists and server identities. Two integrations can expose the same unqualified tool name without a Chapchoo alias table. The registration list and local client setup/discovery work grow with the number of integrations, and membership changes must be applied to the next execution. Failure isolation is not automatic if setup waits on all external dependencies.
- **One optional registration for all external integrations.** Native startup and resume see at most one external registration, regardless of the upstream count. Chapchoo still needs per-integration discovery, expiration, failure handling and dispatch authority. A combined `tools/list` needs stable collision-safe tool names and a mapping to the original integration and tool; it must omit failed sources without failing the whole catalogue. All external tools would share one native server identity and the aggregate endpoint/catalogue would be a shared failure point. A stable declaration does not by itself refresh the tools the harness has loaded.

[Codex `0.171.0` configuration](https://github.com/openai/codex/blob/rust-v0.171.0/codex-rs/config/src/mcp_types.rs#L397-L453) supports optional servers; [its source tests](https://github.com/openai/codex/blob/rust-v0.171.0/codex-rs/codex-mcp/src/connection_manager_tests.rs#L3330-L3406) exercise bounded waiting for optional startup, not this Chapchoo path. [ACP v2](https://github.com/agentclientprotocol/agent-client-protocol/blob/f3ed0fca2fd6d85ef7ae77cfd074cc8605e490dc/docs/protocol/v2/session-setup.mdx#L535-L543) supports multiple uniquely named declarations and a complete current list on Resume. It does not establish optional-start behavior in every released harness. [MCP tool naming](https://modelcontextprotocol.io/specification/2026-07-28/server/tools#tool-names) explicitly anticipates proxy aggregation and recommends disambiguating collisions. Neither topology is excluded by these protocol contracts.

The distinguishing verification is to start and resume with two healthy integrations exposing the same tool name and another unavailable or slow integration, then change the saved set. Chat, built-in `send`, and each healthy connection must remain usable; removed tools must no longer be callable. Compare startup and applied configuration behavior at the supported scale before adopting a count limit or revisiting aggregation. These cases have not yet been exercised end to end.

## Evidence: credentials and outbound destination enforcement

Both options can keep a typed MCP credential encrypted with the Agent's integration record, using the existing installation key and an authenticated context dedicated to MCP and bound to the Agent and integration identity. The existing external-secret implementation protects User/Team AI credentials; it is a primitive and pattern to reuse, not an ownership model for this connection. Metadata reads are redacted. The outbound server path uses the token for the selected MCP endpoint; the Connector and harness receive no upstream credential.

- **Enforce at the Chapchoo server's outbound HTTP client.** Connection tests, discovery and tool calls use the same transport policy. The client allows only public HTTPS destinations, verifies the actual dial destination, and does not automatically follow redirects or inherit an unvalidated proxy route. A URL string check at save time is not sufficient. This introduces no mandatory deployment component and applies in Standalone as well as Hosted. The policy must be exercised against the actual HTTP transport, including reconnects and both supported HTTP response forms.
- **Require a dedicated egress proxy or gateway.** It can enforce the outbound destination policy independently of the application and centralize policy across server instances. It needs a trusted configuration and a deployment guarantee that the application cannot bypass it. The current peer NetworkPolicy restricts ingress; it is not a general outbound enforcement path. Standalone and Hosted would both need equivalent proxy provisioning and failure handling.

Recommend enforcing the boundary at the outbound client for the first release, without making a proxy a prerequisite. The pinned `rmcp` client exposes a public strict discovery mode and a configured HTTP-client hook; the workspace HTTP client exposes the relevant transport controls. Reuse these interfaces and verify the exact behaviour rather than implementing the MCP wire protocol independently. Do not fall back to older revisions or retry a tool request after possible dispatch. This is source-backed feasibility, not runtime proof. Deployment restrictions may add defense in depth; they do not substitute for the application contract.

## Evidence: external catalogue selection and application

The existing [configuration refresh](../spec/domain/agent-instructions.md#common-refresh-and-continuing-input) reuses an unchanged native configuration or applies a changed selection using the same context. The [built-in `send` catalogue](../spec/domain/server-tools.md#native-provision-and-verification) is part of required readiness. External network responses must not become a condition of that readiness. The [upstream TTL](https://modelcontextprotocol.io/specification/2026-07-28/server/utilities/caching#time-to-live-ttl-field) is a freshness hint checked on use, not a requirement to poll continuously.

- **Select external catalogues on the server before the next execution.** Use an unexpired complete catalogue for the exact current connection settings; refresh missing or expired catalogues with a bounded parallel budget, independent of required preparation. Seal a bounded result per integration: a usable catalogue (including a confirmed empty list), or an availability reason. The Connector answers the harness's local discovery from that prepared result without waiting on upstream discovery. An unavailable integration contributes no tools to that execution, does not block unrelated work, and can recover on a subsequent execution. Saved changes are considered on the next execution even for a retained harness; apply declaration or tool changes through supported refresh or exact-context resume. An unchanged effective catalogue does not cause a resume solely because its freshness was renewed.
- **Discover on demand when the harness lists a local MCP registration.** The Connector still delegates misses to the Chapchoo server; credentials and TTL policy stay on the server. This can avoid lookups the harness never needs and preserves per-integration registrations, but local discovery latency depends on upstream availability. Next-execution freshness needs an explicit supported re-list or re-provision boundary even when the harness reuses its local catalogue. A first `tools/list` at native startup is not such a guarantee for retained sessions. Each supported harness needs a verified bounded wait and failure path.

Recommend server-side catalogue selection. It makes the refresh attempt independent of each harness's discovery timing and caps the shared delay without making upstream health a readiness check. Delayed results from superseded settings must not become the selected catalogue; credentials and destinations may not be retargeted by a catalogue entry. The optional connection test does not have to be run before a usable catalogue is selected. Multi-harness start/resume, expiry including TTL zero, stale in-flight discovery after an edit, and independent failure are verification obligations. The cache is expendable and must not authorize dispatch. Physical deletion versus egress ordering remains a separate decision.

## Evidence: deletion versus invocation

The [built-in invocation owner](../spec/domain/server-tools.md#bounded-invocation-owner) distinguishes waiting, started, and final observations. Transport completion and configuration deletion are different facts. Waiting for an outbound send fence or a bounded transport drain was considered to enforce the former pre-send deletion rule. Both were superseded when the requester made saved deletion independent of ongoing calls.

## Accepted decisions

### D1. Share Agent integration management; keep capabilities type-specific

Accepted on 2026-11-26. Resolves [mcp-261126/REQ-1](../requirements/mcp-261126-agent-external-connections.md#req-1-manage-connections-on-the-agent) and [mcp-261126/REQ-6](../requirements/mcp-261126-agent-external-connections.md#req-6-start-with-mcp-without-limiting-agent-integrations-to-mcp).

Use a common management boundary for saved integration identity, Agent association, management authority, and add/edit/delete. Each integration type owns and validates its configuration and authentication, and defines how it runs and what capabilities it provides. MCP URL, protocol revision, tool discovery and invocation are specific to the first type. The shared boundary does not require future types to speak MCP, offer tools, or use the same credential schema.

Compared with composing independent management paths later, this reuses the known Agent ownership and lifecycle rules while leaving unknown provider behavior out of the shared contract. Ship only the first MCP type and its agreed settings flow, without a type picker or a general plugin framework. Physical persistence, outbound transport, and harness delivery are separate decisions.

### D2. Reuse the Connector path with separate harness declarations

Accepted on 2026-11-26 at the topology level. Addresses [mcp-261126/REQ-2](../requirements/mcp-261126-agent-external-connections.md#req-2-connect-from-the-chapchoo-server-using-the-declared-profile), [mcp-261126/REQ-4](../requirements/mcp-261126-agent-external-connections.md#req-4-discover-and-use-the-tools-in-the-same-agents-chat), and [mcp-261126/REQ-5](../requirements/mcp-261126-agent-external-connections.md#req-5-keep-other-work-available-and-preserve-truthful-outcomes).

Present external MCP connections through Connector-local HTTP and existing authenticated component connectivity rather than adding a harness-facing inbound HTTP route at the Chapchoo server. Keep optional external registrations distinct from the required, unchanged built-in `chapchoo` registration. The Connector forwards requests; the Chapchoo server owns outbound MCP and its authentication. This choice concerns MCP tool delivery, not the management or runtime contract of every Agent integration. It reuses existing reachability for both local and remotely hosted Computers. The routing refinement is not part of this acceptance.

**Accepted refinement (2026-11-26):** Project each saved external MCP integration as its own optional harness registration on the shared loopback listener. Use the integration identity, not the editable display name, to distinguish registrations and route calls. The local route selects the integration but does not confer authority; the actual execution and current integration must still authorize the request. Apply the current saved set rather than appending declarations across executions. This uses the native server boundary and avoids a combined catalogue and external tool-name mapping. Exact wire names and paths are implementation details. Multi-registration start/resume, native approval, name collisions, and isolation from a stalled external server remain verification obligations, not proven compatibility.

### D3. Keep MCP credentials on the server and enforce public HTTPS at the outbound client

Accepted on 2026-11-26. Addresses [mcp-261126/REQ-1](../requirements/mcp-261126-agent-external-connections.md#req-1-manage-connections-on-the-agent), [mcp-261126/REQ-2](../requirements/mcp-261126-agent-external-connections.md#req-2-connect-from-the-chapchoo-server-using-the-declared-profile), and [mcp-261126/REQ-3](../requirements/mcp-261126-agent-external-connections.md#req-3-test-discovery-without-executing-a-tool).

Encrypt saved Bearer material on the server with the installation credential protection, bound to the owning Agent and integration under an MCP-specific purpose. Never return it as read metadata or deliver it to the Connector or harness. Use one server-side outbound policy for testing, discovery, and invocation. Check the actual dial destination for public HTTPS reachability, preserve certificate verification, and do not follow redirects or inherit proxy routing that could bypass that check. Use the supported `rmcp` and HTTP client interfaces with the required external revision and no legacy fallback. Hosted and Standalone do not require a dedicated egress proxy. Runtime enforcement and retry behavior must be verified at the transport boundary.

### D4. Select and prepare external catalogues on the server for the next execution

Accepted on 2026-11-26. Addresses [mcp-261126/REQ-3](../requirements/mcp-261126-agent-external-connections.md#req-3-test-discovery-without-executing-a-tool), REQ-4, and REQ-5.

The server selects the saved integration set for the next execution and prepares a result per integration. Reuse a complete, unexpired catalogue only for the same current connection settings; refresh missing or expired catalogues within a bounded shared budget with parallel discovery. Do not apply a late result for superseded settings. The Connector answers local discovery from the selected result without waiting for upstream I/O. An unavailable integration is omitted from the tool set for that execution while its failure cause is preserved independently of a successful empty list. Built-in readiness and unrelated tools do not depend on external discovery success.

Apply changed declarations or tool lists through supported harness refresh or exact-context resume, without interrupting current work. A renewal that leaves the effective selection unchanged does not by itself require a resume. A catalogue is discovery evidence, not dispatch authority. Next-execution adoption and failure isolation remain release verification obligations.

### D5. Report deletion at commit; end new admission without draining calls

Accepted on 2026-11-26. Addresses [mcp-261126/REQ-5](../requirements/mcp-261126-agent-external-connections.md#req-5-keep-other-work-available-and-preserve-truthful-outcomes), as revised.

Serialize deletion and fresh outbound admission against the authoritative Agent integration record. Once deletion commits, show the integration as deleted. Reject subsequent admissions even if a retained harness still advertises the tool. Let an already admitted call proceed under the exact settings that authorized it, even if its HTTP exchange starts or finishes after deletion commits. Do not inspect, wait for, or cancel calls as part of deletion. There is no transport fence, deletion drain, or pending state tied to tool activity. Bounded call ownership and existing failure classification still apply independently: a blocked admission is known nonexecution; possible dispatch without a definitive response is uncertain and must not be replayed.
