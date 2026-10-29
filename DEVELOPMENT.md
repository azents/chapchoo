# Development

## Rust foundation

The account/team implementation is being delivered in phases. The first phase
contains typed domain values and common cryptographic primitives, not a runnable
backend service. Install the pinned toolchain and run its integrated checks:

```sh
rustup toolchain install 1.99.1 --profile minimal --component clippy,rustfmt
bash scripts/check-rust.sh
```

`rust/rust-toolchain.toml` is the toolchain authority. The workspace lockfile is
committed; checks deny unsafe code, unhandled lint warnings, production unwraps/
expects and secret-revealing diagnostic formatting. Domain and crypto crates
have explicit dependency boundaries and do not depend on persistence or RPC.
Password hashing is synchronous CPU work: the future service layer must provide
bounded blocking admission rather than calling it on an async executor thread.

These primitives do not authorize an installation-key change or persist data.
No actual installation secret belongs in source control, test fixtures or logs.

The persistence phase adds real SQLite/PostgreSQL integration tests. The Rust
quality script starts and removes a private Unix-socket PostgreSQL instance when
`initdb` and `pg_ctl` are available. Otherwise explicitly supply
`CHAPCHOO_TEST_POSTGRES_URL` for a disposable test server with schema-creation
permission. Never point this setting at a production database. Tests use isolated
schemas and do not count missing PostgreSQL prerequisites as skipped successes.
CI supplies a fresh PostgreSQL service to each validation job.

## Frontend

The scaffold uses the same frontend foundation as Azents: Next.js App Router,
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
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web
```

The dev app uses port 3003; Storybook uses port 6006:

```sh
pnpm run storybook --filter=@chapchoo/web
```

The app renders one localized Hello World screen. Supported locales match the
upstream web app: `en-US`, `ko-KR`, `ja-JP`, and `fr-FR`. Locale selection uses the
`locale` cookie, then Accept-Language, then English; no locale URL segments.
There is no account-preference lookup because authentication is not implemented.
Storybook renders the same theme and translation providers with static fixtures.

Backend clients, tRPC routes, authentication, Sentry upload configuration, and
Azents product pages were deliberately not copied. There is no backend or
deployment command yet.

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
