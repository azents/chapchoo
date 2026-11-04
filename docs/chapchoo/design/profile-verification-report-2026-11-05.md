---
title: "Agent profiles and Repository identity — integrated verification"
created: 2026-11-05
tags: [agents, identity, verification, security, operations]
document_role: supporting
document_type: supporting-verification
---

# Integrated verification

This supporting record closes implementation evidence for
[profile-261104/REQ](../requirements/profile-261104-agent-identity.md) and the
[approved revision-1 Design](profile-261104-agent-identity.md). It is not a second
current Spec or permission to operate a deployed installation. Design delta: None.
The current contract is owned by the [Spec Wiki](../spec/index.md).

## Source and execution provenance

- Foundation: [PR #71](https://github.com/azents/chapchoo/pull/71).
- Repository/native/creation consumers: [PR #72](https://github.com/azents/chapchoo/pull/72),
  final source `dd3cb91d2e71c1b81d4a27bafc4c18e93bbf2ce0`.
  [CI run 41917173467](https://github.com/azents/chapchoo/actions/runs/41917173467)
  passed the complete matrix, including both architecture artifacts, browsers and
  actual Docker/disposable Kubernetes execution. Earlier run
  [41912605989](https://github.com/azents/chapchoo/actions/runs/41912605989) also
  established the native integration result at `71b0198` before the final browser
  fixture correction.
- Integrated RPC/web: [PR #73](https://github.com/azents/chapchoo/pull/73), source
  `1efb13fc6e618321070ccd57efdcbeb7d609b31f`.
  [CI run 41923208218](https://github.com/azents/chapchoo/actions/runs/41923208218)
  passed the complete matrix at that exact source, including Rust, validation,
  both architectures' web/binary/packages, all assembled browser journeys and
  actual Docker/disposable Kubernetes execution. Codex completed at that SHA
  without findings.
- This final phase changes documentation only. Unchanged implementation evidence
  is reused rather than relabeled as a newly executed deployed migration.
  Final-phase CI/review results are recorded on its PR, not predicted here.

Local validation used Linux, Cargo/Rust 1.99.1, Node 24.22.1 and pnpm 11.26.0,
SQLite plus a private Unix-socket PostgreSQL fixture. Credentials and root keys
were disposable synthetic inputs, never production inputs. Native CI used an
explicit disposable Docker Engine and a disposable Kubernetes cluster. Missing
prerequisites fail; they are not counted as skipped passes.

## Requirement and mechanism evidence

| Authority | Implemented behavior | Evidence and limits |
| --- | --- | --- |
| REQ-1, M1/M12 | Saved image/name/handle/palette, two desktop/one mobile cards, diagnostics and separate deletion | Real-component stories and General component checks; assembled owner journey rereads saved state. Static visuals are not deployed screenshots. |
| REQ-2, M1/M6 | One-screen Provider-backed creation, profile preview and final handle address, unchanged uncertain-result ownership | Assembled lifecycle journey and client/core checks; final fingerprints and removed receipts in both-engine/RPC tests. |
| REQ-3, M1/M6 | Desktop dialog/mobile sheet, draft preview, cancel/close/Escape, failed-save retention, immutable handle | UI 32, General 37 and Admin 10 component checks; saved update and script-blocked SSR reread in assembled browser. |
| REQ-4, M7 | Six bundled initial assets, six product palettes, deterministic one-time defaults, preserved edits and extensions | Both-engine populated preparation tests and profile/catalogue Repository tests; no upload/catalogue-management feature implied. |
| REQ-5..6, M2/M6 | Exact 3–16 ASCII grammar, global Agent/live-Team uniqueness, personal NULL, bounded client proposals and manual override | Pure candidates/controller and database acceptance/race checks, including embedded-NUL rejection on both engines; availability remains advisory. |
| REQ-7..8, M3/M12 | Exact Team+Agent scope, personal `my`, mount-aware paths, name+handle picker and prospective Team address | Both-engine scoped resolution/authorization and actual RPC; all four assembled browser journeys include General/Admin Team preview and navigation. |
| REQ-9, M7..10 | Populated transition, retained relationships/receipts/cipher contexts/native witnesses, transactional failure/retry | The preservation tests described below, synthetic native-adapter witnesses and separately actual native lifecycle execution. No deployed populated-native cutover has run. |
| REQ-10, M4..6/M11 | Repository-private UUIDv7/compact IDs and opaque DTOs; configuration key distinct from canonical Provider | Private codec/projection and issuer-boundary inspection, full Rust quality, generated Node/Rust RPC on both engines, frontend builds/typechecks. |
| REQ-10, M8/M12 | Explicit original-root preparation, stopped compatible release boundary and source-qualified recovery | Source inspection of CLI/config/runtime plus existing lifecycle contracts; executable operator procedure in the owning Wiki. This is a procedure, not evidence of performed maintenance. |

## Populated database preservation

[`profile_preparation/tests.rs`](../../../rust/crates/persistence/src/profile_preparation/tests.rs)
installs the unchanged migrations 1–8, populates account/Team/Agent/Computer and
operational references, then runs real explicit preparation through migration 9.
The SQLite and PostgreSQL tests verify:

- compact canonical identity conversion, relational joins and personal-context
  ownership; nullable personal handles and collision-safe collaborative backfill;
- original resource names/UIDs and exact allocation Computer/Provider/execution
  witness values independently of their new canonical product IDs;
- removed-Agent creation receipts without a live assigned Agent or Team, plus
  authenticated historical present/removed/never-accepted reads and foreign denial;
- same recovered account/Provider/Connector tokens under new authenticated
  contexts, rejection under old contexts, original digest and expiry/overlap facts;
- six initial catalogue records, deterministic saved profiles and subsequent
  profile edits/installation extensions surviving repeated preparation;
- refused destructive downgrade and no remaining preparation-local runtime map.

SQLite failure tests corrupt a registration envelope after other tentative
rewrites, require byte-identical rollback and migration-8 history, reject unknown
incoming foreign keys and attached triggers without dropping them, then explicitly
retry successfully after fixture correction. PostgreSQL compares snapshots across
all identity tables after a late failed rewrap, checks retained migration history,
corrects the fixture and explicitly retries; repeated preparation preserves stored
rows and catalogue identities. These fixture-only corrections are not operator
instructions to edit deployed ciphertext or remove foreign schema dependencies.
Original-root/key-readiness rejection is also covered by preparation tests.

[`native_binding.rs`](../../../rust/crates/persistence/tests/native_binding.rs)
checks both-engine immutable original daemon binding, foreign-owner denial,
concurrent different-first-binding acceptance and same-binding recovery.
Native-adapter fixture tests establish witness comparison/containment and
uncertainty behavior; they do not constitute a live substrate migration.

## Assembled product and native execution

Local final-source checks passed the complete Rust quality entrypoint and actual
generated Node/Rust interoperability: **105 checks on SQLite and 105 on PostgreSQL**.
Normal commit hooks repeated those gates. Frontend checks passed format, lint,
14-workspace typecheck, both Next production builds, all three Storybooks and
focused core/app/component suites. After the sole independent review's two
REQ-8 omissions were corrected, all four assembled browser journeys passed
together, including private context/authorization, workspace membership,
General/Admin destination previews, profile persistence and lifecycle cleanup.

The native CI scenario uses public generated RPC with actual backend/Control,
Docker Provider and execution-local Connector. It verifies non-root saved data,
distinct execution replacement with the **original volume and no copy**, normal
deletion and unrelated canaries. Disposable Kubernetes chart acceptance exercises
ServiceAccount/TokenReview admission and lifecycle/cleanup. Neither run starts
from an existing deployed pre-transition database and saved environment: combine
this evidence with populated DB and native-witness tests without claiming an
end-to-end production migration rehearsal.

The browser fixture initially used Asia/Seoul while its account defaults were UTC.
The resulting legitimate display-settings suggestion toast covered Create.
The final correction matched fixture display settings, retaining product
notifications, real pointer actionability and all validation. No force-click,
longer timeout, disabled security check or temporary diagnostic catch remains.

## Reproduction entrypoints

Run from the repository root with disposable prerequisites documented in
[DEVELOPMENT.md](../../../DEVELOPMENT.md):

```sh
bash scripts/check-rust.sh
bash scripts/check-rpc.sh
pnpm format:check
pnpm lint
pnpm typecheck
pnpm --filter @chapchoo/web build
pnpm --filter @chapchoo/admin-web build
pnpm build-storybook
pnpm --filter @chapchoo/e2e test:e2e
python scripts/validate_docs.py
python -m unittest scripts.tests.test_validate_docs
```

Component reproduction uses each owning UI/web/admin Storybook and component-test
script; inspect current manifests and prerequisites rather than substituting mock
screens. Native reproduction requires the explicit disposable inputs and commands
in the [integration README](../../../typescript/apps/integration/README.md) and
[acceptance support](../../../testenv/chapchoo/README.md). No command above is an
instruction to use a production database, daemon or cluster.

## Visual evidence and review

Final real-component PNGs are retained as Session artifacts with a SHA-256
manifest and render-condition record. Chromium uses light scheme, 100% zoom,
DPR 1, 1280×900 desktop and 375×812 mobile viewports. Captures use the real Mantine
theme, bundled assets and Pretendard, with font readiness awaited. Full-page
mobile height is actual document height, not resampling. Saved cards, creation
and mobile edit use credential-free English/Korean fixtures. The mobile creation
capture starts at scrollY=0; no compositing, cropping or stretching was used.
The images are presentation evidence, separate from assembled browser persistence.

One assigned independent reviewer examined each integrated implementation phase.
The root applied its SQLite grammar and two Team-picker/address-preview findings;
the same reviewer confirmed the corrections. Final-phase evidence/procedure
review and exact-SHA automated results remain attached to the final PR.

## Promotion, removal and operational limits

The owning Wiki covers Repository identity/preparation, Agent profiles, account
and Computer RPC, workspace navigation and Computer web behavior. The
[cutover procedure](../spec/domain/repository-identity.md#operator-procedure-for-the-identity-cutover)
connects quiescence, safe retirement, paired database/root backup, explicit
`prepare`, matching-process restart and original saved-data checks. It distinguishes
transaction rollback from separately approved post-commit restoration.

Consumer inspection confirms the old General ID/query routes, mutable Team-handle
updates, Service ID issuance, public row/codec exports and DTO-derived native
names were replaced in their owning phases. Necessary historical request hashes
and original allocation witnesses remain preserved authority, not runtime aliases.
Only this feature's temporary master/phase plans are removed after promotion;
unrelated account/Computer plans and source experiments remain untouched.

Implementation completion is not deployment completion. No merge, live Helm/Argo
rollout, production credential/data access, deployed backup/restore rehearsal,
old-process retirement, live identity preparation or populated-native cutover was
performed. Do not downgrade old binaries against a converted database, rotate the
root, recreate saved environments or replay uncertain effects as a substitute for
approved maintenance and exact evidence.
