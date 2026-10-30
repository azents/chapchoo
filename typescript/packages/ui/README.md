# Shared product UI

`@chapchoo/ui` contains the existing Chapchoo Mantine theme and presentation-only
building blocks for the independent general and Admin apps. Applications import
explicit subpaths and transpile this workspace package with Next.js.

Import `@mantine/core/styles.css` in each app's root layout, then wrap with
`AppMantineProvider` from `@chapchoo/ui/provider`. Supply the existing Geist font
CSS variables in each app. The package keeps the original theme's warm palettes,
spacing, semantic tokens, contrast resolution and component defaults.

The provider's default/forced color-scheme props retain the original behavior.
The app owns color-mode storage, locale providers, cookie settings and fonts.

## Presentation API

- `AuthPanel`: title, optional description/footer and app-owned form content.
- `ApplicationShell`: visible context label, title, identity label, navigation
  data/callbacks, actions and workspace content.
- `Feedback`: loading/error/empty/loaded discriminated states; individual
  `LoadingFeedback`, `ErrorFeedback` and `EmptyFeedback` are also available.
- `EmailField` and `PasswordField`: controlled values and callbacks with Mantine
  input props. Password visibility labels are supplied by the application.
- `ConfirmationDialog`: controlled opened state, localized labels, confirm/cancel
  callbacks, ready/pending/error state and optional destructive treatment.
- `createReactContainer`: connect an app's state hook to its pure component,
  retaining typed optional component overrides.

There is no RPC, environment, credential, cookie, routing or domain authority in
this package. Do not pass session tokens into any presentation component.
Navigation destinations and localized copy belong to each application.

Colocated stories use static fixtures. Apps can include
`../../packages/ui/src/**/*.stories.tsx` in their Storybook story globs
and retain their existing Mantine/locale/color-scheme decorators. No live API
provider is needed. The shell collapses to visible wrapping navigation on mobile;
tabular app content should use `Table.ScrollContainer` to constrain overflow.
