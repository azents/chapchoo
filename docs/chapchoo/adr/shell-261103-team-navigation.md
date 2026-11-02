---
title: "Consistent Team and Administration Navigation Decisions"
created: 2026-11-03
document_role: primary
document_type: adr
snapshot_id: shell-261103
tags: [frontend, navigation, authentication, architecture]
---

# Consistent Team and Administration Navigation Decisions

- Snapshot: `shell-261103`
- Requirements: [shell-261103/REQ](../requirements/shell-261103-team-navigation.md)

## Fixed and Derived Outcomes

The requester approved the final visual behavior and implementation, then explicitly corrected the missing Agent team association. These are not optional technical alternatives:

- Each Agent has one required team and retains its individual owner.
- The existing personal-team mapping supplies `Your space` and the preservation target for pre-existing personal Agents.
- Both team admission and private ownership remain authoritative; no team-public Agent permission is introduced.
- General and Admin share the presentation rules but retain separate protected business surfaces.
- Administration is a selector destination; it is not a new Team record.
- Same-domain authentication continuity must not imply administrator privilege or erase a valid General credential on Admin authorization failure.
- Server-rendered primary data, URL authority, request-owned credentials and existing asynchronous Computer cleanup remain unchanged constraints.

## Material Decision Map

| Topic                                                                            | State                                                                                | Owner                                                                             |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- |
| Mandatory Agent team association with private owner                              | Required by requester; fixed                                                         | Requester                                                                         |
| Backfill existing Agents to owner's personal team, preserve Computer/environment | Required correction; fixed                                                           | Requester                                                                         |
| Team departure/removal/deletion disposition                                      | Requester selected Agent deletion, not rehoming; archiving deferred                  | Requester                                                                         |
| Remembered context persistence and unavailable-context recovery                  | Derived browser hint + current membership validation; General verified               | Requester for product behavior; implementation owner for equivalent local details |
| Same-origin auth continuity, Admin authorization separation                      | Existing General root-cookie scope; request-owned rotation and Admin denial verified | Implementation owner; escalate any new installation/security authority            |
| Initial font-discovery and late-swap prevention                                  | Local REQ-9 remediation; four delayed-font conditions verified                       | Implementation owner                                                              |

No pending policy is treated as accepted merely because it is easy to implement. In particular, cascading Team deletion to Agent/Computer rows is not authorized: it bypasses the existing asynchronous external-resource cleanup authority.

## Current Evidence and Remaining Decision Boundary

The fixed Team/private-owner/deletion outcome now has both-engine integration
and preserving migration evidence. Context removal records the existing terminal
Computer intent before withdrawing membership; a pending Team row exists only
while asynchronous cleanup still owns Agent FKs. Team collection runs after the
Provider/Computer transaction commits to preserve admission lock order. This is
a derived cleanup mechanism, not a new archive policy.

Same-host identity continuity reuses the existing General root-cookie scope.
Admin renewal publishes a committed rotated result before denying non-admin
business dispatch; authorization failure is not credential invalidation. Exact
Origin checks and independent administrator admission remain unchanged.

The material unresolved interface question is Admin workspace discovery/creation
under split listeners. Using an explicitly configured General backend adapter
and adding a narrow shared RPC have different interface/configuration outcomes.
The [Design draft](../design/shell-261103-team-navigation.md) records both; neither
is accepted by this evidence note. Full TeamManagement must not silently become
an Admin service.

## ADR-D1. Admin Workspace API Connection — Pending

- Affected requirements: REQ-4, REQ-5, REQ-6, REQ-8.
- Decision owner: requester; no choice accepted yet.
- Current boundary: only the Client RPC union exposes TeamManagement. Admin's
  ordinary business clients remain on their Admin backend destination.
- Option 1 (recommended): Admin Node explicitly connects to the configured
  General backend for current-user Team reads/creation, reusing the ordinary
  service and request-owned session authority. Pass canonical public peer
  destinations at runtime; expose no backend address or credential to the browser.
- Option 2: introduce a narrow shared context discovery/creation RPC and register
  it on the required listener unions. This creates a new public contract and
  capability surface instead of reusing the current General contract.
- Asked after independent General, database, auth and rendering work was verified.
  Neither option authorizes broad TeamManagement exposure on the Admin listener,
  new administrator privileges, secret copying or a generic proxy.
- Split-listener acceptance remains required; the combined Assembly is not proof
  of this boundary.

## ADR-D1 Resolution — Admin API Only

The requester rejected the General-backend connection on 2026-11-03 KST and
required Admin web to communicate only with the Admin API. That requirement
supersedes the pending recommendation above.

Provide a narrow `AdminWorkspaces` service on the Admin union with only current
administrator's own accessible-Team listing and ordinary Team creation. Reuse
the existing Team persistence and permission rules; verify current administrator
status inside the same complete storage transaction. It does not expose member
management, another user's Team inventory or private Agents. Full
TeamManagement remains Client-only. Admin Node retains one Admin backend target.

General/Admin public web destinations are runtime navigation metadata, not API
connections. They carry no credentials; choosing a Team on Admin navigates the
browser to that General context and normal General admission revalidates it.
Split-listener tests must prove both the new narrow service and continued absence
of General business services on Admin.

## Local Implementation Categories

Component/file identifiers, equivalent presentation composition, icon paths, translated labels, bounded fixture setup and test selectors do not create new product authority. Experimental English-only component-render code is evidence, not production implementation authority.
