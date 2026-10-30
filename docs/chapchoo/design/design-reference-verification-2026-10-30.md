---
title: "Design Reference Workspace Verification"
created: 2026-10-30
tags: [design-reference, frontend, verification]
document_role: supporting
document_type: supporting-verification
---

# Design Reference Workspace Verification

Scope: [reference-261030](reference-261030-design-workspace.md), Design revision 3,
including coexistence of general references with web-specific common UI.

## Deterministic checks

- Frozen pnpm installation succeeds. Lock changes add only the new workspace
  importer; no package versions, overrides or production dependency graph change.
- Root format, lint and typecheck pass.
- Dependency boundary fixtures: 21 pass. Actual workspace boundaries pass.
  Fixtures include named/relative/dynamic/type imports, TypeScript aliases,
  dependency sections, workspace alias ranges and relative/absolute local links.
- Existing unit suites: UI 9, web-core 17, General 9, Admin 14, E2E helpers 5:
  **54 passed**, no skips or failures.
- Root product/package build and static builds for General, Admin and the new
  design-reference Storybook pass.
- Documentation frontmatter validation and independent read-only review pass.

## Real browser smoke

Served the generated package-local `storybook-static/` over local HTTP.
The Session Browser rendered all three story IDs:

- `start-here-workspace--purpose`
- `ownership-examples-common-ui-reuse--provisional-reuse`
- `ownership-examples-non-web-representation--local-outline`

Inspected actual text and screenshots at an 800 × 1120 capture surface.
The initial examples show provisional status and ownership guidance, not a
palette or screen-design catalog. The non-web outline explicitly disclaims
native implementation or a proposed mobile screen.

Also used Storybook's navigation controls, without reloading the preview
document, for **neutral → common reuse → neutral**. The common example loaded
its shared component inside a nested document and rendered successfully.
Returning to the neutral reference removed that nested frame and preserved
neutral presentation rather than inheriting the common theme's background.
The reference documents continued to render with no visible Storybook error UI.
This is visual smoke evidence, not a claim of exhaustive console, accessibility,
responsive, or native-platform validation.

## Independent review and corrections

The independent reviewer identified global Mantine CSS/provider document effects
and manifest alias gaps. The final implementation lazy-loads Mantine only in the
nested reuse document and recognizes workspace alias ranges and absolute local
links. Targeted re-review found no remaining issues in those fixes or the
requester's general-reference/web-specialization clarification.

## Environment observations

General web tests first failed to materialize SWC native bindings because the
Runtime's default cache ancestors were group-writable without sticky protection.
Rerunning with `SWC_NATIVE_BINDING_CACHE` pointed at a private user-owned cache
under a safe home directory passed, without repository code or dependency edits.

Existing peer warnings involve TS 6/tsconfck and ESLint 10/plugins. Reference's
Vite build reports ignored `use client` module directives when bundling shared
Mantine modules for client-only Storybook. These are not build failures.
Production code, app Storybook configurations and app visual styling are unchanged.

## Reproduction

From `typescript/`:

```sh
pnpm install --frozen-lockfile
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm --filter @chapchoo/ui test
pnpm --filter @chapchoo/web-core test
pnpm --filter @chapchoo/web test
pnpm --filter @chapchoo/admin-web test
pnpm --filter @chapchoo/e2e test:unit
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web --filter=@chapchoo/admin-web --filter=@chapchoo/design-reference
```

Run the documented reference Storybook command for interactive visual review.
No backend, login, private service, deployment or cluster modification is needed.
