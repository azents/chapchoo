# Confirmed harness setup concept

The requester confirmed this integrated product design on **2026-11-06 KST**.
This is a curated visual record, not feature implementation, a current-system
Spec or proof of authentication, permissions or Computer application.

## Core states

- [Account management](accounts-desktop.html)
- [Creation: registered cloud environment](computer-desktop.html)
- [Creation: Codex authentication](auth-mobile.html)
- [Creation: existing profile/name/handle/color](profile-mobile.html)
- [Existing Agent authentication row](agent-desktop.html)
- [Device Auth and official-page entry](device-mobile.html)
- [API key input](key-mobile.html)
- [Shared account application and partial recovery](application-desktop.html)

The current application's real components and shell were used as the starting
copy, with proposed authentication controls and the three-step creation journey.
Device Auth uses a line-SVG external-link icon; the AI connection navigation item
uses the shared frame with a proposed connection glyph. No other harness, persona,
personal Computer/connector UI, or user cloud registration was added.

Only eight representative states are retained. Intermediate options, exhaustive
responsive/error combinations and verification PNGs are excluded. Styles are
local and shared in `shared.css`; font subsets and artwork are deduplicated in
`assets/`. Total payload is approximately 1.32 MB before this short index.
See [asset licenses](../ASSETS.md) and [archive rules](../AGENTS.md).

## Capture and fidelity

Desktop: 1440 × 1100 CSS px; mobile: 390 × 844; DPR 1, ko-KR, light, reduced motion.
The exported resolved DOM/styles and form values replay independently with
JavaScript disabled and external requests blocked. All referenced local resources
loaded. Seven optimized pages matched their pre-optimization replay PNG exactly;
the profile page showed cold font-weight/glyph-metric variation while retaining
its fields, content, selected image/color and layout. Do not claim byte-identical
rendering across browsers, OS font fallbacks or unreviewed widths.

The controls are static illustrations, including the explicit prototype-only
approval-example control. They do not initiate real authentication or execute
changes. The actual approved behavior opens `https://auth.openai.com/codex/device`
from its external-link action; following that URL is not authentication success.
Codes, identities, plans and Agent counts are illustrative. Approval of this
concept does not authorize secret delivery, a token refresh architecture, Team
role mappings or visibility into another user's private Agent; those contracts
belong to subsequent Requirements and technical design.
