import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { Sheet, SheetViewport } from "@chapchoo/ui/Sheet";
import { Button, Stack, Text } from "@mantine/core";
import { useState } from "react";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

/** Uses the production web pattern; no separate reference implementation. */
export default function SheetPreview(): ReactElement {
  const [opened, setOpened] = useState(true);
  return (
    <AppMantineProvider forceColorScheme="dark">
      <FontFaces basePath="" />
      <Button onClick={() => setOpened(true)}>Open sheet</Button>
      <Sheet
        opened={opened}
        onClose={() => setOpened(false)}
        title="History"
        size="80dvh"
        closeButtonProps={{ "aria-label": "Close" }}
      >
        <SheetViewport>
          <Stack p="lg">
            {Array.from({ length: 20 }, (_, index) => (
              <Text key={index}>
                Record {index + 1}: native body scrolling with stationary
                chrome.
              </Text>
            ))}
          </Stack>
        </SheetViewport>
      </Sheet>
    </AppMantineProvider>
  );
}
