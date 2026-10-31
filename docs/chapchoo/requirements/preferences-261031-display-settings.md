---
title: "User Display Settings Requirements"
created: 2026-10-31
updated: 2026-10-31
implemented: 2026-10-31
tags: [accounts, preferences, frontend, localization]
document_role: primary
document_type: requirements
snapshot_id: preferences-261031
---

# User Display Settings Requirements

- Snapshot: `preferences-261031`
- Document reference: `preferences-261031/REQ`

## Problem

The web applications couple translation language and regional formatting, do not
provide a user time zone, and have no persistent personal display-settings UI.
Visitors and account holders need one understandable settings experience without
losing deliberate choices when browser detection or authentication changes.

## Primary Actor

A visitor or account holder using the General or Admin web application.

## Primary Scenario

The person opens the global display-settings panel from any page, changes a
language, time zone or regional format, and continues with the selected display.
For an account holder, the profile exposes the same values and either editing
surface updates the other. For a visitor, explicit selections survive refresh
and subsequent account creation.

## Supporting Scenarios or Effects

- A signed-in person receives and independently accepts relevant display-change
  suggestions after browser preferences differ from account settings.
- An administrator changes installation defaults without changing existing users.
- An existing installation gains complete user display values while preserving
  accounts, credentials, sessions, teams and other product data.

## Goals

- Independent translation language, time zone and regional formatting.
- The approved global panel, duplicate profile controls, and minimal titled,
  category-icon suggestion toast on both web applications.
- Deterministic server rendering and first hydration, with visible real content.
- Explicit user intent takes precedence over automatic browser detection.

## Non-Goals

- Appearance-policy changes or new account/profile capabilities beyond display
  settings and the existing account identity presentation.
- Per-user currency, calendar or separate hour-cycle overrides.
- Changes to authentication, authorization, credential lifetime, team behavior,
  automatic operation replay, deployment or merge policy.
- Browser-wide active-tab synchronization infrastructure or settings versioning.

## Requirements

### REQ-1. Independent display dimensions

Translation language, time zone and regional format are independent settings.

**Acceptance criteria**

- Initial translation languages are English, Korean, Japanese and French.
- Initial regional formats are `en-US`, `ko-KR`, `ja-JP` and `fr-FR`.
- A supported language, a supported regional format and a valid IANA time zone
  can be selected independently; changing one does not change the others.
- Real instants, including Admin issuance and expiry, use the selected time zone
  and regional format. Date-only values do not undergo time-zone conversion.
- Every account has a time zone and complete language/format values.

### REQ-2. Global, authentication-independent settings experience

The global entry and panel work outside login forms and are available before and
after sign-in in both General and Admin.

**Acceptance criteria**

- The approved header entry is available on public, authentication and signed-in
  pages, including responsive narrow layouts.
- The panel preserves the approved three-control arrangement, preview and
  immediate-application interaction regardless of authentication.
- Visitor controls additionally offer following the browser for each dimension.
- Visitor scope and account scope are accurately identified.
- A normal signed-in page can open settings without navigating to the profile.
- Keyboard operation, accessible names, modal dismissal and mobile reflow work
  without horizontal page scrolling or overlapping controls.

### REQ-3. Account authority and duplicate profile controls

Signed-in display uses the account's values. The profile separately exposes the
same three controls, rather than replacing the global panel.

**Acceptance criteria**

- Server-rendered and first hydrated signed-in content use the same account
  language, time zone and regional format.
- A successful panel edit updates the profile; a successful profile edit updates
  the panel without an additional user save action.
- Both General and Admin edit the authenticated account's personal values,
  independently of permission to change installation defaults.
- Saving failure is localized and actionable, preserves user intent for recovery,
  and does not claim persisted success or silently repeat an uncertain operation.
- Sign-out or account switching cannot leave the previous account's values as
  the new account's authoritative settings.

### REQ-4. Visitor explicit choice and browser-following state

Each visitor setting can be explicit or follow the browser. The states remain
semantically distinct across refresh and subsequent visits.

**Acceptance criteria**

- An explicit choice survives refresh/revisit and is not overwritten by browser
  detection; choosing follow-browser returns that dimension to automatic mode.
- Initial server rendering and first hydration use an explicit choice, otherwise
  the last valid browser projection, otherwise the installation default.
- After hydration, current available browser values update automatic dimensions
  and their projection without replacing real server content with a blank or
  client-only loading screen.
