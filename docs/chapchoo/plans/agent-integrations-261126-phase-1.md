---
title: "Agent Integrations Phase 1 Execution Plan"
created: 2026-11-26
tags: [agent, mcp, backend]
---

## Phase Execution Plan

- Phase: 1 — Server foundation
- Branch/base: `feat/agent-integrations-foundation` → `main`
- PR boundary: protected storage, management and outbound discovery building blocks
- Inputs: confirmed [REQ](../requirements/mcp-261126-agent-external-connections.md), [ADR D1–D5](../adr/mcp-261126-agent-external-connections.md) and [Design v1](../design/mcp-261126-agent-external-connections.md)
- Deliverables: typed settings, owner-validated saves and reads, commit-time deletes independent of calls, and public HTTPS discovery with no tool invocation
- Non-goals: no UI, external native declarations, component invocation, OAuth, activation mode, new deployment component, or changes to built-in `send`
- Interfaces: ownership-aware repository operations and server-owned policy-controlled discovery; no stored credential in management projections
- Approved mechanisms: M1 and server work from M2, M4, M7 and M8
- Design delta: `None`
- Removal obligations and absence verification: none for this additive phase; existing `send` and native contracts stay unchanged

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Managed settings | Primary agent | `rust/crates/{crypto,domain,persistence}` | Existing private Agent, migration and key rules | Persisted identity, protected typed settings, transactional writes, redacted metadata | SQLite and PostgreSQL owner/replay/delete contract tests |
| Outbound discovery | Primary agent | `rust/apps/chapchoo/`, `rust/crates/service/` if required by composition | Typed settings, existing reqwest and rmcp interfaces | Strict profile and common policy; paginated discovery without invoking tools | Tests through the owned transport and explicit refusal paths |

- Integration order: schema and storage → discovery → integrated checks
- Independent review: `/root/agent-integrations-reviewer`, read-only, after the integrated diff stabilizes
- Final validation: focused Rust tests and formatting/lint, documented database prerequisites, docs validator; root owns results
- Scope-drift check: no user-facing surface or execution/approval policy in this phase
- Context checkpoint: Design v1 approved, baseline `875f69c`. Phase 1 delivered typed owner-scoped settings, mutation receipts, protected credentials, and a list-only test operation. Verified the full crypto and persistence suites against SQLite and disposable PostgreSQL, 66 service tests, affected Clippy, workspace formatting, application all-targets compilation, and documentation checks. Independent review and targeted re-review closed five findings at code tree `ff54d0c`. No Design delta or removals; existing Client RPC, Connector, and built-in declarations remain unchanged. Phase 2 starts on `feat/agent-integrations-runtime` and owns the product RPC and runtime connection, including delete/admit ordering and released-harness interoperability.
