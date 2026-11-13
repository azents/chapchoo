---
title: "Direct Conversation Sender Decisions"
created: 2026-11-13
document_role: primary
document_type: adr
snapshot_id: sender-261113
tags: [tools, mcp, harness, architecture]
---

# Direct Conversation Sender Decisions

This snapshot records the narrowly approved correction in
[sender-261113/REQ](../requirements/sender-261113-direct-conversation-tool.md).
It does not settle the broader operation-outcome design.

## ADR-D1. Restore the production name and retain historical evidence

- Authority: sender-261113/REQ-1, REQ-3; unchanged private conversation storage and
  readable captured-history contracts.
- Decision: expose and execute only `send_message`. The read-only captured-record
  classifier recognizes the original stored identity as well as the corrected
  identity. It does not mutate a stored record or make an old identity callable.
- Rationale: changing an execution contract does not invalidate evidence already
  written under the previous contract. Removing its display interpretation would
  regress history without improving invocation safety.
- Alternatives: an executable alias contradicts REQ-1; rewriting or dropping old
  records violates preservation. Treating old publications as generic shell tools
  loses their already-supported meaning.

## ADR-D2. Use the native direct-only namespace policy

- Authority: sender-261113/REQ-2 and the direct-only policy shown to the requester
  before the explicit implementation instruction on 2026-11-13 KST.
- Decision: configure only `mcp__chapchoo` as direct-only and disable strict
  third-party deferral for this native profile. Retain ACP HTTP MCP provisioning.
- Feasibility: inspected Codex 0.165.0 tool planning converts that namespace to
  DirectModelOnly; its strict-third-party policy would otherwise override direct
  exposure. ACP 2.6.0 forwards its native configuration on New and exact Resume.
- Alternatives: changing descriptions still depends on search. Making every MCP
  tool direct changes unrelated integrations. Replacing MCP with dynamic tools
  changes transport/ownership unnecessarily. A statically configured MCP endpoint
  would risk adapter name-conflict filtering and stale scoped credentials.
- Local wiring: the actual adapter launch supplies the non-secret native policy
  through its supported configuration input. The dynamic endpoint and bearer stay
  solely in the existing ACP session declaration. This introduces no second
  credential or recipient authority.

## Decision provenance

The requester owns the name and direct-visibility decisions above. This record
captures the already given, specific implementation instruction; it does not
assert a later separate approval conversation or authorize new state-machine,
rollback, migration or operational mechanisms.
