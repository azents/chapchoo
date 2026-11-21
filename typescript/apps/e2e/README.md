# Chapchoo E2E application

`@chapchoo/e2e` is an executable Playwright app, not a shared package. It owns
three serial account/team journeys and one Computer owner/operator journey.
Fixture preparation comes from
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
fail. Compiler cache stays inside the private fixture. Its ancestors must be
securely owned or protected by a trusted sticky directory. Set a suitable
`TMPDIR` before starting the runner if the host temporary root does not satisfy
that native compiler requirement; do not relax shared-directory permissions.

Docker-native lifecycle/storage checks belong to the separate browser-free
[`@chapchoo/integration`](../integration/README.md) app and required CI job.
The default browser suite needs no Docker daemon and does not duplicate its UI
flow for every native substrate.

### Native text-conversation acceptance

`pnpm --filter @chapchoo/e2e test:conversation` runs one separate core browser
journey with the actual Docker Provider, Connector, backend and official ACP
adapter. It selects the deterministic app-server executable as an explicit
existing Codex installation before releasing its owned Provider to create the
Computer. It waits for a real selected native session before the first Send and
checks that the first input retains that exact conversation/native target. The
owner then stops the Computer and submits the first ordinary message while it
is stopped; queue-owned startup/restoration delivers it without another Send.
The native work barrier remains on the original saved volume across replacement.
The fixture refuses a duplicate primary New or a synthetic first prompt, and
validates the effective `send` catalog on New and exact Resume. Official SDK
calls start and update the same work, then a qualified automatic follow-up asks
a question without a user receipt. A fresh reply precedes the final result and
summary removal. Ordinary/final native output stays out of Chat while full
records retain the native utterances, captured tool inputs/results and typed
inline answer. Reentry checks server-rendered summary and receipt HTML before
hydration. This proves assembled plumbing, not real Codex/model behavior or
successful real persisted-context restoration. Missing Docker/native
prerequisites fail rather than skip.

Build an architecture-matched binary directory containing `chapchoo`, then build
an isolated image from the repository root:

```sh
pnpm --filter @chapchoo/testenv build:fixtures
docker build --target runtime-base -f images/chapchoo/Dockerfile \
  -t chapchoo-runtime-base:acceptance .
docker buildx build --load \
  --build-context "binary-artifacts=/absolute/path/to/binary-directory" \
  --build-context "fixture-artifacts=$PWD/testenv/chapchoo/dist/fixtures" \
  -f testenv/chapchoo/fixtures/Dockerfile -t chapchoo-conversation:acceptance .
```

Select only a disposable, authorized daemon using the same
`CHAPCHOO_INTEGRATION_DOCKER_{AUTHORIZED,SOCKET,IMAGE,CONTROL_HOST}` inputs as the
browser-free integration app (`AUTHORIZED=disposable`; `CONTROL_HOST` is optional
when its bridge gateway reaches the fixture host). In package mode, use the
extracted package as `binary-artifacts` when building the fixture. In source
mode, set `CHAPCHOO_INTEGRATION_BINARY` to the same absolute executable used
to build the fixture. Clear ambient
Docker transport variables; Unix socket selection remains explicit. The native
security profile is unchanged. No real key or model is required. Where the host
temporary root fails native compiler-cache validation, set
`SWC_NATIVE_BINDING_CACHE` to a private user-owned cache with trusted ancestors;
never disable validation or change shared-directory permissions.

`CHAPCHOO_E2E_PACKAGE_ROOT` selects an absolute extracted source-free bundle.
Both browser suites use it without source builds or host Node. The test
substrate prepares a private PostgreSQL database using the official product
CLI and launches separate Client+Control and Admin API processes and both
webs with root mounts. Docker mode keeps the Control listener reachable from
its owned bridge. Source mode uses the same public contracts with `/app` and
`/admin` mounts and production web listeners.

### Rehearse the Codex cutover

