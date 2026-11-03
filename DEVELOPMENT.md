# Development

## Rust backend

The account/team implementation is being delivered in phases. The workspace now
contains domain/crypto, both-engine persistence, application services, generated
RPC and a foreground backend/operator executable. Install the pinned toolchain
and run its integrated checks:

```sh
rustup toolchain install 1.99.1 --profile minimal --component clippy,rustfmt
bash scripts/check-rust.sh
```

`rust/rust-toolchain.toml` is the toolchain authority. The workspace lockfile is
committed; checks deny unsafe code, unhandled lint warnings, production unwraps/
expects and secret-revealing diagnostic formatting. Domain and crypto crates
have explicit dependency boundaries and do not depend on persistence or RPC.
Password hashing uses bounded blocking admission in the application service.
Cancellation retains its permit until the started CPU worker actually completes.

Semantic Rust guidance is indexed by `.claude/rules/rust-conventions.md`.
Mechanical policy lives in the workspace lint tables and `rust/clippy.toml`;
Rustfmt follows the pinned toolchain. The canonical gate also checks rustdoc,
member lint inheritance and compile-only negative/positive policy probes.
Those probes use cached dependencies offline after workspace Clippy and never
execute the intentionally invalid fixture program. Research and the complete
Python/TypeScript disposition are in
`docs/chapchoo/design/rust-conventions-research-2026-11-04.md`.

These primitives do not authorize a server-secret-key change or persist data.
No actual installation secret belongs in source control, test fixtures or logs.

The persistence phase adds real SQLite/PostgreSQL integration tests. The Rust
quality script starts and removes a private Unix-socket PostgreSQL instance when
`initdb` and `pg_ctl` are available. Otherwise explicitly supply
`CHAPCHOO_TEST_POSTGRES_URL` for a disposable test server with schema-creation
permission. Never point this setting at a production database. Tests use isolated
schemas and do not count missing PostgreSQL prerequisites as skipped successes.
CI supplies a fresh PostgreSQL service to each validation job.

### Local backend and operator commands

Supply one database through `CHAPCHOO_SQLITE_PATH` or
`CHAPCHOO_POSTGRES_URL`, plus `CHAPCHOO_SECRET_KEY` containing the original
32-byte root encoded as padded URL-safe base64. Obtain/store the root securely
outside the checkout. There is no automatic generation or replacement.

```sh
cd rust
cargo run --locked -p chapchoo -- prepare
cargo run --locked -p chapchoo -- serve --groups client,admin,control
# Independent choices: serve --groups client; serve --groups admin; serve --groups control
cargo run --locked -p chapchoo -- grant-admin --email existing@example.invalid
```

Preparation explicitly migrates and binds the installation. Normal service startup
only checks readiness and refuses missing/wrong schema or key.
Preparation also initializes display defaults from its machine's IANA time zone
and supported locale (English/en-US for an unsupported locale), and backfills
missing existing-user display values atomically. Set the preparation process's
ordinary environment correctly before its first successful run; subsequent runs
preserve initialized defaults and user choices. Missing existing-user time zones
come from that migration machine, not an administrator-edited default.
Timezone name validation uses bundled data rather than requiring host tzdata.
The selected service union uses `127.0.0.1:50051` by default. Override it with
`--listen` or `CHAPCHOO_LISTEN`. This is plaintext HTTP/2; do not publish it directly.
Deployments require a trusted internal route and an operator-managed TLS gRPC
edge for external callers.

Control-only local compositions own a same-handler memory peer server. Distributed
Control additionally requires paired `--peer-listen`/`--peer-advertise` (or
`CHAPCHOO_PEER_LISTEN`/`CHAPCHOO_PEER_ADVERTISE`) on a distinct restricted-network
listener. PeerRelay is never registered on the public component/client listener.
The operator must enforce network isolation; no peer application credential or
mandatory peer mTLS is introduced.

