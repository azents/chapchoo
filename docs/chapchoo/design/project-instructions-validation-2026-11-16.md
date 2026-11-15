---
title: "Agent Project Instructions Validation"
created: 2026-11-16
document_role: supporting
document_type: supporting-validation
tags: [agent, conversation, verification]
---

# Agent Project Instructions Validation

Snapshot: [prompt-261115](../requirements/prompt-261115-system-prompt-management.md).
Approved Design: revision 4, exact mechanisms M1–M8. This report describes
verification boundaries, not additional product authority.

## Implemented boundaries

Owner setting writes are direct overwrite/empty clear under private owner and
current membership authority. Immutable selected sources and the actual tool
catalogue are sealed before component preparation. An explicit current operation
identity is reserved at admission; late results cannot select an older bundle.
The final forwarding transaction checks the current applied target/configuration
and does not grant delivery through pending preparation or earlier uncertainty.

The existing persistent workspace is the default project. Atomic managed
AGENTS.md publication first preserves an unmanaged original once and leaves
ordinary project files intact. Common prompt/catalogue refresh retains existing
work and queued input, then retires the physical owner and restores the exact
native context only after supported lifecycle and external-effect completion.
Unknown state fails preparation rather than interrupting work to force progress.

## Executed verification

- Full Rust quality entrypoint: formatting, Clippy with warnings denied, tooling,
  private-item rustdoc, complete workspace tests on isolated SQLite/PostgreSQL,
  and dependency boundaries.
- Generated Node/Rust public/component interoperability: 129 checks on SQLite and
  129 on PostgreSQL; no generated client was hand-edited.
- Frontend formatting, lint, typecheck and production builds; shared/app unit
  suites; 178 General browser component checks, followed by a targeted rebuilt
  transport test for the corrected history-selection labels.
- Core browser E2E: all four account/Team/Computer journeys passed locally.
- Released-program fixed-response acceptance: stock ACP 2.6.0 and Codex 0.165.0,
  fresh isolated state, exact title-persisted restoration before first model
  input, A→B→empty AGENTS.md across exact-context physical replacement, three
  loopback Responses requests and zero native action callbacks. The assertions
  verify the new instruction item's role/order, not marker presence in old history.

The released-program check uses fixed assistant SSE, not live inference. It does
not certify model obedience, operator credentials or arbitrary native engine
internals. Source comparison verified that the required ACPv2 tool/terminal and
standard-child translation is already present in adapter 2.5.0; this is not an
additional 2.5.0 runtime execution claim.

## Review corrections and failure evidence

One independent complete integrated review and targeted re-reviews identified
and closed these boundaries:

1. Strict-v2 first tool-call upserts and independent terminal updates replace the
   incorrect v1-shaped tracking fixture. A completed tool cannot release a live
   terminal; null exits remain unresolved. SDK-driver and native-owner gate tests
   use the actual v2 wire shape.
2. Evidence options include a visible unique operation identity, including when
   time and Computer execution are equal. Browser transport tests retain exact
   selection across paging failure/retry.
3. Deferred-expiry capacity errors are per-request results, not actor-task
   failures that can drop an existing child. The 256-entry uncertainty test
   verifies rejection without eviction and explicit known-result release.

Initial assembled Docker verification failed while a queued input waited for
refresh; its test used the default five-second poll despite asynchronous native
preparation. The observation bound was made explicit and preparation failures
now fail immediately. The controlled native/stock-adapter answer sequence was
also checked separately through actual Running→Idle/end_turn and tool completion.
Final assembled verification and exact-head CI results are recorded below.

## Final delivery evidence

[CI run 43752080847](https://github.com/azents/chapchoo/actions/runs/43752080847)
completed successfully on implementation commit
`1e6ae75124573b2261b08a8e7a5cf80df9237ef6`. This includes both architecture builds,
repository/Rust/frontend gates, Docker integration and owned-cluster Provider
checks, packaged browser acceptance and the released-program instruction check.
The assembled model-free journey completed Save during existing work, queued
input, typed-answer completion, same-target refresh/once-only continuation and
historical original selection after a later Save. Its normal cleanup succeeded.

The complete integrated independent review and scoped corrections closed all
known findings. The final documentation-promotion commit is checked by the same
CI workflow; its exact head and status remain available on
[PR #131](https://github.com/azents/chapchoo/pull/131). Merge and deployment are
separate approvals and were not performed.

## Scope and removals

Configuration-blind changed Setup reuse and catalogue-mismatch-only handling are
replaced at the owned refresh boundary. Composer submission is separate from
native readiness. Consumed input rows no longer own instruction originals.
The root captured journal remains single-session; child work is live evidence,
not fabricated root history. No settings-version/conflict/rollback interface,
arbitrary MCP management, additional harness or live-model certification is added.
