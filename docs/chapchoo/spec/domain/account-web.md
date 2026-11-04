---
title: "General and Admin web ownership"
---

# General and Admin web ownership

General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.

## Applications and shared presentation

General and Admin are independently built Next.js/React/Mantine applications.
The general app provides password/configured-code login, email-bound signup,
password reset, personal/team context, membership and invitation operations.
Admin provides initial setup, administrator login and signup/reset-link
inventories with issuance/revocation. Team ownership does not grant server-wide
administration. Backend operations remain the authority for current permissions.

Personal Agent/Computer routes and read-only Admin Provider/resource inventory
reuse these same presentation/session boundaries. Their server-derived power,
connectivity, confirmation and bounded observation behavior is specified in
[Agent Computer web contracts](computer-web.md). Existing account/team routes
retain Team-management and profile entrypoints. General app entry selects a
validated recent Team instead of rendering the former combined contexts hub.

Both apps use the same workspace UI theme, semantic CSS variables, provider,
form controls, application shell and confirmation/feedback primitives. The
shared theme/token sources implement the selected DESIGN.md values. Product
routes and stories replace Hello World; there is no second Admin theme or Refine
layer. App containers own data/state/navigation; shared presentation receives
props/callbacks and imports no business RPC, environment, cookie or token store.
Both web apps support System, Light and Dark and load locally bundled Pretendard
with Geist Mono reserved for code. Pretendard uses the complete official
unicode-range subset set; the browser fetches only ranges used by rendered text,
without a whole-font or unused mono preload. Core subset 91 is preloaded with
anonymous CORS; optional font display retains readable fallback on a slow cold
view rather than replacing metrics late. The runtime mount is passed into
font stylesheet, font and decorative mascot URLs. The 72px mascot uses a 216px
alpha-preserving production derivative with normal composition. Six transparent
PNG masters and small WebPs share both appearances; unchanged generated originals
and extraction provenance remain in common UI assets.
Same-size single-line form fields and buttons share common frame heights through
the theme's Input and Button variables: xs 36px, default/sm 44px, md 48px,
lg 56px and xl 64px; narrow screens retain a 44px minimum. Textareas and
multi-value fields can grow rather than clipping content. General workspace
selection lives in the sidebar selector or inline mobile picker, not the
obsolete contexts-row hub.
Shared tables default to the `lg` (16px) horizontal cell spacing token for both
headers and body cells, including the outer columns. Explicit table spacing
props still override this default. The table's outer edge remains aligned with
the section; narrow views retain the existing bounded horizontal scrolling.

Desktop workspace sidebar and content regions scroll independently within the
viewport below the variable-height global header. The sidebar footer stays
independent of main-content length; the entire sidebar can scroll in short
windows or with long navigation. Narrow layouts use ordinary document scrolling.

