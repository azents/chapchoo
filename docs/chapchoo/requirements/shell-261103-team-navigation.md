---
title: "Consistent Team and Administration Navigation Requirements"
created: 2026-11-03
document_role: primary
document_type: requirements
snapshot_id: shell-261103
tags: [frontend, navigation, authentication, design]
---

# Consistent Team and Administration Navigation Requirements

- Snapshot: `shell-261103`
- Document reference: `shell-261103/REQ`

## Problem

Route-specific menus change shape while users navigate. Personal/team destinations, account settings and installation administration are mixed at the same level. On narrow screens persistent navigation consumes primary-content space. Repeated authentication across same-origin General/Admin surfaces and late font replacement further disrupt continuity.

## Primary Actor

A signed-in user working in their personal context or an accessible team; server administrators additionally enter installation administration.

## Primary Scenario

Open the app, resume the recently selected accessible working context, navigate its features through a stable menu, switch to another context, and continue without losing orientation. The account/profile and administration entry points remain clear and consistent on desktop and mobile.

## Supporting Scenarios or Effects

- Create a collaborative team from the selector and enter it after confirmed creation.
- View/edit account display preferences through a single upper-right entry and a separate profile page.
- An authorized server administrator enters and leaves administration using the same selector.
- Refresh or move between same-origin General/Admin without font-induced layout churn or contradictory authentication state.

## Goals

- Preserve clear context and stable navigation across routes and screen sizes.
- Keep narrow primary content visible until navigation is explicitly opened.
- Separate personal account information, team operations and installation management without redundant escape routes.

## Non-Goals

- Merge, deployment, production provisioning or credential changes.
- New Agent execution, provider provisioning, or shared/team-public Agent permissions. Team membership never grants access to another user's private Agent.
- Shared login between unrelated installations or untrusted origins.
- New visual branding, new mascot art, or unrelated page-content redesign.

## Requirements

### REQ-1. Select the working context before its features

**Acceptance criteria**

- The top of desktop navigation and mobile navigation identifies the current working context and opens the same personal/team/administration selector.
- The personal destination is named `Your space` in English and is distinct from account profile settings.
- Accessible team names are choices within the selector, not peers of feature links.
- Every Agent belongs to exactly one team. `Your space` represents the user's existing personal team, not a teamless exception.
- A personal Agent can belong to the personal team or to a collaborative team while retaining its individual user owner.
- Selecting a team shows only the current user's personal Agents within that team. Membership does not expose other members' private Agents.
- Agent creation, listing, detail and lifecycle operations respect both team membership/context and individual ownership.
- Existing personal Agents and their Computer/environment data are preserved in the owner's personal team during the corrective migration.
- Leaving or being removed from a collaborative team deletes that user's personal Agents within the team. Deleting the team deletes all Agents within that team.
- These deletions use the existing Agent/Computer cleanup lifecycle, not row cascades that bypass resource cleanup.
- Do not automatically rehome these Agents. Archiving, restore or intermediate retention experiences are explicitly deferred to a later discussion.
- Selection changes the working target and corresponding feature navigation/content, with no cross-context data exposure.
- Installation administration is a privileged destination, not a collaborative team.

### REQ-2. Retain stable context navigation across routes

**Acceptance criteria**

- Within a selected working target, feature entries and their order remain stable when changing list, detail, settings or profile routes.
- Current selection and primary content change without replacing the menu with an unrelated subset.
- URL-selected content remains reloadable and standard browser back/forward navigation remains meaningful.
- Selecting a team does not implicitly grant access to another user's private Agents or installation administration.

### REQ-3. Keep mobile navigation out of the resting content area

**Acceptance criteria**

- Mobile feature navigation opens from a header menu control in a left Drawer rather than persistent rows above primary content.
- The current context is at the top; activating it shows the selector inside that Drawer and selecting a target returns to its menu.
- The context banner separates the leading icon/name and trailing chevron without crowding the close control.
- Existing mascot artwork appears below the menu in available lower-left space without overlapping navigation; overflow keeps actions reachable.
- Desktop retains its sidebar and existing independent scrolling/content geometry except the agreed selector/chrome changes.

### REQ-4. Unify personal identity and display controls

**Acceptance criteria**

- One upper-right control opens a desktop popover or mobile right Drawer with mini-profile, appearance, language, time zone, regional format and applicable account actions.
- Signed-in users see a user icon; visitors see a settings icon and applicable sign-in entry, not a fictitious profile.
- Appearance retains System/Light/Dark and display settings retain existing account/device scope, persistence, failure and retry behavior.
- Profile is reached from this account entry, not the former combined personal/team hub.
- Responsive controls remain keyboard accessible, return focus on close, and do not leave conflicting modal layers active.

