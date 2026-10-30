# Shared product UI

`@chapchoo/ui` owns the selected web theme, semantic tokens, bundled assets and
presentation-only components used by general/Admin apps and design-reference.
Visual principles live only in root [DESIGN.md](../../../DESIGN.md).

Import `@mantine/core/styles.css` in the app layout and wrap presentation in
`AppMantineProvider`. It forces the selected light design; there is no unreviewed
dark palette. `FontFaces` loads bundled Pretendard and Geist Mono with the caller's
runtime mount. Apps keep locale/data/transport ownership.

`typescript/scripts/stage-web-fonts.mjs` stages common UI fonts/licenses and
small production illustration derivatives into each app's ignored public directories before dev/build and
Storybook. Build caches restore both staged directories. Storybook serves these
assets at root; actual apps use `basePath` props so root and non-root deployment
URLs remain correct. The original mascot prompts/model provenance are under
`assets/mascots/GENERATION.md`; font licensing remains with the files.

## Presentation API

- `AuthPanel`: title, description/footer, app-owned form content and `basePath`.
- `ApplicationShell`: context/title/identity, navigation, actions, content and
  `basePath`. Desktop sidebar and wrapping narrow navigation share the same links.
- `ContentSection`: semantic section heading/description and grouped content.
- `Mascot`: decorative identity art using an explicit caller mount path.
- `Feedback`: controlled loading/error/empty/loaded states and individual feedback.
- `EmailField` / `PasswordField`: controlled input values and callbacks.
- `ConfirmationDialog`: app-owned action, labels, pending/error and confirmation.
- `createReactContainer`: typed state-hook/view connection.
- `designTokens` / `theme`: common token values and the web Mantine adapter.

No RPC, credentials, cookie storage or business routing live here. Reference
imports these implementations for equivalent roles; fictional compositions and
iOS typography representation remain reference-local. Colocated static stories
are included in both apps' own Storybooks and use the same font/provider setup.
