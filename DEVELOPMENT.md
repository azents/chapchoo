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
cargo run --locked -p chapchoo -- serve --role both
# Independent choices: serve --role general; serve --role admin
cargo run --locked -p chapchoo -- grant-admin --email existing@example.invalid
```

Preparation explicitly migrates and binds the installation. Normal service startup
only checks readiness and refuses missing/wrong schema or key. Local listeners
default to `127.0.0.1:50051` (general) and `127.0.0.1:50052` (Admin). Override
with `--general-listen` / `--admin-listen` or `CHAPCHOO_GENERAL_LISTEN` /
`CHAPCHOO_ADMIN_LISTEN`. They are plaintext HTTP/2; do not publish them directly.
Deployments require a trusted internal route and an operator-managed TLS gRPC
edge for external callers.

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

Email is optional. Configure both `CHAPCHOO_SES_SENDER` and
`CHAPCHOO_SES_REGION`, with ordinary AWS SDK credential resolution. An optional
`CHAPCHOO_SES_ENDPOINT` selects an operator-controlled SDK endpoint. Mail-only
flows fail when it is absent; manual signup/reset grants remain usable.
Email is submitted after commit, without automatic submission retries; failure
does not compensate state and provider acceptance is not mailbox delivery.

SIGINT/SIGTERM stops admission and drains all selected backend tasks. Combined
roles form one failure unit. Startup has a 60-second budget; shutdown has
20 seconds for graceful drain and a further 5 seconds for forced task cleanup.
Liveness is the gRPC health service name `chapchoo.liveness`; readiness checks
the selected business service or empty service and rereads schema/key readiness.
An unhealthy database can leave liveness serving while readiness is not serving.

### Generated RPC clients

`proto/chapchoo/accounts.proto` is the Rust/Node contract source. Rust generation
uses vendored protoc at build time; committed Node output is reproduced with:

```sh
bash scripts/generate-rpc-clients.sh
bash scripts/check-rpc.sh
```

The RPC gate verifies generation, Node codec tests and live Node-to-Rust calls
against both disposable database engines. It requires pnpm/Node and `psql`,
plus the PostgreSQL prerequisite described above. Missing prerequisites fail.
The Node package is server-side only: browsers do not speak gRPC or hold backend
credentials in JavaScript storage.

## Frontend

The apps use the same frontend foundation as Azents: Next.js App Router,
React, TypeScript, Mantine and its theme, Geist fonts, cookie/header-based
next-intl localization, pnpm workspaces, Turborepo, ESLint, Prettier, and Storybook.
The initial dependency ranges and pinned Next.js version come from the recorded
upstream snapshot.

```sh
cd typescript
pnpm install --frozen-lockfile
pnpm run dev --filter=@chapchoo/web
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm --filter @chapchoo/web test
pnpm --filter @chapchoo/admin-web test
pnpm --filter @chapchoo/ui test
pnpm --filter @chapchoo/web-core test
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web
pnpm run build-storybook --filter=@chapchoo/admin-web
```

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
Both use `@chapchoo/ui` for the original shared theme and controls, and
app-owned tRPC adapters backed by `@chapchoo/web-core` and generated Node gRPC.
Supported locales match the
upstream web app: `en-US`, `ko-KR`, `ja-JP`, and `fr-FR`. Locale selection uses the
`locale` cookie, then Accept-Language, then English; no locale URL segments.
There is no account-preference lookup in the UI.
Storybook renders the same theme and translation providers with static fixtures.

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

The three assembled browser journeys and disposable PostgreSQL/process ownership
are documented in
[`@chapchoo/e2e`](typescript/packages/chapchoo-e2e/README.md).
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
standalone outputs/Node, bundle lifecycle checks and the same three PostgreSQL
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

Each inherited skill's companion scripts and references are preserved. Service
examples inside those skills are not runnable unless the referenced package and
infrastructure exist. Audit checkpoints start fresh; no Azents audit state is
inherited.

The Claude startup hook installs pre-commit. Codex hooks load path-scoped rules.
The optional upstream Claude marketplace plugin is not required or enabled.

Empty layout directories do not select a backend framework, runtime protocol,
cloud provider, or local adapter architecture.
