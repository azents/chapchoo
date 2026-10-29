---
name: python-quality-check
description: Run Python code quality tools (ruff, the configured type checker, pytest) for an Chapchoo Python subproject. Use when the user asks to check code quality, run linting, type checking, or tests for Python code.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# Python Quality Check

Run code quality tools from the relevant Python subproject directory.

## Standard Check

```bash
uv run ruff check --fix .
uv run ruff format .
uv run pytest
```

Run the whole-subproject type checker:

```bash
uv run ty check --error-on-warning
```

## Quick Reference

### Backend

```bash
cd /path/to/chapchoo/python/apps/chapchoo
uv run ruff check --fix .
uv run ruff format .
uv run ty check --error-on-warning
uv run pytest
```

### Shared Library

```bash
cd /path/to/chapchoo/python/libs/az-common
uv run ruff check --fix .
uv run ruff format .
uv run ty check --error-on-warning
uv run pytest
```

### E2E

```bash
cd /path/to/chapchoo/testenv/chapchoo/e2e
uv run ruff check --fix .
uv run ruff format .
uv run ty check --error-on-warning
uv run pytest
```

E2E tests require Docker.

## Line Length Guidelines

- For LLM prompts, preserve prompt quality. Use `# noqa: E501` or a file-level exception when needed.
- For regular code, fix by breaking lines, using intermediate variables, or multi-line formatting.

## When to Run

- After completing a logical unit of work.
- Before marking any task as complete.
- After adding or modifying type annotations.
- After making changes that could affect existing functionality.
