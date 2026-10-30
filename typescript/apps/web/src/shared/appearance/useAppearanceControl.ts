"use client";

import { useMantineColorScheme } from "@mantine/core";
import { useTranslations } from "next-intl";
import type { ColorModeSelectorProps } from "@chapchoo/ui/ColorModeSelector";

export function useAppearanceControl(): ColorModeSelectorProps {
  const text = useTranslations("Appearance");
  const { colorScheme, setColorScheme } = useMantineColorScheme();
  return {
    value: colorScheme === "auto" ? "system" : colorScheme,
    label: text("label"),
    labels: {
      system: text("system"),
      light: text("light"),
      dark: text("dark"),
    },
    onChange: (value) => setColorScheme(value === "system" ? "auto" : value),
  };
}
