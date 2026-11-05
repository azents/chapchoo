---
name: product-design-snapshot
description: Start Chapchoo product-design concepts by copying the current UI, iterate on rendered feedback, and freeze explicitly confirmed designs as independent HTML snapshots. Use for mockups, wireframes, UI variants, screenshot review, roadmap imports, and design finalization; not production feature implementation.
---

# Product Design Snapshot

## Authority and destination

Read `typescript/packages/design-reference/snapshots/AGENTS.md`, the reference
package's `AGENTS.md`/`README.md`, and root `DESIGN.md` first. Follow the existing
product-design brief and requester decisions; an implementation can have drifted
from an approved roadmap. Separate target behavior, this delivery's scope, and
current implemented behavior before drawing.

A concept, design approval, HTML export, PR, implementation and deployment are
different outcomes. Never advance one to the next without authorization.

## 1. Prepare the actual baseline

- Start by copying the current UI into a disposable authoring surface. Copy the
  actual screen composition, components, layout, styles, content and state setup;
  do not start from a blank canvas or redraw a similar-looking screen. Render
  this unchanged copy first and compare it with the current UI before modifying
  only the requested parts.
- Locate the latest design-reference representations and real components,
  providers, theme, fonts, icons, assets and responsive rules.
- Define requested changes and invariants. Preserve surrounding layout, fields,
  content and behavior unless the requester changes them. Compare with the approved
  roadmap as well as current code; do not replace a step flow with a form silently.
- Use the smallest working render: existing story, temporary story around the real
  component, isolated app fixture, then reference-local experiment. Do not rebuild
  an existing screen in standalone HTML for convenience.
- Keep app-owned components in app fixtures; reference may import shared UI but
  must not import apps. Keep fixture state local with fake data and callbacks,
  without backend requests, authentication or resource mutation.
- Temporary authoring source is disposable. Keep it while feedback is active,
  outside production entrypoints. It is not a second maintained implementation.

## 2. Produce and review concepts

Create the meaningful normal, empty, selection, progress and recovery states.
Use restrained task-oriented UI and the existing visual system. Prefer working
controls and state-aware actions over explanatory banners. Preserve material
costs, destructive effects, field labels and accessible names. Repeated accounts
can be cards; do not turn every paragraph into a card or add marketing decoration.

Compare stable numbered options under identical content, viewport, theme, locale,
font, zoom and DPR. Change one material decision axis at a time. Include desktop
and mobile for each principal flow when requested. Do not invent a native mobile
interaction merely because a window is narrow.

Render at explicit native resolution, wait for fonts/images, and inspect every
final PNG. Check overflow, modal/footer clipping, keyboard/focus, long labels,
empty/error states, returning to the original task, and responsive order. Correct
failed renders before presentation. Never stretch or resample screenshots; any
composition uses original pixels plus external padding only.

Label evidence accurately:

- **Component render**: actual component/providers executed, with proposed additions
  identified separately.
- **Product screenshot**: actual running application.
- **Concept mock**: standalone approximation or proposal.

Attach actual images and state conditions/limits once. A local prototype check is
not real authentication, product integration testing or user validation.

## 3. Iterate and confirm

Restate feedback as the exact changed variable and preserved invariants. Re-render
under the same conditions. Keep pending requests cumulatively. Update the product
case with current behavior, not a growing transcript of discarded alternatives.

Obtain explicit confirmation of the design and its scope before freezing a newly
created design as confirmed. Approval of one state does not approve missing states,
future features, implementation or deployment. If approval is withdrawn, mark the
candidate unconfirmed and stop technical handoff/implementation.

## 4. Dump confirmed results

After confirmation, freeze the rendered result, not the React source or a loader
that imports the product at viewing time. Store it under
`snapshots/<product-design>/<descriptive-state>.html`, with required local static
assets. Curate only the core states needed to explain the retained design; HTML
is a visual snapshot, not a functioning prototype. Use separate state pages for
those selected states. Do not import every draft, variant, responsive combination
or error state merely because it exists.

Bundled exporter: `scripts/freeze-page.mjs`. From the repository root with the web
workspace dependencies installed:

```sh
node .claude/skills/product-design-snapshot/scripts/freeze-page.mjs \
  --url http://127.0.0.1:6006/iframe.html?id=YOUR_STORY \
  --output typescript/packages/design-reference/snapshots/PRODUCT/STATE.html \
  --viewport 1440x1000
```

The exporter captures the resolved DOM, form values and CSS, localizes image/font
resources, strips authoring scripts/handlers/runtime imports, and verifies an
HTTP-served replay without external network requests. Each dump records one reviewed
viewport and its locale/color/motion conditions. Original responsive CSS is retained
for fidelity; export desktop and mobile as separate named pages without claiming
unreviewed widths are verified. The adjacent PNG is verification
output; keep it outside Git after visual review. Run the exporter tests with
`node --test .claude/skills/product-design-snapshot/scripts/freeze-page.test.mjs`.
Do not commit its temporary
server or authoring harness. If it cannot preserve a surface (for example canvas
or inaccessible cross-origin frames), fix/export that surface deliberately and
validate it; do not silently omit content.

Verify before accepting the dump:

1. Serve it independently of Storybook, OpenDesign, product bundles and backend.
2. Block external requests; fonts, icons and artwork must still load.
3. Re-render at the reviewed desktop/mobile conditions, inspect final PNGs and
   compare with the approved render. Record hashes/viewport only in capture notes.
4. Check that the output contains no credentials, private signed URLs, live API
   calls, external stylesheet/script imports, or workspace/user runtime paths.
5. Confirm known interactions are represented as separate static states. Do not
   call a script-free snapshot an executable behavior test.

Before committing, remove unused font subsets and image assets, deduplicate shared
bytes within the product folder, and report the actual file count and total size.
The exporter localizes resources but is not an archive-selection or size-budget
authority: inspect its output and retain only required assets. Verification PNGs,
full source bundles, intermediate alternatives and exhaustive state matrices stay
outside Git. Prefer a small representative collection over a historical mirror.

## 5. Retain the result, discard authoring source

Only after dump verification, remove temporary React components, stories, loaders,
source scripts and worktrees created for the concept. Preserve shared production
and design-reference components: they are not disposable authoring source.
Do not add snapshots to Storybook, product bundles or a regression suite that
requires updating them after component changes.

Write a short product-folder index with entry links, meaning, confirmation scope,
source date and fidelity limitations. Historical imports retain their actual
accepted/proposed/superseded status; importing an unconfirmed artifact does not
confirm it. Do not copy sensitive private research into this public repository.

For a new design iteration, start from current components, obtain feedback and
confirmation, then create a new descriptive snapshot. Do not retrofit old dumps
to current components or create a version-management subsystem.

## Delivery

Report the archived relative paths, reviewed states, confirmation boundary and
validation performed. For requested repository delivery, open a focused PR with
rules/imports only; keep feature implementation separate. Include a direct PR
link and state whether CI/reviews finished. Merge requires explicit approval.

Useful optional visual references:

- `references/openai-delightful-frontends.md`
- `references/product-admin-design-laws.md`
