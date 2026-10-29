# Assembled account acceptance

This private package owns three representative serial Chromium journeys against a
fresh, fixture-owned PostgreSQL backend and two production Next.js applications.
It imports the official generated RPC package for secondary-user setup; no
production app modules, database writes, shared cluster resources or preexisting
browser state are used. A separate focused browser-boundary test imports the
production web-core cookie functions and uses only its own controlled HTTP server.

## Prerequisites and public commands

Run from `typescript/` after the root-owned frozen workspace installation:

```sh
pnpm --filter @chapchoo/e2e format:check
pnpm --filter @chapchoo/e2e lint
pnpm --filter @chapchoo/e2e typecheck
pnpm --filter @chapchoo/e2e test:unit

# Root/integrator only: install the exact package's Chromium prerequisite.
PLAYWRIGHT_BROWSERS_PATH=/home/agent/.cache/chapchoo-playwright \
  pnpm --filter @chapchoo/e2e exec playwright install chromium

# Executes all three real journeys; implementation workers do not run this.
PLAYWRIGHT_BROWSERS_PATH=/home/agent/.cache/chapchoo-playwright \
SWC_NATIVE_BINDING_CACHE=/home/agent/.cache/chapchoo-swc \
  pnpm --filter @chapchoo/e2e test:e2e
```

The cache paths above are examples of the integrator's selected user-owned cache
locations, not prerequisites hardcoded into the fixture. The runner must provide
`initdb`, `pg_ctl`, PostgreSQL's `postgres` binary, `cargo`/the repository Rust
toolchain, Node 24, pnpm, and Chromium runtime libraries. Missing prerequisites
fail; there is no SQLite fallback or skipped browser execution. Package metadata
pins `@playwright/test` to **1.64.1**.

## Fixture API and lifecycle

- `Assembly.create(): Promise<Assembly>` prepares one private temporary data and
  socket directory, runs `initdb` and `pg_ctl -w`, builds the locked `chapchoo`
  binary, invokes its official `prepare`, and starts foreground `serve --role
both` on distinct reserved loopback ports. Installation key and setup proof
  are fresh per run; only backend children receive DB/key/setup configuration.
- The backend readiness line is authoritative. Next production builds use actual
  fixture public origins with `/app` and `/admin`; the real `next start` listeners
  bind `127.0.0.1` and await their readiness line. No compiled-path constant is
  forged at runtime. Node app children receive only an explicit safe environment
  allowlist plus their own public URL, backend URL, bind host and production mode.
  Both build and listener launch the already installed app-local Next executable
  directly. Preparation does not invoke a package-manager wrapper that could try
  to reinstall workspace modules under the restricted child environment.
- `generalUrl`, `adminUrl`, `administrator`, `member`, `second`, `setupProof`,
  `resetPassword`, `teamName`, and `teamHandle` are private-run in-memory fixture
  values. They must not be printed or attached to reports.
- `seedSecondUser()` logs in through generated Auth RPC after the first Admin UI
  bootstrap, issues a signup grant through generated Admin RPC, and redeems it
  through generated General RPC. This prepares only the registered second user,
  without duplicating the first browser journey or writing SQL.
- `close()` terminates only tracked owned process groups with finite TERM/KILL
  bounds and reaps their children, stops only the exact private PostgreSQL data
  directory, then removes that directory. Partial startup uses the same cleanup.
  Cleanup failure fails the run and retains the private directory rather than
  deleting data beneath a potentially live process. There are no global process
  scans or port-based kills.
- `test` adds the worker-scoped `assembly` fixture with an explicit preparation
  deadline. The three tests share only this newly owned assembly and isolated
  browser contexts. There is one worker and no automatic test or RPC retries.

Finite startup/shutdown timeouts bound failure. PostgreSQL uses its own readiness
wait; backend/Next readiness uses process output events, browser actions await
roles/URLs/state, and shutdown polls only owned process-group existence. There
are no fixed sleeps for test ordering or automatic mutation replays.

