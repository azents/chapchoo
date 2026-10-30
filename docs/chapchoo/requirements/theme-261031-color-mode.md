---
title: "System, Light and Dark Appearance Requirements"
created: 2026-10-31
implemented: 2026-10-31
tags: [frontend, appearance, accessibility]
document_role: primary
document_type: requirements
snapshot_id: theme-261031
---

# System, Light and Dark Appearance Requirements

- Snapshot: `theme-261031`
- Document reference: `theme-261031/REQ`

## Problem

The general and Admin applications currently force light appearance. Users cannot
choose a dark appearance or follow their device preference, and the decorative
character artwork has a baked background unsuitable for reuse across appearances.

## Primary Context

### Primary Actor

A person using the general or installation Admin web application.

### Primary Scenario

The person chooses System, Light or Dark, continues using the application, and
returns later to the remembered appearance without an incorrect-theme flash.

## Supporting Scenarios or Effects

- Authentication and initial Admin setup remain usable in both appearances.
- System appearance changes update an open application when System is selected.
- Shared character artwork remains recognizable on either background.

## Goals

- Complete light/dark treatment of the existing general and Admin surfaces.
- Remember an explicit three-way preference and produce a consistent initial page.
- Retain the existing visual hierarchy and interaction behavior.

## Non-Goals

- New product features, navigation redesign, decorative selection markers or
  changes to authentication, permissions and business operations.
- Per-account server-side appearance settings, native apps or deployment work.
- Deployment monitoring.

## Requirements

### REQ-1. Complete adaptive presentation

Both web applications support the selected light appearance and the reviewed warm
olive-charcoal dark direction, including auth/setup, populated/empty/error states,
controls, menus, confirmation dialogs and loading/disabled/focus states.

**Acceptance criteria**

- Representative real product renders remain legible in both appearances.
- Existing layout, typography, control geometry, mobile editable-text minimum,
  complete action-label line boxes, zoom and navigation treatment are preserved.
- Text and meaningful control/state cues are checked against applicable WCAG 2.2
  AA contrast thresholds; decorative separators are not treated as controls.

### REQ-2. Three-way remembered selection

A person can choose System, Light or Dark on signed-out and signed-in surfaces in
either application. An absent or invalid preference defaults to System.

**Acceptance criteria**

- The selected preference survives navigation, reload and a later visit through
  cookies rather than an account setting.
- General and Admin applications on the same host share the preference across
  runtime mounts; different hosts remember their preferences independently.
- System follows the current browser/device appearance and subsequent changes;
  an explicit Light or Dark choice overrides those changes.
- Changing appearance preserves entered form values, focus and current work.

### REQ-3. Consistent initial appearance

Server rendering uses available appearance cookies, and the document exposes its
appearance before visible content is painted. Browser system information corrects
an unknown or stale system resolution without a light/dark flash.

**Acceptance criteria**

- Explicit preferences appear in server-rendered document mode attributes.
- On a first visit, System matches a dark or light browser before visible paint,
  although a server without browser information cannot know the device setting.
- Initial appearance and hydration do not rely on conflicting persistence stores
  or cause a hydration error, content remount or a later wrong-mode repaint.
- Behavior holds at root and configured runtime mounts for both applications.

### REQ-4. Shared transparent character assets

Use the prepared transparent five-character artwork as common masters rather than
separate baked light/dark backgrounds.

**Acceptance criteria**

- Production small derivatives preserve alpha transparency and character colors.
- Artwork uses normal composition and is visible in both appearances.
- Runtime-prefixed asset paths, original generation provenance and packaging work.

## Fixed Constraints

- Implement SSR, HTML/document mode indication and cookie persistence as explicitly
  requested, adapting the established three-mode architecture.
- Preserve shared UI ownership: product-independent appearance primitives may be
  shared; request reading and product localization remain application-owned.
- No changes to session cookies, backend authority, runtime configuration trust,
  authentication or installation boundaries.
- Changes are delivered through a focused PR; merge and deployment require approval.

## Open Assumptions

- A first HTTP request may not contain the device appearance. Pre-paint browser
  resolution is required; SSR alone is not represented as knowing that information.

## Confirmation

Confirmed by the requester on 2026-10-31 KST following the bounded scope brief.
This includes same-host general/Admin preference sharing across runtime mounts.
The original explicit implementation request fixes the three choices, SSR,
document mode indication, cookies, transparent artwork and existing-layout scope.
