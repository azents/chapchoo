---
title: "Direct Conversation Sender Design"
created: 2026-11-13
document_role: primary
document_type: design
snapshot_id: sender-261113
tags: [tools, mcp, harness, frontend]
---

# Direct Conversation Sender Design

Scope is the focused [sender-261113/REQ](../requirements/sender-261113-direct-conversation-tool.md)
correction, not the broader outcome-reliability snapshot. Decisions are recorded
in [sender-261113/ADR](../adr/sender-261113-direct-conversation-tool.md).

## Current Behavior and Correction

The previous catalog exposes `publish_text` although the approved public name is
`send_message`. The model searches a native catalog whose filtering is not a
reliable discovery mechanism for its essential reply capability. Historical
captured invocations retain their original MCP server/tool identity.

The correction aligns the server catalog and dispatch validator with
`send_message`, updates typed fixtures and new captured-record examples, and
configures the actual native adapter to expose `mcp__chapchoo` directly. The
read-only publication classifier recognizes both historical and current stored
identities; the live server recognizes only the current one.

## Ownership and Interfaces

- Existing Rust service owns fixed-recipient authorization, argument validation
  and atomic chat publication. No sender arguments, result shape or data owner
  change except the corrected tool name.
- The native launch owner passes a non-secret `CODEX_CONFIG` object containing
  `code_mode.direct_only_tool_namespaces = ["mcp__chapchoo"]` and
  `features.code_mode_only_strict_3p_tools = false`.
- ACP 2.6.0 merges this configuration into both New and exact Resume requests.
  It separately merges the runtime HTTP MCP declaration. No static endpoint or
  bearer is added to native configuration; no name-conflict bypass is introduced.
- Codex 0.165.0's native exposure policy produces DirectModelOnly for this
  namespace. Other namespaces retain their existing exposure policy.
- Web publication presentation consumes immutable captured server/tool metadata.
  Historical recognition grants no execution or receipt-replay permission.

## Design Authority

- Design revision: `1`

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Correct production catalog and invocation identity to `send_message` | REQ-1; ADR-D1 | required |
| M2 | Direct-only Chapchoo namespace through the existing native configuration path on New and Resume | REQ-2; ADR-D2 | decided |
| M3 | Preserve fixed-recipient MCP, authority checks, effect lifetime and native context | REQ-2; current server-tools Spec | existing |
| M4 | Read both stored publication identities without execution alias or record rewriting | REQ-3; ADR-D1; readable-history Spec | existing |

All references above are snapshot-local unless named otherwise. M4 retains an
existing historical capability instead of adding a legacy execution mode.

## Failure, Rollout and Compatibility

There is no database migration. Existing captured rows remain byte-identical.
The prior executable name is removed from the live catalog/validator together.
Existing owners retain the policy/configuration sealed at their actual start;
this change does not restart or reconfigure a running native process. Normal
future New/Resume receives the corrected policy. Missing MCP HTTP capability,
incorrect catalog, unavailable authority and native restoration failures keep
existing rejection semantics. The policy does not fabricate model-use evidence.
Merge, image publication and operational rollout require separate authorization.

## Removal and Replacement

| Existing behavior | Authority | Replacement | Boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Incorrect live `publish_text` catalog/validator name | REQ-1 | `send_message` | Same production correction | Owned argument/catalog/SDK tests; inspect production validators |
| Search-dependent essential sender exposure | REQ-2 | Native direct-only namespace | Actual New/Resume launch | Native policy unit contract and controlled fixture request validation |
| Current-only stored publication classifier | REQ-3 | Read-only interpretation of original/current identities | Record rendering | Retained historical identity unit/component fixtures |

Historical implemented Requirements/ADRs/Designs remain immutable references.

## Test Strategy

- Existing representative Docker browser conversation journey remains the primary
  assembled-product flow: accepted input and one intentional SDK publication.
  Update its expected production name; do not multiply E2E permutations.
- Existing model-free native acceptance covers actual ACP configuration and HTTP
  discovery on controlled New/Resume. The fixture rejects missing direct-only
  configuration. It does not prove real model inference.
- Unit/RPC tests cover production name, argument validation, authorization,
  nonexecution versus uncertain result and native policy. SQLite/PostgreSQL live
  Node/Rust checks cover supported backend parity.
- Captured publication unit/component checks preserve both stored identities,
  readable body/result and localized role/icon without relying on title guessing.
- Root runs Rust fmt/Clippy/docs/tests, generated-contract checks, app
  lint/typecheck/build, Storybook and component checks. The single assigned
  independent reviewer reviews the integrated diff and material corrections.
- Any local native integration requires an explicitly authorized disposable
  Engine and current image; missing inputs are reported, not silently converted
  into a live-user test or certified model-use result. CI owns its declared
  isolated Docker/browser prerequisites.

## Authority and Feasibility Validation

REQ-1 maps to M1; REQ-2 to M2/M3; REQ-3 to M4; REQ-4 to the existing owned
verification boundaries. Native-policy behavior was checked against pinned
Codex/ACP sources and the adapter fixture. No new credential, persistence,
recipient, operational or fallback authority is introduced. All mechanisms are
within the specific correction requested after the direct-only proposal.

## Design Approval

- Mode: Collaborative; requester decision owner.
- Authorization date: 2026-11-13 KST.
- Recorded revision: `1`; mechanism set: `M1, M2, M3, M4`.
- Source of authorization: explicit instruction to implement the earlier feedback
  after the approved name and this exact direct-only namespace policy were
  discussed. Existing private authority and persisted-history preservation remain
  fixed constraints, not newly delegated choices.
- This records that prior instruction transparently. No separate later approval
  exchange is asserted, and no approval of the wider outcome-reliability Design
  is claimed.
