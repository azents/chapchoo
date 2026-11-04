---
title: "Agent Profiles Phase 1: Validated Values and Candidates"
created: 2026-11-04
tags: [agents, identity, implementation]
---

# Phase Execution Plan

- Phase: 1/4, validated values and candidate generation.
- Branch/base: design/agent-profile-handles-261104 -> main at 7ba03553.
- PR boundary: executable shared value/candidate contracts and approved baseline.
- Inputs: profile-261104/REQ, ADR-D1..D8, primary Design revision 1.
- Deliverables: backend Handle/AgentPalette values, client-only candidate proposal
  and bounded checking with explicit manual-value preservation, own-contract tests.
- Non-goals: schema activation, production ID cutover, API/UI activation or effects.
- Interfaces: Handle canonical string, palette canonical key; candidate generation
  accepts resource kind/display name/random source and returns canonical handle;
  checking is injected, bounded and cancellable. No Repository ID codec is exposed.
- Approved Design mechanisms: M1, M2, M7, retained M12.
- Authority: REQ-4..6,9; ADR-D3,D5; Design handle/profile boundaries.
- Design delta: None.
- Removal obligations: no active behavior replaced in this prerequisite phase.
  Existing mutable Team/wire/hex-ID paths are owned by phases 2/3, not retained
  fallback modes. No unsupported feature is exposed to users in phase 1.
- Absence verification: only domain/client helper exports and their tests added;
  no migrated database, new wire field or UI mutation is activated.

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Backend values | /root | rust/crates/domain/src/{handle,agent_profile,lib}.rs | Requirements | Typed canonical values | cargo test/clippy/fmt domain |
| Browser candidates | /root | typescript/packages/web-core/{src/handle.ts,test/handle.test.mjs,package.json}, pnpm-lock.yaml | Same grammar | Client proposal/check helpers | package tests, lint/type/build |
| Baseline and plan | /root | docs/chapchoo/{requirements,adr,design,plans}/profile* | Approvals | Traceable scope | metadata/link/authority checks |

- Integration order: baseline -> domain values -> client helper/dependency -> tests.
- Independent review: `/root/profile-reviewer`, root requests stable complete diff.
- Final validation: root runs focused Rust and web-core quality checks, docs checks,
  hooks and new-SHA CI. Wider unaffected suites run in integrated later phases.
- Scope drift: grammar, short/empty/collision/manual behavior exactly match approval;
  no new source of truth, server generation, compatibility or profile edit mode.
- Context checkpoint: pending initial implementation; original checkout protected.
