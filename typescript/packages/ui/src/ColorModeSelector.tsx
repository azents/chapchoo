import { NativeSelect } from "@mantine/core";
import type { ReactElement } from "react";

export interface ColorModeSelectorProps {
  value: "system" | "light" | "dark";
  label: string;
  labels: { system: string; light: string; dark: string };
  onChange: (value: "system" | "light" | "dark") => void;
}

/** Presentation only; the application owns persistence and localized copy. */
export function ColorModeSelector({
  value,
  label,
  labels,
  onChange,
}: ColorModeSelectorProps): ReactElement {
  return (
    <NativeSelect
      aria-label={label}
      value={value}
      w="auto"
      data={[
        { value: "system", label: labels.system },
        { value: "light", label: labels.light },
        { value: "dark", label: labels.dark },
      ]}
      onChange={(event) => {
        const next = event.currentTarget.value;
        if (next === "system" || next === "light" || next === "dark") {
          onChange(next);
        }
      }}
    />
  );
}
