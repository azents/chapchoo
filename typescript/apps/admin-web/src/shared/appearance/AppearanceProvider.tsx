"use client";

import { AppMantineProvider } from "@chapchoo/ui/provider";
import {
  createColorSchemeManager,
  syncDocumentColorMode,
  toMantineMode,
} from "@chapchoo/web-core/color-mode";
import { useComputedColorScheme, useMantineColorScheme } from "@mantine/core";
import { useIsomorphicEffect } from "@mantine/hooks";
import { useMemo } from "react";
import type { ColorModeSnapshot } from "@chapchoo/web-core/color-mode";
import type { ReactNode } from "react";

function DocumentAppearance(): null {
  const { colorScheme } = useMantineColorScheme();
  const mode = useComputedColorScheme("light", {
    getInitialValueInEffect: false,
  });
  useIsomorphicEffect(() => {
    syncDocumentColorMode({
      preference: colorScheme === "auto" ? "system" : colorScheme,
      mode,
    });
  }, [colorScheme, mode]);
  return null;
}

export function AppearanceProvider({
  initial,
  children,
}: {
  initial: ColorModeSnapshot;
  children: ReactNode;
}): ReactNode {
  const manager = useMemo(
    () => createColorSchemeManager(initial.preference),
    [initial.preference],
  );
  return (
    <AppMantineProvider
      defaultColorScheme={toMantineMode(initial.preference)}
      colorSchemeManager={manager}
    >
      <DocumentAppearance />
      {children}
    </AppMantineProvider>
  );
}
