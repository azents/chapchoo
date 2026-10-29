---
name: python-dependency-tree-analysis
description: Analyze Python dependency paths and version constraints in Chapchoo uv projects. Use when a transitive Python dependency will not update, a security advisory requires finding the constraining parent, or an agent must determine whether a patched version is resolvable without overrides.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# Python Dependency Tree Analysis

Run every command from the affected Python subproject, never the repository root.

## 1. Classify the dependency

```bash
grep -Ri --include='pyproject.toml' '<package>' .
grep -n -A4 -B2 'name = "<package>"' uv.lock
uv tree --package <package> --invert --no-dedupe
```

- If the package is declared in `pyproject.toml`, treat it as direct.
- Otherwise, use the inverted tree to identify every parent path to a direct dependency.
- Add `--universal` when platform markers may produce different paths.

## 2. Test normal lock resolution

```bash
uv lock --upgrade-package <package> --dry-run
uv lock --upgrade-package <package>
uv tree --package <package> --invert --no-dedupe --locked
```

Use a command-only target constraint when a minimum patched version must be tested without changing project declarations:

```bash
uv lock --upgrade-package '<package>>=<patched-version>' --dry-run
```

Do not add `[tool.uv.override]` unless the requester explicitly requires it.

## 3. Resolve a blocking parent

When the target remains below the required version:

1. Read each parent requirement from `uv.lock` and the parent's published metadata.
2. Identify the nearest direct dependency that introduces the restrictive parent.
3. Check whether a newer compatible release of that direct or parent dependency widens the constraint.
4. Update that dependency through its normal `pyproject.toml` declaration or `uv lock --upgrade-package <parent>`.
5. Regenerate the lock and repeat the inverted-tree check.

Do not replace one forced version with another hidden constraint.

## 4. Stop when no compatible path exists

Do not modify the lockfile manually. Stop and report:

- requested target and patched version
- every dependency path from the project to the target
- the exact version constraint that blocks resolution
- the newest parent version checked and whether it retains the constraint
- viable upstream options, such as waiting for a release, replacing the parent, or accepting an explicit requester-approved override

## 5. Verify

```bash
uv lock --check
uv tree --package <package> --invert --no-dedupe --locked
uv run ty check --error-on-warning
```

Run the affected project's tests before completing an update.
