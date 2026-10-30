---
title: "System, Light and Dark Appearance Architecture"
created: 2026-10-31
tags: [frontend, appearance, architecture]
document_role: primary
document_type: adr
snapshot_id: theme-261031
---

# System, Light and Dark Appearance Architecture

Requirements authority: [theme-261031/REQ](../requirements/theme-261031-color-mode.md).

## Decision Map

- Fixed: cookie-backed System/Light/Dark preference, SSR initialization, document
  mode indication and pre-paint browser resolution (`REQ-2`, `REQ-3`).
- Accepted by the requester: same-host general/Admin preference sharing, including
  different runtime mounts (`REQ-2`).
- Fixed: complete adaptive semantic colors, existing geometry and normal
  transparent-art composition (`REQ-1`, `REQ-4`).
- Existing: apps own request reading and localization; common UI remains
  presentation-only; browser imports do not pull Node RPC/configuration code.
- Derived: preference and cached system resolution are distinct values; the
  latter is not authority over an explicit preference or current browser media.
- Pending material technical choices: none. Document revision approval remains
  required before implementation.
- Agent-owned details: helper boundaries, CSS variable names, cookie names, fixture
  composition and equivalent component decomposition.

## D1. Host-Scoped Shared Preference

- Authority: `theme-261031/REQ-2`.
- Decision owner: requester.
- Accepted: 2026-10-31 KST.

General and Admin share one host-only appearance preference at root cookie path.
Different hosts remain independent. This is presentation state, not authentication
or a cross-origin account setting.

Alternatives discussed were independent per-application preferences and one
same-host preference. The requester chose sharing. Root path allows runtime mounts
such as `/app` and `/admin` to observe the same choice. Omitting Domain prevents
unrequested subdomain sharing. Ports cannot isolate cookies; this also applies to
local HTTP.

## Fixed Architecture Consequences

These record required outcomes rather than additional discretionary decisions:

- Server rendering reads the preference and last resolved mode. Explicit Light or
  Dark wins over any cached resolution.
- System is resolved by current browser media before visible paint and while the
  page is open. A server without that information can only use a cached resolution
  or its initial light placeholder.
- Cookies are the only persisted appearance authority; Mantine's default
  localStorage manager must not introduce a competing preference.
- Shared web policy is browser-safe and isolated from session-cookie and RPC
  modules. Shared presentation receives state and callbacks.
- Existing forced-light bootstrap/provider behavior and baked production artwork
  are replaced; original art-generation provenance is retained.
