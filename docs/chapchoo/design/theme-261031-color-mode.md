---
title: "System, Light and Dark Appearance Design"
created: 2026-10-31
implemented: 2026-10-31
tags: [frontend, appearance, accessibility]
document_role: primary
document_type: design
snapshot_id: theme-261031
---

# System, Light and Dark Appearance Design

Authority: [theme-261031/REQ](../requirements/theme-261031-color-mode.md) and
[theme-261031/ADR](../adr/theme-261031-color-mode.md).

## Current Behavior and Gaps

Both application root layouts render a forced-light Mantine script. Common UI's
provider also defaults to forced light. The shared CSS resolver defines light
values and an empty dark map. Brand variant colors use static light ink for both
filled and text-like actions, which cannot provide readable dark links. Stories
expose only light appearance. Mascot rendering uses a cream-backed 216px WebP and
multiply compositing. No appearance cookies or selection controls exist.

The current account-web Spec remains authority for business behavior, runtime
mounts, authentication, cookies and shared-presentation ownership; only its
forced-light description and artwork treatment are replaced.

## Architecture and Ownership

- `@chapchoo/web-core` provides a new browser-safe appearance policy subpath:
  preference validation, SSR resolution, cookie serialization, a cookie-backed
  Mantine-compatible manager and static pre-paint bootstrap generation. It imports
  neither Node transport/configuration nor product UI. Appearance code must not
  reuse or alter the authentication-cookie policy.
- Both application root layouts read request cookies and supply typed initial
  appearance state. They emit initial HTML/body attributes and pre-paint scripts.
- App-owned client bridges connect the cookie manager and initial state to the
  existing common Mantine provider. System-media synchronization uses Mantine's
  browser hooks. Bridges synchronize resolved attributes/cookies without keying or
  remounting the application, RPC provider, forms or query caches.
- `@chapchoo/ui` owns adaptive tokens, pure selection presentation and provider
  adapter props. It receives values/callbacks rather than importing a cookie store
  or request context. App adapters supply translated labels and selection actions.
- General and Admin share the policy and theme; neither app imports the other.

## Preference, Persistence and State Transitions

Preference is `system | light | dark`; resolved mode is `light | dark`. The policy
maps System to Mantine's `auto` at the provider boundary. Cookies are host-only,
root-path, SameSite=Lax, one-year presentation preferences, with Secure on HTTPS.
They are intentionally JavaScript-readable and contain only validated enum values,
not identities or credentials. Use Chapchoo-specific names to avoid another
product's cookies on a shared host. There is no Domain attribute, account record,
localStorage mirror or deployment-specific configuration.

The selected preference and last resolved mode are separate cookie values. Missing
or invalid preference means System. Missing or invalid cached resolution gives a
light server placeholder. Explicit Light/Dark always overrides a stale cached
resolution. The cached system result is only a hint until current browser media is
available. Cookie values never become arbitrary HTML or script text.

Selecting a choice updates in-memory state, document attributes and both cookies.
Selecting System immediately resolves current media. A subsequent system-media
change updates resolution only while System is selected; explicit choices remain
unchanged. Reloads and later visits, including another same-host runtime mount,
read the shared cookie. This does not add a cross-tab messaging service; an already
open independent tab need not repaint until its own choice/media change or reload.

Cookie storage denial does not stop an in-page change; persistence cannot be
promised when the browser refuses cookies. There is no backend request or mutation
retry for appearance changes.

## SSR, Pre-Paint Resolution and Hydration

1. Server layouts validate cookies and resolve explicit preferences or the cached
   System hint. They render `data-mantine-color-scheme` on HTML and a resolved mode
   indication on body, plus a preference attribute for observation.
2. A small synchronous head bootstrap uses validated preference and current
   `matchMedia` when System is selected. It updates the HTML mode and native
   `color-scheme` before product content can paint. It does not consult Mantine's
   default localStorage key.
3. A body-start prelude mirrors the already resolved HTML mode before the body
   content is parsed. Root-scoped CSS variables remain the styling authority;
   body indication is not a second competing theme source.
4. The client manager starts from the same preference and uses current media for
   System. Mantine controls its existing mode attribute without a forced-light
   fallback. Attribute synchronization must not write the stale server hint back
   over the browser-resolved mode during hydration.
