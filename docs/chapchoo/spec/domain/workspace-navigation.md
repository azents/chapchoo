---
title: "Workspace context and navigation"
---

# Workspace context and navigation

Navigation selects an accessible context without granting access to another owner’s private Agents.

## General working context and navigation

General `/` validates current identity and the current accessible Team list, then
redirects to that user's remembered accessible Team or existing personal Team at
`/agents?team=<Team ID>`. A missing personal context or unavailable required list
is an error, not an invented Team. Host-root `chapchoo-workspace` stores only
the public User/Team IDs for 30 days, SameSite=Lax and Secure on HTTPS. This
JavaScript-readable preference is untrusted, scoped to its User and checked
against current membership on every app-entry request. It is not authorization.

The General root layout server-loads selector context and hands it to one
identity-scoped client query. The URL selects Agent context or Team settings;
profile and invitation routes retain the validated recent context. Navigation
items remain Agents, Invitations and collaborative-Team settings across list,
detail, settings and profile. A Team switch navigates to only the current user's
private Agents in that Team. Desktop retains the independently scrolling sidebar;
mobile hides it and uses the header's left Drawer. Its banner contains the
current icon/name and an adjacent chevron, with a bounded content-width trigger.
The independent close control retains a 44px target and at least the shared
large gap. Team choices open in a bounded Popover below the banner on both
desktop and mobile; the feature navigation and companion remain mounted and
unchanged behind it. Escape closes the picker first and returns focus to its
banner without dismissing the mobile Drawer. A closed touch trigger has no
sticky hover fill; keyboard focus and current-route selection remain distinct.
The common DropdownButton owns same-trigger toggle, outside dismissal and
Escape/focus behavior. Only the current responsive selector is mounted;
a hidden desktop overlay cannot intercept mobile activation.
Shared navigation rows retain their 44px minimum height and use 14px inline
padding, balancing the existing single-line text's visible vertical inset.
The shared NavigationIcon SVG frame supplies workspace symbols, Back, add and
chevron geometry with non-shrinking, vertically centered icon slots. Selected
rows retain their background and `aria-current` without a trailing decorative
tick. Icon alignment does not depend on a font glyph's baseline.

Create Team sits immediately below the Team list and opens a name/handle modal.
It closes the picker while retaining the open navigation Drawer. The modal owns
focus and Escape/outside dismissal until closed; cancel restores the same
navigation context without mutating or selecting. Successful ordinary creation selects the new
Team after the server response. Validation/pending/uncertain errors retain their
actual outcome; there is no automatic creation retry. Received invitation
acceptance is reachable at `/invitations` and server-renders its required list.
Existing `/teams/<ID>` membership/role/ownership operations remain reachable.
Leave, kick and Team-delete confirmations disclose Agent and saved Computer-data
deletion, without introducing archive or automatic rehoming.

Admin uses the same shared workspace chrome with Administration selected,
a shield/check icon and a lowercase `admin` brand badge. Its stable menu contains
signup links, password reset links, Computer Providers and server settings,
including on detail and profile routes. It has no duplicate subtitle or
separate back-to-app row. The same left mobile Drawer and overlay picker apply.

Admin Node communicates only with its configured Admin API. The narrow
AdminWorkspaces service lists the current administrator's accessible Teams and
creates an ordinary caller-owned Team; full TeamManagement stays Client-only.
Selecting a Team or confirmed creation navigates the browser to General, where
ordinary admission independently runs. Both logos point to General entry and
therefore resume its validated recent context. Public peer web URLs are optional
validated runtime navigation metadata; they are not backend targets, inferred
paths or compiled deployment hosts. Without a configured General destination,
Admin renders a non-link brand and unavailable workspace selection rather than
inventing a destination or submitting an unusable creation.

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Private Agent ownership and Team deletion](agent-ownership.md) — Team membership selects context while individual ownership remains the private Agent boundary. Context deletion accepts cleanup intent, not completed native deletion.
- [Primary rendering and URL-owned state](web-rendering.md) — Server admission supplies primary content and stable selections; client queries own subsequent interactions rather than a second initial truth.

## References

- [`typescript/apps/web/src`](../../../../typescript/apps/web/src)
- [`typescript/apps/admin-web/src`](../../../../typescript/apps/admin-web/src)
- [`typescript/packages/ui/src`](../../../../typescript/packages/ui/src)