To authorize system-owned Computer Provider metadata/key management, configure
the complete `CHAPCHOO_SYSTEM_OWNER_ID`, comma-separated
`CHAPCHOO_SYSTEM_OWNER_PROVIDER_IDS`, and `CHAPCHOO_SYSTEM_OWNER_TOKEN` set in
Control. Generate the bearer as 32 random bytes encoded as unpadded URL-safe
base64 (for example `python3 -c 'import secrets; print(secrets.token_urlsafe(32))'`).
Supply it through your runtime secret manager/environment, never a CLI argument
or tracked file. Provision the same bearer only to the scoped Computer Provider;
it grants owner-level key management for those IDs. Preserve it until explicit
operator rotation/revocation, and never publish its value in logs or reports.

Computer components run independently from the backend:

- `chapchoo computer-provider` requires `CHAPCHOO_CONTROL_ENDPOINT`,
  `CHAPCHOO_PROVIDER_ID`, `CHAPCHOO_PROVIDER_NAME`,
  `CHAPCHOO_PROVIDER_OWNER_TOKEN`, `CHAPCHOO_KUBERNETES_NAMESPACE`,
  `CHAPCHOO_COMPUTER_IMAGE`, and `CHAPCHOO_COMPUTER_SERVICE_ACCOUNT`.
  Select either `--in-cluster` or `--kubeconfig <explicit-file>` (also
  `CHAPCHOO_KUBECONFIG`); there is no implicit current-cluster selection.
  The Provider owner token is the scoped bearer provisioned above, not the
  installation encryption root. Kubernetes credentials must have only the
  designated namespace's managed Pod/PVC/Secret permissions, with Pod watch for
  termination evidence. Never provision production credentials for tests.
- Runtime Computer policy uses `CHAPCHOO_COMPUTER_STORAGE_CLASS`,
  `CHAPCHOO_COMPUTER_STORAGE_REQUEST` (10Gi),
  `CHAPCHOO_COMPUTER_SAVED_MOUNT` (/workspace),
  `CHAPCHOO_COMPUTER_IMAGE_PULL_POLICY` (IfNotPresent), and CPU/memory
  `CHAPCHOO_COMPUTER_CPU_REQUEST` (100m), `CHAPCHOO_COMPUTER_MEMORY_REQUEST`
  (128Mi), `CHAPCHOO_COMPUTER_CPU_LIMIT` (1),
  `CHAPCHOO_COMPUTER_MEMORY_LIMIT` (1Gi). StorageClass omission deliberately
  uses the operator's default class; no class is provisioned by the process.
- `chapchoo docker-provider` requires `CHAPCHOO_CONTROL_ENDPOINT`,
  `CHAPCHOO_PROVIDER_ID`, `CHAPCHOO_PROVIDER_NAME`,
  `CHAPCHOO_PROVIDER_OWNER_TOKEN`, and `CHAPCHOO_COMPUTER_IMAGE`. It does not
  load backend DB/root configuration. Default Docker selection uses the
  executing account's CLI environment/context once; it never changes context or
  starts Docker Desktop. Linux workloads require the selected daemon's supported
  architecture, seccomp and CPU/memory enforcement.
  An explicit `--docker-host` / `CHAPCHOO_DOCKER_HOST` selects a complete override:
  optional `CHAPCHOO_DOCKER_TLS`, `CHAPCHOO_DOCKER_TLS_SKIP_VERIFY`,
  `CHAPCHOO_DOCKER_CA_FILE`, paired `CHAPCHOO_DOCKER_CERT_FILE`/
  `CHAPCHOO_DOCKER_KEY_FILE`, and `CHAPCHOO_DOCKER_API_VERSION` do not borrow
  ambient context credentials. Without a pinned API version, startup negotiates
  once. TLS material is captured; SSH keeps one authenticated OpenSSH master
  and requires local `ssh` plus remote `docker system dial-stdio`. Lost SSH
  source cannot reauthenticate under changed account settings; restart the
  Provider explicitly to select new credentials/target.
  TCP configurations requiring `HTTP_PROXY`/`HTTPS_PROXY` are currently rejected
  before direct connection, not silently bypassed; Docker-style `NO_PROXY`
  exclusions and loopback are honored. Unix/SSH do not use these HTTP proxy inputs.
  No proxy credentials are printed.
