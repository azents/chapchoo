import {
  createTheme,
  defaultVariantColorsResolver,
  rem,
  ActionIcon,
  Anchor,
  Badge,
  Button,
  Checkbox,
  Chip,
  Input,
  Loader,
  Menu,
  Modal,
  NavLink,
  Paper,
  Radio,
  Switch,
  Table,
  Tabs,
  ThemeIcon,
} from "@mantine/core";
import { toneColors, darkToneColors, platformFonts } from "./designTokens";
import classes from "./theme.module.css";
import "./editors.css";
import type { CSSVariablesResolver } from "@mantine/core";

function modeVariables(colors: typeof toneColors): Record<string, string> {
  return {
    ...Object.fromEntries(
      Object.entries(colors).map(([key, value]) => [
        `--chapchoo-color-${key}`,
        value,
      ]),
    ),
    "--chapchoo-color-surface-raised": colors.surfaceWarm,
    "--chapchoo-color-overlay": colors.overlay,
    "--chapchoo-color-surface-hover": colors.selected,
    "--chapchoo-color-border-strong": colors.borderStrong,
    "--chapchoo-color-error": colors.danger,
    "--chapchoo-color-subtle": colors.secondary,
    "--chapchoo-color-inverse": colors.ink,
    "--chapchoo-color-on-inverse": colors.surface,
    "--chapchoo-color-inverse-muted": colors.navigation,
    "--mantine-color-body": colors.canvas,
    "--mantine-color-text": colors.ink,
    "--mantine-color-dimmed": colors.secondary,
    "--mantine-color-default": colors.surfaceWarm,
    "--mantine-color-default-hover": colors.selected,
    "--mantine-color-default-border": colors.border,
    "--mantine-color-placeholder": colors.muted,
    "--mantine-color-anchor": colors.accentText,
    "--mantine-color-error": colors.danger,
  };
}

export const themeCssVariables: CSSVariablesResolver = () => ({
  variables: {
    "--chapchoo-font-input-mobile": "max(16px, 1rem)",
    "--chapchoo-line-height-control": "1.5",
  },
  light: modeVariables(toneColors),
  dark: modeVariables(darkToneColors),
});

