# TypeScript Monorepo Rules

Use pnpm workspaces and Turborepo. Run commands from `typescript/`.
Read `.claude/rules/typescript-conventions.md` and applicable bodies.

The current apps are `apps/chapchoo-web` (`@chapchoo/web`) and
`apps/chapchoo-admin-web` (`@chapchoo/admin-web`). Shared presentation and Node
transport live in `packages/`; apps must not depend on other apps.

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
```

Do not copy generated API clients or add backend contracts before those contracts
are designed. Keep route entrypoints thin, feature UI under `features/`, and
shared theme/providers/utilities under `shared/`.
