import type { MantineColorsTuple } from "@mantine/core";
/** Saved-profile colors from the approved character/profile study. */
const profileColors = {
  butter: ["#f4efdc", "#766019", "#e5dcbe", "#d7c788"],
  sage: ["#edf0e8", "#4b653e", "#dbe2d3", "#a7b997"],
  apricot: ["#f4e9df", "#955831", "#ead9c8", "#d8ab89"],
  teal: ["#eaf0ee", "#386869", "#d9e4df", "#91b7b2"],
  lilac: ["#efebf4", "#6c5683", "#ded4e8", "#baabcf"],
  rose: ["#f5e9eb", "#8d515f", "#e8d4da", "#d2a1af"],
} satisfies Record<string, [string, string, string, string]>;
const darkProfileColors = {
  butter: ["#343020", "#e3d394", "#514831", "#d7c788"],
  sage: ["#293127", "#bfd0ac", "#424f3b", "#a7b997"],
  apricot: ["#382c25", "#e2b998", "#574235", "#d8ab89"],
  teal: ["#253231", "#afd3cd", "#3b5150", "#91b7b2"],
  lilac: ["#302b38", "#d0bfe4", "#4c4259", "#baabcf"],
  rose: ["#382a30", "#e2b5c1", "#57404a", "#d2a1af"],
} satisfies Record<string, [string, string, string, string]>;
export type AgentPalette = keyof typeof profileColors;
export const agentThemeColors = Object.fromEntries(
  Object.entries(profileColors).map(([key, [surface, ink, border, swatch]]) => {
    const shades: MantineColorsTuple = [
      surface,
      surface,
      border,
      border,
      swatch,
      swatch,
      swatch,
      ink,
      ink,
      ink,
    ];
    return [`agent-${key}`, shades];
  }),
);
export function profileVariables(dark: boolean): Record<string, string> {
  return Object.fromEntries(
    Object.entries(dark ? darkProfileColors : profileColors).flatMap(
      ([key, [surface, ink, border, swatch]]): [string, string][] => [
        ...Object.entries({ surface, ink, border, swatch }).map(
          ([role, value]): [string, string] => [
            `--chapchoo-profile-${key}-${role}`,
            value,
          ],
        ),
        [`--mantine-color-agent-${key}-filled`, swatch],
        [`--mantine-color-agent-${key}-filled-hover`, border],
        [`--mantine-color-agent-${key}-light`, surface],
        [`--mantine-color-agent-${key}-light-hover`, border],
        [`--mantine-color-agent-${key}-light-color`, ink],
        [`--mantine-color-agent-${key}-outline`, ink],
        [`--mantine-color-agent-${key}-outline-hover`, surface],
        [`--mantine-color-agent-${key}-text`, ink],
      ],
    ),
  );
}
