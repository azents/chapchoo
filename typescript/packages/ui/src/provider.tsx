"use client";

/**
 * Mantine theme provider.
 *
 * variantColorResolver contains a function, so Server Component cannot pass
 * theme directly to MantineProvider. This provider is separated as a Client
 * Component to provide the theme.
 */
import { MantineProvider } from "@mantine/core";
import { ModalsProvider } from "@mantine/modals";
import { theme, themeCssVariables } from "./theme";
import type { MantineColorSchemeManager } from "@mantine/core";
import type { ReactNode } from "react";

type MantineColorScheme = "light" | "dark" | "auto";

export interface AppMantineProviderProps {
  children: ReactNode;
  defaultColorScheme?: MantineColorScheme;
  forceColorScheme?: "light" | "dark";
  colorSchemeManager?: MantineColorSchemeManager;
}

export function AppMantineProvider({
  children,
  defaultColorScheme = "light",
  forceColorScheme,
  colorSchemeManager,
}: AppMantineProviderProps): ReactNode {
  return (
    <MantineProvider
      theme={theme}
      cssVariablesResolver={themeCssVariables}
      defaultColorScheme={defaultColorScheme}
      forceColorScheme={forceColorScheme}
      colorSchemeManager={colorSchemeManager}
    >
      <ModalsProvider>{children}</ModalsProvider>
    </MantineProvider>
  );
}
