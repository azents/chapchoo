---
name: e2e-debugging
description: Debug Chapchoo E2E tests, fixtures, browser state, and local integration failures.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# Chapchoo E2E Debugging

Chapchoo E2E tests live under `testenv/chapchoo/e2e/`.

## Common Commands

```bash
cd testenv/chapchoo/e2e
uv run pytest ./src -v
```

Run a focused test:

```bash
cd testenv/chapchoo/e2e
uv run pytest ./src/tests/chapchoo/public/test_health.py -v -s
```

## Fixture Preparation

```bash
cd testenv/chapchoo
uv run testenv bootstrap local
uv run testenv prerequisite prepare --profile live --json
uv run testenv fixture doctor <fixture-id> --json
uv run testenv fixture up <fixture-id> --json
```

## Debugging Pattern

1. Reproduce with the smallest E2E test selection.
2. Capture server logs and browser traces when available.
3. Check fixture readiness before changing product code.
4. Fix fixture support only when the product behavior is already correct.
5. Prefer adding or updating E2E coverage for product behavior verification.

## Notes

- E2E tests may require Docker.
- Live/external tests require prepared prerequisite snapshots.
- Do not write product verification as legacy setup scripts; use E2E tests as the primary verification layer.
