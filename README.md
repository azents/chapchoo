# chapchoo

An independent, channel-first agent project being built alongside Azents.

**Current state:** development foundation and a Hello World web app. Agent
execution, channels, users, workspaces, integrations, and routines are not
implemented here yet.

## Quick start

Prerequisites: Node.js 24, pnpm (the version pinned in `package.json`),
Python 3.14, and uv.

```sh
uvx pre-commit install
pnpm install --frozen-lockfile
pnpm run dev --filter=@chapchoo/web
```

Open <http://localhost:3003>. No backend, database, cloud account, or secrets are
needed to run the Hello World app.

See [DEVELOPMENT.md](DEVELOPMENT.md) for checks and repository structure.

## Provenance

Repository-local skills, coding conventions, Claude/Codex hooks, documentation
tooling, frontend configuration, and theme are adapted from
[azents/azents](https://github.com/azents/azents) at
`e71f614e2a8131b3db3fab7d0152c462b2447216`.
The original MIT notice is preserved in [LICENSES/azents-MIT.txt](LICENSES/azents-MIT.txt).
The skill-creator package retains its own license.

This is a development scaffold, not a copy of the Azents service or its data.
Inherited service-specific skill examples are conditional on those services
actually being added to this repository.
