import { ReferenceFrame } from "./ReferenceFrame";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Ownership examples/Non-web representation",
  component: ReferenceFrame,
} satisfies Meta<typeof ReferenceFrame>;

export default meta;
type Story = StoryObj<typeof meta>;

export const LocalOutline: Story = {
  args: {
    title: "Non-web reference ownership",
    note: "A web-rendered structural placeholder, not a mobile screen proposal or native implementation.",
    children: (
      <>
        <section className="reference-outline" aria-label="Non-web outline">
          <h2>Platform-specific region</h2>
          <p>Future platform-specific content can be represented here.</p>
          <hr />
          <p>Layout, controls and styling have not been selected.</p>
        </section>
        <p>
          This representation stays in the reference package after review. A
          native implementation belongs to its target platform, not common web
          UI.
        </p>
      </>
    ),
  },
};
