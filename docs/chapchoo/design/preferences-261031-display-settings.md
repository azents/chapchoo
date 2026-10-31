---
title: "User Display Settings Design"
created: 2026-10-31
updated: 2026-10-31
implemented: 2026-10-31
tags: [accounts, preferences, architecture, frontend]
document_role: primary
document_type: design
snapshot_id: preferences-261031
---

# User Display Settings Design

Authority: [Requirements](../requirements/preferences-261031-display-settings.md)
and [ADR](../adr/preferences-261031-display-settings.md).

## Current Behavior and Gaps

Users have no persisted display values, profile route or settings RPC. Both web
apps currently couple translation and formatting through a cookie/header locale;
Admin timestamps force UTC. The existing request-cached identity and browser
identity query are the account authority. Common UI is presentation-only.

## Architecture, Data and Interfaces

- Persist language, IANA time zone and regional format on each user. Return them
  with the existing User snapshot on identity and authentication responses. A
  protected independent-field patch returns the committed current User; panel
  and profile share the existing identity query, not a second account store.
- Store installation defaults in a separate singleton. A public read exposes
  only these presentation values through the existing Auth service on both
  listeners. Only Admin's protected update can change installation defaults.
- Extend signup and initial Admin creation with optional partial display input.
  Each omitted dimension resolves current installation defaults inside the
  account-creation transaction. Browser callers supply explicit choices or
  available current detection independently per field, without materializing
  stale SSR defaults. Existing creation/proof/session atomicity is retained.
- Generated Protobuf remains the Rust/Node authority. Both ingress boundaries
  validate complete settings and independent patches. Browser bundles import no
  Node transport or credentials. IANA names use bundled timezone data so value
  validation does not introduce a host tzdata installation requirement.

## Preparation, Upgrade and Readiness

The existing prepare lock serializes the entire operation. Schema migration,
key binding, initial defaults and missing-user backfill are one transaction.
The preparation environment is detected outside that transaction. A failed or
cancelled transaction commits none of the new schema/data changes.

The new migration adds columns with incomplete markers solely within the
preparation transaction; production creation always writes complete values.
Defaults initialize once. Missing existing-user time zones use the current
preparation machine, not the persisted default. Missing language/region use the
installation default. Existing values are never overwritten on repreparation.
Read-only readiness verifies the complete expected migration set, singleton and
absence of incomplete markers. `serve` never initializes or migrates.

An undetectable/invalid machine time zone fails a preparation that needs initial
values; it is not silently replaced with UTC. A supported environment locale is
mapped to its supported language/format; unknown locale uses the confirmed
English/en-US default. No new runtime override or recovery command is introduced.

## SSR and Browser State

Each app resolves one request-cached display snapshot using the existing cached
identity and the public defaults read. Account values win when authenticated.
A visitor resolves explicit cookies, then projection cookies, then defaults.
The existing cookie/header locale resolver is replaced, not retained as another
authority. Server content and first hydration use the same resolved snapshot.
Authentication failure stays a failure, not a signed-out account transition.
Required defaults-read failure reaches the sanitized rendering boundary rather
than inventing a static installation default.

After hydration, browser detection updates automatic visitor dimensions and the
projection cookie. Explicit visitor choices are stored separately and never
overwritten by detection. Host-only, root-path presentation cookies follow the
existing one-year, SameSite=Lax, HTTPS-Secure presentation-cookie policy. No
credential or signed-in account settings are persisted as visitor authority.

The application display controller derives account values from the identity
query. It owns transient editor attempts, pending/error state and the global
panel. A successful mutation updates that query from its committed User result.
Account-switch and unmount fences prevent a late response populating a different
identity. Requests are not automatically retried. Explicit failure recovery
keeps the requested edit without claiming saved success.

Each app's client localization provider uses the current display language and
its existing four message catalogs without remounting page state. The document
language follows it. Formatting uses the independent region/time-zone context.
Subsequent SSR resolves the persisted account or visitor authority again.

## UI and Suggestions

Promote the approved panel, fields, preview, icons and toast into prop-driven
common UI. App-owned controllers supply persistence, scope, translated copy and
callbacks. An application-chrome context supplies the always-visible header
control to existing public/auth/workspace/error frames; it carries only rendered
presentation, not authentication or RPC ownership.

General and Admin add protected profile routes with duplicate controls over the
same controller. Admin additionally adds a protected installation-default page;
editing it does not change the account controller. The illustrative public
backdrop does not become a new product route.

Suggestions include only detected differences, apply independently, and disappear
when no row remains. Language suggestions use autonyms and current/target-language
title, label and action. Dismissal memory is browser-local and partitioned by
account and dimension/detected value; a newly observed value resets only that
dimension's rejection. It is not account authority. A denied storage facility
must not crash the page or imply guaranteed persistence.

