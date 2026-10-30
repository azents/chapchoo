---
title: "Accounts Phase 4: Shared General and Admin Web"
created: 2026-10-30
tags: [accounts, implementation, frontend, authentication]
---

# Phase Execution Plan

- Phase: 4/5, shared product UI and independently runnable general/Admin web.
- Branch/base: `feat/accounts-web` → `feat/accounts-rpc-cli` at `8b8a771` (PR #4).
- PR boundary: replace the scaffold with the approved account/team journeys,
  integrate app-owned tRPC/gRPC adapters and verify cookie/auth/permission behavior.
- Inputs: reviewed backend phases 1–3, generated Node RPC contract, existing
  general-app Mantine theme/providers, approved Design revision 36.
- Approved Design mechanisms: M1, M3, M4, M7, M25, M27, M34, M38, M42;
  backend mechanisms remain dependencies, not new authority.
- Authority references: REQ-1 through REQ-11 and REQ-13 as applicable;
  ADR-D3-D6, D8-D14, D20-D24, D27, D30-D31, D34, D36-D37;
  current credential/storage/RPC Specs.
- Design delta: None.
- Deliverables: shared theme and presentation primitives; general login/code,
  signup/reset, personal/team context and authorized team/invitation flows;
  Admin setup/login/signup-grant/reset-grant flows; separate app navigation,
  current-authority admission and backend endpoints; generated RPC calls from
  Node; writable HttpOnly cookie renewal and read-only server-render auth;
  localized states, component stories and representative browser acceptance.
- Non-goals: agent execution, profile/account deletion, broad server management,
  Refine, external identity providers, token registries, replay/outbox services,
  installable distribution bundles, native apps and infrastructure writes.

## Interfaces and Execution Contract

- Browser → owning Next.js Node app uses tRPC and TanStack Query. Each app owns
  its router, configuration, navigation and permission-sensitive operations.
  Reusable server primitives may live in a package; app-specific adapters remain
  in each app. Apps never import another app.
- Node calls the generated RPC package with fresh per-call metadata. Shared UI
  receives typed presentation props/callbacks and never imports RPC, cookies,
  environment, application navigation or DB/root/provider credentials.
- Both apps retain the existing theme/tokens/provider behavior from one package.
  General uses personal/team working surfaces; Admin keeps an explicit Admin
  context and signup/reset grant inventories. Reuse controls and form feedback,
  not an invented generic administration framework.
- Each app has explicit public origin, listener and backend endpoint configuration.
  Loopback HTTP requires localhost/parsed loopback host and loopback bind;
  remote public origins require HTTPS. Origin trust is configured, not inferred
  from forwarded headers. TLS backend calls validate certificate/hostname with
  supported grpc-js credentials; no silent plaintext fallback.
- Writable auth/mutation and auth-renewing query boundaries admit POST only and
  reject missing/mismatched Origin before forwarding work. The tRPC HTTP link
  overrides queries to POST. Public/read-only routes do not implicitly rotate.
- Protected writable requests renew once in request-local context, then forward
  business work with that result. Successful renewal survives later ordinary
  business failure. Invalid session clears the app cookie and stops dispatch;
  post-renewal invalidation overrides the candidate cookie. Infrastructure auth
  failure retains a potentially valid cookie and stops business dispatch.
- General/Admin cookies have distinct names and selected paths; ports are not
  isolation boundaries. HTTPS uses Secure; permitted local HTTP uses non-prefixed
  non-Secure HttpOnly/SameSite=Lax cookies. Browser persistence is 30 days capped
  by actual backend expiry. Session tokens never appear in React props/tRPC
  results/query caches/storage/log fields. Server Components only authenticate.
- Stable machine reasons are validated against the generated canonical enum.
  Unknown/missing metadata remains generic failure. Potentially executed
  mutations and auth transitions are not automatically replayed.
- UI states use typed loading/error/loaded ADTs in containers, pure props-based
  components and thin route entrypoints. Forms retain pending/error state and
  prevent duplicate submission. Destructive actions have explicit confirmation;
  role/owner restrictions remain enforced by the backend.

## Ownership

| Workstream | Owner | Owned paths | Depends on | Output | Validation |
| --- | --- | --- | --- | --- | --- |
| Shared presentation | /root/accounts-protocol-generation | typescript/packages/ui | existing theme and approved screens | theme/providers, shared controls/layout, pure presentation states | package typecheck/lint, static stories/state tests |
| Web transport/auth | /root/accounts-rpc-adapter | typescript/packages/web-core; both apps src/config, src/shared/server, src/shared/rpc, app/api/trpc | generated Node RPC and cookie contract | reusable primitives plus app-owned routers/client wiring | origin/cookie/error/renewal/per-call isolation tests |
| General product UI | /root/accounts-team-storage | typescript/apps/web excluding transport/auth-owned paths | shared UI and general app router | login/signup/reset, personal/team/membership/invitation UI and locales/stories | app quality/build, static component states |
| Admin/integration | /root | typescript/apps/admin-web excluding transport/auth-owned paths; root manifests/lock/tooling/docs/testenv | shared UI, Admin router, backend | Admin setup/login/grant UI, complete assembled verification | both apps/build/storybooks, minimal real-browser journeys |

- After General UI freeze, `/root/accounts-team-storage` owns only the new
  `typescript/apps/e2e/` acceptance-test implementation. Root owns
  installing browser prerequisites and executing all integrated browser evidence.
- After shared presentation freeze, `/root/accounts-protocol-generation` owns
  only Admin `messages/{ko-KR,ja-JP,fr-FR}.json` and
  `src/shared/lib/localization.test.mts` for bounded localization completion.
  Root keeps the English source, remaining Admin paths and final integration.
- Package owners create their own manifests but root alone resolves the shared
  pnpm lock and coordinates public package exports/typed app interfaces. Workers
  do not stage/commit or change another owner's paths. The previous backend
  reviewer is not an implementation owner.
- Integration order: shared manifests/interfaces → theme/primitives and typed
  transport context → app containers/screens → origin/cookie/error integration
  → representative browsers and responsive capture → Specs/quality → review/PR.
- Independent review: `/root/accounts-reviewer`, the same exact read-only
  reviewer. Root requests review only after the complete integrated diff and
  root-owned validation are stable, then applies corrections directly.

## Removal and Validation

- Removal obligations: replace the Hello World product route/stories and move
  app-local theme/tokens/provider definitions to the shared source. Remove
  superseded copies/imports, without changing the selected theme behavior.
- Absence verification: no Hello World route/story remains authoritative; both
  apps import the same theme and controls. Verify no app-to-app dependency,
  browser RPC/server credential import, JS session-token persistence, copied
  product schema/key access, implicit renewal GET or ambiguous-mutation retry.
- Final root validation: frozen pnpm install; all frontend format/lint/typecheck,
  tests and builds; shared/app storybooks; generated RPC check and backend gates;
  cookie/origin/renewal/permission focused integration; three minimal browser
  journeys (Admin bootstrap/signup/personal, collaborative invitation/roles/
  transfer-leave, Admin reset/relogin/revocation); desktop/mobile visual review;
  current Specs/docs and full hooks; exact-head CI and review follow the phase PR.
- Controlled lower-level tests cover response loss/reordering, within/outside
  overlap, successful renewal followed by business failure, revocation after
  renewal and auth infrastructure failure. Do not inflate those combinations
  into the minimal browser suite or claim framework headers prove network order.
- Scope-drift check: approved screens and cookie mechanics only. Any new product
  contract or material mechanism returns to authority; local identifiers and
  equivalent implementation layout stay implementation-owned.
- Context checkpoint: PR #4 at `8b8a771` is open, independently reviewed with no
  issues, root/in-reviewer Rust 42+3 and Node SQLite/PG 71 checks each passed.
  Its exact-head CI passed (rust 4m7s, validate 5m45s). Backend data/protocol
  state is stable; packaged web children and deployment acceptance remain open.

## Root Integration Checkpoint

- Implemented shared original theme/presentation, both independent account apps,
  app-owned nonbatched tRPC adapters, request-local generated gRPC auth and cookie
  lifecycle, four locales, static stories, typed failure states and SSR admission.
- Integrated browser execution found and fixed internal/prefixed tRPC endpoint
  confusion, prefix-doubled server redirects, BigInt inventory query-key hashing,
  reset success hidden by consumed-proof cache clearing, and test-only required
  label/route-announcer selection errors. No product/Design contract changed.
- Root-owned Chromium execution passed all three planned product journeys using
  fresh private PostgreSQL and real `/app`/`/admin` production builds. A separate
  focused cookie-boundary test passed actual delayed overwrite/deletion and
  HttpOnly non-exposure; backend overlap/revocation remains lower-layer evidence.
  Fixture cleanup completed; no proof/password screenshot was retained/published.
- Native product PNGs inspected: Admin inventory and personal space 1280×900,
  team-owner mobile375×1403 at viewport375×812. Product GeistSans loaded; heading
  26px/600/35.1px; light/en-US/Asia-Seoul/DPR1; no mobile document overflow.
- Root frozen install, format, lint, typecheck, UI/core/general/Admin/E2E-safety
  tests, both default production builds and both Storybooks passed. Root Rust
  42 tests plus3 compile-fail and generated/live Node RPC71 checks on each of
  SQLite/PostgreSQL passed. Docs catalog18 tests and frontmatter passed.
- Full hooks first normalized convention indexes, new CLAUDE import and JSON
  formatting; package format scope leaves tsconfig JSON to the existing JSON
  hook to avoid conflicting formatters. Final clean hooked validation and the
  exact same read-only reviewer `/root/accounts-reviewer` precede commit/PR.
- CI includes both app/package tests and both Storybooks, plus a separate owned
  assembled-browser job. Typecheck/E2E build package dependencies on a clean
  checkout. No merged/deployed/whole-feature-complete assertion is made.
- The same independent reviewer completed the entire frozen tree with one
  Warning: Admin explicit refresh omitted the identity query and could pin a
  transient client identity failure. Root corrected the shared invalidation scope,
  added two focused regressions (Admin now12 tests), rebuilt Admin/Storybook and
  verified the real production Next/Chromium recovery with an owned mock gRPC:
  identity2/list1 before → identity3/list2 after; the error/refresh screen cleared.
  No security/authority boundary changed; the review policy requires affected
  validation rather than another full review for this correction.
- The first PR #5 remote CI at `5823521` passed Rust but exposed clean-checkout
  tooling prerequisites: typed lint ran before web-core declarations existed;
  the restricted fixture child invoked pnpm and triggered noninteractive module
  reconfiguration before Next build. Root added package-build ordering and emitted
  output restoration to workspace tasks, hashed the public build URL, and used
  the installed Next executable directly for fixture build/start. Cold lint with
  removed library outputs and a second cached-output restore both passed. The
  three product journeys plus focused cookie-order test passed again using direct
  Next builds; both default builds/Storybooks and affected quality gates passed.
  A new exact-head remote CI result is still required before phase progression.
- Design delta: None. No new mechanism or deferred approved phase-4 removal.
  The next phase remains Linux packages/owned children/Docker/Helm, with explicit
  cluster/namespace/exposure/data scope required before live infrastructure writes.
