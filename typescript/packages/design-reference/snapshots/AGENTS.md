# Product-design HTML snapshots

This directory owns frozen visual review artifacts, grouped by product design.
It is an archive, not a maintained component library or current-system spec.
Use the repository's `product-design-snapshot` skill for the complete workflow.

## Lifecycle

1. Start new concepts by copying the current UI into a disposable authoring
   surface. Verify the unchanged copy against the actual screen, then change only
   the requested parts using the latest design-reference and real components,
   fonts, theme and assets. Preserve approved flows and unchanged surrounding UI.
   Keep app-only fixtures with their app; reference must not import apps.
2. Iterate on actual rendered desktop/mobile feedback. Keep temporary authoring
   source until the requester explicitly confirms the design and scope.
3. Dump the confirmed rendered DOM, styles and required static assets as independent
   HTML. Verify independent, network-free replay before deleting authoring source.
4. Retain the HTML dump and local static assets; discard temporary components,
   stories, render loaders and source scripts. Do not discard shared components.
5. Old dumps do not change when shared components or design tokens change. Do not
   maintain, auto-regenerate or promote them into production. Create a new reviewed
   snapshot for a new design; do not retrofit historical artifacts.

## Layout and contract

- Keep a curated core, not an exhaustive historical mirror. Retain representative
  flows and only indispensable states; omit intermediate drafts, duplicate options
  and automatic desktop/mobile/error permutations. Report final file count and size.
- Share identical assets within a product folder. Keep only needed font subsets,
  artwork and icons; remove unused assets. Render PNGs and full source bundles stay
  outside Git. Selection and byte size are part of accepting a dump.
- Use `snapshots/<product-design>/<descriptive-state>.html` and a short `README.md`
  linking states, meaning, capture date, fidelity and confirmation scope.
- Keep needed assets local (or embedded), including fonts and icons; no CDN,
  OpenDesign iframe, live Storybook bundle, workspace path, backend or product
  module is required to view a dump. Keep the licenses for copied assets.
- A static snapshot's controls are illustrative. Represent important overlays,
  errors and transitions as separate linked states. Never claim it implements auth,
  permissions, persistence, infrastructure or real account data.
- Historical migration is an explicit exception to the new-design confirmation
  sequence: import old visual records without redesigning or inventing approval.
  Identify accepted direction, unconfirmed drafts and superseded alternatives.
- Do not commit tokens, cookies, real auth files, signed URLs, real personal data,
  private research text, source prompts or full private tool outputs.
- Capture/provenance notes describe evidence; they do not grant product authority.
  Approval, implementation and deployment are separate decisions.
- Exclude snapshots from Storybook catalogs, production imports/exports, component
  lint/format rewrites, design-token syncing and visual regression maintenance.
  Check new exports for self-containment, rendering fidelity and safety only.
- Generic design-reference examples outside this directory retain their existing
  component ownership and maintenance rules.

Root AGENTS.md links here for discovery. Keep these lifecycle rules scoped here.
