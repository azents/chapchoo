---
title: "Docker Computers Implementation Plan"
created: 2026-11-03
tags: [computer, docker, implementation]
---

# Docker Computers Implementation Plan

## Authority and ownership

- Inputs: [Requirements](../requirements/docker-261103-server-computers.md),
  [accepted ADR D1–D6](../adr/docker-261103-server-computers.md), and
  [approved Design revision 6](../design/docker-261103-server-computers.md).
- Requester approval: 2026-11-03, 15:45:15 KST.
- Approved mechanisms: M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12,
  M13, M14, M15, M16.
- Design delta: None.
- External execution boundary, explicitly clarified by the requester after
  approval: either Provider may be run by a trusted third party, with no product
  DB or installation-root access. All persisted work in this plan belongs to the
  server. Independent third-party credential-management changes require their
  own confirmed authority; do not silently expand the single configured owner.
- Implementation, integration, corrections and validation owner: `/root`.
- Single independent reviewer for the complete feature and every stable
  integrated phase: `/root/computer-reviewer`, read-only, one-way opinions.
- Delivery: four sequential reviewable PRs, rooted at main `210db53`.
  Each PR opens before implementation of its successor starts. No merge,
  deployment or production substrate mutation is authorized.

## Delivery phases

| Phase / PR title | Boundary and mechanisms | Dependencies and interfaces |
| --- | --- | --- |
| Docker computers [1/4]: native resource contracts | Domain-owned native kinds, scopes, identities and roles; strict persisted kind decoding replaces the storage-owned K8s enum. M3/M4/M7 foundations, M10/M12 owner-layer evidence. No Docker activation or schema change. | Base main; fixes typed contracts before persisted/wire integration. Existing K8s behavior unchanged. |
| Docker computers [2/4]: durable native authority and protocol | Both-engine preservation migration, typed store/service native reservations and evidence, immutable daemon binding, wire/DTO/cursor changes, generated clients and complete server-side exact retirement manifest/admission. M1/M3/M4/M5/M8/M10/M11/M16. | Phase 1; DB-first prepare required, coordinated components required. No Docker effect forwarding before the complete server once-only gate is integrated. |
| Docker computers [3/4]: selected Docker transport and lifecycle | CLI context/default/override resolver, public SDK transport, native adapter, shared task owner and Provider-side once-only native stages matching the server retirement manifest. M2/M3/M5/M6/M7/M8/M9/M13/M15/M16. | Phase 2; only isolated explicitly selected fixture resources. Existing K8s contract remains green. |
| Docker computers [4/4]: platform delivery and acceptance | Native Mac build/distribution, Linux image ownership, General/Admin labels and generated-client consumption, minimal real Linux E2E, distinct native-Mac/remote acceptance evidence and final Spec promotion. M9/M10/M11/M12/M14 and complete-feature regression. | Phase 3; missing native acceptance blocks support claims, not permission to narrow scope. |

All paths remain root-owned; phase plans specify exact current paths, integration
order and commands. A later phase may refine execution decomposition without
changing authority. A material discovery returns to Requirements/ADR/Design and
requester approval.

## Removal and absence obligations

- Phase 1 replaces the storage-local resource-kind declaration and permissive
  kind decoding with one domain authority; native roles cannot be caller-relabeled.
- Phase 2 replaces K8s-only schema checks, shared namespace-only storage/DTO/cursor
  authority and reservation/evidence assumptions. Preserve exact K8s identities,
  routes, key BLOBs, receipts and current execution cycles in both engines.
  SQLite stages the closed eleven-table graph with foreign keys enabled, verifies
  lossless copies before clearing the old graph, and rolls back any mismatch.
- Phase 3 removes shared `Arc<Kubernetes>` ownership and K8s-only dispatch/cleanup
  assumptions at activation. No Docker object is represented as Pod/PVC/Secret.
  Every retirement entry point uses the exact Docker admission fence; correlation
  loss permits observation only, never reconstructed mutations.
- Phase 4 replaces universal K8s-only inventory/docs/fixture claims and establishes
  non-root image mount ownership without privileged repair. Regenerate outputs,
  search obsolete assumptions and verify both substrates' complete contracts.
- No Kubernetes mode, owner permission, original saved data or credential recovery
  authority is removed. Never prune caches, volumes or unrelated host objects.

## Validation and prerequisites

Root runs format, Clippy, focused owner tests and interface-dependent checks before
freezing each integrated phase for the assigned reviewer. Root fixes findings
directly and runs affected checks, then opens the phase PR and checks exact-head
CI. Hook execution is mandatory.

- Domain: strict decoding, native bounds, role/substrate consistency, exact
  identity comparison and original-volume metadata preservation.
- Persistence: real SQLite and disposable PostgreSQL; complete migration graph,
  credentials/recovery/receipts/routes, immutable binding, resource constraints,
  current-source evidence, sticky disappearance and no replay.
- Protocol/service: generated source, mixed/unknown payload rejection, unchanged
  K8s field meanings, owner/current-source/request/execution checks, bounded
  inventory/cursors and privacy.
- Adapter/Control: supported SDK synthetic transport tests; endpoint/context/TLS/
  SSH configuration fidelity; no alternate-host fallback; retained task lifetime,
  ACK boundaries, native retirement stages and uncertain progress.
- Linux: one serial Docker Computer lifecycle added to required existing
  acceptance using current built artifacts and runner-owned disposable Engine.
  Missing declared prerequisites fail; do not duplicate the browser matrix.
- macOS: native build checks for claimed Apple/Intel targets, plus separately
  recorded real already-running Docker Desktop acceptance on authorized resources.
  Linux and cross-builds do not satisfy real-Mac acceptance.
- Remote SSH/TCP/TLS modes: explicit opt-in real selected-mode evidence;
  unavailable, failed and passed remain distinct. Ordinary CI owns resolver and
  transport fixture coverage.

Current environment is Linux. Docker CLI presence establishes neither a reachable
daemon nor mutation authorization. No native Mac/real Desktop infrastructure has
been established. These gates stay visible throughout implementation.

## Checkpoints, rollout and completion

Each phase records behavior, interfaces, commands/results, removal evidence,
remaining scope, risks and PR/head in its phase plan. Do not mark the snapshot
implemented before full implementation and required validation. Promote current
Specs at their owning boundary (especially schema), then run final spec review.
Only after validated promotion remove this plan and phase plans in the final
delivery diff; do not create an extra plan-only cleanup PR. Operator rollout
requires explicit DB prepare and coordinated updated binaries and is outside
this implementation authorization.
