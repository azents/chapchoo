---
title: "Agent Profile and Identity Implementation Plan"
created: 2026-11-04
tags: [agents, identity, implementation]
---

# Agent Profile and Identity Implementation Plan

- Inputs: [Requirements](../requirements/profile-261104-agent-identity.md),
  [ADR](../adr/profile-261104-agent-identity.md),
  [approved Design revision 1](../design/profile-261104-agent-identity.md).
- Approved mechanisms: M1..M12, with exact authority in Design Authority.
- Design delta: None.
- Implementation/integration/validation owner: primary agent `/root`.
- Single independent reviewer: `/root/profile-reviewer` (read-only); root evaluates
  findings and performs all edits and validation.

## Delivery boundaries

Use a four-PR stack, opening each integrated phase before starting its dependent
phase. No automatic merge or deployment. Shared title prefix:
`Agent profiles [n/4]`.

1. **Validated values and candidate generation** (M1, M2, M7): Rust handle/palette
   domain values and client-only AnyAscii candidate generation with bounded
   availability checking, explicit manual override, syntax and deterministic
   tests. Carries the complete approved design baseline. Does not activate new
   IDs, schema or wire behavior in isolation.
2. **Preserving Repository transition** (M4, M5, M7..M11): private identity codec,
   DB adapters, catalogue/profile/handle storage, repository-generated IDs,
   historical receipt and native witness preservation, crypto rewrap and complete
   PostgreSQL/SQLite preserving preparation. Update Service/private projections
   and native allocation-name boundaries. Activation waits for matching contracts.
3. **Public contracts and scoped web behavior** (M1..M6, M12): generated wire/Node
   contracts, DTO validation, authoritative availability and profile update,
   team/null/immutable handle and exact Team+handle reads, General/Admin routing,
   selector, single-screen creation, profile/list/detail and locale copy/stories.
4. **Integrated preservation and evidence** (M8..M12): minimal existing/new browser
   journeys, both-engine populated migration and actual RPC, native saved-resource
   continuity tests, maintenance operator procedure, visual verification, Spec
   promotion and temporary-plan cleanup. Required fixes belong here or in the
   responsible earlier branch, not outside approved authority.

This stack is one coordinated release, not four independently deployable schema
or runtime formats. Do not publish or execute a migration between incompatible
phases. Reviewability does not imply mixed-binary rollout compatibility.

## Interfaces and removals

Domain grammar and palette keys precede persistence constraints; generated wire
clients follow the authoritative Protobuf source. Repository DTOs carry only
complete canonical IDs. Phase 2 removes Service ID construction and personal
synthetic handles; phase 3 removes Team handle edits and ID-based General routes.
No profile or identity defaults are recomputed on reads. Native/credential proof
is preserved, not replaced with alias fallback. Removal table and absence checks
in Design are the phase checklist.

## Validation and prerequisites

Root runs domain/client tests in phase 1, both DB/private codec/crypto/native tests
in phase 2, actual generated RPC and General/Admin component/type/build checks in
phase 3, integrated Rust and minimal E2E plus responsive images in phase 4.
Isolated local PostgreSQL and testenv fixtures are permitted; deployed data and
credentials are not inputs to these tests. Optional/live external checks require
explicit prerequisites and cannot be claimed when skipped. Monitor owned build
storage and clean completed scratch/builds, preserving all current source and
other projects' data.

After each stable integrated phase, request one read-only review, apply findings,
rerun affected checks, commit with hooks, open its PR and monitor fresh-SHA CI and
automatic review. Keep the same reviewer and preserve approval mechanisms. A new
material behavior, authority, compatibility mode or data-loss shortcut returns to
Design; local reversible details remain root-owned.

## Checkpoint

Phase 1 starting. No product implementation previously existed. Approved baseline
is pinned to source 7ba03553; existing untracked source experiments in the original
checkout are protected. No migration or resource effect has run.
