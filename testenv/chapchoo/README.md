# Chapchoo test substrate

`@chapchoo/testenv` provides reusable local assembly/process/prerequisite support
through explicit built JavaScript subpath exports. It does not depend on Playwright or app
implementations and does not own product assertions or an E2E command.

- `assembly`: isolated PostgreSQL, backend and web lifecycle, process readiness,
  secondary user fixture seed through public generated RPC, exact
  owned cleanup.
- `processes`: bounded commands, event-driven readiness and owned process groups.
- `config`: explicit child environment allowlist and installed-package prerequisite.
- `seed`: secondary registered-user setup through public Auth/Admin/General RPC.
- `safety`: randomized fixture identities and secret-safe diagnostics.

From the repository root:

```sh
pnpm --filter @chapchoo/testenv... build
pnpm --filter @chapchoo/testenv test
pnpm --filter @chapchoo/testenv lint
pnpm --filter @chapchoo/testenv typecheck
```

`build` emits JavaScript and declarations to `dist/`; public exports run in plain
Node without Playwright or a TypeScript loader. Support tests also import every
built public export in Node without starting infrastructure.

The executable E2E app in `typescript/apps/e2e` supplies its Playwright worker
wrapper and browser scenarios. `Assembly.create()` prepares only owned local
resources; `close()` reaps tracked process groups, stops its private PostgreSQL
and removes its private directory only after cleanup succeeds. No direct product
DB writes, global process scans, credential dumps or mutation retries are used.
Source mode builds the product and independent web listeners. An explicit
`CHAPCHOO_E2E_PACKAGE_ROOT` selects an already-extracted source-free bundle and
never falls back to a host build/Node runtime.

Source web mounts are `/app` and `/admin`; package acceptance uses root paths to
prove deployment-neutral artifacts. Required tools and stale/incomplete package
inputs fail honestly, not as skipped tests. Finite readiness/cleanup budgets are
preserved. Support tests remain Docker/server/browser-free; the E2E integrator
owns assembled execution and its evidence.
