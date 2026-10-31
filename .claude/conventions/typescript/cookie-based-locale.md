---
title: "General/Admin routes stay locale-less; next-intl follows the authoritative account or visitor display snapshot."
---

# Locale-less Display Routing

General/Admin translations follow the shared display snapshot independently of
regional format and time zone. See the account-web Living Spec for resolution.

- ALWAYS keep routes locale-less, e.g. `/login`, `/w/{handle}`.
- NEVER add App Router `[locale]` segments for chapchoo-web.
- Signed-in language comes from the account; visitor language resolves explicit
  choice, browser projection, then installation defaults. SSR and first hydration
  share that snapshot; live language changes do not remount page state.
