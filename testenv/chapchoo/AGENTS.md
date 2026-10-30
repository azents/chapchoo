# Chapchoo test substrate

This package prepares the local infrastructure, fixtures and prerequisite state
needed by executable tests. It is not an E2E runner, test catalog or wrapper CLI.
Browser scenarios and the Playwright fixture wrapper live in
`typescript/apps/e2e`; representative E2E verifies core assembled-product flows.
Detailed copy, conditional UI and responsive layout checks belong to the owning
app's component tests. Read `.claude/conventions/testenv/e2e-core-flows-only.md`.

Use product CLI and public RPC/web paths to create state. Account, grant, session
and membership writes must pass ordinary product boundaries; never write SQL.
`Assembly` owns disposable PostgreSQL data/socket directories and its exact child
process groups. Reuse its bounded event-driven readiness and cleanup. Do not
scan or kill foreign processes, reuse a developer database, or retry mutations.
Missing tools fail explicitly; no SQLite fallback or silently skipped prerequisite.

Built JavaScript subpath exports expose assembly, process ownership, safe configuration,
redaction and secondary-user seeding. This package must not import app source,
Playwright, assertions or browser scenarios. Test output/configuration contains
safe metadata only; credentials, grants and proof-bearing URLs stay private.

Run from the repository root: `pnpm --filter @chapchoo/testenv test`, `lint`, and
`typecheck`. Its support tests exercise pure helpers/configuration and do not
launch products, Docker, PostgreSQL or network listeners. Root/integrator owns
assembled-product execution from `@chapchoo/e2e`.
