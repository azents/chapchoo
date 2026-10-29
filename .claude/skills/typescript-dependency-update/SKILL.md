---
name: typescript-dependency-update
description: Update dependencies in the Chapchoo TypeScript workspace.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# TypeScript Dependency Update

Run from `typescript/`.

```bash
cd /path/to/chapchoo/typescript
pnpm update <package>
pnpm run format
pnpm run lint
pnpm run typecheck
```

Commit `package.json` and `pnpm-lock.yaml` changes together.
