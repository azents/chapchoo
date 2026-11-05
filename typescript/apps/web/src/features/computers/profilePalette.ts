import type { AgentPaletteKey } from "@chapchoo/web-core/agent-profile";
import type { CSSProperties } from "react";

export function profileStyle(key: AgentPaletteKey): CSSProperties & {
  "--profile-surface": string;
  "--profile-ink": string;
  "--profile-border": string;
} {
  const colors = profilePalette[key];
  return {
    "--profile-surface": colors.surface,
    "--profile-ink": colors.emphasis,
    "--profile-border": colors.border,
  };
}

interface ProfilePalette {
  surface: string;
  foreground: string;
  emphasis: string;
  border: string;
  swatch: string;
}
function palette(key: AgentPaletteKey): ProfilePalette {
  return {
    surface: `var(--chapchoo-profile-${key}-surface)`,
    foreground: `var(--chapchoo-profile-${key}-ink)`,
    emphasis: `var(--chapchoo-profile-${key}-ink)`,
    border: `var(--chapchoo-profile-${key}-border)`,
    swatch: `var(--chapchoo-profile-${key}-swatch)`,
  };
}
/** A profile's palette never overrides the workspace's global appearance. */
export const profilePalette: Record<AgentPaletteKey, ProfilePalette> = {
  butter: palette("butter"),
  sage: palette("sage"),
  apricot: palette("apricot"),
  teal: palette("teal"),
  lilac: palette("lilac"),
  rose: palette("rose"),
};
