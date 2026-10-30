import { EmptyFeedback } from "@chapchoo/ui/Feedback";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

/** Loaded only by the isolated reuse document, never by neutral story canvases. */
export default function CommonUiPreview(): ReactElement {
  return (
    <AppMantineProvider forceColorScheme="light">
      <EmptyFeedback
        title="Shared presentation example"
        message="The implementation is imported from @chapchoo/ui/Feedback. Changes to it belong in common UI."
      />
    </AppMantineProvider>
  );
}