### REQ-5. Create a team from the selector

**Acceptance criteria**

- `Create team` is immediately below the accessible team list, not a form on the old hub page.
- It opens a creation modal with the existing team-name and handle inputs, explicit cancel and submit actions.
- Cancel does not create or select a team; confirmed success makes the new team the working target.
- Validation, pending submission, permission/domain errors and uncertain responses remain truthful; a lost response does not trigger automatic repeated creation.
- Team invitation acceptance and existing role/member/ownership management remain reachable after removing the old hub.

### REQ-6. Use the common selector in administration

**Acceptance criteria**

- Only current server administrators see the administration entry below the team list in its distinct group.
- On Admin, the same banner uses an administrator icon, `Administration` title and chevron, with no subtitle.
- Choosing `Your space` or any accessible team switches to that working target using the same selector experience.
- No separate `Back to app` feature entry remains.
- Admin feature navigation remains stable across its pages and uses the same shared spacing, active styles, Drawer, mascot and account controls.
- The top app bar explicitly displays `admin`, including narrow screens.

### REQ-7. Make application entry predictable

**Acceptance criteria**

- The old combined spaces hub is replaced by the separate account profile and context selector responsibilities.
- The app entry resumes the recently selected accessible personal/team context rather than showing the former hub.
- The `chapchoo` logo is an app-entry link on General and Admin.
- Navigation never resumes an inaccessible team or exposes data based only on a remembered choice.

### REQ-8. Keep same-origin authentication consistent without weakening authorization

**Acceptance criteria**

- Logging in or out of one same-origin General/Admin surface yields consistent account state when entering the other.
- Current administrator authorization remains enforced by the server independently of shared identity.
- A non-administrator visiting Admin does not invalidate their valid General login merely because access is denied.
- Session credentials remain HttpOnly, not exposed to page code or logged, and existing CSRF/origin protections remain enforced.
- Unrelated installations and untrusted origins do not gain session authority.

### REQ-9. Stabilize initial text rendering

**Acceptance criteria**

- Refresh and first/cold navigation avoid avoidable late font discovery and visible metric replacement on General and Admin.
- Existing Pretendard appearance, Korean and Latin coverage, text scaling and readable fallback remain intact.
- Measurement distinguishes actual cold font arrival/layout behavior from screenshot automation that waits for fonts.
- The fix does not hide the complete page or substitute prolonged blank text for layout movement.

### REQ-10. Remove redundant visual boundaries

**Acceptance criteria**

- A row/list boundary and an adjacent section boundary do not produce two redundant horizontal rules for the same grouping transition.
- Remove the obsolete hub layout rather than preserving its redundant personal-space explanation and borders elsewhere.
- Keep remaining table/list/section information hierarchy and actual functional boundaries clear.

## Fixed Constraints

- Apply shared requirements to General and Admin unless an explicit surface distinction above applies.
- Admin web communicates only with the Admin API. Workspace selection and creation on Admin must not call the General API or add a General backend connection. This boundary was explicitly reaffirmed by the requester on 2026-11-03 KST.
- Preserve existing protected data loading, ownership rules, permission checks, translated content and account display semantics.
- Do not ship the visual fixture's mocked actions, team/Agent data, transient navigation-only state, or English-only strings.
- Deliver reviewable PR(s), exact-head validation/CI and review responses; merge/deployment require separate authorization.

## Open Assumptions and Feasibility Checks

- The requested last-selected-context behavior must specify a safe first-visit or unavailable-context result using existing personal access; no new access is inferred.
- Current Agent APIs omit team association. The requester explicitly confirmed on 2026-11-03 that every Agent must belong to one team, that personal Agents can belong to personal or collaborative teams, and instructed correction. This association is required; shared ownership/team-public Agent permissions are not implied.
- Team leave/delete currently has no Agent lifecycle integration. The requester chose deletion on 2026-11-03 and explicitly deferred archiving. Integrate deletion without bypassing asynchronous Computer cleanup or introducing a nullable/teamless Agent; any internal deletion bookkeeping must not present an archive/restore feature.
- Same-origin session sharing must retain installation and administrator authorization boundaries; separate-origin integration is not implicitly added.

## Confirmation

The requester approved the final component-render direction and explicitly requested implementation of all accumulated feedback and PR submission on 2026-11-03 KST. This document consolidates that accepted behavior. The feasibility checks above are not additional approved product mechanisms; any newly exposed material scope conflict must be resolved before its affected implementation.
