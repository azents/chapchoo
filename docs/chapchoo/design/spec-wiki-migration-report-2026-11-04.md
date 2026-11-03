---
title: "Chapchoo Spec Wiki Migration and Contract Preservation"
created: 2026-11-04
tags: [documentation, wiki, maintenance]
document_role: supporting
document_type: supporting-verification
---
# Chapchoo Spec Wiki Migration

This report records the requested documentation refactor, not a new product
Design or an implementation-wide behavior audit. The current contract lives in
[the Spec Wiki](../spec/index.md); this report is an independent supporting record.

## Reference and scope

The requester asked to adopt the latest Azents Spec format, including related
documents, prompts and Skills. The reference was Azents main
`a57e32f789d964fdcb4c7296a1d9c38296f8b9eb`, particularly its Spec Wiki maintenance
guide, question-led index, semantic impact/audit Skills and lightweight metadata
validator. The Chapchoo source baseline was
`d6701abeab57be3c48678fc98a78bda4371efebc`.

The thirteen previous specs combined multiple owners behind broad account,
web, delivery and Computer storage/Control titles. They are now **35 owning
topic pages**, with a question-led editorial index and a scoped Agent guide.
Existing page paths remain useful entrypoints; their connections navigate the
contracts that moved rather than duplicating the full original page.

## Contract ownership disposition

| Previous owner | Current division of responsibility |
| --- | --- |
| Account RPC | RPC admission/protocol; account service operations/mail; foreground process ownership/health |
| Account storage | Schema/preparation/key binding; completed account/Team/session transactions |
| General/Admin account web | App boundaries; workspace navigation; appearance; display settings; mascot motion; request/session/cookies; primary rendering/URL state; runtime configuration |
| Linux distribution/deployment | Artifact/image assembly; foreground child ownership; Helm/Argo lifecycle and Secrets; Snapshot publication; verification-layer boundaries |
| Computer storage | Native identities/schema; private Agent/Team ownership; lifecycle admission/Provider keys; Connector credentials; execution reservation/evidence/cleanup |
| Computer Control RPC | Stream/call admission; native retirement correlation; Provider process/ACKs; reconciliation/restoration; Connector process; memory/peer delivery |
| Credential primitives | Reusable value/crypto/hashing responsibility, connected to its actual service/storage consumers |
| Control directory | Durable exact owner selection and current-credential/lease eligibility |
| Computer owner/inventory RPC | Private owner operations and redacted read-only administrator inventory |
| Docker adapter | Selected connection, original volume, container stages and uncertainty |
| Kubernetes adapter | Original resource identity and ordered terminal evidence |
| Computer web | Truthful owner controls, bounded reads and independent operator inventory |
| Design reference | Reference ownership, selective promotion and evidence boundaries |

All named source sections were assigned to an owning page. A paragraph-level
comparison verified that the preexisting body clauses were retained, except for
phase-history statements or the narrow evidence-backed corrections below. This
is content-preservation evidence, not semantic proof of every implementation
claim. The source/reference inventory and section comparison were disposable
migration aids, not a new tracked code-to-page catalog or future audit planner.

## Narrow current-claim corrections

- Credential primitives no longer claim there is no account server, persistence,
  web or CLI. The current composition and linked account RPC/storage pages own
  those capabilities. The crypto primitive itself still does not authorize an
  actor, establish session expiry or replace the installation root.
- Primitive hashing guidance points to the existing bounded service caller rather
  than an eventual future caller. `rust/crates/service/src/password.rs` holds its
  owned permit inside `spawn_blocking`; primitive tests do not certify the service.
- The Computer directory intro distinguishes its storage responsibility from the
  implemented transport owner instead of calling transport later-phase work.
- The Computer relationship explanation uses the already implemented immutable
  Kubernetes/Docker Provider kind and native resource scope. Migration 6 defines
  the typed container/volume identity constraints and one saved-environment index;
  the older Kubernetes-only sentence did not describe the current schema.

No product implementation, API, migration, role, cancellation, physical cleanup,
credential or lifecycle behavior was changed. Existing uncertainty and evidence
limits remain beside the owning claims. This migration does not certify a live
cluster, physical browser/OS behavior or every linked source path's semantics.

## Authoring, prompts and tooling

Only `docs/chapchoo/spec/` is the wiki. Requirements, accepted ADRs and implemented
Designs remain outside it with their original lifecycle and content. Root/scoped
Agent instructions, developer commands and authoring/shipping Skills now navigate
by behavior and ownership, ingest verified changes into the owner, query connected
pages, and lint the touched subgraph.

`spec-review` examines the actual diff's semantic impact. `living-spec-audit`
checks a bounded semantic topic plus the independent preceding-24-hour change
window. Its obsolete path-bucket planner/checkpoint scripts were removed. No
replacement planner or duplicate ingest/query log is introduced. The separate
convention-audit workflow remains independent and unchanged.

The old catalog/query/path-matching script and its obsolete tests were removed.
`validate_docs.py`, adapted from the current Azents implementation, checks minimum
metadata and Requirements → ADR → Design snapshot relationships. Wiki pages
require `title`; it does not infer relevance, ownership, truth or coverage.
Per-page versions, global verification dates, `code_paths`, domain/type discovery
metadata and changelog mechanics are no longer the wiki format. CI and local
hooks call the lightweight validator and its current tests.

The schema convention points to the owning wiki explanation and bounded evidence,
not a timestamp update. Its index is regenerated by the existing isolated
pre-commit hook rather than edited manually. Chapchoo's test-strategy and immutable
snapshot rules are retained independently of Azents-specific historical exceptions.

## Validation and limits

- The lightweight validator and all 18 validator tests passed during migration,
  including title-only pages and progressive/implemented snapshot constraints.
- Local Markdown targets and heading fragments were checked after the split;
  references are ordinary GitHub/local links, not a new wiki framework.
- Topic ownership, section preservation, source-phase corrections and connected
  claims receive semantic review separately from those mechanical checks.
- Required hooks and exact-head CI are delivery evidence attached to the PR.
  Product tests are not rerun locally merely to rename or split documentation;
  focused documentation/tooling checks exercise the changed maintenance behavior.

The final report must distinguish the checked migration/tooling scope from an
implementation-wide audit. Merge and deployment remain separate requester actions.
