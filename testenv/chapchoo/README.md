# Chapchoo test substrate

`@chapchoo/testenv` provides reusable local assembly/process/prerequisite support
through explicit built JavaScript subpath exports. It does not depend on Playwright or app
implementations and does not own product assertions or an E2E command.

- `assembly`: isolated PostgreSQL, backend and web lifecycle, process readiness,
  secondary user fixture seed through public generated RPC, exact
  owned cleanup.
- `processes`: bounded commands, event-driven readiness and owned process groups.
- `config`: explicit child environment allowlist and installed-package prerequisite.
- `seed`: account prerequisites through generated public domain RPC.
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

Computer acceptance uses the explicit Client/Admin/Control union and independent
web targets. `startComputerProvider()` adds an actual scoped Computer Provider
and a private loopback synthetic Kubernetes HTTP service. Standalone Pod
simulation launches the actual execution-bound Computer Connector, retains
original UID identities and emits terminal deletion observations only after that
owned Connector exits. It creates no cluster, PVC filesystem or CNI; those remain
separate live gates. Setup uses ordinary CLI/RPC, never SQL. Compiler cache is
private to each fixture; secure temporary ancestors remain a host prerequisite.

`Assembly.create(true)` instead requires explicit real-Docker fixture inputs:
`CHAPCHOO_E2E_DOCKER_AUTHORIZED=disposable`,
`CHAPCHOO_E2E_DOCKER_SOCKET` (absolute selected Unix socket), and
`CHAPCHOO_E2E_DOCKER_IMAGE` (current built Linux image). This is authorization
for disposable test resources, never an inference from an available user socket.
The fixture uses public `dockerode` operations; optional native ssh2/cpu-features
install scripts are disabled because this fixture uses only Unix sockets.
Ambient Docker host/TLS/path overrides are rejected rather than mixed with
explicit fixture credentials. `startDockerProvider()` registers a unique scoped
Provider and supplies no product DB/root settings to its child.

The real-Docker Assembly exposes its own ephemeral Client/Control listener on
all interfaces so the already-configured bridge can reach it. The default
container route uses that bridge's gateway; an explicit
`CHAPCHOO_E2E_DOCKER_CONTROL_HOST` can name another already-reachable host (for
example an authorized Desktop fixture). It neither publishes guest ports nor
creates networks, host networking or tunnels. Ordinary Assemblies keep loopback
listeners. Use a disposable trusted host/network; this is not production wiring.

Native support observes exact full IDs and the original volume, provides bounded
non-root marker IO via SDK exec, and allocates unrelated labeled canaries.
Product lifecycle assertions stay in the E2E app. Cleanup never prunes or force
removes product resources. Only Control/Provider owns native product cleanup;
unknown effects or failed product cleanup fail teardown and retain diagnostic
responsibility. Canary cleanup is separately scoped and checked.
