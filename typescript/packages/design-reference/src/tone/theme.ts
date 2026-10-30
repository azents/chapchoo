import { mergeThemeOverrides, rem } from "@mantine/core";
import { theme, themeCssVariables } from "@chapchoo/ui/theme";
import {
  platformFonts,
  toneColors,
  toneMetrics,
} from "@chapchoo/ui/designTokens";
import type { CSSVariablesResolver, MantineThemeOverride } from "@mantine/core";
import type { ReferencePlatform } from "@chapchoo/ui/designTokens";

/** General iOS representation shares web values, differing only in typography. */
export function makeReferenceTheme(
  platform: ReferencePlatform,
): MantineThemeOverride {
  return mergeThemeOverrides(theme, {
    fontFamily: platformFonts[platform],
    headings: { fontFamily: platformFonts[platform] },
  });
}

/** Reference compositions retain their local naming, not independent values. */
export const referenceVariables: CSSVariablesResolver = (resolvedTheme) => {
  const shared = themeCssVariables(resolvedTheme);
  return {
    ...shared,
    variables: {
      ...shared.variables,
      ...Object.fromEntries(
        Object.entries(toneColors).map(([key, value]) => [
          `--cc-${key}`,
          value,
        ]),
      ),
      ...Object.fromEntries(
        Object.entries(toneMetrics.space).map(([key, value]) => [
          `--cc-space-${key}`,
          rem(value),
        ]),
      ),
      "--cc-radius-control": rem(toneMetrics.radius.control),
      "--cc-radius-surface": rem(toneMetrics.radius.surface),
      "--cc-measure": rem(toneMetrics.layout.readingMeasure),
      "--cc-shadow": "0 4px 18px rgb(53 58 54 / 4%)",
    },
  };
};
