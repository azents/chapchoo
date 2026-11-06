---
title: "Codex Authentication Implementation Plan"
created: 2026-11-06
tags: [harness, implementation, security]
---

# Codex Authentication Implementation Plan

- Requirements: [harness-261106/REQ](../requirements/harness-261106-codex-authentication.md).
- Decisions: [harness-261106/ADR-D1 and D2](../adr/harness-261106-codex-authentication.md).
- Approved Design: [harness-261106/DESIGN](../design/harness-261106-codex-authentication.md), revision1 exact M1-M9, approved2026-11-06 07:41KST with implementation authorization.
- Design delta: None.
- Implementer/integration/validation owner: primary agent `/root`.
- Single independent read-only reviewer for every stable integrated phase: `/root/harness-feature-reviewer`.
- Base: main294f9ff1bfc5baf5017bd38f4c9bdae0df4c6161. Research base89bf569 has no Rust/proto changes before this base.

## Delivery Phases

1. `Codex authentication [1/3]: protected credential schema and primitives` — branch `feat/codex-auth-backend` -> main. M1/M2/M4/M6/M9 foundation: purpose-bound variable-size secrets, keyed request identity, preserving SQLite/PostgreSQL schema and typed IDs. No reachable feature API until phase2.
2. `Codex authentication [2/3]: backend acquisition and saved Agent selection` — stacked on phase1. Complete Store/Service/RPC operations, provider HTTP, concurrency/recovery, creation receipt extension, lifecycle integration and generated Node clients. M1-M7/M9. No Computer credential delivery, model call, runtime refresh or web activation.
3. `Codex authentication [3/3]: authentication management and Agent flows` — stacked on phase2. M8 integration, minimal fixture-backed E2E, component checks, final same-SHA review, Spec promotion, Requirements/Design implemented date only after verified completion, temporary plan cleanup.

No extra docs-only or cleanup PR is created to satisfy a ritual. Open each completed phase PR before later implementation. Never merge or deploy without explicit approval.

## Integration and Prerequisites

- Root reads all relevant path instructions and semantic conventions; no implementation delegation.
- Phase1 owns crypto/schema and prepares APIs phase2 consumes. Phase2 owns proto, persistence acceptance and composition; phase3 owns General UI and testenv/browser fixture wiring.
- Require pinned Rust1.99.1, existing disposable PostgreSQL test lifecycle, SQLite, existing Node/pnpm client generator. No real provider credentials required for deterministic checks. Live OAuth remains optional, human-authorized and never invokes paid inference.
- Root runs `scripts/check-rust.sh`, `scripts/check-rpc.sh` where applicable, document checks, frontend quality/build/component checks and fixture-backed E2E. CI is not complete until the exact new head passes.
- Review stable whole integrated phase diff once through the named reviewer; root makes corrections and runs affected checks. New authority/scope choices return to design, local reversible details remain root-owned.

## Removal and Preservation

Retain existing session envelope/root binding, all existing schema data and accepted creation receipts. Replace only new-create contract/creation composition and Agent projections at their activating phase. Remove intermediate account confirmation and proposed Computer application states from production target; frozen historical HTML remains unchanged. No Control/Connector/Provider protocol, TLS, native credential file or runtime-refresh feature is introduced.

## Checkpoints and Spec Promotion

Record phase SHA/branch/base, tests, unchanged prerequisites, reviewer findings/resolution, implemented vs remaining mechanisms and actual migration guarantees. Phase1 updates credential/schema owning Specs immediately for shipped primitive behavior; phase2 updates backend owning pages; final `/spec-review` audits cross-page semantic impact. Do not label proposed UI implemented in an earlier phase.

Phase1 checkpoint: scoped crypto and empty schema implemented; canonical Rust gate passed268 cases across workspace groups, actual private PostgreSQL and SQLite, Clippy/rustdoc/tooling/dependency checks. Documentation validator and18 validator tests passed. Single reviewer found no material defect in the stable18-file staged diff. No reachable auth API/UI yet; compact DTO kind issuance remains phase2 with its first consumer. No approved removal or Computer transport change omitted/added. Exact commit/PR evidence is recorded at delivery.

Feature risks: provider protocol/applicability, one-time exchange uncertainty, Team cleanup lock order and old creation-receipt recovery. No timer alone grants replay of an uncertain exchange. No secret in receipts/logs/public projections. Keep disk usage bounded and remove finished owned worktrees only when remaining work/results are safely preserved.
