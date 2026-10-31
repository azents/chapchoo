# Chapchoo design

## Overview

**A small, soft companion in a calm working space.** Warm cream surfaces, a
butter-yellow action, clear quiet type and a simple dog-blob character bring
cheerfulness without turning useful tools into toys. Think of the selected
character-chat study's warmth, Amie's composed workspace and Dia's readable
conversation, not a generic colorful dashboard.

The requester selected this **visual tone** on 2026-10-30 KST. The chat study's
features differ from their product plan: its character picker, navigation,
research steps, suggested replies, cards and agent capabilities are illustrations,
**not approved product requirements**. New product screens in the reference
library are fictional compositions for applying the same tone.

This file is the single authority for visual design principles and discussion
context. It follows the useful section structure of Google Labs' draft DESIGN.md
format. The exact values below are canonical; the common-UI adapter is checked
against them. It does not replace technical Requirements, ADRs or Design records,
The requester subsequently authorized applying it to the existing general/Admin UI on 2026-10-30; this promotes the shared web theme and assets, not fictional product features.

### Canonical design tokens

The following JSON flow object is valid YAML. Its structure combines semantic
colors, spacing, rounded corners and platform typography with a small layout
extension. `check-design.mjs` verifies the implementation adapter matches it.
No design-version scheme or external format tooling is required.

```yaml
{
  "name": "Chapchoo",
  "colors": {
    "canvas": "#fdfcf8",
    "navigation": "#f3f2e9",
    "surface": "#ffffff",
    "overlay": "#ffffff",
    "surfaceWarm": "#fffefa",
    "selected": "#e9eadf",
    "border": "#e1e3d7",
    "borderStrong": "#9aab82",
    "ink": "#353a36",
    "secondary": "#626c58",
    "muted": "#707765",
    "accent": "#edca6e",
    "accentHover": "#e5bd54",
    "onAccent": "#51431e",
    "accentText": "#51431e",
    "accentSoft": "#fbf2d0",
    "success": "#4f6b3b",
    "successSoft": "#edf3e6",
    "warning": "#805a1e",
    "warningSoft": "#fbf0d8",
    "danger": "#a24237",
    "dangerSoft": "#fbece8",
    "focus": "#866716"
  },
  "darkColors": {
    "canvas": "#181c18",
    "navigation": "#1d221c",
    "surface": "#222820",
    "overlay": "#2c3528",
    "surfaceWarm": "#20261f",
    "selected": "#333d2d",
    "border": "#3f4a3b",
    "borderStrong": "#77856f",
    "ink": "#e6e9df",
    "secondary": "#b8c0af",
    "muted": "#a0ab94",
    "accent": "#d9bc78",
    "accentHover": "#e4ca8b",
    "onAccent": "#272718",
    "accentText": "#e5c987",
    "accentSoft": "#393421",
    "success": "#b4ce98",
    "successSoft": "#293623",
    "warning": "#e5c282",
    "warningSoft": "#3b3222",
    "danger": "#edb1a5",
    "dangerSoft": "#402b27",
    "focus": "#edca6e"
  },
  "typography": {
    "body-web": { "fontFamily": "Pretendard, sans-serif", "fontSize": "1rem", "fontWeight": 400, "lineHeight": 1.7 },
    "body-ios": { "fontFamily": "Helvetica, \"Apple SD Gothic Neo\", sans-serif", "fontSize": "1rem", "fontWeight": 400, "lineHeight": 1.7 }
  },
  "spacing": { "xs": "4px", "sm": "8px", "md": "12px", "lg": "16px", "xl": "24px", "xxl": "32px" },
  "rounded": { "control": "12px", "surface": "18px", "avatar": "24px" },
  "typeScale": { "caption": 12, "small": 14, "body": 16, "title": 24, "display": 32 },
  "layout": { "navigation": 258, "readingMeasure": 790, "mobileGutter": 16, "touchTarget": 44 }
}
```

These concrete values encode the approved direction; they were extracted and
refined for usable contrast, not separately approved one by one. Layout numbers
are reference starting points, not universal product dimensions. Components use
semantic tokens through the common-UI Mantine adapter instead of scattering
sample colors through production code.

## Colors

- **Canvas / navigation:** almost-white warm cream for work; a slightly
  olive-tinted neutral for quiet navigation. Distinguish regions without drawing
  heavy boxes around everything.
