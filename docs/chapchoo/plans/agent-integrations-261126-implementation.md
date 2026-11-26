---
title: "Agent Integrations Implementation Plan"
created: 2026-11-26
tags: [agent, mcp]
---

# Agent Integrations Implementation Plan

Authority: [Requirements](../requirements/mcp-261126-agent-external-connections.md), [ADR](../adr/mcp-261126-agent-external-connections.md), and [approved Design v1](../design/mcp-261126-agent-external-connections.md). Approved mechanisms: M1–M8. Design delta: `None`.

The primary agent owns implementation, integration, and verification. The single independent read-only reviewer is `/root/agent-integrations-reviewer`.

1. **Agent Integrations [1/3]: Server foundation** — common Agent-owned identity and typed MCP settings, credential protection, application management operations and policy-controlled SDK discovery. Verify owner isolation, replay outcomes, edit/delete and credential handling in SQLite and PostgreSQL; verify HTTP policy separately. No web or harness feature is exposed before the complete path is ready. Implement M1, the server portions of M2/M4/M7/M8.
2. **Agent Integrations [2/3]: Execution integration** — build on the foundation branch. Wire Client contracts and generated bindings, component admission, per-execution discovery, Connector registrations and released Codex/ACP integration. Validate M3/M5/M6/M7/M8, including the existing built-in path and the admission/commit race. This phase owns replacement and absence evidence for the fixed single-declaration representation.
3. **Agent Integrations [3/3]: Settings and acceptance** — complete M2, wire saved state and edit/test/delete UI, run the representative browser journey and integration matrix, promote verified behavior to living specs, and remove phase plans. Use disposable fixtures and unmodified released harnesses; no production credentials or live operator action are required.

After each phase, record exact branch and validation, review, affected interfaces, remaining scope, and open the PR before starting the next. Merge or deployment is separate. Do not advertise external-tool readiness until the entire path works. Document any conditional or unverified prerequisite rather than adding a transport fallback.