- Docker workload policy uses `CHAPCHOO_COMPUTER_SAVED_MOUNT` (/workspace),
  `CHAPCHOO_COMPUTER_IMAGE_PULL_POLICY` (IfNotPresent; public images only),
  `CHAPCHOO_DOCKER_NANO_CPUS` (1000000000),
  `CHAPCHOO_DOCKER_MEMORY_BYTES` (1073741824) and optional
  `CHAPCHOO_DOCKER_NETWORK` (existing outbound network; default bridge).
  `CHAPCHOO_CONTAINER_CONTROL_ENDPOINT` overrides only the Container's outbound
  Control target when the Provider's endpoint is not container-reachable.
  The operator owns trusted listeners, network and firewall; the Provider creates
  no tunnel, published port, host-network workaround or network. Its image must
  supply the non-root saved-directory ownership. Existing data is never chowned,
  refilled or replaced. Source fixtures are not real Engine/Mac acceptance;
  see [the native contract](docs/chapchoo/spec/domain/computer-docker.md).
- `chapchoo computer-connector` requires only its explicit Control endpoint,
  `CHAPCHOO_CONNECTOR_SESSION_ID`, and `CHAPCHOO_CONNECTOR_INITIAL_KEY`,
  supplied by the execution's immutable Kubernetes Secret or Docker container
  configuration. It receives neither
  Provider/owner credentials nor Kubernetes service-account API tokens. The
  Provider issues this initial key under an exact execution; do not manually
  reuse another execution's key or copy its Secret. Retire/revoke the session
  when its execution ends. Raw keys are environment/secret-manager inputs only.

Control runs owned lease and 15-second reconciliation workers. The current
Computer Connector owner reconciles through the current Computer Provider owner;
the Provider owner handles pre-bootstrap/absent-Connector work. Supported-client
fixtures exercise automatic create/stop/start/restart/restoration/delete through
real local streams and peer delivery. These tests are not browser or live
Kubernetes release-gate evidence. Do not deploy or provision a cluster merely to
run the offline suites.

The Docker Provider respects the selected CLI context/socket and keeps host
behavior portable so it can naturally be built on Mac later. This effort adds
no Mac-only build workflow, archive or distribution, and real-Mac validation is
not its completion gate. It never installs or launches Docker Desktop. A future
claim of a tested Desktop or remote SSH/TCP/TLS combination still needs explicit
real-target evidence; Linux socket results do not imply those results.

The browser-free Docker integration job requires a disposable authorized Engine,
not merely a CLI. It uses the same run's native binary artifact for actual
backend/Control/Provider and the Computer image, without web servers or Chromium.
The existing four browser E2E journeys remain unchanged. Run
`pnpm --filter @chapchoo/integration test:integration` with
`CHAPCHOO_INTEGRATION_DOCKER_AUTHORIZED=disposable`, an absolute
`CHAPCHOO_INTEGRATION_DOCKER_SOCKET`, and `CHAPCHOO_INTEGRATION_DOCKER_IMAGE`
naming the current built Linux image. `CHAPCHOO_INTEGRATION_BINARY` selects an
explicit built executable without source-build fallback. Keep ambient Docker
transport variables unset. The default existing bridge gateway must reach the
fixture Control listener; optional `CHAPCHOO_INTEGRATION_DOCKER_CONTROL_HOST`
selects an already-reachable host. No networks, guest ports or tunnels are
created by the Provider. Missing required inputs fail rather than skip. See
[the native owner](typescript/apps/integration/README.md) for evidence boundaries.

The Docker supported-SDK/real-Control fixture checks allocation ACK order,
normal stop/start/restart/delete, original-volume retention, source replacement
before ACK and unknown Start across stream reconnect. A separate explicit local
OpenSSH fixture uses disposable keys and an unprivileged inetd-mode server:
`cargo test --locked -p chapchoo-docker --features ssh-fixture` from `rust/`.
It requires `ssh`, `sshd`, `ssh-keygen` and `id` on PATH and fails for missing
prerequisites. It changes no account SSH configuration or system daemon. This
proves mux policy/lifetime, not remote Engine or Mac/Desktop compatibility.