- **Surface / surfaceWarm:** raised or interactive areas. Group an actual output
  or meaningful object, not every paragraph inside a card.
- **Ink / secondary / muted:** warm charcoal and olive hierarchy. Secondary
  information is quieter, not illegible. The liked sample's very pale small
  labels are not a requirement to preserve poor contrast.
- **Accent / onAccent:** butter-yellow primary actions with dark warm text.
  White on butter is not an acceptable default. Use the accent sparingly so the
  important action remains easy to find.
- **Success / warning / danger:** semantic tints with readable text and an icon
  or label. Brand yellow alone does not identify a warning. Do not communicate
  status only through color.

Both light and warm olive-charcoal dark appearances are selected for the existing
general/Admin web surface as of 2026-10-31 KST. Dark mode uses tonal separation
and brighter raised overlays, subdued butter actions and ivory text. Use
`accentText` for links/text-like actions and `onAccent` for ink on filled butter.
Preserve navigation selection with background/text, without decorative markers.
System is the default; an explicit Light or Dark choice overrides it.
Theme changes do not change layout, type metrics or business behavior.

## Typography

Respect each platform instead of forcing one font everywhere:

- **Web:** Pretendard, including narrow mobile-web windows.
- **Initial iOS representation:** Helvetica for Latin and Apple SD Gothic Neo
  for Korean. Reference stack: `Helvetica, "Apple SD Gothic Neo", sans-serif`.
- **macOS / Android:** identity should remain coherent, but exact typography and
  native navigation are not selected. Do not invent them to complete a catalog.

The requester reaffirmed Pretendard for both English and Korean on 2026-10-31
KST after comparing it with Geist plus system Korean and Noto Sans plus
Noto Sans KR on the same login/chat examples.

Viewport width is not platform identity. Linux does not have the Apple fonts;
its substitute render cannot establish iOS typography. Use an Apple device to
review those fonts and do not redistribute their files. The bundled Pretendard
font retains its SIL Open Font License.

Use a small semantic ladder: 12px captions, 14px labels/compact supporting copy,
16px body, 24px titles and 32px optional greeting/display. Body line height around
1.7; headings around 1.3. Medium/semibold headings and regular text establish
hierarchy before color or ornament. Do not use mono for ordinary labels or agent
identities; actual code can use a code font. These roles are not permission to
make long-form mobile content tiny.

Mobile editable controls use at least **16 CSS px**, with a default **20px line
height** (1.25 ratio), without enlarging the surrounding labels or spacing.
Same-size form fields and actions share a common frame-height scale (xs 36px,
sm/default 44px, md 48px, lg 56px, xl 64px), with a 44px minimum on narrow
screens. This applies to actual text/password/search/email/code inputs,
textareas, selects and editable text surfaces, including inner compound controls.
Preserve larger user text settings. This is the selected treatment for Safari's
focus zoom behavior, not a reason to disable pinch zoom or enlarge checkboxes,
toggles and other non-text controls. The requester selected the input-only
comparison on 2026-10-30 after reviewing both auth and chat examples.
The requester added shared form-frame alignment on 2026-10-31. Multiline and
multi-value fields may grow with their content; wrapped action labels retain
their complete line boxes.

Textual controls keep a complete line box, including descenders and Korean glyph
extents. Use a 1.5 line-height for action labels and let controls grow when text
wraps or enlarges. Do not trim labels to Latin cap/baseline metrics or mask glyphs
with a fixed-height inner label. Keep intentional data truncation separate from
action-label typography; do not compensate with screen-specific text offsets.

Write natural, concise labels. Warmth is not baby talk or cheerful filler.
Explain scope, consequence, unusual behavior and recovery only where useful;
avoid repeating what the visible control already says. Preserve recognizable
technical terms when appropriate and test long Korean/Latin content.

## Layout

The existing web apps share a sticky global header across authentication, setup,
general workspaces and Admin. The brand stays left; an icon-only appearance
button stays at the far right (24px desktop inset, 16px narrow inset). This is
application chrome, not a form field or page-specific action. Use one 44×44 hit
area, a localized accessible name/hover title, monitor/sun/moon preference icons,
and a menu with System/Light/Dark and a current-selection check. The requester
selected this composition on 2026-10-31 KST. Account identity/logout sit before
the icon on desktop; narrow layouts keep them in a separate context row below
navigation. The header grows for wrapped or enlarged account text, with the
content frame filling the remaining viewport instead of subtracting a fixed
header height. Preserve modal layering above the header and body typography.

