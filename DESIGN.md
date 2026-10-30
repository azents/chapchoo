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
    "accentSoft": "#fbf2d0",
    "success": "#4f6b3b",
    "successSoft": "#edf3e6",
    "warning": "#805a1e",
    "warningSoft": "#fbf0d8",
    "danger": "#a24237",
    "dangerSoft": "#fbece8",
    "focus": "#866716"
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

Light mode is the selected condition. Dark mode has not been designed; do not
invert the palette and call that approved.

## Typography

Respect each platform instead of forcing one font everywhere:

- **Web:** Pretendard, including narrow mobile-web windows.
- **Initial iOS representation:** Helvetica for Latin and Apple SD Gothic Neo
  for Korean. Reference stack: `Helvetica, "Apple SD Gothic Neo", sans-serif`.
- **macOS / Android:** identity should remain coherent, but exact typography and
  native navigation are not selected. Do not invent them to complete a catalog.

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
height** (1.25 ratio), without enlarging the surrounding labels, buttons, control
frames or spacing. This applies to actual text/password/search/email/code inputs,
textareas, selects and editable text surfaces, including inner compound controls.
Preserve larger user text settings. This is the selected treatment for Safari's
focus zoom behavior, not a reason to disable pinch zoom or enlarge checkboxes,
toggles and other non-text controls. The requester selected the input-only
comparison on 2026-10-30 after reviewing both auth and chat examples.

Write natural, concise labels. Warmth is not baby talk or cheerful filler.
Explain scope, consequence, unusual behavior and recovery only where useful;
avoid repeating what the visible control already says. Preserve recognizable
technical terms when appropriate and test long Korean/Latin content.

## Layout

- Start with the user's work: content, current context, relevant state and next
  actions. Use stable alignment and proximity to explain relationships.
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

The selected family is five **abstract dog-like blob creatures**, not literal
retrievers: bichon, corgi, dachshund, husky and border collie. Use low, wide,
plump continuous soft matte volume with little visible texture. Differentiate
through broad colors/patterns and ear shape. Keep simple cream eye shapes/pupils;
avoid Ditto-style blank dots and realistic eye detail. No projecting muzzle,
inflated cheeks, hanging jowls, separate chin or shaggy hair outlines.

Use a character as a speaker/companion, near identity, a useful empty state or a
small greeting, not as wallpaper. On occupied screens it should not compete with
information. The selected PNGs in the common UI's `assets/mascots/` are raster
explorations, not claimed vector masters or a completed expression/motion library.
Do not infer breed-specific jobs, personalities or automatic multi-agent behavior.

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
  where they improve feedback; no motion library is approved yet.

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

Open: actual product IA/features, native navigation beyond the font choice, dark
mode, final icon/logo/motion systems and production illustration masters.
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
