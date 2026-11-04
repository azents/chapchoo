import type { AgentPaletteKey } from "@chapchoo/web-core/agent-profile";

/** Coordinated, theme-aware profile tokens; never a workspace-theme override. */
export const profilePalette: Record<
  AgentPaletteKey,
  { surface: string; foreground: string; emphasis: string }
> = {
  butter: {
    surface: "var(--chapchoo-color-accentSoft)",
    foreground: "var(--chapchoo-color-ink)",
    emphasis: "var(--chapchoo-color-accentText)",
  },
  sage: {
    surface: "var(--chapchoo-color-selected)",
    foreground: "var(--chapchoo-color-ink)",
    emphasis: "var(--chapchoo-color-success)",
  },
  apricot: {
    surface: "var(--mantine-color-orange-light)",
    foreground: "var(--mantine-color-text)",
    emphasis: "var(--mantine-color-orange-light-color)",
  },
  teal: {
    surface: "var(--mantine-color-teal-light)",
    foreground: "var(--mantine-color-text)",
    emphasis: "var(--mantine-color-teal-light-color)",
  },
  lilac: {
    surface: "var(--mantine-color-grape-light)",
    foreground: "var(--mantine-color-text)",
    emphasis: "var(--mantine-color-grape-light-color)",
  },
  rose: {
    surface: "var(--mantine-color-pink-light)",
    foreground: "var(--mantine-color-text)",
    emphasis: "var(--mantine-color-pink-light-color)",
  },
};
