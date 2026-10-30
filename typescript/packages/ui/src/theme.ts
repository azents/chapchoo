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
import { toneColors, platformFonts } from "./designTokens";
import classes from "./theme.module.css";
import "./editors.css";
import type { CSSVariablesResolver } from "@mantine/core";

export const themeCssVariables: CSSVariablesResolver = () => ({
  variables: {
    "--chapchoo-font-input-mobile": "max(16px, 1rem)",
    ...Object.fromEntries(
      Object.entries(toneColors).map(([key, value]) => [
        `--chapchoo-color-${key}`,
        value,
      ]),
    ),
    "--chapchoo-color-surface-raised": toneColors.surfaceWarm,
    "--chapchoo-color-surface-hover": toneColors.selected,
    "--chapchoo-color-border-strong": toneColors.borderStrong,
    "--chapchoo-color-error": toneColors.danger,
    "--chapchoo-color-subtle": toneColors.secondary,
    "--chapchoo-color-inverse": toneColors.ink,
    "--chapchoo-color-on-inverse": toneColors.surface,
    "--chapchoo-color-inverse-muted": toneColors.navigation,
  },
  light: {
    "--mantine-color-body": toneColors.canvas,
    "--mantine-color-text": toneColors.ink,
    "--mantine-color-dimmed": toneColors.secondary,
    "--mantine-color-default": toneColors.surfaceWarm,
    "--mantine-color-default-hover": toneColors.selected,
    "--mantine-color-default-border": toneColors.border,
    "--mantine-color-placeholder": toneColors.muted,
    "--mantine-color-anchor": toneColors.onAccent,
  },
  dark: {},
});

/** Selected light web design; DESIGN.md is the visual authority. */
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
    if (color === "butter") {
      if (input.variant === "filled") {
        return {
          ...resolved,
          color: toneColors.onAccent,
          hoverColor: toneColors.onAccent,
        };
      }
      if (
        ["outline", "light", "subtle", "transparent"].includes(input.variant)
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
    }
    return resolved;
  },
  components: {
    Button: Button.extend({
      defaultProps: { variant: "filled" },
      classNames: { root: classes.button },
    }),
    ActionIcon: ActionIcon.extend({
      defaultProps: { variant: "subtle" },
      classNames: { root: classes.touchTarget },
    }),
    Anchor: Anchor.extend({ classNames: { root: classes.anchor } }),
    Badge: Badge.extend({ defaultProps: { variant: "light" } }),
    ThemeIcon: ThemeIcon.extend({ defaultProps: { variant: "light" } }),
    Chip: Chip.extend({ defaultProps: { variant: "light" } }),
    Input: Input.extend({ classNames: { input: classes.input } }),
    Paper: Paper.extend({
      defaultProps: { radius: "lg" },
      classNames: { root: classes.paper },
    }),
    Loader: Loader.extend({ defaultProps: { color: "dark" } }),
    Modal: Modal.extend({
      defaultProps: { radius: "lg", padding: "xl" },
      classNames: { content: classes.modal },
    }),
    NavLink: NavLink.extend({
      classNames: { root: classes.navLink, label: classes.navLabel },
    }),
    Menu: Menu.extend({
      defaultProps: { radius: "md", shadow: "xs" },
      classNames: { item: classes.menuItem },
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
