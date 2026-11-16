---
title: Match floating-component translucency and material to the upper chat status summary; keep the summary as the baseline.
---

# Floating surface baseline

- ALWAYS use the upper chat status summary as the material reference for floating controls: match transparency, blur, border and highlight/shadow.
- Reuse the existing treatment within its owning context; preserve opaque foreground content and role-specific sizing.
- Read [DESIGN.md](../../../DESIGN.md#floating-control-surfaces) for the canonical reference. Keep ordinary content surfaces in their existing material.

## Bad

Give a floating return button its own stronger blur and opaque tinted fill, then alter the status summary to match.

## Good

Match the return button to the unchanged status-summary surface; change its label and width for unread state while preserving that material.