- Start with the user's work: content, current context, relevant state and next
  actions. Use stable alignment and proximity to explain relationships.
- Desktop workspaces give the sidebar and main content independent scroll
  regions beneath the variable-height global header. Main content length does
  not move the sidebar footer. Short windows and long navigation scroll the
  complete sidebar; narrow layouts retain normal document scrolling.
- Divider-list rows use transparent backgrounds and the same inline content
  baseline as neighboring headings and sections. Do not turn a zero-inset row
  into a filled box; use the common Paper row variant for this treatment.
- Bounded tables keep their outer edges on the surrounding content grid.
  Header and body cells share a 16px (`lg`) horizontal inset by default,
  including the first and last columns. Explicit density choices can override
  the common spacing; use bounded horizontal scrolling on narrow screens.
- Desktop may have quiet side navigation and a bounded reading measure. The
  selected sample used about 258px navigation and 790px reading width; other
  workflows may need different measures.
- Use a 4/8/12/16/24/32 rhythm. Keep controls close to affected content and choose
  rows, lists, panes or inspectors when those communicate structure better than
  cards. Cards are appropriate for meaningful grouped outputs or repeated objects.
- On mobile, prioritize useful information density. Remove empty side gutters,
  excessive padding, oversized decoration and duplicated explanation before
  shrinking text. In conversation-like layouts, avatar/name belong **above the
  body at upper left**, then text/results use the available width. Keep essential
  status/actions accessible when content reflows or moves into disclosure.
- Begin narrow references with 16px outer gutters; evaluate actual content rather
  than enforcing the same padding everywhere. Touch hit areas may exceed the
  visible icon without inflating the whole interface.
- Preserve text enlargement and long strings. For web, check 320 CSS px reflow
  without page-level two-axis scrolling, except bounded genuinely two-dimensional
  content such as maps or data tables. Check text at 200% and meaningful empty,
  full and error states. Do not hide decision-critical columns/status merely to
  make a screenshot fit.
- Native representations must account for safe areas, text scaling and keyboard
  occlusion. A responsive web mock is not native platform verification.

## Elevation & Depth

Use tonal layers, subtle borders and shallow shadows only where they clarify
layering. Reference surface shadow: `0 4px 18px rgb(53 58 54 / 4%)`. A calm pane or
separator is often enough. Avoid heavy floating dashboards, decorative gradients,
colored card mosaics and deeply nested rounded boxes.

Requester-supplied Pickle and Apple Liquid Glass were taste references, not a
requirement to blur everything. If explored, materials belong selectively to
navigation/control layers without reducing content clarity. The selected cream
composition does not need glass.

## Shapes

Soft rectangles rather than pills everywhere: controls around 12px radius,
grouped surfaces around 18px, identity art may be rounder. Keep a consistent
family while allowing role-specific differences. Simple line icons should have
consistent weight and clear meaning. The exact icon library and logo remain open;
the study's temporary star/wordmark is not a finalized brand mark.

## Components

### Drawers

Drawer bodies begin with an `xl` (24px) top inset below the header, including
when the title wraps on narrow screens. This is shared theme behavior, not a
margin on the first paragraph; pages retain their own internal content spacing.
Keep panel titles short and task-level, such as “Display settings”.

### Actions and inputs

- Keep one clearly dominant next action per local task; secondary controls stay
  quiet and consequential destructive actions remain visually distinct.
- Show normal, hover, focus, pressed/selected, disabled, pending and error states
  where meaningful. Disable unsafe repeat submission while an operation is
  pending; a spinner alone must not erase the action's purpose.
- Use persistent input labels, associated help/error text and familiar semantics.
  Keep entered work on recoverable failure. Place actionable errors by the
  affected input/action, not only in a disappearing toast.
- Provide cancel/back or undo when meaningful. Confirm consequential irreversible
  actions, not every ordinary edit. Reuse established dialog focus handling.

### Content and feedback

- Keep orientation and relevant save/progress state visible. Distinguish loading,
  empty, partial/unavailable, completed and error states when their next actions
  differ. Never imply real success before it is established.