**Build side effect:** the fixture deliberately builds the two existing app
outputs for `/app` and `/admin`. Their `.next` outputs retain those baked paths
after the run. The root integrator must restore default-path builds before later
packaging, or deliberately build packaging with the intended public prefixes.

## Three journeys and evidence

1. Admin setup proof/email/password creates the first administrator; Admin UI
   issues an email-bound signup URL; General signup reaches the fixed personal
   context. The test inspects only safe attributes of its own cookie, never the
   credential value. General cookies intentionally use root path `/`; Admin
   cookies use `/admin`, and distinct names provide app isolation.
2. A normal user creates a collaborative team, invites the registered second
   user, and the second user accepts. A representative member permission check
   precedes ownership transfer. The former owner becomes Manager and can leave.
   The new owner remains present. A 375-wide viewport asserts no document-level
   horizontal overflow and captures the safe team surface.
3. Admin issues a reset URL for the normal user. Reset in a fresh context does
   not log in, the old context is rejected, and the new password reaches the
   personal context. The ordinary team owner cannot log into or enter Admin but
   retains General access.

Intentional safe PNGs are written under ignored `artifacts/`: dismissed Admin
inventory and personal context (desktop), team owner (mobile). No automatic
failure screenshots, traces, videos, response attachments, config dumps or raw
child logs are produced. Screenshots refuse active proof/password inputs and
proof-bearing URLs; the integrator must visually inspect them before publication.

Runner output lives beneath an owned `0700` parent at `test-results/`, with the
configured output in its `run/` child. Only public Reporter APIs are used:
`onTestEnd` validates attachment paths and their resolved parent beneath the
configured output, deletes those files and drops inline attachments without
publishing them. `onEnd` removes and verifies absence of the entire owned `run/`
output, and fails the run if privacy cleanup cannot be confirmed. Public `onExit`
rechecks cleanup after any late runner bookkeeping. Automatic DOM
error artifacts may transiently exist inside that private area, but are never
published. Trace/video/automatic screenshots remain off, and safe intentional
PNGs use the separate `artifacts/` directory. Captures wait for the actual Geist
font and record native viewport/DPR, loaded font families, heading typography,
locale, theme and document dimensions in adjacent nonsecret JSON. Product contexts
use explicit light/en-US/Asia-Seoul/DPR-1 conditions; the mobile viewport remains
375×812 with full-page native capture. There are no private Playwright flags or
mutations to the framework's error metadata/status.

The sole reporter drops test/child stdout, stderr, stacks and attachments. It
publishes sanitized first-line failures, masking long base64url credentials,
fixture passwords and proof URL parameters, while preserving failed outcomes.
Uncaught website exceptions are captured as sanitized first-line failures. Owned
child failures include only a bounded recent diagnostic tail: complete lines,
credential/password/proof/Bearer and URL masking, with environment assignments,
command lines and stack frames omitted. Unit tests exercise all known secret
shapes, partial buffer boundaries and output bounds. Service groups remain
tracked even after their original parent exits; completed one-shot commands
(including pg_ctl control operations) are distinct from these long-lived groups.
TERM allows 30 seconds before KILL, respecting backend shutdown grace.

Request Origin, renewal overlap, cookie disposition, unknown errors and response
reordering matrices stay in focused lower-layer tests. The separate
`cookie-order.spec.ts` gates actual HTTP response arrival using the production
cookie functions and a real isolated Chromium context: an old response can
overwrite a newer HttpOnly cookie, a delayed deletion can clear newer state, and
JavaScript cannot read the credential. It does not claim real-backend rotation,
TLS deployment or unlimited recovery; authority and fixed-overlap models remain
lower-layer evidence. The three product journeys are not multiplied by this
boundary test. Root owns actual execution and the independent phase review.

## Implementation validation boundary

Package formatting, strict lint and TypeScript checks can run without launching
PostgreSQL, the backend, Next.js or a browser. Actual assembled execution is a
separate root-owned verification; writing or typechecking these tests is not a
claim that the real journeys have passed.

All environment reads are confined to the defining `src/config/index.ts`
boundary. Child environments explicitly disable Next.js and Turbo telemetry.
