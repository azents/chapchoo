import {
  Button,
  createTheme,
  defaultVariantColorsResolver,
  rem,
} from "@mantine/core";
import { platformFonts, toneColors, toneMetrics } from "./tokens";
import type { CSSVariablesResolver, MantineThemeOverride } from "@mantine/core";
import type { ReferencePlatform } from "./tokens";

/** Only this scoped provider opts into the selected reference tone. */
export function makeReferenceTheme(
  platform: ReferencePlatform,
): MantineThemeOverride {
  return createTheme({
    primaryColor: "butter",
    primaryShade: 6,
    colors: {
      red: [
        "#fff5f2",
        "#fbece8",
        "#f4d7d1",
        "#e9b9ae",
        "#d98d7f",
        "#bd6052",
        toneColors.danger,
        "#8d352d",
        "#742b24",
        "#5c211b",
      ],
      butter: [
        "#fffbee",
        "#fbf2d0",
        "#f8e8b2",
        "#f5dfa0",
        "#f2d68a",
        "#efcf79",
        toneColors.accent,
        toneColors.accentHover,
        "#b38b2d",
        "#705619",
      ],
    },
    fontFamily: platformFonts[platform],
    headings: { fontFamily: platformFonts[platform], fontWeight: "600" },
    fontSizes: {
      xs: rem(12),
      sm: rem(14),
      md: rem(16),
      lg: rem(20),
      xl: rem(24),
    },
    spacing: { xs: rem(4), sm: rem(8), md: rem(12), lg: rem(16), xl: rem(24) },
    radius: { xs: rem(6), sm: rem(8), md: rem(12), lg: rem(18), xl: rem(24) },
    defaultRadius: "md",
    focusRing: "always",
    autoContrast: true,
    variantColorResolver: (input) => {
      const resolved = defaultVariantColorsResolver(input);
      if (
        (input.color ?? input.theme.primaryColor) === "butter" &&
        (input.variant === "outline" ||
          input.variant === "light" ||
          input.variant === "subtle")
      ) {
        return {
          ...resolved,
          color: toneColors.onAccent,
          hoverColor: toneColors.onAccent,
          border:
            input.variant === "outline"
              ? `1px solid ${toneColors.focus}`
              : resolved.border,
        };
      }
      return resolved;
    },
    components: {
      Button: Button.extend({
        styles: (_theme, props) => ({
          root:
            props.variant === undefined || props.variant === "filled"
              ? {
                  color:
                    props.color === "red"
                      ? toneColors.surface
                      : toneColors.onAccent,
                }
              : {},
        }),
      }),
    },
  });
}

export const referenceVariables: CSSVariablesResolver = () => ({
  variables: {
    ...Object.fromEntries(
      Object.entries(toneColors).map(([key, value]) => [`--cc-${key}`, value]),
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
  light: {
    "--mantine-color-body": toneColors.canvas,
    "--mantine-color-text": toneColors.ink,
    "--mantine-color-dimmed": toneColors.secondary,
    "--mantine-color-default": toneColors.surfaceWarm,
    "--mantine-color-default-hover": toneColors.selected,
    "--mantine-color-default-border": toneColors.border,
    "--mantine-color-placeholder": toneColors.muted,
  },
  dark: {},
});