- Unavailable browser values retain the applicable installation/default display;
  invalid or unsupported untrusted values are not accepted as account settings.
- The automatic projection and visitor's explicit intent remain distinguishable.

### REQ-5. Account creation preserves applicable visitor choices

Signup and initial administrator creation initialize all three dimensions from
explicit visitor choices, then current browser values, then installation defaults.

**Acceptance criteria**

- The precedence is evaluated independently per dimension.
- A deliberate visitor time zone is preserved on account creation just like
  language and regional format.
- Account creation without usable browser values still produces complete values.
- Signup authority, initial-setup admission and their atomic account/session
  outcomes retain existing security and integrity behavior.

### REQ-6. Independent, minimal browser-change suggestions

When signed-in browser values differ from account values, available changes are
suggested with the approved toast rather than silently applied.

**Acceptance criteria**

- The toast has the approved display-settings question title, close action, and
  language, clock and globe icons for the respective available rows.
- Only applicable language, time-zone and regional-format rows appear.
- Every row has its own change action; language and region are not atomic.
- Accepting a row immediately changes only that dimension, removes the accepted
  row, and requires no navigation or additional save. No rows means no toast.
- Language names use recognizable autonyms and current-to-target direction.
  When language is suggested, title, language label and action include both the
  current and proposed languages so one unreadable language is not the sole cue.
- The proposal omits browser-origin explanations and unrelated prose.
- Dismissal leaves account values unchanged and is remembered for that account
  in that browser, keyed by detected values. The same dismissed detected value
  is not offered again until the relevant browser value changes.

### REQ-7. Installation defaults

An administrator can edit installation default language, time zone and regional
format separately from personal settings.

**Acceptance criteria**

- Initial time zone comes from the first successful preparation environment.
- Initial language/format map that preparation environment's locale to the
  supported set; unknown or unsupported locale uses English/`en-US`.
- Initialization happens once. Restart, preparation rerun or subsequent default
  edits do not overwrite an already initialized installation default.
- Default changes affect fallback/initialization behavior, not existing account
  settings or a visitor's explicit choices.
- Unauthenticated visitors can render the applicable defaults without receiving
  administrator capabilities, secrets or unrelated installation data.

### REQ-8. Existing-account upgrade

The upgrade preserves existing product data and initializes missing display
values according to the agreed source environments.

**Acceptance criteria**

- Existing users' missing time zones use the machine executing migration, not
  the configured server default, remote database server or a browser cookie.
- Existing language/format values, if present, are preserved; otherwise the
  applicable initial server defaults supply them.
- An interrupted upgrade or rerun cannot expose incomplete values as ready,
  change already established values or delete unrelated data.
- Both SQLite and PostgreSQL have executable populated-installation upgrade and
  fresh-installation verification.

### REQ-9. Consistent product fidelity and verification

The production result implements the approved component-render composition,
shared theme, typography and responsive interaction, not a separate visual copy.

**Acceptance criteria**

- Both General and Admin use the shared Chapchoo presentation and Pretendard.
- Translated copy is natural and aligned across the four supported languages.
- Account/visitor authority, first SSR/hydration, browser projection, bidirectional
  profile edits, independent suggestions, dismissal memory and default edits have
  focused lower-layer coverage plus minimal representative browser journeys.
- Real production-component desktop/mobile renders are inspected after integration.
- Existing authentication/team/grant regressions, generation, format, lint,
  typecheck, builds and required CI pass for the delivered PR head.

## Fixed Constraints

- Preserve existing read-only SSR authentication and request-local credential
  ownership. No credentials enter browser-readable preference persistence.
- Initialization and existing-user backfill belong to explicit preparation.
  Normal service startup remains read-only and rejects incomplete preparation.
- No deployment, merge, destructive reset, data copy or legacy fallback is
  authorized by this feature request.

## Open Assumptions

- Settings changes do not require active synchronization to already-open
  independent tabs. Subsequent request/navigation observes current authority.
- Installation defaults are initialized only once, whereas a later migration
  uses its own execution environment for any still-missing user time zone.

## Confirmation

The requester confirmed the cumulative display behavior and final UI proposal and
requested implementation on 2026-10-31. After this transcription was presented,
the requester confirmed preparation-owned initialization on the same date,
including the interpretation of initial environment as the first successful
preparation environment. The UI remains confirmed. This confirmation does not
delegate unrelated material technical decisions.
