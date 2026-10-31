---
title: "Display Settings Initialization and Authority"
created: 2026-10-31
tags: [accounts, preferences, architecture]
document_role: primary
document_type: adr
snapshot_id: preferences-261031
---

# Display Settings Initialization and Authority

Requirements authority:
[preferences-261031/REQ](../requirements/preferences-261031-display-settings.md).

## Decision Map

- Accepted: explicit preparation owns installation-default initialization and
  existing-user backfill; service startup stays read-only (`D1`).
- Fixed: account values govern authenticated SSR and hydration; visitor explicit
  choices override browser projection, which overrides installation defaults.
- Fixed: one account authority is edited through both the global panel and
  profile; each suggestion changes one dimension.
- Existing: app-owned state/transport, generated RPC, request-local read-only
  authentication, protected mutation admission, no automatic replay, and common
  presentation without authentication or browser-persistence ownership.
- Agent-owned details: equivalent DTO nesting, helper/module boundaries,
  component composition, field/cookie identifiers, fixture composition and
  translation-key names. These do not authorize additional authority or modes.
- Existing identity propagation and local provider handoff implement the
  confirmed single account authority, without an additional source of truth
  or runtime mode.
- Timezone validation retains distribution portability through bundled data;
  environment detection and preparation failure remain within explicit prepare.
  No timezone fallback or new operator setting is introduced.

## D1. Preparation-Owned Initialization

- Authority: `preferences-261031/REQ-7`, `preferences-261031/REQ-8`.
- Decision owner: requester.
- Accepted: 2026-10-31 KST.

Explicit `prepare` initializes installation display defaults once and backfills
missing user settings. The first successful preparation environment supplies
initial installation time zone and locale. A migration's execution environment
supplies missing existing-user time zones, independently of any subsequently
edited installation defaults.

Normal `serve` performs read-only readiness checks and rejects incomplete
preparation. It does not initialize defaults, migrate schema or race other
replicas to select an initial machine's display settings.

The existing SQLite file lock and PostgreSQL preparation advisory lock continue
to own preparation serialization. Initialization/backfill must be crash-safe,
repeatable and non-overwriting; readiness must reject an incomplete result.

### Alternatives Considered

- First-service-start initialization would preserve a separate serving-machine
  environment, but introduce startup writes and first-writer selection across
  distributed replicas. The requester rejected this as a less natural boundary.
- Supplying separately detected serving-environment values to preparation would
  retain startup's read-only boundary but add an operator input/procedure.
  It is unnecessary with the confirmed preparation-environment interpretation.

### Consequences

- Deployment continues to prepare explicitly before starting application
  replicas; no new startup migration or background reconciliation mode exists.
- Repreparation never overwrites administrator-edited defaults or established
  user preferences.
- Initial defaults and an existing user's migration-time value may coincide,
  but remain different authorities for subsequent upgrades/default changes.