/** Adaptive web design; DESIGN.md is the visual authority. */
export const theme = createTheme({
  primaryColor: "butter",
  primaryShade: 6,
  autoContrast: true,
  colors: {
    butter: [
      "#fffbee",
      toneColors.accentSoft,
      "#f8e8b2",
      "#f5dfa0",
      "#f2d68a",
      "#efcf79",
      toneColors.accent,
      toneColors.accentHover,
      "#b38b2d",
      "#705619",
    ],
    red: [
      "#fff5f2",
      toneColors.dangerSoft,
      "#f4d7d1",
      "#e9b9ae",
      "#d98d7f",
      "#bd6052",
      toneColors.danger,
      "#8d352d",
      "#742b24",
      "#5c211b",
    ],
    green: [
      "#f7faf3",
      toneColors.successSoft,
      "#e1eacc",
      "#cbdbb0",
      "#abc58a",
      "#7d9a5e",
      toneColors.success,
      "#425c31",
      "#354b26",
      "#293b1d",
    ],
    blue: [
      "#f3f7fa",
      "#edf0f3",
      "#dbe5eb",
      "#c3d4df",
      "#9eb7c8",
      "#7494ac",
      "#476b84",
      "#3b5b73",
      "#304c61",
      "#263b4b",
    ],
  },
  fontFamily: platformFonts.web,
  fontFamilyMonospace: '"Geist Mono", Menlo, Consolas, monospace',
  headings: {
    fontFamily: platformFonts.web,
    fontWeight: "600",
    sizes: {
      h1: { fontSize: rem(28), lineHeight: "1.3" },
      h2: { fontSize: rem(24), lineHeight: "1.35" },
      h3: { fontSize: rem(20), lineHeight: "1.4" },
      h4: { fontSize: rem(18), lineHeight: "1.4" },
      h5: { fontSize: rem(16), lineHeight: "1.5" },
    },
  },
  fontSizes: {
    xs: rem(12),
    sm: rem(14),
    md: rem(16),
    lg: rem(20),
    xl: rem(24),
  },
  lineHeights: { xs: "1.5", sm: "1.6", md: "1.7", lg: "1.5", xl: "1.4" },
  spacing: {
    "2xs": rem(4),
    xs: rem(4),
    sm: rem(8),
    md: rem(12),
    lg: rem(16),
    xl: rem(24),
    "2xl": rem(32),
    "3xl": rem(48),
    "4xl": rem(64),
  },
  radius: { xs: rem(6), sm: rem(8), md: rem(12), lg: rem(18), xl: rem(24) },
  defaultRadius: "md",
  focusRing: "auto",
  focusClassName: classes.focus,
  variantColorResolver: (input) => {
    const resolved = defaultVariantColorsResolver(input);
    const color = input.color ?? input.theme.primaryColor;
    const role =
      color === "butter"
        ? "accent"
        : color === "red"
          ? "danger"
          : color === "green"
            ? "success"
            : null;
    if (role) {
      const foreground = `var(--chapchoo-color-${role === "accent" ? "accentText" : role})`;
      if (input.variant === "filled") {
        return {
          ...resolved,
          background:
            role === "accent"
              ? "var(--chapchoo-color-accent)"
              : resolved.background,
          hover:
            role === "accent"
              ? "var(--chapchoo-color-accentHover)"
              : resolved.hover,
          color:
            role === "accent"
              ? "var(--chapchoo-color-onAccent)"
              : resolved.color,
          hoverColor:
            role === "accent"
              ? "var(--chapchoo-color-onAccent)"
              : resolved.hoverColor,
        };
      }
      if (
        ["outline", "light", "subtle", "transparent"].includes(input.variant)
      ) {
        return {
          ...resolved,
          background:
            input.variant === "light"
              ? `var(--chapchoo-color-${role}Soft)`
              : resolved.background,
          hover: `var(--chapchoo-color-${role}Soft)`,
          color: foreground,
          hoverColor: foreground,
          border:
            input.variant === "outline"
              ? `1px solid ${foreground}`
              : resolved.border,
        };
      }
    }
    return resolved;
  },
  components: {
    Button: Button.extend({
      defaultProps: { variant: "filled" },
      classNames: {
        root: classes.button,
        inner: classes.buttonInner,
        label: classes.buttonLabel,
      },
    }),
    ActionIcon: ActionIcon.extend({
      defaultProps: { variant: "subtle" },
      classNames: { root: classes.touchTarget },
    }),
    Anchor: Anchor.extend({ classNames: { root: classes.anchor } }),
    Badge: Badge.extend({ defaultProps: { variant: "light" } }),
    ThemeIcon: ThemeIcon.extend({ defaultProps: { variant: "light" } }),
    Chip: Chip.extend({
      defaultProps: { variant: "light" },
      classNames: { label: classes.chipLabel },
    }),
    Input: Input.extend({ classNames: { input: classes.input } }),
    Paper: Paper.extend({
      defaultProps: { radius: "lg" },
      classNames: { root: classes.paper },
    }),
    Loader: Loader.extend({
      defaultProps: { color: "var(--chapchoo-color-secondary)" },
    }),
    Modal: Modal.extend({
      defaultProps: { radius: "lg", padding: "xl" },
      classNames: { content: classes.modal, header: classes.modalHeader },
    }),
    NavLink: NavLink.extend({
      classNames: { root: classes.navLink, label: classes.navLabel },
    }),
    Menu: Menu.extend({
      defaultProps: { radius: "md", shadow: "xs" },
      classNames: { item: classes.menuItem, dropdown: classes.menuDropdown },
    }),
    Table: Table.extend({
      classNames: { th: classes.tableHeading, td: classes.tableCell },
    }),
    Tabs: Tabs.extend({ classNames: { tab: classes.tab } }),
    Checkbox: Checkbox.extend({
      defaultProps: { iconColor: toneColors.onAccent },
      classNames: { input: classes.choice },
    }),
    Radio: Radio.extend({
      defaultProps: { iconColor: toneColors.onAccent },
      classNames: { radio: classes.choice },
    }),
    Switch: Switch.extend({ classNames: { track: classes.switchTrack } }),
  },
});