Control policy is runtime-configurable: `CHAPCHOO_CONTROL_VALIDITY_MS` and
`CHAPCHOO_COMPONENT_VALIDITY_MS` default to 60000,
`CHAPCHOO_CONTROL_HEARTBEAT_MS` to 20000,
`CHAPCHOO_COMPONENT_KEY_VALIDITY_MS` to 86400000,
`CHAPCHOO_COMPONENT_KEY_REFRESH_MS` to 43200000,
`CHAPCHOO_COMPONENT_KEY_OVERLAP_MS` to 300000, and
`CHAPCHOO_START_DELAYED_MS` to 300000. Invalid timing/overlap combinations fail
before admission. Schema preparation includes the durable Prepare admission
fence; it remains distinct from actual first-start dispatch timing.

Set `CHAPCHOO_SETUP_PROOF` to a separately secured 256-bit unpadded URL-safe
bearer before the first administrator flow. Consumed setup never reopens from
configuration. `grant-admin` acts directly on an existing account using local
database/key authority; it is not a remote RPC.

`CHAPCHOO_GENERAL_WEB_URL` controls issued link destinations (default
`http://localhost:3000`). HTTP accepts loopback hosts only; remote URLs require
HTTPS. The independent Node web applications below serve the product screens.
Optional `CHAPCHOO_SESSION_VALIDITY_DAYS` defaults to 180; optional
`CHAPCHOO_SESSION_ABSOLUTE_DAYS` imposes a positive absolute lifetime cap.
Rotation/previous-token overlap default to 10/5 minutes.
`CHAPCHOO_PASSWORD_WORKERS` bounds active bcrypt jobs (default 4, 1–256).

Email is optional. Configure `CHAPCHOO_SMTP_SENDER` and `CHAPCHOO_SMTP_HOST`.
`CHAPCHOO_SMTP_TLS` defaults to `starttls` (required upgrade, port 587); `implicit`
uses TLS from connection start (port 465). `CHAPCHOO_SMTP_PORT` can explicitly
select another nonzero port. Both modes verify the server certificate and never
downgrade. Optional `CHAPCHOO_SMTP_USERNAME` and `CHAPCHOO_SMTP_PASSWORD` must be
provided together. For an unauthenticated local relay only, explicit
`loopback-plaintext` permits a numeric loopback IP (default port 25), never a
remote host or authentication. SMTP credentials are backend-only runtime inputs.
The former SES variables and AWS credential chain are removed; configure your
SMTP relay instead, including an SES SMTP endpoint if desired. Mail-only flows
fail when SMTP is absent; manual signup/reset grants remain usable.
Email is submitted after commit, without automatic submission retries; failure
does not compensate state and provider acceptance is not mailbox delivery.

SIGINT/SIGTERM stops admission and drains all selected backend tasks. Combined
groups form one failure unit. Startup has a 60-second budget; shutdown has
20 seconds for graceful drain and a further 5 seconds for forced task cleanup.
Liveness is the gRPC health service name `chapchoo.liveness`; readiness checks
the selected business service or empty service and rereads schema/key readiness.
An unhealthy database can leave liveness serving while readiness is not serving.

### Generated RPC clients

`proto/chapchoo/*.proto` are the Rust/Node contract sources. Rust generation
uses vendored protoc at build time; committed Node output is reproduced with:

```sh
bash scripts/generate-rpc-clients.sh
bash scripts/check-rpc.sh
```

The RPC gate verifies generation, strict client compilation and live Node-to-Rust calls
against both disposable database engines. It requires pnpm/Node and `psql`,
plus the PostgreSQL prerequisite described above. Missing prerequisites fail.
The Node package is server-side only: browsers do not speak gRPC or hold backend
credentials in JavaScript storage.

The account wire services are domain-scoped: Authentication and
PersonalPreferences are common to Client/Admin; Client exposes AccountLifecycle,
TeamManagement and InvitationManagement, while Admin exposes Installation,
AccountGrants, DisplayDefaults and the narrow current-administrator
AdminWorkspaces Team listing/creation contract. Generated callers must use those contracts;
the former broad Auth/General/Admin RPC service names are replaced, not aliases.
The service union deduplicates shared registrations. Bundled web selections use
independent explicit `CHAPCHOO_GENERAL_BACKEND_URL` and
`CHAPCHOO_ADMIN_BACKEND_URL`, even if both intentionally identify one union.
`serve --groups --web general` selects web-only execution without DB/key access.

