---
title: "Web appearance lifecycle"
---

# Web appearance lifecycle

Appearance owns system/light/dark resolution and initial paint; it is independent of account language, time zone and regional format.

## Appearance lifecycle

Both applications reuse the same sticky global AppHeader through their auth/setup
and workspace frames. Its brand is left-aligned and one 44×44 account/settings
entry remains at the far right, including loading and recoverable errors.
Authenticated users see a user icon; visitors see sliders. Desktop opens a
Popover and narrow screens a right Drawer with mini-profile, System/Light/Dark,
display controls and profile/sign-out or visitor sign-in. The avatar, identity and
profile label form one full-banner native link using the common InteractiveSurface
hover/pressed/focus-visible treatment, with a trailing navigation chevron. It is
one keyboard stop, opens Profile in a new tab/window with `noopener`, and closes
the menu on navigation. Static scope/immediate-help
rows are omitted from this menu; actual saved, failure and retry feedback remain.
App-provided controls replace duplicate page account rows and the separate appearance entry.
Escape/outside dismissal uses controlled callbacks and restores focus.
General also supplies a narrow-screen navigation trigger and an app-entry brand link.
The conversation workspace places these existing account/appearance controls
in its right-side presence header, preserving the same preference authority
without a duplicate left-sidebar entry or a second global header.
The shared flex frame accommodates the header's actual height; wrapped or
enlarged identity text grows the header without clipping or covering the content.
The common UI is prop-driven and owns no identity request or authentication
store. The former page-local native select is replaced, including its exports
and examples.

An absent or invalid preference
means System. Explicit Light/Dark overrides device changes; System follows current
browser media and live changes. Theme changes preserve application identity, query
state, entered work and focus rather than remounting the subtree.

Host-only cookies `chapchoo-color-mode` and `chapchoo-color-mode-resolved` store the
preference and last resolution for one year at path `/`, SameSite=Lax, with Secure
on HTTPS. They contain only presentation enums and are intentionally readable by
JavaScript. General/Admin share them across same-host runtime mounts and ports;
different hosts remain independent. They are not session credentials or account
preferences. Cookies are the only persisted appearance source; no localStorage
mirror or default Mantine storage manager participates. Cookie denial permits an
in-page change but cannot guarantee persistence. Already-open independent tabs
are not actively synchronized through a cross-tab service.

Each root layout validates cookies and emits initial HTML/body mode attributes.
Request-time viewport metadata emits `color-scheme` in the initial head before
the CSS-dependent synchronous bootstrap, available while stylesheets are pending,
using a concrete explicit mode or `light dark` for System. The bootstrap and
live appearance bridge keep that metadata aligned with the selected preference;
it informs the browser's initial UA canvas independently of CSS evaluation.
The HTML element also carries inline `color-scheme` and a canonical-token
`light-dark()` canvas background, so the document canvas does not depend on
external stylesheets, the head bootstrap or hydration. System advertises both
schemes so the browser selects the canvas from current media even when the
cached resolution is absent or stale; explicit modes advertise only that mode.
The existing bootstrap
and live bridge update that same color scheme; the canvas follows automatically.
Explicit preferences win over a cached resolution. System uses the cached hint
or a light server placeholder until a synchronous head bootstrap resolves current
browser media before visible paint. A body-start prelude mirrors the result, and
the app provider maintains attributes and the resolution cookie through hydration
and live changes. The server cannot independently know a first-visit device
preference. With JavaScript disabled, explicit cookie modes still render correctly;
unknown System uses the server placeholder. Adaptive CSS variables use the HTML
mode as their sole styling authority. Shared presentation remains cookie-free;
the browser-safe web-core policy and app bridges own persistence/synchronization.
Browser acceptance checks computed canvas colors while external stylesheets are
held, then SSR body colors with application scripts blocked, and actual
content-frame background colors through refresh/hydration. Mode attributes alone
are not treated as proof of initial paint. These checks do not establish the
absence of a pre-content white frame on a physical iPhone Safari.

Light and warm olive-charcoal dark maps retain common layout/type metrics and
use separate filled-accent ink and readable accent-text roles. Menus/dialogs use
raised surface tones. Storybooks expose both appearances with static fixtures.

Routes remain locale-less. The display language selects the existing en-US,
ko-KR, ja-JP or fr-FR catalog independently of regional formatting. Forms,
statuses and failures use aligned localized messages. Normal UI shows the consequence/next action rather than raw
protocol reasons, stack traces or implementation metadata.
Collaborative context badges use localized Team terminology; personal contexts
retain the distinct personal-space label.

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Display language, time zone and regional format](display-settings.md) — Personal values, visitor choices and installation defaults are distinct authorities for the three independent display dimensions.
- [Runtime web configuration and failures](web-runtime.md) — Web deployment configuration is resolved at runtime; decoding and reporting preserve safe, truthful failure boundaries.

## References

- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
- [`typescript/packages/ui/src`](../../../../typescript/packages/ui/src)
- [`typescript/apps/web/src/app`](../../../../typescript/apps/web/src/app)
