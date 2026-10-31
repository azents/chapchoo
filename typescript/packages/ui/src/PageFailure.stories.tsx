import { PageFailure } from "./PageFailure";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
const meta = {
  title: "Common UI/Page Failure",
  component: PageFailure,
  args: {
    title: "Unable to load this page",
    message: "The page could not be loaded. Try again.",
    retryLabel: "Try again",
    onRetry: (): void => {},
  },
} satisfies Meta<typeof PageFailure>;
export default meta;
type Story = StoryObj<typeof meta>;
export const PrimaryDataUnavailable: Story = {};
