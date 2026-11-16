# Native integration tests

## Released-program project instructions

`pnpm --filter @chapchoo/integration test:project-instructions` requires Python 3,
Node and the lockfile-installed development-only ACP 2.6.0/Codex 0.165.0 programs.
Missing prerequisites fail. This browser-free check uses fresh disposable
HOME/native state, a local model catalogue and loopback fixed Responses SSE;
it reads no operator credentials and makes no live inference request.

It verifies title-persisted exact restoration before the first model request,
then AGENTS.md A→B→empty across physical owner replacement and exact same-ID
Resume. Assertions inspect the new instruction item's native role and ordering
before the next fixture message, not mere marker presence in retained history.
Exactly three fixed-response requests and no native action callbacks are allowed.
Every owned process group and listener is explicitly finished.

The existing browser CI gate runs this designated check. The one model-free
conversation browser journey separately verifies owner Save, queued input while
prior work is active, same-context refresh, once-only continuation and retained
original inspection after a later Save. Fixed-response delivery proves neither
model obedience nor successful live credentials.

This private integration application also owns the real Docker backend integration
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

## Model-free native server-tools acceptance

`pnpm --filter @chapchoo/integration test:server-tools` is a separate explicit
acceptance run, not another browser journey. It requires the same disposable
Engine/current binary inputs plus an image built from
`testenv/chapchoo/fixtures/native-tools.Dockerfile`. Supply architecture-matched
`binary-artifacts/chapchoo`, a real compatible `native-artifacts/codex`, and
`fixture-artifacts/codex-app-server.cjs` from
`pnpm --filter @chapchoo/testenv build:fixtures`. The fixture bundle uses the
official MCP SDK; the image performs no package resolution. The real managed
native startup still acquires the supported Node/adapter/Codex closure from its
official registries, so registry/network availability is a prerequisite.

The real-native case uses a synthetic API key and submits no input/model turn.
It proves effective MCP initialization and catalog discovery on New, then
preserves the exact target when native rejects restoration of its unmaterialized
empty session. Codex does not persist that rollout until its first user input;
this test does not claim successful real persisted-session restoration.
Changing the saved supply before restart distinguishes the successor's durable
setup result from the original retained Ready observation without manufacturing
a captured event.

The controlled-native case uses the official adapter plus an explicit deterministic
app-server fixture and actual SDK HTTP initialization/catalog on both New and
exact Resume. A later captured native state distinguishes resumed readiness from
the previous fact. This proves the assembled Connector/harness provision path,
not genuine Codex history recovery or model-side tool use. Both cases preserve
the original source identity, admit no input, and use normal product cleanup.
