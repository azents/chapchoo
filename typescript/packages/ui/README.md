# Shared product UI

`@chapchoo/ui` owns the selected web theme, semantic tokens, bundled assets and
presentation-only components used by general/Admin apps and design-reference.
Visual principles live only in root [DESIGN.md](../../../DESIGN.md).

Import `@mantine/core/styles.css` in the app layout and wrap presentation in
`AppMantineProvider`. `FontFaces` loads bundled Pretendard and Geist Mono with the caller's
runtime mount. Apps keep locale/data/transport ownership.

Pretendard uses the official self-hosted unicode-range subsets, loaded only for
rendered characters through a runtime-prefixed stylesheet. Font files are not
globally preloaded. See `assets/fonts/README.md` for provenance and update scope.

`typescript/scripts/stage-web-fonts.mjs` stages common UI fonts/licenses and
small production illustration derivatives and per-character icons into each app's ignored public directories before dev/build and
Storybook. Build caches restore the staged directories and root favicon. Storybook serves these
assets at root; actual apps use `basePath` props so root and non-root deployment
URLs remain correct. The original mascot prompts/model provenance are under
`assets/mascots/GENERATION.md`; font licensing remains with the files.

## Presentation API

- `AuthPanel`: title, description/footer, app-owned form content and `basePath`.
  An optional `character` renders the caller's server-selected login identity;
  the component does not generate randomness or own selection state.
- `ApplicationShell`: context/title/identity, navigation, actions, content and
  `basePath`. Desktop sidebar and wrapping narrow navigation share the same links.
- `ContentSection`: semantic section heading/description and grouped content.
- `Mascot`: decorative identity art using an explicit caller mount path; defaults
  to the supplied retriever. The typed `character` prop selects any of six characters.
- `webIdentity`: runtime-prefixed browser metadata and home-screen manifest data,
  shared by General and Admin. Icon sources/generation are under `assets/icons/`.
- `Feedback`: controlled loading/error/empty/loaded states and individual feedback.
- `EmailField` / `PasswordField`: controlled input values and callbacks.
- `ConfirmationDialog`: app-owned action, labels, pending/error and confirmation.
- `createReactContainer`: typed state-hook/view connection.
- `designTokens` / `theme`: common token values and the web Mantine adapter.

No RPC, credentials, cookie storage or business routing live here. Reference
imports these implementations for equivalent roles; fictional compositions and
iOS typography representation remain reference-local. Colocated static stories
are included in both apps' own Storybooks and use the same font/provider setup.

## Owned verification

Run `pnpm --filter @chapchoo/ui test` for presentation and canonical adapter checks.
Build `pnpm run build-storybook --filter=@chapchoo/ui`, then run
`pnpm --filter @chapchoo/ui test:components`. This package-local fixture Storybook
has no app, router, localization or backend dependency. It tests the shared
layout/custom CSS contracts once; app tests own their state and composition,
not shared component internals or Mantine token resolution.
