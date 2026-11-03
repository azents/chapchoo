# Native integration tests

This private Node test application owns the real Docker backend integration
journey. It has no browser or Playwright dependency and imports no product app
source. Existing browser E2E remains responsible for the owner/operator UI.

The scenario uses generated public account/Computer RPC against actual isolated
backend/Control, the native Docker Provider and the execution's real Connector.
It checks non-root saved data, distinct execution replacement, original volume
and no-copy, normal deletion and unrelated canaries. `@chapchoo/testenv` supplies
only reusable prerequisite/process/native SDK support, not scenarios or assertions.

```sh
pnpm --filter @chapchoo/testenv... build
pnpm --filter @chapchoo/integration test:unit
pnpm --filter @chapchoo/integration test:integration
```

A real run requires PostgreSQL tools, an explicitly authorized disposable Engine
and a current built Computer image. Set
`CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED=disposable`, an absolute existing
`CHAPCHOO_INTEGRATION_DOCKER_SOCKET`, and `CHAPCHOO_INTEGRATION_DOCKER_IMAGE`.
`CHAPCHOO_INTEGRATION_BINARY` selects the absolute existing current-run executable;
an invalid explicit input fails instead of falling back to a source build.
Without it, local source mode requires the Rust toolchain. Native mode starts no
web servers, Node child or browser. No production or arbitrary developer daemon
is implicitly authorized. Missing prerequisites fail, never skip.

The existing bridge gateway must reach the ephemeral fixture Control listener;
optional `CHAPCHOO_INTEGRATION_DOCKER_CONTROL_HOST` selects an already-reachable
host. Keep ambient Docker transport variables unset. No tunnel, host networking,
guest port publication, new network or guest Docker credential is introduced.

Read-only polling admits expected replacement gaps but never treats absence as
a running successor. Lifecycle writes run once through public RPC, with no
client mutation retry. Startup races the Provider's owned completion so an
exited Provider fails promptly with sanitized diagnostic context.

Only safe typed native evidence is printed after the journey and owned cleanup
succeed. Raw SDK/guest/configuration/bootstrap output remains withheld. Product
cleanup belongs to Control/Provider, not a forced fixture teardown; unresolved
resources fail cleanup rather than being pruned or hidden. Mac build/distribution
and real-Mac validation are outside this feature's scope.
