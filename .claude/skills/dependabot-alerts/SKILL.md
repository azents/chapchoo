---
name: dependabot-alerts
description: Inspect Dependabot alerts for the Chapchoo repository and summarize actionable vulnerabilities.
---

## Bootstrap readiness

This repository currently contains development tooling and a Hello World web app only. Before running any project-specific command below, verify that its target, workflow, package, and required credentials actually exist in this checkout. Examples for backend apps, API clients, cloud runtimes, E2E infrastructure, and an admin web app describe inherited patterns, not implemented Chapchoo features. Do not run commands against the upstream Azents repository or provision missing services merely to follow an example. Use the current README and DEVELOPMENT.md for runnable commands.


# Dependabot Alerts

Use the Chapchoo repository name when querying GitHub alerts.

```bash
gh api 'repos/azents/chapchoo/dependabot/alerts?state=open&per_page=100' \
  --jq '.[] | {number, state, package: .dependency.package.name, ecosystem: .dependency.package.ecosystem, severity: .security_advisory.severity, summary: .security_advisory.summary}'
```

Summarize:

- affected package and ecosystem
- severity
- vulnerable range and patched version
- affected subproject when inferable
- recommended update command

Do not dismiss alerts unless the user explicitly asks.
