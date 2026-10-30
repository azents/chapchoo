---
title: "Design Reference Workspace"
created: 2026-10-30
tags: [design-reference, frontend, developer-tooling]
spec_type: domain
domain: design-reference
code_paths:
  - typescript/packages/design-reference
  - typescript/scripts/design-boundaries.mjs
  - typescript/scripts/design-boundaries.test.mjs
  - package.json
  - .github/workflows/ci.yaml
last_verified_at: 2026-10-30
spec_version: 1
---

# Design Reference Workspace

`@chapchoo/design-reference` is a private, independent React/Vite Storybook for
general-purpose cross-platform design references. Its only story source is its
own package. General/Admin Storybooks retain app-state review responsibility.
Reference requires no Next app, live backend, login or credentials.

## Ownership and represented designs

Reference imports formal web presentation from `@chapchoo/ui`, whose existing
component changes remain in common UI. Reference owns experimental components
without a formal shared equivalent and web representations of non-web UI.
Design approval is separate from selection for production-web reuse. A selected
shared web implementation belongs in common UI; references import it for
equivalent design roles. When common UI is web-specific, reference may retain
a distinct, more general representation and document their relationship.
Equivalent-role copies are avoided. Non-web representations stay in reference
after approval and are not executable native implementations.

Current Azents-derived styling remains provisional. The onboarding, local outline
and common-UI reuse stories demonstrate boundaries, not approved Chapchoo visual
identity. The reuse story's nested document loads the Mantine stylesheet
and `AppMantineProvider`; its global resets and document-root color scheme stay
out of the parent canvas when switching stories. The workspace does not adopt
the inherited theme globally. Local scaffolding CSS is not a token system.

The `Chapchoo/Foundations` and `Chapchoo/Products` stories render the selected
visual tone from root `DESIGN.md`. New reference stories use their own nested
document and local Mantine theme, leaving neutral ownership examples unaffected.
The catalog includes token/type samples, basic control and feedback examples,
mascot assets, fictional library/planner/settings screens, and content/identity
compositions. Fixtures and small local interactions have no app transport or real
business effects. Existing email/password fields, feedback and confirmation
contracts are imported from common UI rather than copied.

Platform selection is a story argument, separate from viewport width. The iOS
stories reference platform fonts; browser rendering does not establish native
execution or actual Apple font availability. All design principles and their
discussion context are maintained in root `DESIGN.md`, not repeated here.

## Commands and verification

From the repository root, run `pnpm run storybook --filter=@chapchoo/design-reference`
on port 6008 or `pnpm run build-storybook --filter=@chapchoo/design-reference`.
Static output is the ignored package-local `storybook-static/`.
`pnpm --filter @chapchoo/design-reference check:design` verifies that reference
token adapters match the canonical document and checks intended text-color pairs.
This check is also part of package build/typecheck and is not a full accessibility
audit.

Root format/lint/typecheck include the package. Root lint runs
`check:design-boundaries`, which executes isolated boundary fixtures and checks
workspace manifests and source imports. Other workspaces cannot depend on
reference; reference cannot depend on product apps. The checker recognizes named,
relative, dynamic and TypeScript-resolved aliased imports, plus CSS `@import`
and CSS Modules composition dependencies. This is a repository
dependency guard, not a runtime security sandbox.

CI builds all three Storybooks. Authors inspect actual rendered stories at
relevant dimensions; builds do not establish visual approval or native behavior.
The [package guide](../../../../typescript/packages/design-reference/README.md)
and scoped Agent instructions describe authoring, review, selective promotion
and coexistence of general references with web-specific implementations.
