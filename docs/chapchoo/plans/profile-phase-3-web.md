---
title: "Agent Profile Phase 3 — Scoped Public Contracts and Web"
created: 2026-11-05
tags: [agents, identity, web, implementation]
---

# Phase Execution Plan

- Phase: 3/4, public contracts and scoped web behavior.
- Branch/base: `feat/profile-ui-261105` -> `feat/profile-storage-261104` at `dd3cb91`.
- PR boundary: one generated public contract and General/Admin web integration
  after [phase 2, PR #72](https://github.com/azents/chapchoo/pull/72); no merge or deployment.
- Inputs: [Requirements](../requirements/profile-261104-agent-identity.md),
  [accepted ADR](../adr/profile-261104-agent-identity.md),
  [approved Design revision 1](../design/profile-261104-agent-identity.md), and
  [stack plan](profile-implementation-plan.md). Phase 2 supplies saved profiles,
  Repository-owned IDs, preserving preparation, exact scoped/profile operations,
  catalogue/creation RPC and the verified minimum creation consumer.
- Approved Design mechanisms: M1, M2, M3, M4, M5, M6, M10, M12. M4/M5 are existing
  phase-2 boundaries to preserve, not permission to change the private codec.
- Authority: REQ-1..10, ADR-D1..8 and Design Authority's exact M1..M12 approval;
  M10's already implemented read-only receipt operation is exposed as a public
  contract in this phase. No new product/compatibility scope is introduced.
- Design delta: None.
- Implementation, integration, decisions and all validation: primary `/root`.
- Exact independent reviewer: `/root/profile-reviewer`, read-only one-way opinion
  after the complete integrated phase diff is stable. Root applies all fixes.

## Deliverables and boundaries

1. Expose authenticated Team handle availability/resolution, exact Team+Agent
   handle resolution, saved profile update and owner/request-only historical
   creation-result reads. Regenerate authoritative clients. Public Agent
   projections carry enough validated local selected-image presentation to
   render saved profiles without an unbounded catalogue scan. Catalogue identity
   remains the persisted reference; logical assets never become arbitrary URLs.
2. Use canonical General paths `/teams/{teamHandle}`, `/teams/{teamHandle}/agents`,
   `/teams/{teamHandle}/new-agent`, and
   `/teams/{teamHandle}/agents/{agentHandle}`. Personal equivalents use `my`.
   Authenticate before resolving personal association or accessible Team handle,
   then resolve the private Agent in that exact Team. A wrong Team or foreign
   private Agent stays not-found.
3. Keep route-selected context, workspace selector, SSR snapshot and client query
   consistent by stable Team ID. Shared General/Admin navigation uses one
   mount-aware General path builder. Remembered preferences and query keys keep
   opaque stable IDs; they never parse IDs or use handles as hidden authority.
   Root remains the authenticated recent/personal resume entry. Existing profile
   and invitation entrypoints retain their approved recent-context behavior.
4. Present the same saved image/name/palette/immutable handle in Agent list and
   detail. Add controlled draft/save/cancel profile editing with bounded catalogue
   choices, current profile retention and ordinary failure/recovery handling.
   Profile changes do not perform native effects or mutate Computer lifecycle.
   Keep existing truthful independent Computer projections and action fences.
   Implement REQ-1's two-column desktop/one-column mobile profile cards,
   coordinated palette surface/foreground/emphasis, collapsed diagnostics and
   separately placed deletion. REQ-3 editing uses a desktop dialog/mobile bottom
   sheet, live draft preview and discard-on-cancel/close/Escape.
5. Carry the already implemented single-screen creation form onto the new Team
   route. Complete Team creation's bounded automatic handle proposal/availability
   and manual-preservation behavior using the phase-1 shared resolver. Final
   handles remain exact and immutable. Do not add hidden input defaults.
   Complete REQ-2's side-by-side desktop preview/inputs, mobile stacking and
   explicit color randomization; unrelated input changes do not randomize choices.
6. Update General/Admin DTO validation, localized English/Korean/Japanese/French
   copy, pure components/stories and focused owning tests. Render primary content
   on the server and hydrate that exact snapshot, including selected profile.

## Fixed interfaces and non-goals

- Repository receives complete opaque IDs or exact handles in their distinct
  operations. Services/browser code must not construct prefixes or decode UUIDs.
- Profile update accepts name/image/palette only. No handle edit, revision field,
  ETag, edit lock or merge mode. Save failure retains the draft; confirmed success
  refreshes related queries through invalidation.
- Availability is not reservation. Full accepted creation payload fingerprints
  and frozen uncertain-request IDs/payloads remain unchanged. Read-only historical
  result lookup cannot allocate; an empty concurrent read is not proof that an
  uncertain prior submission stopped or permission to change its payload.
- Preserve current account/session, membership/private ownership, Admin inventory,
  native witness and lifecycle authority. No new harness/persona/conversation,
  local Mac enrollment, external image/upload, catalogue management or provider
  onboarding product. No broad redesign of unrelated layouts or navigation.
- No new schema/identity transition mechanism, dual-format route mode, legacy
  lookup table, automatic deployed maintenance, merge or deployment.

## Owned paths and workstreams

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Public profile/scoped/receipt contracts | Root | `proto/chapchoo/`, Rust service/RPC projections and focused Store profile reads, generated clients | Phase-2 Store operations | Authenticated typed contracts with private scope preserved | Both-engine Store and actual generated Node/Rust RPC |
| Shared web DTO/path/context | Root | `typescript/packages/web-core/`, General/Admin shared server/navigation adapters | Public contracts | Opaque IDs, canonical paths, exact SSR context and bounded saved image presentation | Core unit tests, both app typecheck/lint/build |
| General profile/list/detail/creation and Team proposals | Root | General `app/` thin routes and owning `features/`, shared profile presentation in UI if genuinely reusable | DTO/path/context | Saved consistent profiles, controlled editing, bounded final handles | Pure stories, owning component checks, desktop/mobile real renders |
| General/Admin assembly and removal evidence | Root | Both app navigation consumers, existing E2E scenarios and owning Wiki subgraph | Integrated web | No old ID/query route authority, truthful current navigation and creation | Minimal existing browser journeys, obsolete-reference search, link/docs checks |

## Integration, removal and validation

Integration order: public service/wire -> generated DTO/path/context -> thin route
admission -> saved profile presentation/editing and Team candidates -> current
General/Admin navigation -> minimum assembled browser and visual evidence.
Do not start phase 4 before opening this phase PR.

Removal obligations: remove obsolete General `/agents?team=ID`,
`/agents/new?team=ID`, `/agents/ID?team=ID` and `/teams/ID` content resolution,
old route builders and query-authoritative context; remove remaining Team handle
edit payload/UI and tests claiming mutability. Preserve non-ID profile/invitation
entrypoints and stable-ID preference/cache keys. No redirects or aliases preserve
obsolete content routes; valid Agent handle `new` needs no reservation because
creation is the separate `new-agent` route.

Absence evidence: inspect route trees and production callers; search obsolete
`?team`, ID-based path interpolation and Team handle updates; verify canonical
browser navigation, wrong-Team/private rejection and no user-content cache leak.
Update references and owning tests rather than retaining old route contracts.

Root final validation: full relevant Rust/SQLite/PostgreSQL and generated actual
RPC; TypeScript format/lint/typecheck, core/app units, owning component checks and
both production builds; existing minimal account/Team and Computer browser flows;
real credential-free desktop/mobile visual captures. Use lower-layer tests for
profile/candidate/error combinations rather than multiplying E2E journeys.
Missing live Docker/cluster prerequisites cannot count as real substrate evidence.

Before review, check both omitted approved outcomes and unauthorized additions.
Record exact primary evidence, removals, remaining release/operator obligations
and current paths. After the single integrated review and corrections, commit
with normal hooks, open the stacked PR, and monitor fresh-SHA CI/automatic review.
Phase 2's CI remains separately monitored while this phase proceeds.

## Context checkpoint

The integrated working diff contains selected-image reads, authenticated scoped
contracts and generated clients, canonical route admission, common General/Admin
paths, saved-profile cards/editing, complete creation preview and bounded Team
candidates. Requirement reconciliation completed the approved two/one-column
cards, coordinated palettes, desktop dialog/mobile sheet, explicit random color,
collapsed diagnostics and separated deletion. No material Design delta arose.

Root validation passed the full authoritative Rust gate and actual generated
Node/Rust RPC 105 checks per database, TypeScript 14/14, core 93, app units
18 General/20 Admin, both production builds/Storybooks and component suites
37 General/10 Admin. Native-resolution component captures were reviewed.
All three account/Team/session assembled browser journeys passed. The Computer
journey exposed a fixture mismatch: a Seoul browser triggered the UTC account's
legitimate display-suggestion toast over an enabled Create button. Safe boolean
diagnostics and a credential-free capture established pointer interception.
The shared fixture now uses UTC and retains display behavior in its account
scenarios; no forced click, hidden product control or longer timeout was added.
Focused Computer acceptance then passed, followed by all four assembled browser
journeys together on the final integrated diff, including profile save and SSR
re-read. Whole format/lint/typecheck gates also passed on that diff.

Phase-2 PR #72 at `71b0198` passed the complete actual Docker lifecycle and
disposable Kubernetes SA/TokenReview/create/Connector/stop/start/delete job,
plus Rust/validation and both-architecture artifacts. The timezone fixture fix
and safe Computer source-location reporting are pushed as `dd3cb91`, whose fresh
CI is now fully successful, including browser and native jobs. Earlier failures
are not rewritten as success. Phase-2's sole integrated review/NUL correction
remains resolved.

The sole integrated phase-3 reviewer found two P2 omissions against REQ-8:
collaborative choices lacked their saved handle, and creation lacked the
prospective Team address. Root added kind-qualified choice presentation and
mount-aware General-address previews on both surfaces. Personal context has no
invented handle; Admin uses its configured General destination. Root reran whole
TypeScript gates, both Next builds and all three Storybooks, shared UI 32,
General 37/Admin 10 component cases and all four assembled browser journeys
successfully. The existing Team-choice acceptance locator now uses the required
name-plus-handle accessible label. The same reviewer confirmed both REQ-8
findings resolved with no remaining concrete material correction issue in that
scope. Normal commit and the stacked PR remain. No merge, deployed migration or
operational rollout has run.
