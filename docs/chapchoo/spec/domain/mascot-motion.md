---
title: "Mascot playback and interaction"
---

# Mascot playback and interaction

Shared mascot playback owns gesture timing, visibility and explicit replay while preserving the surrounding product layout.

## Mascot motion

General/Admin authentication headings and desktop sidebar companions use shared
`AgentMascotMotion` at their existing 72px-square placements. Production keeps the
small WebP URL, character selection and surrounding geometry; per-character alpha
contact metadata anchors the transform inside the square canvas. Authentication
remains visible on narrow screens. General shows the 72px companion at the lower
left of its opened navigation Drawer; fallback sidebar compositions retain the
48em hiding rule. The static `Mascot` primitive remains available for other artwork.

The approved gesture extends by 5% for 1200ms toward 30 degrees clockwise from
upward vertical, with a small damped return bounce and six-second automatic rest.
A native named replay button accepts click/touch/Enter/Space only at rest; input
during playback is ignored without restart or queuing. Hover does not trigger it.
Each app supplies the replay action name in its four supported locales.
Reduced motion disables playback and the replay control. Hidden documents and
offscreen placements cancel active/pending work, with no catch-up replay.
Server-selected login artwork remains stable through hydration and edits.
The independent design reference imports this same common-UI implementation.

## Connections

- [General and Admin web ownership](account-web.md) — General and installation Admin are independent server-rendered applications with shared presentation, not interchangeable authority or API surfaces.
- [Design reference ownership](design-reference.md) — Reference Storybook is a design workspace; approved concepts and rendered examples are not automatically promoted product implementations.
- [Web appearance lifecycle](appearance.md) — Appearance owns system/light/dark resolution and initial paint; it is independent of account language, time zone and regional format.

## References

- [`typescript/packages/ui/src`](../../../../typescript/packages/ui/src)
- [`DESIGN.md`](../../../../DESIGN.md)
