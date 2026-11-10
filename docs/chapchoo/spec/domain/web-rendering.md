---
title: "Primary rendering and URL-owned state"
---

# Primary rendering and URL-owned state

Server admission supplies primary content and stable selections; client queries own subsequent interactions rather than a second initial truth.

## Primary content and page state

General Agent pages, received invitations, selected team details/membership, and the selected Admin
inventory page are loaded and rendered on the server. A request-local read scope
shares validated Identity and lends the app credential only inside server reads;
it never renews or writes cookies. Primary-data failures reach safe route-level
error/not-found handling. Required snapshot values do not gain artificial
client-only initial loading states.

Interactive queries receive the same initial data and selection for SSR and
hydration, without mount/reconnect repeats. Later updates and invalidation remain
client-owned. Received invitations are primary data at `/profile/invitations`. Team
management invitations remain supplementary to the member/settings record.
Workspace-selector data is supplementary on other feature routes; its failure
does not replace independently available primary content with an empty success.

Admin inventory offset is a canonical decimal URL search value with the existing
25-row page size and int64 bound. Admin login method is URL-owned and constrained
by the server-advertised methods. Server and browser use the same first-value
policy for duplicate keys. Native history push updates primary selections and
Next search-parameter subscribers; direct entry/reload resolves the same selection
on the server. Back/Forward restore the selected view. Minor URL state uses
replace; no current feature needs an invented minor selection.

Admin issuance/expiry values use the account's independent regional format and
time zone instead of fixed UTC formatting and its suffix.

Chat hydrates the server's exact initial conversation region, then its
[active read owner](conversation-web.md#initial-rendering-and-active-reads)
continues source-bound reads on a coalesced visible-document cadence. Source
replacement clears the old region before refill; this is ordinary loading, not
a second staged primary snapshot or a durable browser feed.

Existing selected-team and inventory-kind routes remain pathname-owned. Form
inputs, authentication challenge bindings, newly issued proof links and destructive
mutation confirmations remain transient and never become URL state. Shared
read/initial-query/URL-codec patterns are described in the
[implementation guide](../../notes/web-page-patterns.md).

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Web authentication, request ownership and cookies](web-session.md) — Node request contexts and browser authentication have independent lifetimes. Cookie disposition is not an ordinary business operation’s success flag.
- [Workspace context and navigation](workspace-navigation.md) — Navigation selects an accessible context without granting access to another owner’s private Agents.
- [Computer owner and operator web](computer-web.md) — Server-rendered owner controls and redacted administrator inventory project backend authority rather than creating browser lifecycle state.

## References

- [`typescript/apps/web/src/app`](../../../../typescript/apps/web/src/app)
- [`typescript/apps/admin-web/src/app`](../../../../typescript/apps/admin-web/src/app)
- [`typescript/packages/web-core/src`](../../../../typescript/packages/web-core/src)
