---
title: "Display language, time zone and regional format"
---

# Display language, time zone and regional format

Personal values, visitor choices and installation defaults are distinct authorities for the three independent display dimensions.

## Display settings

Both apps expose the same always-visible account/settings entry on public,
authentication and protected surfaces. Its title is “Account and settings” for
authenticated users and “Settings” for visitors, with localized equivalents.
The common theme gives every Drawer body an `xl` top inset below its header.
The account Popover/Drawer applies language, IANA time zone and regional
format independently and immediately. Protected `/profile` routes duplicate
those controls over the same identity-query User; there is no separate account
settings store. Changing language updates catalogs and document language without
remounting entered work. A failed account save retains its attempt with an
explicit retry and never claims success; account/unmount fences reject late
responses belonging to an old identity.

The request-cached display snapshot uses validated account values for signed-in
SSR and hydration. Visitors resolve each dimension from explicit
`chapchoo-display-choice`, then automatic `chapchoo-display-projection`, then
installation defaults. Both host-only root-path cookies are validated,
JavaScript-readable presentation data with one-year lifetime, SameSite=Lax and
Secure on HTTPS. Current post-hydration browser detection updates only automatic
dimensions and their projection. Explicit choices survive detection; selecting
Follow browser settings restores automatic behavior. Cookie denial permits an
in-page change but reports that persistence is unavailable.

Account creation submits only explicit or available detected dimensions; the
backend resolves absent dimensions from current transaction-time defaults.
Neither old NEXT_LOCALE nor Accept-Language is an alternative account authority.
Required defaults/identity infrastructure failure reaches the sanitized render
boundary rather than fabricating a signed-out state or static defaults.

Signed-in differences produce a titled/icon toast containing only applicable
independent rows. Each action changes only its dimension. Language proposals
include current/target language copy and autonyms. Dismissals are browser-local
Mantine localStorage data partitioned by account/dimension/detected value; a new
detected value can be offered again. Storage denial does not crash the page.
IANA aliases compare semantically. A stored server-valid zone absent from local
Intl data does not crash suggestion comparison. Previews and Admin timestamps
show a localized unavailable-format message, preserving the stored value without
substituting UTC or another time zone. Toasts remain below modal layers.
Backend User/default response decoding checks complete bounded field shapes and
supported language/region, with Rust/Jiff retaining semantic zone authority;
older Node ICU data cannot reject a server-approved zone before SSR. Browser
cookie, detection and editable-input validation still checks local renderability.

Admin `/settings` edits installation defaults through its own protected query
and mutation, separate from personal settings. Existing accounts and explicit
visitor choices do not change when defaults change.

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Web appearance lifecycle](appearance.md) — Appearance owns system/light/dark resolution and initial paint; it is independent of account language, time zone and regional format.
- [Account and Team transaction contracts](account-transactions.md) — Account, session, membership and invitation state changes complete under current database authority before external delivery.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.

## References

- [`rust/crates/domain/src/display.rs`](../../../../rust/crates/domain/src/display.rs)
- [`rust/crates/persistence/src/display.rs`](../../../../rust/crates/persistence/src/display.rs)
- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
