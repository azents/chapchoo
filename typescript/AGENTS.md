# TypeScript Monorepo Rules

Use the repository-root pnpm workspace, lockfile and Turbo configuration.
Run installation and workspace commands from the repository root.
Read `.claude/rules/typescript-conventions.md` and applicable bodies.

The current apps are `apps/web` (`@chapchoo/web`) and
`apps/admin-web` (`@chapchoo/admin-web`). Shared presentation and Node
transport live in `packages/`; apps must not depend on other apps.
`apps/e2e` is the executable Playwright runner. Reusable assembly/process/seed
support lives in `testenv/chapchoo` (`@chapchoo/testenv`), which must not import
apps or Playwright. Keep core product journeys in E2E and exact copy/responsive
layout in app-owned executable component checks.

`packages/design-reference` is the independent design reference
Storybook, not app UI QA. Read its README and scoped AGENTS.md for design work.
It may import formal components from `@chapchoo/ui`; apps and common UI must
not import reference. Experiments and non-web representations live in reference.
Design approval does not imply promotion to common UI. Existing Azents-derived
styling remains provisional; do not treat it as Chapchoo's approved design system.
A general reference representation may coexist with a web-specific common-UI
implementation; document their distinct roles and reuse common UI for equivalent roles.

```sh
pnpm install --frozen-lockfile
pnpm run dev --filter=@chapchoo/web
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm --filter @chapchoo/web test
pnpm --filter @chapchoo/admin-web test
pnpm --filter @chapchoo/ui test
pnpm --filter @chapchoo/web-core test
pnpm --filter @chapchoo/testenv test
pnpm --filter @chapchoo/e2e test:unit
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web
pnpm run build-storybook --filter=@chapchoo/admin-web
pnpm run storybook --filter=@chapchoo/design-reference
pnpm run build-storybook --filter=@chapchoo/design-reference
pnpm run check:design-boundaries
pnpm --filter @chapchoo/web test:components
pnpm --filter @chapchoo/admin-web test:components
```

Do not copy generated API clients or add backend contracts before those contracts
are designed. Keep route entrypoints thin, feature UI under `features/`, and
shared theme/providers/utilities under `shared/`.
