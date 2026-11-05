---
title: "Agent chat surface design"
created: 2026-11-05
updated: 2026-11-05
implemented: 2026-11-05
tags: [agent, chat, web]
document_role: primary
document_type: design
snapshot_id: chat-261105
---

# Agent chat surface design

Requirements: [chat-261105/REQ](../requirements/chat-261105-product-surface.md).
Boundary record: [ADR](../adr/chat-261105-product-surface.md).

## Current behavior and gaps

The product has owned Agent list/detail/creation, server admission, profile assets,
workspace/account controls and request-fenced Computer lifecycle. It has no chat
execution contract. The approved study supplies the presentation to promote.

## Design Authority

- Design revision: `2`

| ID | Material design mechanism | Authority | Classification |
| --- | --- | --- | --- |
| M1 | Conversation route reuses SSR Agent admission and hydrated detail query | REQ-2; existing computer-web and web-rendering Specs | derived |
| M2 | Local transcript/activity with no service or persistence | REQ-3 | required |
| M3 | Approved floating layout, always-expanded status, responsive activity/navigation | REQ-1 | required |
| M4 | Existing Computer lifecycle controller and captured-request confirmation | REQ-2; existing computer-web Spec | existing |
| M5 | Handle-aware chat links from list/create; retained profile detail | REQ-2; workspace-navigation Spec | derived |

## Architecture and state

A thin `/teams/{teamHandle}/agents/{agentHandle}/chat` route loads the existing
Agent detail snapshot. A feature container composes the existing Agent controller
with local input/transcript/activity state and passes resolved props to pure UI.
Optional sidebar Agent choices load independently. Existing workspace context and
account controls remain reusable presentation nodes. Agent identity lives in the
URL; draft input and activity disclosure do not. The sidebar follows the study's
Agent list composition and does not introduce conversation history or selection.

Computer observations remain independent of local activity. Input eligibility
requires a fresh completed running Computer with connected Connector. This does
not assert harness readiness. Status actions delegate existing admission and
consent; no optimistic power transition or retargeted confirmation is introduced.
Read failures preserve visible history and disable submission. Auth withdrawal
uses the existing protected-content failure surface.

A measured overlay hook maintains input clearance and latest-follow intent.
Manual scrolling detaches following; a latest button restores it. Status size is
content-driven and independent of scroll. Shared CharacterArtwork owns playback.

## Interfaces, security and operations

No API, event, generated client, credential, configuration or database changes.
Existing detail/list queries remain owner/Team scoped. Route keys remount local
conversation state across identity changes. Existing session withdrawal guards
protected content. No execution or production observability is fabricated.

Shipping is one focused frontend PR. Rollback is reverting the route and entry
links; there is no persisted chat data to migrate. Deployment is outside scope.

## Removal and Replacement

| Existing unit or behavior | Removal authority | Replacement or remaining authority | Removal boundary | Absence verification |
| --- | --- | --- | --- | --- |
| List/create direct detail entry | REQ-2 | Chat entry, detail remains character-linked | entry links only | route/component tests and core journey |
| Prototype scroll-driven status sizing | REQ-1; explicit cancellation | ordinary expanded status | do not promote gesture/compact code or tests | source review and scroll interaction checks |
| Prototype notices and review settings | REQ-1 | production controls and static Storybook fixtures | no product demo controls | renderer review |

## Test Strategy

E2E-first: extend the existing assembled Computer journey to cover list/create to
chat, detail access and existing power consent; retain cross-owner/Team admission
coverage. Do not create another E2E matrix for visual combinations. App-owned
Storybook component checks cover 320/390/desktop overflow, first-click activity,
read-only/stale/auth states, same-sender spacing and local input/latest-follow.
Unit tests cover owned route helpers and local state transformations where useful.

Use existing isolated E2E/testenv prerequisites only; no real credentials. Missing
required prerequisites fail rather than skip. Evidence includes commands, results,
head SHA and screenshots where relevant. CI runs ordinary quality/build/component
and core browser acceptance gates. No optional live-provider experiment is needed.

## Feasibility and risks

Feasible: `loadAgents`, `useAgentsContainer`, `useComputerActions`,
`ComputerControls`, shared navigation/account contexts and CharacterArtwork already
own the required real capabilities. Local conversation requires no backend change.
Narrow-screen measurement and nested disclosure focus require component validation.
Local messages intentionally disappear on reload and are not execution receipts.

## Design Approval

- Mode: `Collaborative`
- Decision owner: requester
- Approved on: `2026-11-05`
- Approved Design revision: `2`
- Approved authority IDs: `M1, M2, M3, M4, M5`
- Approved scope: the explicitly requested promotion of the approved chat study,
  reuse of existing functionality, local unavailable functionality, removal of
  scroll sizing and exclusion of unsolicited mock UI. This records the direct
  implementation/merge instruction, not an additional product approval gate.