5. Suppress hydration warnings only for the document attributes intentionally
   changed by the pre-paint scripts. Do not hide content-level hydration errors.

Without browser information the server cannot know a first-visit system preference.
The pre-paint browser step closes that gap; this design does not claim otherwise.
JavaScript-disabled requests still render explicit cookie preferences correctly;
a first System visit without JavaScript uses the server placeholder. No new CSP
policy is introduced. The scripts contain static code and validated enum literals,
and must follow any existing script policy discovered during implementation.

## Adaptive Presentation

Retain current light tokens and layout/type metrics. Add the reviewed warm
olive-charcoal semantic map rather than inverting colors. Canvas/navigation/pane,
input, selected and elevated/overlay surfaces are separate roles. Use restrained
butter actions, ivory body text and lighter semantic foregrounds. Separate filled
button ink (`onAccent`) from readable accent link/text foreground (`accentText`).
Define mode-specific Mantine body/text/default/anchor/placeholder and semantic
variant roles, including error/success/warning content, subdued backgrounds,
focus, selected, hover, disabled and loading. Menus and confirmation dialogs use
raised tones without changing geometry or turning ordinary groups into cards.
Native controls inherit the resolved CSS color scheme. Preserve action-label line
boxes, mobile input minimum/default line height and text enlargement behavior.

The three-choice accessible control is placed in existing shell header actions
and a compact auth/setup header action slot. It has localized Appearance and
System/Light/Dark labels, a visible current selection and keyboard operation.
There is no new navigation decoration, side stripe, destination or settings page.

Storybook exposes light and dark with the real shared provider/theme. Stories use
static fixtures, not application cookies. Independent reference examples consume
the same common components for equivalent roles. Native-platform experiments
remain outside this implementation.

## Character Assets

Keep the original five generated PNGs and their hashes as generation provenance.
Add the prepared transparent PNG masters as explicit derived assets, with source
and output hashes and extraction methodology. Generate small alpha-preserving WebP
production derivatives from them; never ship black review composites as dark-mode
variants. The current corgi runtime asset is replaced by its transparent
derivative, retaining runtime-prefixed staging and tracing. Render with normal
composition. Verify alpha and light/dark visual edges at actual 72px/40px sizes.
No new character identity, outline or color inversion is introduced.

## Security, Migration and Operations

No backend schema, API, session credential, authorization or installation change
is required. Existing light-only users have no appearance data to migrate and
start with System. Invalid appearance values cannot affect authentication. Do not
log cookie headers, credentials or proof URLs in tests or screenshots.

Delivery is one focused PR. Merge/deployment are separate authorized actions.
There is no deployment monitor or remote infrastructure mutation. Rollback is a
code revert; harmless appearance cookies may remain but old code ignores them.
No legacy theme path, feature flag or dual persistence fallback is added.

## Removal and Replacement

| Existing behavior/unit | Removal authority | Replacement/remaining authority | Boundary | Absence verification |
| --- | --- | --- | --- | --- |
| Forced-light root bootstrap and production provider default | REQ-1/2/3 | Typed SSR and cookie-managed modes | Both app layouts/common provider | Source scan plus SSR and browser checks |
| Empty dark resolver/static light variant foregrounds | REQ-1 | Adaptive semantic roles | UI theme and token adapter | Dark component renders/contrast checks |
| Light-only app Storybook toolbar/decorators | REQ-1 | Explicit light/dark render conditions | Both app Storybooks | Build/render both globals |
| Cream-backed runtime corgi/multiply | REQ-4 | Alpha derivative/normal composition | Shared mascot, staging and asset tests | Pixel alpha, bundle path and render checks |
| Light-only assertions/descriptions | REQ-1/3/4 | Current adaptive behavior assertions/Spec | Owning tests, DESIGN.md and account-web Spec | Focused tests/docs validation |

Original generated PNGs and their provenance remain authoritative for the original
outputs, not production compositing. Black review assets outside the repo are not
production input. No obsolete backend contracts/configuration or generated RPC
surfaces were found for this bounded change.

## Test Strategy

### Primary Browser Verification

