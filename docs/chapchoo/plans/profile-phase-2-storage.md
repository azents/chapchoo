---
title: "Agent Profiles Phase 2: Repository Identity and Preserving Storage"
created: 2026-11-04
updated: 2026-11-05
tags: [agents, identity, implementation, migration]
---

# Phase Execution Plan

- Phase: 2/4, Repository identity and preserving storage.
- Branch/base: feat/profile-storage-261104 -> design/agent-profile-handles-261104
  (phase 1 PR #71, 9a6a549).
- PR boundary: repository-only issuance/codec and compact storage, preserving
  initialization and complete database profile/handle operations, corresponding
  Service/native contracts needed before public activation in phase 3.
- Inputs: confirmed profile-261104/REQ, ADR-D1..D8, primary Design revision 1,
  validated phase-1 Handle/AgentPalette and client candidate contract.
- Approved mechanisms: M4, M5, M7, M8, M9, M10, M11, retained M12; storage aspects
  of M1, M2 and M6 plus their required creation-form input consumers. Authority
  is the exact primary Design authority table.
- Design delta: None.
- Interfaces: private EntityId and backend SQL type never exported; public DTO IDs
  are complete opaque strings. Complete Store operations issue IDs; native names
  and expected original allocation proof stay distinct from canonical entity IDs.
  Client candidate generation is not moved into normal server creation.
- Compiler-facing dependency: nullable Team DTOs and immutable Team patch require
  matching Protobuf/generated-client and minimal existing consumer type changes
  in this phase. These changes only carry the approved storage contract; new
  scoped navigation and saved profile surfaces remain phase 3. Do not preserve a
  synthetic/empty personal handle to avoid this work.
  The completed Agent acceptance similarly needs final handle/image/palette
  inputs through Service and Protobuf, with matching generated client fields and
  existing creation-form consumers. Carry final selections and client candidate
  handling in phase 2 rather than introducing server-generated normal defaults
  or an interim old payload mode. This is a dependency refinement within approved
  M1/M2/M6, not new Design authority. Add catalogue and availability calls, SSR
  initial catalogue data, one-time independent image/palette draft initialization,
  exact handle input and the existing uncertain-payload lock to the current form.
  Team-qualified routes and saved profile/list/detail surfaces remain phase 3.
  Owned client paths: General computers creation types/container/component,
  shared server computers adapter/SSR loader, web-core profile DTO/copy helpers,
  and matching stories/component/client-state fixtures. Validate actual
  catalogue calls, candidate/manual/uncertain state ownership, unit/component
  checks and General type/build before the integrated phase review.
- Non-goals: operating deployed preparation/maintenance, activating partial mixed
  versions, introducing legacy-ID lookup/compatibility flags, new scoped-route or
  saved-profile surfaces, or mention work.

| Workstream | Owner | Owned paths | Dependencies | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Private ID codec/adapters | /root | persistence identity modules, Cargo manifests/lock, boundary checks | UUID library | Canonical prefixed DTO/BLOB16/PG uuid boundary | module tests, actual SQLite/PG SQL roundtrip, wrong prefix/bits |
| Storage/preparation | /root | persistence modules and paired forward migrations | private adapters | Profiles/catalogue/handle constraints and preserving references | populated both-engine prepare/rollback/retry |
| Issuer/service integration | /root | service ID call sites, DB drafts, credential protection | new complete Store ops | Repository-issued IDs and unchanged secret contracts | existing account/lifecycle suites plus recovery proofs |
| Native/evidence integration | /root | domain native records, persistence witnesses, provider adapters | preserving maps | Original native identity/labels/UID continuity, safe names | Docker/K8s owned behavior tests |
| Documentation | /root | current owning Specs, phase checkpoint | verified behavior | accurate implemented internal boundary | semantic review and metadata validator |

Removal obligations: remove Service entity-ID generation and public row/DTO
aliasing in this phase; replace TEXT identity storage and personal synthetic
handles with preserving forward migrations; retain native allocation witnesses,
credential/session validity and historical receipt semantics. Existing public
routes/handle edit controls are removed by phase 3, not by a hidden fallback.
Absence checks enumerate issuer/codec imports, DB type/constraint catalogs, old
helper uses, empty/malformed new columns, plaintext credentials and stale DTO
projection paths.

Integration order: private codec/actual backend adapters -> preserving preparation
maps and constraints -> profile/handle ops -> issuer/credential/native integration
-> both-engine invariants/complete suites. Public wire activation belongs to phase 3.
No intermediate WIP is represented as a passing integrated phase.

Single independent reviewer: `/root/profile-reviewer`, read-only review requested
only by root after the complete integrated diff and checks stabilize. Root owns
all validation/fixes. Context checkpoint tracks current WIP and safe restart
instructions; no operations against real credentials or deployed data.

## Delegated execution decisions

The requester instructed root at 2026-11-04 17:18:54 UTC to finish by the morning,
choose and implement recommended intermediate decisions without approval waits,
and consolidate the decision briefing at the end. Preserve Requirements/ADR/Design
authority and record any material additions before implementing them; use this
explicit delegation rather than pausing for a new requester response. This does
not expand the separately excluded deployed migration/deployment boundary.

Local creation dependency decisions: initialize the presentation draft once in
its authenticated SSR snapshot using independent image/palette draws, and hydrate
that exact draft. This is not Repository or POST defaulting. Keep final handle
validation/client candidates distinct from IDs, and retain one frozen complete
creation payload across uncertain outcomes. Only definitive pre-acceptance
rejections release that payload; handle conflicts restart candidate resolution.

## Active checkpoint

Integrated so far (not a completed phase):

- Private UUIDv7 DTO codec and both-engine compact SQL adapters.
- New forward migration 9, consumed only through explicit preparation-local
  identity/profile assignments. PostgreSQL converts identity columns to UUID
  with validated foreign keys; SQLite rebuilds the closed graph with foreign
  keys enabled and BLOB16 checks. Existing migrations 1–8 remain unchanged.
- One preparation transaction retains relationships, original native allocation
  Computer/Provider/Execution witnesses, registration keys, removed-Agent
  receipts and original accepted hashes. Account, operational Provider,
  Connector bootstrap/operational and registration envelopes are authenticated
  with the original root and rewrapped without changing tokens or validity.
- One-time Agent handle/profile and Team handle initialization, null personal
  handles, compact catalogue seed/extension and readiness validation.
- Runtime User, account, auth session, grant, login challenge and display
  operations now use private compact rows and Repository-issued IDs. Their
  Service drafts no longer contain proposed entity IDs.
- Authenticated catalogue read and owner/request-only creation-result recovery
  are implemented; recovery performs no mutation or allocation.
- Team, membership and invitation operations now use private compact rows.
  Repository accepts final canonical handles and issues Team, invitation and
  invitation-signup-grant identities. Team updates accept name only. Context
  deletion keeps its Team -> Provider -> Computer lock/effect ordering.
- Nullable Team Protobuf and generated clients, explicit browser null/handle
  validation and the minimum existing settings consumer changes are integrated.
  New scoped routes and profile UI remain phase 3.
- Provider registration now resolves the configured registration key inside the
  accepting transaction, assigns its canonical ID in Repository, and protects
  operational credentials under the canonical context. Existing configuration
  updates use the same key/owner/substrate boundary. The Store, Service, RPC and
  Provider process carry an explicit Provider ID plus operational key result;
  reauthentication rejects a changed canonical identity.
- Control registration and Provider/Connector connection preparation now assign
  their IDs in complete Repository operations. Directory rows are private compact
  records with explicit DTO projections. Exact captured-owner admission, renewal,
  replacement, close and monotonic draining keep their previous authority.
- Connector session issuance, credentials and routes now use private compact IDs;
  the accepting operation owns session issuance and encryption context. Initial
  and operational secrets, overlap, retry recovery and revocation remain distinct.
- Agent/Computer acceptance and lifecycle projections now use compact private IDs;
  Repository issues both IDs atomically. Creation fingerprints include Team,
  Provider, name, handle, image and palette. Catalogue references and exact final
  handle/palette values are validated; duplicate request recovery retains its
  accepted identity even after profile editing or Agent removal.
- Agent profile updates change name/image/palette only and perform no native or
  Computer mutation. Exact Team+handle lookup retains individual ownership and
  membership checks. Agent availability uses the global live-row namespace;
  Team handle lookup/availability retain authenticated live-Team boundaries.
  Inventory and Agent keyset queries use compact nullable cursors rather than
  empty identity sentinels.
- Final Agent create/result fields now pass through Service and generated wire
  contracts. Existing General creation now reads catalogue data through SSR,
  hydrates the same independent initial selections, uses bounded cancellable
  handle candidates with manual preservation, and submits all final fields.
  A frozen full request survives uncertainty and locks editing/pagination.
  Current old-format General paths remain scheduled for removal in phase 3;
  there is no legacy input/default mode.
- Authenticated bounded catalogue pages now cross Store, Service and
  Agents.ListProfileImages with compact keyset cursors. The Docker integration
  fixture consumes a real catalogue selection and the actual returned Provider
  ID; its configured registration key is not used as an entity ID.
  Logical asset keys are validated local identifiers, not paths or URLs.
  Browser-safe profile values live separately from Node RPC decoding.

Root-executed local evidence:

- All 28 persistence library tests pass against SQLite and a real disposable
  PostgreSQL server. Includes populated forward preparation, both-engine
  credential/native/reference preservation, original-root validation, rollback
  after tentative envelope rewrites and explicit retry, malformed handle/palette
  constraints, cancellation-owned leases and compact account/session operations.
- SQLite rejects unknown incoming identity dependencies before rebuilding.
- PostgreSQL rollback compares every inventoried table's complete row snapshots.
- The successful repeated preparation retains saved assignments; staging has no
  runtime lookup path.
- The owning authentication and Team integration suites each pass on both
  engines (four integration tests), using Repository-assigned identities.
  Credential/signup races, membership/role authority, invitation/transfer races,
  post-grant insertion rollback and failed-collection retry remain covered.
  Temporary duplicate authentication-flow checks were consolidated into the
  owning suite; private ID/catalogue/receipt-owner checks remain at their boundary.
- Web-core has 89 passing tests; General/Admin and browser consumers use the
  nullable contract rather than an empty or synthetic handle.
- General/Admin unit suites pass (18/20), all 14 workspace typecheck tasks and
  affected lint checks pass, generated clients match Protobuf, and both Next
  builds pass. General Storybook and its six Accounts component checks pass,
  including the narrow-screen owner state.
- Local SWC materialization required a private command-scoped cache because the
  shared Runtime HOME/tmp ancestry is world-writable. An owned 0700 cache under
  `/home/agent` passed the native check; no shared permissions, product/deployment
  configuration or validation policy were changed.
- One Rust 1.99.1 incremental compiler ICE (`unstable fingerprints`) was
  eliminated by cleaning only this worktree's persistence package artifacts and
  rerunning the same suite. This is not claimed as a source-level defect fix.
- The expanded both-engine component boundary exercises configured-key/canonical
  ID separation, authenticated registration recovery, corrupt-envelope rejection,
  concurrent bootstrap/session result recovery, private Control and route rows,
  exact connection replacement/stale-close refusal, draining, Connector bootstrap
  versus operational authority, key refresh overlap and revocation.
- At the Provider boundary, Control process CLI and external Kubernetes/Docker Provider CLI checks passed
  (three tests). The two Provider-auth unit checks pass, including canonical ID
  returned through private Trust exchange. These checks predate the final Agent
  create input; remaining native fixture compilation is not claimed current.
- Latest Protobuf regeneration and whole TypeScript graph typecheck pass
  (14 tasks). Strict persistence/service/RPC library Clippy and
  full Rust workspace check pass. This does not replace full integration hooks.
- The subsequent Agent boundary tests exercise both-engine concurrent same-request
  recovery, a cross-owner global handle race, changed-payload conflict across all
  immutable fields, catalogue validation, exact Team/owner lookup, immutable handle
  under profile update, typed inventory and pagination, lifecycle deletion and
  removed-result recovery without reallocation.
- Catalogue and handle availability pass actual Node/Rust RPC checks on SQLite
  and PostgreSQL (89 checks on each engine), including auth, Team privacy, exact
  grammar, paging and invalid limit/cursor rejection.
- General creation passes 12 Computer component checks, Storybook and current
  Next production build. Actual 1280x900 and 375-wide rendered captures were
  inspected. Unit checks preserve the full uncertain payload and independent
  initial image/color draws. A Storybook runtime failure exposed accidental
  Node RPC imports through profile constants; a pure browser-safe module fixed
  the boundary, without polyfills or disabled checks.

Latest native-boundary integration:

- Execution and delivery now use private compact rows with explicit DTO
  projections. Repository owns execution issuance; native names are independently
  generated DNS-safe correlation values. Managed-Computer cursors are nullable
  compact keysets, not empty-string sentinels.
- Every managed resource carries a required validated original allocation
  witness through persistence, Control wire, RPC decoding and exact peer resource
  matching. Kubernetes annotations and Docker labels use this committed witness,
  independently of current canonical Computer/Provider/Execution correlation IDs.
  Both adapters no longer own configured registration-key-derived Provider IDs.
- Native adapter tests preserve original volume labels and Kubernetes annotations
  across changed canonical contexts. Missing, malformed and role-mismatched
  witnesses reject before native I/O. Existing exact UID/daemon/container/volume
  identity and uncertain-effect retirement fences remain unchanged.
- External fixture consumers now use Repository-issued result/preparation IDs.
  Historical populated fixtures prepare the eight shipped migrations, retain the
  original installation binding and authenticated encryption contexts, then run
  explicit migration 9. They discover current IDs through public Store results;
  no private ID codec is exposed to tests or ordinary consumers.
- Root verified strict workspace/all-target Clippy, current generated-client
  equality and all 14 TypeScript typecheck tasks. Adapter/domain checks pass:
  Docker library 22, synthetic native integration 20, domain 26, Kubernetes 11,
  plus two compile-fail doctests. The both-engine persistence library 28, Team
  departure/cleanup 2, computer operations/credentials 2, native execution 3 and
  Provider registration/retention 2 checks pass. The expanded execution/directory
  fixture currently passes SQLite; its PostgreSQL run is in the full matrix.
- The complete authoritative `scripts/check-rust.sh` passes after fixture
  conversion: format, strict workspace/all-target Clippy, Rust tooling tests,
  warning-free docs, every workspace test on SQLite/isolated PostgreSQL, and
  dependency boundaries. Actual generated Node/Rust RPC passes 89 checks per
  engine; all 14 TypeScript typecheck tasks pass after the native wire change.
  Synthetic native tests are not real Docker lifecycle evidence.
- The full matrix exposed a real SQLite migration defect: rebuilding could
  silently discard a local trigger before display backfill. Migration 9 now
  rejects triggers attached to its closed converted graph. Populated preparation
  verifies byte-identical rollback, retention of that trigger and explicit retry;
  both display-upgrade scenarios pass. Historical migration tests retain their
  owned migration boundary rather than asserting the total number of all future
  migrations. Current schema constraint fixtures use explicit native UUID/BLOB
  values, not a public codec or caller-issued DTO identity.

- The selected assembled Computer browser journey passes with actual backend,
  PostgreSQL, Control, Provider, Connector and both product webs over synthetic
  Kubernetes HTTP. It exercises the final profile-aware creation consumer and
  existing stop/start/original-PVC/operator/normal-cleanup flow. The first attempt
  rejected an unsuitable private PostgreSQL socket path before product startup;
  a shorter secure command-scoped TMPDIR fixed that prerequisite without changing
  product code, shared permissions or validation policy.
- Semantic owning-Spec ingestion covers shared identity, profile/catalogue,
  account/Team/Provider/directory boundaries, native allocation witness and final
  creation. New pages own their detailed contracts and neighbors link rather
  than duplicate them. Touched local links, document metadata and all 18 metadata
  validation tests pass. New Team routes and saved-profile public/UI operations
  are intentionally not promoted before phase 3.

Independent integrated review:

- The assigned single read-only reviewer inspected the complete phase-2 diff,
  including new source, against approved revision 1 and the phase contract.
  One material P2 finding: SQLite GLOB/substr can stop at an embedded NUL while
  the handle byte-length guard still admits its suffix.
- Both Team/Agent SQLite CHECKs now explicitly reject NUL. Direct parameterized
  SQL regressions cover empty/short/Unicode/newline and NUL suffixes for each
  constraint on both engines, independent of Store parsing. Root reran all 28
  persistence library tests, both current schema tests and both Team operation
  suites successfully. The same reviewer confirmed the focused correction
  resolves the sole finding, with no remaining concrete correction issue.

Post-PR CI checkpoint:

- PR #72 at `e331565` passed native x86_64/aarch64 web, binary and package builds,
  Rust and validation. Codex completed without findings. Browser CI is separate.
- The required real-Docker job reached registration then rejected the test's
  Create RPC with INVALID_ARGUMENT before allocation. Its literal final handle
  `native-integration` has 18 characters, violating the already accepted 3–16
  contract. The fixture now submits `native-agent`; product validation is not
  relaxed. This correction is delivered on this phase branch and its new SHA
  must pass the actual native job. No local Engine pass is fabricated.

Remaining delivery:

1. Push the fixture correction with normal hooks and monitor the new-SHA matrix.
2. Resolve any current CI/automatic review finding before final delivery.

No phase-2 release, merge, deployed preparation, native effect or rollout has
been executed. Design delta: None.