- Use concise status messages accessible without unnecessarily stealing focus.
  Diagnostic internals belong in secondary disclosure, not ordinary status copy.
- Prefer recognition over recall: show selected scope, active view and necessary
  context where the user acts. Follow familiar interaction conventions before
  inventing novel navigation.

### Identity and illustration

The selected family is six **abstract dog-like blob creatures**: the original
golden companion (called the retriever), plus bichon, corgi, dachshund, husky and
border collie. Chapchoo means golden retriever: the retriever is the primary
identity, asset `01`, and first in all character catalogs. The requester reaffirmed the original retriever as the default on
2026-10-31 and supplied its recovered PNG. Preserve that exact character and its
texture rather than generating a new retriever. Use low, wide,
plump continuous soft matte volume with little visible texture. Differentiate
through broad colors/patterns and ear shape. Keep simple cream eye shapes/pupils;
avoid Ditto-style blank dots and realistic eye detail. No projecting muzzle,
inflated cheeks, hanging jowls, separate chin or shaggy hair outlines.

Use a character as a speaker/companion, near identity, a useful empty state or a
small greeting, not as wallpaper. On occupied screens it should not compete with
information. The selected PNGs in the common UI's `assets/mascots/` are raster
explorations, not claimed vector masters or a completed expression/motion library.
Do not infer breed-specific jobs, personalities or automatic multi-agent behavior.
Use derived transparent masters in `assets/mascots/transparent/` across both
appearances with normal composition. Small production WebPs preserve alpha;
original generated PNGs and generation provenance remain unchanged. General,
Admin and the reference default to the retriever; login screens are the exception,
with one of six characters selected on the server for each fresh page request.
The client retains that serialized choice through hydration and form edits.
Explicit character props can
render the other five without introducing a product character picker. Browser
favicons and home-screen icons derive from the same transparent masters.
Apple icons use an opaque unrounded cream canvas; maskable icons have a separate
safe-zone composition. See the common UI's `assets/icons/README.md` for exports
and reproducible generation.

Use motion when the character represents **the Agent as the subject of
interaction**. Use static artwork when the character is an accessory, supporting
illustration or decoration unless an explicit placement decision selects motion.
On 2026-10-31, after reviewing desktop/mobile screenshots, the requester selected
all existing General/Admin authentication-heading and sidebar-companion placements
for this motion. Sidebar companions retain their existing narrow-screen hiding.

The requester approved the grounded stretch gesture on 2026-10-31: **5% vertical
extension, 1200ms active motion, extension displacement 30 degrees clockwise from
upward vertical**, a brief hold and one small damped return bounce. The feet stay
on a level ground line. The character initiates the gesture rather than jumping
or appearing externally pulled. The return dips by 0.75% and rebounds by 1%
before settling; repeated elastic wobbling should not dominate. Reference
automatic playback rests for six seconds between gestures. Click or touch on a
resting animated character starts the gesture immediately; Enter/Space provides
the same replay through a named button. Input during an active gesture is ignored
without restarting or queuing another gesture. Hover has no playback effect.
Pausing automatic playback permits explicit one-shot replay; reduced motion,
offscreen artwork and hidden documents stop all playback.

The reusable React implementation belongs to common web UI and is imported by
the independent design reference's actor/accessory examples. Production motion
retains the static artwork's square box and WebP asset, anchoring at the artwork's
actual contact line inside that box. Static `Mascot` remains available for other
accessory placements. This rollout does not implement Agent lifecycle behavior.

### Accessible operation

- Aim for applicable WCAG 2.2 AA: 4.5:1 normal-text contrast, 3:1 qualifying large
  text and 3:1 meaningful UI/state graphics. Decorative dividers do not need to
  look like interactive boundaries.
- Use semantic controls, accessible names, associated labels, logical headings
  and keyboard operation. Keep focus visible and not obscured by sticky layers.
  Modal interactions contain and restore focus and have a suitable exit.
- WCAG AA pointer targets are at least 24×24 CSS px with defined exceptions;
  **44×44 is a practical comfortable mobile target here, not a claimed AA
  mandate**. Native points and CSS pixels are distinct; validate actual platforms.
- Avoid hover-only essentials, drag-only controls without alternatives and
  color-only status. Respect reduced motion. Use short subtle transitions only
  where they improve feedback; the grounded mascot gesture is approved for its
  interaction-subject role, without selecting a third-party motion library.