## Frontend

The apps use the same frontend foundation as Azents: Next.js App Router,
React, TypeScript, Mantine and the shared UI theme, bundled Pretendard/Geist Mono fonts, cookie/header-based
next-intl localization, pnpm workspaces, Turborepo, ESLint, Prettier, and Storybook.
The initial dependency ranges and pinned Next.js version come from the recorded
upstream snapshot.

```sh
pnpm install --frozen-lockfile
CHAPCHOO_BACKEND_URL=http://127.0.0.1:50051 pnpm run dev --filter=@chapchoo/web
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm --filter @chapchoo/web test
pnpm --filter @chapchoo/admin-web test
pnpm --filter @chapchoo/ui test
pnpm --filter @chapchoo/web-core test
pnpm --filter @chapchoo/testenv test
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web
pnpm run build-storybook --filter=@chapchoo/admin-web
pnpm --filter @chapchoo/e2e exec playwright install chromium
pnpm --filter @chapchoo/web test:components
pnpm --filter @chapchoo/admin-web test:components
```

The repository root owns the pnpm workspace, lockfile and Turbo task graph.
It includes `typescript/apps/*`, `typescript/packages/*` and `testenv/chapchoo`.
Run the commands above from the repository root.

CI's existing `browser-e2e` job consumes the native build's x86-64 archive and runs
the core journeys once. The reusable native workflow only builds; it has no E2E
or runtime acceptance steps. Local E2E can still assemble source for development.

Workspace lint/typecheck, dev and Storybook commands build their library
prerequisites first. Turbo restores emitted `dist`/Next outputs from its cache;
deployment configuration is read only by running listeners. Development passes only
the explicit public URL, listener and backend client settings to app commands.
This does not pass database/root/bootstrap credentials into a web process.

General uses port 3003 and Storybook 6006; Admin uses 3004 and Storybook 6007:

```sh
pnpm run storybook --filter=@chapchoo/web
pnpm run dev --filter=@chapchoo/admin-web
```

General renders account/password and personal/team journeys. Admin renders
initial administrator setup, administrator login, and signup/reset link management.
Both use `@chapchoo/ui` for the selected shared theme and controls, and
app-owned tRPC adapters backed by `@chapchoo/web-core` and generated Node gRPC.
Supported locales match the
upstream web app: `en-US`, `ko-KR`, `ja-JP`, and `fr-FR`. Locale selection uses the
`locale` cookie, then Accept-Language, then English; no locale URL segments.
There is no account-preference lookup in the UI.
Storybook renders the same theme and translation providers with static fixtures.

### Independent design references

`@chapchoo/design-reference` is a separate React/Vite Storybook on port 6008:

```sh
pnpm run storybook --filter=@chapchoo/design-reference
pnpm run build-storybook --filter=@chapchoo/design-reference
pnpm run check:design-boundaries
```

See its [authoring and promotion guide](typescript/packages/design-reference/README.md).
It holds general cross-platform references, local experiments and non-web
representations, not app functional tests or UI QA. Common UI owns the promoted web theme/assets and is reused by reference. See root [DESIGN.md](DESIGN.md) for visual design context and the
reference package guide for its current catalog.

Each app reads configuration only through `src/config/index.ts`.
`CHAPCHOO_PUBLIC_URL` supplies the exact public origin/path (defaults:
`http://localhost:3003` and `http://localhost:3004`). One deployment-neutral build
can run at root or a different mount without rebuilding. `CHAPCHOO_WEB_HOST` defaults to `127.0.0.1`; local HTTP requires a
loopback public host and listener. Remote public URLs require HTTPS.
`CHAPCHOO_BACKEND_URL` selects the corresponding general/Admin endpoint (defaults:
`http://127.0.0.1:50051` and `http://127.0.0.1:50052`). HTTPS backend calls use
normal CA/hostname verification. Node apps never receive the installation key,
database URL or setup proof through their ordinary configuration.

Browser calls use POST and exact configured Origin admission, one operation per
request, no automatic retry and distinct HttpOnly cookies. SSR authenticates
read-only. See [the account web Spec](docs/chapchoo/spec/domain/account-web.md)
for renewal and cookie policy; do not copy real credentials into commands/logs.