The requester-supplied golden retriever is the default. Shared `Mascot` callers
can select bichon, corgi, dachshund, husky or border collie with a typed character
prop; this does not add a user-facing character picker.
General and Admin login entrypoints select one of the six characters on the
server, after their existing request-bound login-method read has resolved.
The selected string is serialized as a prop and rendered unchanged on the
client, including hydration and form-state updates. A fresh page request can
choose a new character; repeats are allowed. Signup, reset, setup and workspace
defaults remain the retriever. This follows Next.js's
[request-time random-value guidance](https://nextjs.org/docs/messages/next-prerender-random)
and React's [identical hydration output contract](https://react.dev/reference/react-dom/client/hydrateRoot#caveats),
without client-side random generation, seed storage or an effect-driven swap.
Both apps emit runtime-prefixed PNG/ICO favicon and 180px Apple Touch Icon links.
Their request-time manifests distinguish General and Admin names and scope
start URLs and icon URLs to each runtime mount. They use opaque 192/512px
regular icons and a separate 512px maskable safe-zone composition. Manifest
display stays `browser`; this adds home-screen identity artwork, not offline
support or standalone application behavior. All six characters have equivalent
icon assets and 1024px exports; the shared staging step also publishes the
retriever at the conventional mount-relative `/favicon.ico` path.
Narrow editable controls, including compound inner inputs and editable text
surfaces, use a 16px minimum while preserving larger user text and pinch zoom;
native selects retain their platform-owned line box. Visual principles remain
in root DESIGN.md. Shared button and chip labels use untrimmed intrinsic line
boxes; action controls grow for wrapped or enlarged text rather than clipping
glyph extents. Auth footers link only to other auth modes and omit the current
login/signup/reset destination. Desktop uses a side navigation pane. General
narrow views open navigation in a left Drawer instead of persistent rows above
primary content. Sectioned forms/rows/tables replace
the former scaffold layout without changing app-owned actions or permissions. General-purpose design work lives in the independent
[design reference workspace](design-reference.md); app Storybooks retain their
product-state review role.

## Verification boundary

Focused tests cover Origin/POST admission, cookie attributes/caps/disposition,
renewal ownership and concurrent context isolation, post-renewal domain failure,
revocation and authentication infrastructure failure, decoded response validation,
read-only SSR, safe source preservation and runtime prefix validation.
Backend persistence tests own overlap/rotation semantics. Web-core tests own
response disposition and its real-Chromium/HTTP logout boundary, including
independent app cookies and failed revocation. Browser cookie-arrival semantics
and generated-code round trips are not re-tested. Common UI runs its layout and
custom typography checks in its own fixture Storybook; app component checks
cover only app-owned state and composition. Tokens are not asserted as resolved
pixels in consumer tests. SSR and core product journeys retain their distinct
integration responsibilities.

Three root-executed product journeys passed against a fresh private PostgreSQL
installation and both production Next applications at `/app` and `/admin`:
bootstrap/signup/personal, invitation/transfer/leave, and reset/relogin/Admin
rejection. Confirmed reset completion remains visible after identity cache
clearing consumes/removes the proof preview. Owned process/DB cleanup succeeded.
Desktop1280×900 and mobile375-wide native PNGs were inspected with the real
GeistSans font, light theme, en-US, Asia/Seoul and DPR1; mobile had no document
horizontal overflow. No proof/password screen was captured.

Both apps' format/lint/typecheck, unit tests, production build and Storybook build
have passed in root-integrated validation, together with backend/protocol gates.
Independent review precedes the phase PR; exact-head CI follows PR creation.
The same three journeys also pass from the extracted x86-64 distribution using
bundled Node, with no host Node/pnpm/Turbo on the product PATH. This is local
artifact evidence, not a native ARM64 or deployed Helm/Argo acceptance claim.

Display-settings verification extends the existing signup journey with visitor
time-zone/region choices, account creation, panel/profile synchronization,
persisted refresh and script-blocked profile SSR. Admin default edits preserve
the administrator's personal time zone. The same three journeys and cookie
boundary passed again after the final production build. Focused component tests
cover independent suggestions, dismissal/value changes, save failure/explicit
retry, visitor following and 1280/390/320 keyboard/reflow. Native DPR1 product
screenshots cover profile/panel desktop and 390-wide mobile, plus Admin defaults
in dark appearance; seeded component renders cover all three suggestion rows.
All use loaded bundled Pretendard. These are local Chromium checks, not physical
Safari or deployment validation.

## Connections

- [Workspace context and navigation](workspace-navigation.md) — Navigation selects an accessible context without granting access to another owner’s private Agents.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.
- [Primary rendering and URL-owned state](web-rendering.md) — Server admission supplies primary content and stable selections; client queries own subsequent interactions rather than a second initial truth.
- [Display language, time zone and regional format](display-settings.md) — Personal values, visitor choices and installation defaults are distinct authorities for the three independent display dimensions.
- [Web appearance lifecycle](appearance.md) — Appearance owns system/light/dark resolution and initial paint; it is independent of account language, time zone and regional format.
- [Mascot playback and interaction](mascot-motion.md) — Shared mascot playback owns gesture timing, visibility and explicit replay while preserving the surrounding product layout.
- [Computer owner and operator web](computer-web.md) — Server-rendered owner controls and redacted administrator inventory project backend authority rather than creating browser lifecycle state.
- [Runtime web configuration and failures](web-runtime.md) — Web deployment configuration is resolved at runtime; decoding and reporting preserve safe, truthful failure boundaries.

## References

- [`typescript/apps/web`](../../../../typescript/apps/web)
- [`typescript/apps/admin-web`](../../../../typescript/apps/admin-web)
- [`typescript/packages/web-core`](../../../../typescript/packages/web-core)
- [`typescript/packages/ui`](../../../../typescript/packages/ui)
