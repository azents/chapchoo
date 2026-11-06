"use client";

import { Box, MantineThemeProvider, useMantineTheme } from "@mantine/core";
import type { AgentPalette } from "./profileTokens";
import type { MantineThemeOverride } from "@mantine/core";
import type { ReactElement, ReactNode } from "react";

/** React theme scope follows portals without changing the surrounding workspace. */
export function AgentTheme({
  palette,
  children,
}: {
  palette: AgentPalette | null;
  children: ReactNode;
}): ReactElement {
  const parent = useMantineTheme();
  if (palette === null) {
    return <>{children}</>;
  }
  const primary = `agent-${palette}`;
  const color = (role: string): string =>
    `var(--chapchoo-profile-${palette}-${role})`;
  const variables = {
    "--chapchoo-color-accent": color("swatch"),
    "--chapchoo-color-accentHover": color("border"),
    "--chapchoo-color-accentSoft": color("surface"),
    "--chapchoo-color-accentText": color("ink"),
    "--chapchoo-color-focus": color("ink"),
    "--chapchoo-color-selected": color("surface"),
    "--mantine-color-anchor": color("ink"),
    "--profile-surface": color("surface"),
    "--profile-ink": color("ink"),
    "--profile-border": color("border"),
  };
  const theme: MantineThemeOverride = {
    primaryColor: primary,
    autoContrast: false,
    variantColorResolver: (input) => {
      const resolved = parent.variantColorResolver(input);
      if ((input.color ?? input.theme.primaryColor) !== primary) {
        return resolved;
      }
      if (input.variant === "filled") {
        return {
          ...resolved,
          background: color("swatch"),
          hover: color("border"),
          color: "var(--chapchoo-color-onAccent)",
          hoverColor: "var(--chapchoo-color-onAccent)",
        };
      }
      return {
        ...resolved,
        color: color("ink"),
        hoverColor: color("ink"),
        background:
          input.variant === "light" ? color("surface") : resolved.background,
        hover: color("surface"),
        border:
          input.variant === "outline"
            ? `1px solid ${color("ink")}`
            : resolved.border,
      };
    },
    components: {
      Anchor: {
        vars: () => ({ root: { "--mantine-color-anchor": color("ink") } }),
      },
      Modal: { vars: () => ({ content: variables }) },
      Drawer: { vars: () => ({ content: variables }) },
    },
  };
  return (
    <MantineThemeProvider theme={theme}>
      <Box style={variables} display="contents">
        {children}
      </Box>
    </MantineThemeProvider>
  );
}
