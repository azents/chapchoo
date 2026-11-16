---
title: "Latest System Prompt Page Design"
document_role: primary
document_type: design
snapshot_id: prompt-261116
created: 2026-11-16
implemented: 2026-11-16
tags: [agent, instructions, frontend, persistence]
---

# Latest System Prompt Page Design

Authority: [Requirements](../requirements/prompt-261116-latest-system-prompt.md)
and [fixed outcome map](../adr/prompt-261116-latest-system-prompt.md).

## Current behavior and replacement

The saved owner setting is already overwrite-only. Operation-keyed preparation
payloads currently accumulate independently, with owner history and exact-record
APIs feeding an inline expandable browser. Replace that complete unit with one
current payload and one separately addressable read-only page.

## Design Authority

- Design revision: `1`

| ID | Material mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Latest-only payload per Agent; transactionally remove the superseded payload at new preparation admission and enforce Agent uniqueness | REQ-3, REQ-4; existing causal preparation identity | derived |
| M2 | Forward migration keeps only the current Agent conversation's selected payload; deletes older payloads and history index | REQ-3 | required |
| M3 | Owner latest-prompt API with optional current payload; remove historical APIs and cursors | REQ-3, REQ-4 | derived |
| M4 | Server-rendered separate System prompt route and real activity navigation; localized terminology and read-only content | REQ-1, REQ-2; existing primary-rendering conventions | derived |
| M5 | Preserve saved settings, route/execution-qualified component reads, native continuity and final forwarding gates | REQ-4; current Agent instructions Spec | existing |

## Persistence and lifecycle

New startup, input-preparation and configuration-refresh admission remove the
Agent's old payload within the same transaction that selects the new operational
identity. A sealed current selection has at most one payload; pending unsealed
selection has none. First seal checks the current operation; reseal can return
only the current exact content. Delayed completion updates only its operation
and cannot restore deleted content. An Agent-unique index prevents accumulation.

The migration uses `agent_conversations` and the exact
`current_preparation_operation_id`, not creation timestamps, to select the sole
surviving payload. Settings, conversations, input receipts, applied native targets
and execution ownership remain untouched. Downgrade can restore schema shape but
cannot recover intentionally deleted historical content.

## Interfaces and security

Remove owner instruction-history and arbitrary historical-operation reads.
Expose an optional latest system prompt under current private Agent authority,
joining only its current conversation and selected preparation. Component reads
retain exact operation/execution/Connector authority. Operational tool catalogue
and credentials are not exposed through the web view.

## Page and failures

The existing Team/Agent route resolver admits the new read-only route. Primary
content is fetched and rendered on the server; request failures remain route
failures, not empty success. The activity entry pushes meaningful page navigation.
The page links back to the same Agent's chat, displays the latest content and
preparation/application status, and has no history selection or collapse.
Existing settings editing is unchanged.

## Removal and Replacement

| Existing unit | Removal authority | Replacement | Boundary | Verification |
| --- | --- | --- | --- | --- |
| Per-operation historical payloads/index | REQ-3 | Agent-unique current payload | Forward migration and admission transactions | Both-engine migration and repeated-selection tests |
| Owner history and arbitrary operation API | REQ-3 | Optional latest-prompt read | Proto, service, RPC, generated client and web adapters | Generation and RPC tests |
| Collapse, history selector, paging hooks/fixtures/tests | REQ-1–3 | Server-rendered page and navigation | Web feature and affected E2E | Component/browser tests and symbol search |
| Historical web-core projection | REQ-3 | Current-payload decoder | Shared web core | Decoder tests |
| Stale current-system wiki claims | REQ-1–4 | Latest-only explanation | Owning wiki and connected pages | Semantic lint and document validator |

## Test Strategy

Primary browser verification covers activity navigation, addressable server-rendered
content, return navigation and localized System prompt copy. Reuse the existing
isolated owner/Agent/Connector fixtures; no production credentials are required.
Migration and Store tests on SQLite/PostgreSQL verify preserving current content,
deleting older content, uniqueness, exact-current authority, late operations and
repeated replacement. Decoder and RPC tests cover absent/current content and
authorization. Existing refresh/once-only native tests retain their authority.
CI runs repository Rust, RPC and frontend gates on the exact PR head; preserve
logs and distinguish synthetic native fixtures from live-model execution.
No live-model tests or operational deployment are required.

## Feasibility and risk

Research verified the three admission paths in `startup.rs`, `preparation.rs`
and `project_refresh.rs`; all subsequent forwarding uses the operational current
pointer. Historical reads are not a runtime recovery dependency. Existing private
route loading and the server-rendered application shell support the new page.
All mechanisms are feasible under the existing transaction and route boundaries.
Intentional history deletion is irreversible; no rollback can recreate it.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Approved on: `2026-11-16`
- Approved Design revision: `1`
- Approved authority IDs: `M1, M2, M3, M4, M5`
- Approved scope: requester-directed latest-only retention, history removal and
  separate system-prompt page; immediate implementation explicitly reiterated.
  All mechanisms above are fixed outcomes or derivations of that scope and the
  unchanged execution/authorization contracts, not additional product choices.

## Implementation verification

Verified locally on 2026-11-16 through the normal commit hooks: Rust
format/Clippy/rustdoc/full tests (both database engines), generated-client
verification and live Node/Rust RPC interoperability, document validation, and
TypeScript format/lint/typecheck. Also passed 168 shared web tests, 38 General
web tests, 20 Admin tests, both production web builds, the web Storybook build
and all 179 web browser component tests. Independent integrated review and
targeted re-review found no issues. The assembled Docker browser journey is
updated and remains part of PR CI; no live-model compliance claim is made.
