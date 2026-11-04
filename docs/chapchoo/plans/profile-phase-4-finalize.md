---
title: "Agent Profile Phase 4 — Preservation Evidence and Coordinated Cutover"
created: 2026-11-05
tags: [agents, identity, implementation, validation, operations]
---

# Phase Execution Plan

- Phase: 4/4, integrated preservation, operator procedure and verified promotion.
- Branch/base: `docs/profile-finalize-261105` -> `feat/profile-ui-261105` at `1efb13f`.
- PR boundary: final reviewable evidence/documentation/cleanup stacked after [phase 3, PR #73](https://github.com/azents/chapchoo/pull/73). No merge, deployed preparation or rollout.
- Inputs: [Requirements](../requirements/profile-261104-agent-identity.md), [ADR](../adr/profile-261104-agent-identity.md), [approved Design revision 1](../design/profile-261104-agent-identity.md), and [stack plan](profile-implementation-plan.md). Phases 1–3 provide all implemented primitives, Repository preservation, native boundaries, generated contracts and web behavior.
- Approved Design mechanisms: M8, M9, M10, M11, M12; verify M1–M7 against the integrated evidence without adding another mechanism.
- Authority: REQ-1..10, ADR-D1..8 and the exact M1..M12 approval. Existing explicit maintenance authority and preservation invariants remain fixed.
- Design delta: None.
- Implementation, decisions, integration and validation: primary `/root`.
- Exact independent reviewer: `/root/profile-reviewer`, read-only one-way review after the complete phase diff is stable; root applies corrections.

## Deliverables and fixed boundaries

1. Consolidate truthful requirement/mechanism evidence for populated SQLite and PostgreSQL preservation, original-root ciphertext rewrap, historical receipts, native witnesses and saved-resource continuity. Reuse fresh unchanged source evidence; rerun only invalidated tests. Distinguish synthetic substrate, actual Docker, disposable Kubernetes, browser and deployed-system evidence.
2. Publish a source-grounded coordinated maintenance/backup/preparation/recovery procedure using the actual current CLI and contracts. Explain mutation/dispatch quiescence and retirement of old execution processes, preserving original saved environments, exact original installation root, transactional preparation, coherent component replacement and revalidation. Describe failure recovery without hidden adoption, relabeling, recreation, replay or mixed formats.
3. Complete semantic Wiki reconciliation and reference validation; record a supporting verification report with commands, environment, covered contracts and explicit limits. Mark Requirements and primary Design with the same implemented date only after implementation and required validation are complete. ADR remains unchanged.
4. Preserve trustworthy real-component desktop/mobile renders and their conditions outside tracked source. Do not turn a concept mock or synthetic native fixture into deployment evidence.
5. Remove only this feature's master and phase plans after verified promotion. Keep unrelated account/Computer plans and protected source experiments. Preserve evidence and clean only owned finished scratch/build resources; never prune shared storage or foreign native objects.

Non-goals: new product features, identity/schema/wire mechanisms, compatibility aliases, runtime migration switches, versioning, new edit-conflict authority, infrastructure changes, merge, deployment or operational migration. A newly material finding returns to Requirements/ADR/Design authority rather than being smuggled into an operator guide.

## Owned workstreams

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Integrated evidence closure | Root | Existing focused preservation/native/RPC/browser tests and Session evidence; supporting report in `docs/chapchoo/design/` | Phases 1–3 and exact source SHAs | Explicit coverage and non-coverage, requirement/mechanism trace | Fresh-SHA CI/automatic review, assertion/source inspection, affected gates only |
| Operator procedure | Root | Owning Repository-identity Wiki and a focused `spec/flow/` maintenance page if needed, connected index/references | Actual preparation CLI and preservation contracts | Executable operator steps and safe stop/failure boundaries | CLI/source cross-check, semantic Wiki lint, local links |
| Verified promotion | Root | Owning Wiki subgraph, this snapshot's Requirements/primary Design metadata | Complete validation | Current behavior and matching implemented date | Docs metadata tests and consistency review |
| Plan cleanup | Root | Only `docs/chapchoo/plans/profile-*.md` | Evidence/report/procedure/promotion complete | No feature-plan authority or broken inbound links | Exact tracked-file/removal and inbound-reference search |

## Integration, review and completion

Integration order: inspect exact evidence and gaps -> cross-check current CLI and write operator procedure -> close affected validation/CI and visual evidence -> promote verified contracts -> remove feature plans -> freeze complete diff -> sole integrated review -> root corrections/affected checks -> normal commit and final stacked PR -> monitor its exact-SHA CI/automatic review.

Removal obligations: this feature's temporary master/phase execution plans only. The obsolete ID/query routes, Team handle edits, Service ID issuance and native-ID derivation were removed in their owning prior phases; recheck their absence rather than creating replacement legacy paths or obsolete-feature tests. The execution plan is committed before implementation so its authority remains available in Git history when final cleanup removes it.

Root final validation: docs metadata and its unit suite, touched local references, source-grounded operational steps and authority/removal checks; reuse the unchanged integrated Rust/SQLite/PostgreSQL/RPC/browser/native evidence with exact source provenance. Wait for required fresh-SHA CI and handle concrete automatic review findings. Missing deployment permission remains an explicit boundary, never skipped execution reported as success.

## Context checkpoint

PR #71 is green; PR #72 at `dd3cb91` is fully green including actual Docker and disposable Kubernetes authentication/lifecycle. Phase 3 source `1efb13f` is open as PR #73 after whole Rust/RPC/TypeScript/build/component/all-four-browser validation and the sole review plus confirmation of two REQ-8 corrections. PR #73 fresh-SHA CI/automatic review is in progress. This phase has not performed operator maintenance or changed implementation. The requester delegated recommended intermediate decisions; no material Design delta or new approval wait is introduced.
