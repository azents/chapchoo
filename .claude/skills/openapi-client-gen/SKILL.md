---
name: openapi-client-gen
description: Regenerate Chapchoo API clients from OpenAPI specifications. Use when API routes are added or modified, or when the user asks to regenerate clients, update OpenAPI spec, or sync API changes.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# OpenAPI Client Generation

Regenerate API clients from Chapchoo OpenAPI specifications.

## Public API

1. Generate OpenAPI specs:

```bash
cd /path/to/chapchoo/python/apps/chapchoo
uv run python src/cli/dump_openapi.py
```

2. Regenerate Python client:

```bash
cd /path/to/chapchoo/python/libs/chapchoo-public-client
make generate
```

3. Regenerate TypeScript client:

```bash
cd /path/to/chapchoo/typescript
pnpm run generate --filter=@chapchoo/public-client
```

## Admin API

1. Generate OpenAPI specs:

```bash
cd /path/to/chapchoo/python/apps/chapchoo
uv run python src/cli/dump_openapi.py
```

2. Regenerate Python client:

```bash
cd /path/to/chapchoo/python/libs/chapchoo-admin-client
make generate
```

3. Regenerate TypeScript client:

```bash
cd /path/to/chapchoo/typescript
pnpm run generate --filter=@chapchoo/admin-client
```

## When to Run

- API routes are added or modified.
- Request/response schemas change.
- New endpoints are added.
- Endpoint paths or methods change.

## Notes

- TypeScript clients use @hey-api/openapi-ts.
- Generated files are typically gitignored; do not manually edit them.
- After generation, run type checks to ensure compatibility.
