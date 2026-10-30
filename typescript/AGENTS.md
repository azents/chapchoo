# TypeScript Monorepo Rules

Use pnpm workspaces and Turborepo. Run commands from `typescript/`.
Read `.claude/rules/typescript-conventions.md` and applicable bodies.

The current apps are `apps/chapchoo-web` (`@chapchoo/web`) and
`apps/chapchoo-admin-web` (`@chapchoo/admin-web`). Shared presentation and Node
transport live in `packages/`; apps must not depend on other apps.

`packages/chapchoo-design-reference` is the independent design reference
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
pnpm run build
pnpm run build-storybook --filter=@chapchoo/web
pnpm run build-storybook --filter=@chapchoo/admin-web
pnpm run storybook --filter=@chapchoo/design-reference
pnpm run build-storybook --filter=@chapchoo/design-reference
pnpm run check:design-boundaries
```

Do not copy generated API clients or add backend contracts before those contracts
are designed. Keep route entrypoints thin, feature UI under `features/`, and
shared theme/providers/utilities under `shared/`.