In a disposable Docker fixture, use the checksum-verified previous package as
`CHAPCHOO_E2E_PACKAGE_ROOT` and its matching fixture image as
`CHAPCHOO_INTEGRATION_DOCKER_IMAGE`. Build each image with its release's own
`codex-app-server.cjs` fixture. Set `CHAPCHOO_E2E_NEXT_PACKAGE_ROOT` to the
checksum-verified replacement package and `CHAPCHOO_E2E_NEXT_DOCKER_IMAGE` to
its matching image, then run:

```sh
pnpm --filter @chapchoo/e2e test:conversation --grep 'a stopped Cloud Computer'
```

This variant seeds a turn before the change and uses the product's Stop
operation. Only after confirming the previous container is absent does the
fixture replace the synthetic API key on the same authentication record,
replace its product and provider processes, migrate its private PostgreSQL
database, and send to the original conversation after observing native login
with the replacement key. The test checks
native and conversation identity, old message IDs, the original workspace
marker and a single delivery after the change. It then creates a separate
conversation with a different synthetic key and verifies its first delivery
and empty prior history. The fixture distinguishes a direct `thread/resume`
for the original conversation from a direct `thread/start` for the new one.
This is not a production rollout or a real-model test.

## Ownership and lifecycle

- `testenv/chapchoo` owns reusable Assembly/process/configuration/seed support.
  Assembly allocates exact private resources, awaits process readiness and
  provides public fixture methods; it does not import Playwright or app modules.
- This app owns the worker-scoped Playwright adapter, canonical-redirect product
  assertions, browser scenarios, safe reporter and private runner output.
- Account prerequisites use generated Authentication/Installation/AccountGrants/
  AccountLifecycle RPC, never SQL. Required fixture credentials stay in memory.
- Cleanup reaps only tracked process groups and stops the exact private
  PostgreSQL directory. Cleanup failure fails the run and retains private data.
  Finite startup/shutdown budgets and no-replay behavior are unchanged.

## Retained critical journeys

1. Administrator setup, Admin workspace creation and General/Administration
   navigation use separate API listeners; issued signup reaches the member's personal context;
   inspect the real document/CSS-arrival boundary, server-rendered primary
   content, shared General/Admin login and session-cookie attributes without
   exposing credentials. A committed profile preference survives reload.
2. Team creation, invitation acceptance, ownership transfer and former-owner
   leave preserve membership/authority boundaries.
3. Password reset does not authenticate its recipient, revokes the old context,
   allows the new password, and retains ordinary-user exclusion from Admin while
   General access remains available. Admin logout withdraws the same browser's
   General session without signing out independent browser contexts.
4. Register synthetic authentication in the retained computer-first creation flow,
   create one personal Agent, change only its saved reference through in-place
   registration, inspect SSR saved identity, Computer detail and independent Connector
   connectivity, stop/start with one original PVC identity, inspect read-only
   operator resources, then confirm normal removal and retained cleanup evidence.
   Actual Control, Computer Provider and Computer Connector processes use isolated
   synthetic Kubernetes HTTP beneath the supported SDK. This is not live
   cluster/storage/CNI evidence.

Web-core owns the focused real-browser logout boundary in
`test/logout.browser.spec.mjs`. This E2E app owns the four assembled product
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

These journeys do not automatically capture screenshots. Visual review is a
separate owning-UI validation using real app components, settled product fonts,
explicit viewport/theme/locale/DPR metadata and credential-free states. It does
not add a screenshot/copy matrix to the product journeys.

The reporter drops raw test/child output, stack traces and attachments. Runner
artifacts remain under a verified owned 0700 parent, with bounded attachment
cleanup and absence verification. It reports sanitized first-line failures;
website exceptions remain failures. No automatic traces/videos/failure screenshots
or mutation retries are enabled. Substrate safety tests cover secret shapes,
partial log lines and diagnostic bounds; the E2E app's unit test checks private
runner output ownership.