## Do's and Don'ts

- Do let the character and a few warm accents carry the cheerfulness.
- Do keep populated workspaces calm, content-first and operationally clear.
- Do change prose, canonical tokens and rendered examples together when refining
  a design rule; `check-design.mjs` catches value drift.
- Don't adopt the fictional screens' capabilities or navigation as product plans.
- Don't promote reference-local components into common UI without selecting them
  for actual shared-web reuse. Existing common-UI contracts stay owned there.
- Don't treat inherited Azents styling, a merge or a screenshot as design approval.
- Don't disguise font substitution or web-rendered samples as native verification.

## Decision context

The discussion first explored monoline golden-retriever icons. Those became too
stiff or emoji-like. A dimensional furry close-up improved softness, but inflated
cheeks/chin and increasingly realistic portraits were rejected. The requester
then abandoned literal retriever anatomy for a dog-like blob. They chose the
lower, wider composition over a taller interpretation, requested five breeds,
rejected excessive fur and selected the smoother five-variant set. That set's
cheerfulness became the UI benchmark. The warm chat composition was liked on its
first presentation; subsequent confirmed adjustments were platform fonts and
identity above mobile content. Later the requester explicitly selected the tone
for a design system while declining the sample chat's feature plan.

Earlier discussion established code-authored Storybook, not mandatory Figma or
Pencil originals; cross-platform identity; real reference research; and separate
reference/common-UI/app QA ownership. Primer/Storybook research concerned editable
Git-managed context and usage guidance, not GitHub's visual style or adoption of
Primer React. Current product implementation remains accounts/teams, not the
future channel/agent vision. None of this authorizes invented product flows.

Open: actual product IA/features, native navigation beyond the font choice,
extended logo/motion systems and editable vector/3D illustration masters.
The existing general/Admin web surface now uses the selected shared theme;
further feature-specific promotion remains separate from fictional compositions.

## References and implementation map

- [Reference library](typescript/packages/design-reference/README.md): runnable
  Storybook, ownership, stories and review conditions; no duplicate principles.
- [Mascot generation record](typescript/packages/ui/assets/mascots/GENERATION.md):
  original prompts, code-verified model identifier, provenance limits and hashes.
- [Token adapter](typescript/packages/ui/src/designTokens.ts) and
  [Mantine adapter](typescript/packages/ui/src/theme.ts):
  shared-web implementation and reference import, validated against this document's values.
- [Google Labs format](https://github.com/google-labs-code/design.md/blob/main/docs/spec.md)
  and [philosophy](https://github.com/google-labs-code/design.md/blob/main/PHILOSOPHY.md):
  specific visual context plus values and rationale. It is a draft, not automatic
  discovery in every tool. Our layout/type-scale extensions are local.
- [WCAG 2.2](https://www.w3.org/TR/WCAG22/),
  [reflow](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html),
  [target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
  [focus](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html)
  and [status](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html):
  accessibility source, not a claim that reading it proves implementation conformance.
- [Primer's mobile input change](https://github.com/primer/css/pull/430) and
  [Nuxt UI input styling](https://github.com/nuxt/ui/blob/200b0bfb7a5666c870742f3f43c83cf09234aed9/src/theme/input.ts):
  evidence for keeping editable text at 16px without inflating the surrounding UI.
  Safari's automatic focus zoom threshold is browser behavior, not a universal
  Apple visual-design mandate. Scale/viewport workarounds were researched but not selected.
- [NN/g heuristics](https://www.nngroup.com/articles/ten-usability-heuristics/):
  visibility, user control, consistency, recognition, prevention and recovery.
  Apply as heuristics, not invented numeric laws.
- [Amie](https://amie.so/), [Granola Chat](https://www.granola.ai/chat),
  [Dia](https://www.diabrowser.com/), [Littlebird Hover](https://www.littlebird.ai/features/hover):
  quiet hierarchy, readable conversation and lightweight contextual help.
- [Pickle](https://www.pickle.com/), [Claude Cowork](https://claude.com/product/cowork),
  [Manus Desktop](https://manus.im/desktop),
  [Apple materials](https://developer.apple.com/design/human-interface-guidelines/materials):
  earlier taste research. Official marketing/demo samples were examined; this
  does not claim authenticated product testing or native Apple sample execution.
