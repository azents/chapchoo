# Writing and applying DESIGN.md

This is Chapchoo's visual-design authoring and review guide, researched on
2026-10-30. It adapts Google Labs' **draft** DESIGN.md format and established
accessibility/usability guidance. It is not a new technical architecture process,
product feature spec, design approval, or replacement for existing conventions.

## Document responsibilities

- Root `DESIGN.md`: concise canonical visual identity and reusable application
  rules. Link it from Agent instructions so people/tools actually read it.
- Reference-local context: discussion evidence, rejected approaches, source links
  and remaining questions. Keep long history out of the everyday design brief.
- Code tokens and rendered Storybook examples: concrete design application, with
  a documented canonical value source and synchronization check. Avoid two
  independently editable authorities.
- `docs/chapchoo/design/`, Requirements and ADRs: the existing technical process.
  A visual DESIGN.md does not rewrite those records or bypass that process.
- Design-reference owns experiments, fictional compositions and non-web
  representations. Common UI owns existing shared web contracts. Apps own
  product functionality and app UI QA. Design approval alone is not promotion.

## Content and structure

Use this core order from the upstream format. Add focused sections where the
product needs them; do not create a large checklist of irrelevant categories.

1. **Overview:** describe a specific visual world, audience/task and intended
   feeling. A concrete approved reference is more informative than generic
   adjectives such as modern, clean or premium. State what is approved and what
   is merely illustrative.
2. **Colors:** semantic roles, foreground/background pairs, values and exceptions.
   Explain where each color belongs, rather than listing swatches alone.
3. **Typography:** roles, platform font stacks, sizes, weights, line heights and
   language coverage. State real-font rendering limitations.
4. **Layout:** spacing/density, reading measure, responsive behavior and content
   priorities. Explicitly describe how narrow screens preserve information.
5. **Elevation & Depth:** tonal hierarchy, borders, layering and shadows; flat
   designs can explain their alternatives to elevation.
6. **Shapes:** control/surface radii, silhouettes and any intentional differences.
7. **Components:** meaningful variants, states and use rules. Include identity,
   inputs, actions and data presentation as relevant, not every possible widget.
8. **Do's and Don'ts:** a short actionable set of distinctive guardrails.

Add accessibility/interaction, illustration, iconography or motion guidance when
useful. Explain both *what* and *why*. Describe missing decisions honestly instead
of inventing dark mode, platform navigation or business behavior to fill sections.

### Tokens and prose

- The upstream format supports YAML front matter and/or fenced YAML blocks. Give
  the design a name; metadata `version` is optional and is not needed here. Do not
  introduce product/design versioning as a side effect of adopting this format.
- Use valid named token groups such as `colors`, `typography`, `spacing`,
  `rounded` and `components`. Use semantic names and role references such as
  `{colors.primary}`. Quote hex strings and use units for dimensions.
- Define each top-level YAML key at most once across blocks. Avoid duplicate
  headings and broken references. Omit irrelevant sections with a short reason.
- Exact values belong in one canonical source. If root DESIGN.md embeds the
  normative values, implementation adapters must derive from them or verify
  parity. If a linked code/token file is canonical, identify that explicitly and
  do not claim full upstream token-schema conformance for an index-only document.
- Tokens give precision; prose supplies intent and application. Neither a giant
  token dump nor adjectives alone is a sufficient design brief.
- Tooling is optional. Do not add a generator, external CLI dependency, token
  transport pipeline or design versioning merely to adopt a Markdown format.
- Upstream examples are examples, not Chapchoo defaults. In particular, do not
  import their generous cards, display sizes, mandated number of type roles or
  two-weight limits as universal rules.

## Quality principles

These are implementation/review expectations compatible with existing rules, not
claims that every current screen already conforms. Test actual applicable states;
a source link, screenshot or linter pass alone is not a conformance certificate.

### Useful density and responsive content

- Optimize the primary task, not decoration. Keep controls near affected content;
  prefer rows, lists or panes when they communicate structure more efficiently
  than nested cards. Cards remain appropriate for genuine repeated object tiles.
- **Mobile information density is a core product principle.** Remove wasted side
  gutters, repeated labels and oversized decoration before shrinking text. In a
  conversation-like view, identity goes above the full-width body at upper left.
- Reflow secondary material into accessible disclosure or a subsequent view;
  retain access to essential status, actions and content. Dense is not tiny.
- For web, check reflow at 320 CSS pixels without page-level two-axis scrolling;
  genuinely two-dimensional content (for example a data table or map) can have a
  bounded scrolling region. Preserve enlarged text (200%) and long/localized
  strings. Device widths are test conditions, not new product breakpoints.
- On native representations, account for safe areas, text scaling and keyboard
  occlusion. A web-rendered native concept is not proof of native correctness.

### Accessible hierarchy and control