The four core assembled browser journeys (three account/team and one Computer)
and disposable PostgreSQL/process ownership are documented in
[`@chapchoo/e2e`](typescript/apps/e2e/README.md). Agent/Computer primary content is
server-rendered, then observed through visible-view bounded unary reads; see
[Computer web contracts](docs/chapchoo/spec/domain/computer-web.md).
Sentry upload configuration and Azents product pages were not copied.

## Installed Linux and deployment validation

The source-free Linux bundle contains the executable, matching official Node,
both standalone applications and static assets. It needs no host Node/pnpm/Turbo.
See [the delivery Spec](docs/chapchoo/spec/domain/account-delivery.md),
[independent image commands](images/chapchoo/README.md), and
[chart prerequisites](infra/charts/chapchoo/README.md). Assembly consumes explicit
built inputs and rejects overwrites; it does not build or download dependencies.

```sh
python3 -m unittest scripts.tests.test_package_linux
uv run --locked --project .codex python scripts/check-chart.py
python3 scripts/test-installed-linux.py --root /absolute/extracted/chapchoo-linux-x86_64 --architecture x86_64
python3 scripts/test-container-images.py --backend local-backend:tag --general local-general:tag --admin local-admin:tag
```

Native x86-64/ARM64 CI uses the fixed Bookworm release builder, architecture-matched
standalone outputs/Node, bundle lifecycle checks and the same four PostgreSQL
browser journeys. The image check owns only its ephemeral Docker containers and
network. Neither command authorizes cluster deployment. Live Helm/Argo acceptance
still requires explicit environment, exposure, data/Secret and full-sync scope.

## Repository tooling

```sh
uvx pre-commit install
uvx pre-commit run --all-files
uv run --project .codex python -m unittest discover -s .codex/hooks -p '*_test.py'
python -m unittest scripts.tests.test_docs_catalog
python scripts/docs_catalog.py validate
for skill in .claude/skills/*; do
  uv run --locked --project .codex python .claude/skills/skill-creator/scripts/quick_validate.py "$skill"
done
python -m unittest discover -s .claude/skills/ci-optimization/scripts -p '*_test.py'
for skill in convention-audit living-spec-audit skill-creator; do
  uv run --locked --project .codex --with 'pytest==9.1.1' python -m pytest ".claude/skills/$skill/scripts" -q
done
```

CI gives each quality check one owner: the Rust job runs the full Rust gate,
and explicit frontend steps run read-only format/lint/typecheck. Its repository
hook step omits those duplicate calls only; local commits still run all hooks.
Separate CI build jobs produce the target's release binary and complete Next
standalone/static/public outputs. Image and tarball packaging reuse those exact
same-run outputs. Docker image assembly does not run Next, pnpm or Cargo.

For local Mac/Linux Docker development, one script runs the target builds in
pinned Linux builder containers, then packages their outputs. Mac node_modules
never enter Linux images; Apple Silicon defaults to Linux ARM64.

```sh
bash scripts/build-images.sh
# Or package outputs produced by a separate native build:
export WEB_ARTIFACTS=/absolute/build-artifacts/web
export BINARY_ARTIFACTS=/absolute/build-artifacts/binary
docker buildx bake --allow "fs.read=$WEB_ARTIFACTS" --allow "fs.read=$BINARY_ARTIFACTS" \
  -f images/chapchoo/ci-bake.hcl
python3 -m unittest scripts.tests.test_ci_structure scripts.tests.test_build_artifacts
```

No images are pushed by this command. Standard CI still verifies both native
architectures, archive checksums, SQLite/PostgreSQL journeys and runtime images.
Cold and cache-restored timing are separate measurements, not coverage modes.

Each inherited skill's companion scripts and references are preserved. Service
examples inside those skills are not runnable unless the referenced package and
infrastructure exist. Audit checkpoints start fresh; no Azents audit state is
inherited.

The Claude startup hook installs pre-commit. Codex hooks load path-scoped rules.
The optional upstream Claude marketplace plugin is not required or enabled.

Empty layout directories do not select a backend framework, runtime protocol,
cloud provider, or local adapter architecture.
