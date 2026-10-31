# Account and team E2E application

`@chapchoo/e2e` is an executable Playwright app, not a shared package. It owns
three representative serial Chromium product journeys. Fixture preparation comes from
[`@chapchoo/testenv`](../../../testenv/chapchoo/README.md); product app source is
not imported. No direct DB writes, mutation retries or existing browser state.

## Commands and prerequisites

From the repository root, after frozen workspace installation:

```sh
pnpm --filter @chapchoo/testenv test
pnpm --filter @chapchoo/e2e test:unit
pnpm --filter @chapchoo/e2e lint
pnpm --filter @chapchoo/e2e typecheck
pnpm --filter @chapchoo/e2e exec playwright install chromium
pnpm --filter @chapchoo/e2e test:e2e
```

The runner requires PostgreSQL `initdb`, `pg_ctl`, `postgres`, repository Rust
and Node 24 toolchains, pnpm, and Chromium runtime libraries. Missing prerequisites
fail. Select user-owned browser/SWC caches as needed using the existing environment
variables; cache paths are not hardcoded into the fixture.

`CHAPCHOO_E2E_PACKAGE_ROOT` selects an absolute extracted source-free bundle.
Package mode never falls back to source builds or host Node. The test substrate
prepares a private PostgreSQL database using the official product CLI and launches
both backend/web roles with root mounts. Source mode uses the same public
contracts with `/app` and `/admin` mounts and production web listeners.

## Ownership and lifecycle

- `testenv/chapchoo` owns reusable Assembly/process/configuration/seed support.
  Assembly allocates exact private resources, awaits process readiness and
  provides public fixture methods; it does not import Playwright or app modules.
- This app owns the worker-scoped Playwright adapter, canonical-redirect product
  assertions, browser scenarios, safe reporter and private runner output.
- Secondary registered-user setup uses generated public Auth/Admin/General RPC
  after Admin bootstrap, never SQL. Required fixture credentials stay in memory.
- Cleanup reaps only tracked process groups and stops the exact private
  PostgreSQL directory. Cleanup failure fails the run and retains private data.
  Finite startup/shutdown budgets and no-replay behavior are unchanged.

## Retained critical journeys

1. Administrator setup and issued signup reach the member's personal context;
   inspect session-cookie attributes without exposing credential values.
2. Team creation, invitation acceptance, ownership transfer and former-owner
   leave preserve membership/authority boundaries.
3. Password reset does not authenticate its recipient, revokes the old context,
   allows the new password, and retains ordinary-user exclusion from Admin while
   General access remains available.

Web-core owns the focused real-browser logout boundary in
`test/logout.browser.spec.mjs`. This E2E app owns the three assembled product
journeys, not browser cookie semantics or dependency component tests.

## Detail coverage moved to owning components

The following checks execute against real app components rendered by their
built static Storybook, without a product backend or testenv assembly:

| Former full-product assertion                             | Executable lower-layer owner                                                             |
| --------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Exact password-reset completion copy                      | `web/src/features/accounts/components/Accounts.component.spec.ts`, reset completion case |
| Exact Admin permission-denied copy                        | `admin-web/src/features/admin-auth/components/AdminAuth.component.spec.ts`               |
| 375 × 812 owner workspace no horizontal document overflow | `web/src/features/accounts/components/Accounts.component.spec.ts`, owner viewport case   |

The product E2E still awaits status/error visibility and checks actual reset,
session and Admin authorization consequences. These component checks were run
successfully before removing exact-copy/viewport assertions from product E2E.
Static story definitions alone are not the replacement evidence.

Build app Storybooks, install Chromium once, then run:

```sh
pnpm --filter @chapchoo/web --filter @chapchoo/admin-web build-storybook
pnpm --filter @chapchoo/web test:components
pnpm --filter @chapchoo/admin-web test:components
```

## Evidence privacy

Intentional screenshots capture credential-free desktop inventory/personal/team
surfaces under ignored `artifacts/`. They refuse proof-bearing URLs, password
fields and visible issued links, wait for loaded product fonts and record safe
render metadata. They are not an exhaustive UI QA matrix.

The reporter drops raw test/child output, stack traces and attachments. Runner
artifacts remain under a verified owned 0700 parent, with bounded attachment
cleanup and absence verification. It reports sanitized first-line failures;
website exceptions remain failures. No automatic traces/videos/failure screenshots
or mutation retries are enabled. Substrate safety tests cover secret shapes,
partial log lines and diagnostic bounds; the E2E app's unit test checks private
runner output ownership.