## Security and Failure Boundaries

Existing exact Origin/POST admission, request-local renewal, per-app cookies,
read-only SSR and no-automatic-replay policies are retained. New protected paths
participate in invalid-session handling. Admin access remains current backend
identity authority; personal editing never grants administrator capabilities.
Public defaults disclose only language/timezone/region. Invalid partial values
are rejected; omitted fields are unchanged, not cleared or filled speculatively.
No secrets, proof URLs or database strings enter user-facing errors.

## Test Strategy

Representative assembled E2E extends existing journeys with guest choice through
signup, persisted profile/panel editing and a script-blocked SSR refresh. Admin
verifies personal/default separation and selected-zone grant formatting. Keep
language/region/timezone permutations, row combinations and failure/pending states
in policy, RPC and app-owned component tests rather than multiplying E2E flows.

Use existing testenv PostgreSQL/process ownership and ordinary product account
creation. No operational credentials or live production data are prerequisites.
Rust integration tests cover fresh and populated SQLite/PostgreSQL upgrade,
repreparation, interruption/rollback, independent patches, denied permissions and
preservation of prior settings. Generated-code, ingress, authentication and
request-cache tests cover boundary propagation. Missing local DB prerequisites
fail rather than silently substituting SQLite or skipping coverage.

Capture real production components at explicit desktop/mobile viewport and DPR1,
wait for bundled Pretendard/layout, inspect native PNGs, and record overflow,
keyboard and immediate-apply evidence. CI must pass for the final PR SHA. Builds,
unit/component/E2E results and migration checks are distinct evidence; prior
reference-only renders do not prove production behavior.

## Design Authority

- Design revision: `1`

| ID | Mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Three independent complete account display values | REQ-1, REQ-3 | required |
| M2 | Preparation-owned atomic initialization/backfill; read-only startup | ADR-D1, REQ-7, REQ-8, account-storage Spec | decided |
| M3 | Account mutation, signup and Admin-default persistence through existing protected RPC | REQ-3, REQ-5, REQ-7, account-rpc Spec | derived |
| M4 | Account SSR authority; explicit/projection/default visitor handoff | REQ-3, REQ-4 | required |
| M5 | Same global panel and duplicate protected profile controls in both apps | REQ-2, REQ-3 | required |
| M6 | Independent titled/icon suggestion rows and scoped dismissal | REQ-6 | required |
| M7 | Existing security, request/cache and no-replay boundaries | REQ-3, fixed constraints, account-web Spec | existing |
| M8 | Existing package portability with IANA validation and preparation-environment detection | REQ-1, REQ-7, REQ-8, account-delivery Spec | derived |
| M9 | Natural four-language UI and real-component verification | REQ-9 | required |

Identifiers, equivalent DTO nesting, callback/helper boundaries, storage field
names and fixture organization are agent-owned implementation details. No settings
version, background reconciler, second account authority or extra runtime mode is
introduced.

## Removal and Replacement

| Existing behavior | Authority | Replacement / boundary | Absence verification |
| --- | --- | --- | --- |
| Locale cookie/header authority over signed-in display | REQ-3, REQ-4 | Account/visitor snapshot when enabled | No request-config header/cookie fallback chain |
| Fixed UTC formatting/suffix on Admin grants | REQ-1 | Independent region/time zone | Formatter/component assertions |
| Reference-local equivalent panel/fields/toast | REQ-9, shared-UI ownership | Common presentation reused by product stories/reference | Reference imports common equivalent components |
| One-version-only migration readiness | REQ-8 | Exact expected migration set and completed data preparation | Old/populated and incomplete-readiness tests |

Original artwork, account/team/authentication contracts unrelated to display,
and existing data are retained. Character replacement is outside this session.

## Feasibility and Non-Blocking Risks

Existing creation transactions, User response mapping, request cache, identity
query, common header/shell, NextIntl provider and app component test infrastructure
provide the integration paths. Local verification includes both-engine populated
upgrades/rollback/retry, Rust and generated live RPC gates, three assembled
account/team journeys, focused cookie-response ordering, unit/component tests and
both production builds. Real product and seeded production-component PNGs use
bundled Pretendard at native DPR1; 1280/390/320 component checks cover reflow and
keyboard focus admission/return. Browser/ICU aliases compare semantically to avoid
spurious suggestions. Browser storage denial remains an explicit limitation,
not guaranteed persistence. Local verification does not establish a physical
device, deployed installation or final-head CI result.

## Design Approval and Execution Scope

Mode: requester-approved implementation. Decision owner: requester.
Date: 2026-10-31. Scope: the confirmed UI, preparation-owned initialization
and mechanisms M1–M9. Approval applies to that material scope; the revision
identifier is a repository traceability label, not a separately approved
artifact. New product scope, data-loss authority or permission changes require
new authority.
