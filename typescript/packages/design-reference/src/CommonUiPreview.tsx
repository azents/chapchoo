import { EmptyFeedback } from "@chapchoo/ui/Feedback";
import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { ColorModeSelector } from "@chapchoo/ui/ColorModeSelector";
import { Mascot } from "@chapchoo/ui/Mascot";
import { Stack } from "@mantine/core";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

/** Loaded only by the isolated reuse document, never by neutral story canvases. */
export default function CommonUiPreview(): ReactElement {
  const mode = new URLSearchParams(window.location.search).has("dark")
    ? "dark"
    : "light";
  return (
    <AppMantineProvider forceColorScheme={mode}>
      <FontFaces basePath="" />
      <Stack p="lg">
        <ColorModeSelector
          value={mode}
          label="Appearance"
          labels={{ system: "System", light: "Light", dark: "Dark" }}
          onChange={() => {}}
        />
        <Mascot size={72} />
        <EmptyFeedback
          title="Shared presentation example"
          message="The implementation is imported from @chapchoo/ui/Feedback. Changes to it belong in common UI."
        />
      </Stack>
    </AppMantineProvider>
  );
}