Extend the existing assembled-product journeys minimally, not with a fourth
exhaustive journey: exercise a signed-out selection, reload, same-host Admin
observation and continued authenticated work in representative dark mode. Retain
all existing account/team/session-isolation assertions. Production apps run at
`/app` and `/admin` using the existing isolated testenv support, fresh database
and API-created account fixtures. Appearance itself needs no new backend seed,
external account, private service or live credential. Testenv is prerequisite
assembly, not a substitute for browser verification.

Use a focused real-browser boundary test for first navigation before hydration,
explicit-cookie SSR, fresh System under dark media, stale system-cache correction
and hydration console failures. Observe initial mode at first visible frame and
through hydration, not merely a screenshot taken after an effect. Verify root and
prefixed serving through existing fixture facilities; do not introduce public-path
build constants.

### Lower-Layer and Component Matrix

- Web-core: invalid/missing values, explicit-versus-cached precedence, safe cookie
  attributes, manager semantics, bootstrap behavior and no Node imports.
- Real component browser checks: all three choices, system live changes, explicit
  override, keyboard/focus and entered-value continuity. Detailed combinations and
  mobile/200%-text/320px reflow stay here rather than bloating product E2E.
- Common UI: actual light/dark token and semantic pair contrast checks; meaningful
  control and overlay states; localized long labels.
- Assets/staging: alpha-preserving derivatives, transparent corners and current
  runtime-prefixed packaging; original provenance hashes remain unchanged.

Capture native-resolution real-component/product PNGs at declared viewport, DPR1,
font, locale and mode; inspect representative auth, populated workspace and Admin
confirmation/error states. Do not label CSS simulations as implemented renders or
Chromium as physical Safari verification. Retain only safe evidence without
passwords/proof links. Record commands, SHA, mode, dimensions and executed counts.

Run normal format/lint/typecheck/unit tests, application and Storybook builds,
component checks and existing product E2E through current root commands. Commit
hooks remain enabled. Exact-head CI/review must pass. Required missing fixtures,
build failures, unexpected skips and hydration/appearance failures block delivery;
there are no optional live-provider tests to silently skip.

## Authority and Feasibility Audit

- REQ-1 is covered by M1/M4/M5: current resolver, component CSS, stories and browser
  component runners exist. Feasible without changing geometry or business state.
- REQ-2 is covered by M2/M3/M5: Next cookie reads and Mantine's injectable manager,
  auto mode and state setter exist; same-host root-path cookies need no API.
- REQ-3 is covered by M2/M3: both async root layouts can read cookies. Installed
  Mantine resolves `auto` via media in its root-attribute path and accepts a custom
  manager. Feasible with explicit pre-paint/body synchronization; implementation
  must prove no stale hydration overwrite rather than assume it.
- REQ-4 is covered by M6: five prepared RGBA masters exist; staging already copies
  WebP derivatives into each app's runtime asset directory. Feasible without an
  additional product dependency or image-generation service.
- Every removal traces to the corresponding replacement requirement. There is no
  unapproved account preference service or second persistence authority.
- Remaining risks: browser cookie/script policies can limit persistence or
  pre-paint execution; initial unknown System needs browser media. Contrast and
  first-frame correctness are verification obligations, not completed evidence.

## Design Authority

- Design revision: `1`

| ID | Material mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Adaptive semantic colors with unchanged geometry and typography | REQ-1 | required |
| M2 | Shared same-host root-path preference and cached resolution cookies; no competing persistent store | REQ-2/3, ADR-D1 | derived |
| M3 | Cookie-initialized SSR HTML/body indication plus pre-paint System resolution and non-remounting client synchronization | REQ-2/3 | derived |
| M4 | App-owned request/localization adapters and presentation-only common UI | Fixed Constraints; unchanged account-web Spec | existing |
| M5 | Accessible three-way selection on auth/setup and shell surfaces | REQ-2 | required |
| M6 | Transparent five-character masters, alpha runtime derivatives and normal composition | REQ-4 | required |

## Design Approval

- Mode: Collaborative.
- Decision owner: requester.
- Approved on: 2026-10-31 KST.
- Approved Design revision: `1`.
- Approved authority IDs: `M1, M2, M3, M4, M5, M6`.
- Approved scope: complete adaptive presentation, host-shared cookie persistence,
  SSR/pre-paint synchronization, existing ownership, three-way selection and
  transparent common artwork. The requester explicitly instructed implementation
  after receiving the complete revision brief.
