# Project Rules for Chapchoo

Chapchoo is a new project alongside Azents. This checkout contains general/Admin
account web applications and the account/team Rust backend. Agent execution and
deployment packaging remain distinct from the implemented account surface.
Do not treat product intentions or inherited examples as implemented behavior.

## Language

Write tracked artifacts in English. Localized messages and locale samples may
intentionally use other languages.

## Structure

- `typescript/apps/web/`: general Next.js account/team application.
- `typescript/apps/admin-web/`: independent installation Admin application.
- `typescript/apps/e2e/`: executable browser acceptance app.
- `typescript/packages/`: shared packages, including the generated Node RPC client.
- `typescript/packages/design-reference/`: independent cross-platform
  design reference Storybook; read its README and scoped Agent instructions for
  ownership and promotion. Existing Azents-derived styling is provisional, not
  Chapchoo's approved design system.
- `rust/crates/`: typed domain, crypto, persistence, protocol, services and adapters.
- `rust/apps/chapchoo/`: foreground backend and operator composition root.
- `testenv/chapchoo/`: reusable isolated environment, process and fixture support;
  browser scenarios remain in the E2E app.
- `proto/chapchoo/`: Protobuf service/message authority.
- `infra/charts/`, `testenv/chapchoo/`, `images/`: deployment, acceptance guidance,
  target-build and packaging support.
- `docs/chapchoo/`: Requirements, ADR, Design, and Living Spec documentation.
- `.claude/skills/`: source of truth for repository-local skills.
- `.agents/skills`: relative symlink exposing the same skills to compatible tools.
- `.claude/conventions/` and `.claude/rules/`: convention bodies and scoped indexes.
- `.codex/`: Claude-style rule loading hooks and their isolated Python environment.

## Conventions

Start with `.claude/rules/conventions.md`. Read the relevant language/path index
and only the convention bodies that apply. Other language and service rules are
available for future work, not authorization to implement those services.

Replace a narrowly scoped incorrect unit when its responsibilities, control flow,
or state model are wrong. Preserve surrounding public contracts unless the task
changes them. Avoid adding flags or fallback branches to preserve incorrect logic.

## Commands

See DEVELOPMENT.md for authoritative runnable commands. Run frontend format,
lint, typecheck, tests, and build before completing frontend changes.
Never bypass commit hooks.

## Documentation

Read `docs/chapchoo/AGENTS.md` for the inherited documentation process.
Read current specs first. Confirm Requirements before material technical design.
Requirements snapshots become immutable after implementation; ADRs are append-only;
Design records development-time decisions; Living Specs describe implemented behavior.
Product discovery is not permission to implement features or provision services.

## Safety and delivery

- Never copy credentials, browser state, `.env` files, runtime data, or audit state.
- Validate project-specific skill commands against this checkout before execution.
- Work on a focused branch and open a PR; do not push directly to main.
- Merge and deployment require explicit approval.
- Perform work directly by default. Delegate only when explicitly requested or
  applicable instructions authorize a bounded task.
