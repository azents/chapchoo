import type { Preview } from "@storybook/react-vite";
import "../src/reference.css";

const preview: Preview = {
  parameters: {
    layout: "padded",
    options: {
      storySort: {
        order: ["Start here", "Ownership examples"],
      },
    },
  },
};

export default preview;
