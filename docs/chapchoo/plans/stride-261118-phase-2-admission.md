---
title: "Conversation Work phase 2: goal and follow-up authority"
created: 2026-11-18
tags: [backend, engine]
---

# Phase execution plan

- Branch/base: `feat/stride-conversation-work-admission` → PR #142.
- Inputs: [approved revision 17](../design/stride-261118-conversation-work-continuation.md)
  and [phase 1](stride-261118-phase-1-native.md).
- PR boundary: authoritative goal storage, atomic continuation decision,
  source-scoped delivery and restore, before `send` activation.
- Approved mechanisms: `M1`–`M5`, `M9`, `M10`. Authority: `REQ-1`–`REQ-5`,
  `REQ-8`, `ADR-D1`–`ADR-D5`, `ADR-D7`–`ADR-D12`, `ADR-D23`–`ADR-D26`.
- Design delta: `None`.
- Non-goals: tool cutover, work-summary UI, rollout or activation on unverified
  native versions.
- Removal obligations: none. The tool replacement belongs to phase 3.

| Workstream | Owner | Owned paths | Output | Validation |
| --- | --- | --- | --- | --- |
| Persistence | Primary agent | Rust persistence migrations, conversations and repository tests | Authoritative set, reset and single committed admission under current native source | SQLite/PostgreSQL parity, stale source, rollback, duplicate, and FIFO contention |
| Component boundary | Primary agent | Control proto, generated clients, service/RPC and Connector | Typed report and restore with exact source ownership | Authenticated route checks, replay, ACK loss and unsupported events |
| Native handoff | Primary agent | Connector and harness runtime | Independently fenced ACP input; managed restore tied to the applied owner | No user receipt or implicit replay, stale-source rejection and supported native compaction |

- Integration order: storage and tests; typed transport; fenced input and
  isolated restore; Spec alignment; integrated validation; read-only review.
- Independent review: `/root/stride-reviewer`.
- Scope checkpoint: no standalone goal tool, unverified bridge activation,
  tool replacement or extra native runner.
- Restore acceptance gate (`M5`, `ADR-D25`): the pinned native pair has
  demonstrated in-turn `SessionStart(source=compact)` injection using an
  isolated fixture. The product's Connector-local caller boundary is not
  established: the fixture's hook hash identifies configured code to Codex,
  not the caller of a Connector transport. The restricted container and
  cleared native environment alone do not distinguish the managed helper
  from native commands. Keep hook installation and restore activation out
  of the integrated implementation until a supported caller boundary is
  demonstrated. The source-qualified server read is not the local bridge.
