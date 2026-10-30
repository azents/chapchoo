---
title: "Design Reference Workspace"
created: 2026-10-30
tags: [design-reference, frontend, developer-tooling]
spec_type: domain
domain: design-reference
code_paths:
  - typescript/packages/chapchoo-design-reference
  - typescript/scripts/design-boundaries.mjs
  - typescript/scripts/design-boundaries.test.mjs
  - typescript/package.json
  - .github/workflows/ci.yaml
last_verified_at: 2026-10-30
spec_version: 1
---

# Design Reference Workspace

`@chapchoo/design-reference` is a private, independent React/Vite Storybook for
general-purpose cross-platform design references. Its only story source is its
own package. General/Admin Storybooks retain app-state review responsibility.
Reference requires no Next app, live backend, login or credentials.

## Ownership and provisional design

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
identity. Only the reuse story's nested document loads the Mantine stylesheet
and `AppMantineProvider`; its global resets and document-root color scheme stay
out of the parent canvas when switching stories. The workspace does not adopt
the inherited theme globally. Local scaffolding CSS is not a token system.

## Commands and verification

From `typescript/`, run `pnpm run storybook --filter=@chapchoo/design-reference`
on port 6008 or `pnpm run build-storybook --filter=@chapchoo/design-reference`.
Static output is the ignored package-local `storybook-static/`.

Root format/lint/typecheck include the package. Root lint runs
`check:design-boundaries`, which executes isolated boundary fixtures and checks
workspace manifests and source imports. Other workspaces cannot depend on
reference; reference cannot depend on product apps. The checker recognizes named,
relative, dynamic and TypeScript-resolved aliased imports. This is a repository
dependency guard, not a runtime security sandbox.

CI builds all three Storybooks. Authors inspect actual rendered stories at
relevant dimensions; builds do not establish visual approval or native behavior.
The [package guide](../../../../typescript/packages/chapchoo-design-reference/README.md)
and scoped Agent instructions describe authoring, review, selective promotion
and coexistence of general references with web-specific implementations.
