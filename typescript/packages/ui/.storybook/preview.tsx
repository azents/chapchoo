import { AppMantineProvider } from "../src/provider";
import { FontFaces } from "../src/FontFaces";
import type { Preview } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";
import "@mantine/core/styles.css";

const preview: Preview = {
  parameters: { layout: "fullscreen" },
  initialGlobals: { colorScheme: "light" },
  decorators: [
    (Story, context): ReactElement => (
      <>
        <FontFaces basePath="" />
        {context.parameters.ownsAppearanceProvider === true ? (
          <Story />
        ) : (
          <AppMantineProvider
            forceColorScheme={
              context.globals.colorScheme === "dark" ? "dark" : "light"
            }
          >
            <Story />
          </AppMantineProvider>
        )}
      </>
    ),
  ],
};
export default preview;