- Meet WCAG 2.2 AA where applicable: normal text contrast at least **4.5:1**,
  qualifying large text **3:1**, meaningful UI boundaries/state graphics **3:1**.
  Decorative separators need not masquerade as actionable boundaries. Do not
  preserve illegible pale captions just because they appeared in a liked mock.
- Use semantic controls, associated persistent labels, accessible names and
  logical headings. Icons need names; tooltips do not replace labels for touch.
- Make keyboard focus visible and not hidden behind sticky UI. Keep reading and
  focus order meaningful; modal interactions must contain/restore focus and have
  an appropriate exit.
- WCAG 2.2 AA target size minimum is **24×24 CSS pixels**, with defined exceptions
  including spacing. This is not a 44px AA mandate. Chapchoo's comfortable touch
  baseline can be **44×44** for independent mobile controls; preserve density via
  hit area and spacing rather than giant icons. Native points and CSS pixels
  are different units; validate the eventual platform separately.
- Pair color with text, shape or an icon for meaningful status. Keep content
  usable with text enlargement and reduced motion. Avoid hover-only essentials,
  motion-only state and drag-only actions without an alternative.

### Feedback, errors and user agency

- Show relevant system state and immediate action feedback. Distinguish loading,
  empty, successful, partial/unavailable and error states when they differ for
  the user's next action. Do not invent these states' backend contracts in mocks.
- Do not communicate real success before it is established. Mark fixture actions
  as demonstrations; distinguish a concept from a product screenshot.
- Preserve entered work on recoverable failures. Explain the problem near its
  affected input/action with a useful recovery; keep diagnostics secondary.
- Keep labels familiar, concise and consistent. Explain non-obvious consequences,
  not what the control already says. Do not add filler to make the UI friendly.
- Provide cancel/back or undo where meaningful; use explicit confirmation for
  consequential irreversible actions rather than warning on every ordinary edit.
- Surface important status updates accessibly without unnecessarily stealing
  focus. Avoid toasts as the only location for errors requiring action.

### Cross-platform and truthful review

- Respect requested platform type: web Pretendard, initial iOS Helvetica plus
  Apple SD Gothic Neo. Viewport width is not platform identity. Preserve native
  conventions where selected; do not force pixel-identical native navigation.
- Inspect actual rendered components at stated viewport/DPR/font conditions.
  Do not upscale/downscale comparison captures or call font substitution native
  verification. Preserve useful context around overlays and focus.
- Keep a shared visual grammar while allowing platform differences. Do not
  silently invent typography/navigation for platforms still undecided.

## Compatibility review

- Existing frontend-design guidance already favors operational clarity, quiet
  hierarchy, restrained borders and useful copy: retained and made concrete.
- Existing common UI/reference/app boundaries and technical approval process:
  unchanged. Root DESIGN.md documents visual identity, not architecture.
- Existing immutable Requirements/ADR/Design history: untouched.
- Requester-selected mobile density and font stacks: explicit; no spacious-card
  pattern or system-font substitution is imported to contradict them.
- Existing Mantine token discipline: unchanged. Reference token definitions may
  contain actual values; downstream implementation must use its semantic adapter
  rather than scatter hardcoded colors through app code.
- No adoption of Stitch services, GitHub/Google visual identity, new UI framework,
  mandatory token generator or versioning is implied.

## Sources and interpretation

1. [Google Labs DESIGN.md specification](https://github.com/google-labs-code/design.md/blob/main/docs/spec.md)
   — draft structure, YAML/prose, semantic tokens, references and duplicate keys.
2. [DESIGN.md philosophy](https://github.com/google-labs-code/design.md/blob/main/PHILOSOPHY.md)
   — specific references, rationale over adjectives, concise negative constraints.
3. [Google's announcement](https://blog.google/innovation-and-ai/models-and-research/google-labs/stitch-design-md/)
   — portable shared visual context, published as a draft rather than a standard
   every tool necessarily loads automatically.
4. [WCAG 2.2](https://www.w3.org/TR/WCAG22/)
   — accessibility criteria, including contrast, text resizing, focus, keyboard,
   labels and status. Understanding pages below explain rather than replace it.
5. [Reflow](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html),
   [Target size minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
   [Focus not obscured](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html),
   [Status messages](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html)
   — narrow-layout, target-size nuance, sticky UI and accessible feedback.
6. [Nielsen Norman Group's usability heuristics](https://www.nngroup.com/articles/ten-usability-heuristics/)
   — visibility, real-world language, user control, consistency, prevention,
   recognition, efficiency, minimalist focus and recovery. These are heuristics,
   not numeric mandates or a substitute for observed user needs.

Apple's accessibility page required client-side rendering in this research fetch;
no unverifiable numeric Apple claim is used here. The 44×44 touch baseline is our
practical design target, not falsely attributed as WCAG AA or checked iOS proof.
