import { lazy, Suspense } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { ReactElement } from "react";

const MascotMotionDocument = lazy(
  (): Promise<typeof import("./MascotMotionDocument")> =>
    import("./MascotMotionDocument"),
);

/** Keep the chosen web theme scoped to a nested reference document. */
function MascotMotionStory({
  mode,
}: {
  mode: "comparison" | "family";
}): ReactElement {
  const params = new URLSearchParams(window.location.search);
  if (params.has("mascot-motion-document")) {
    return (
      <Suspense fallback={<p>Loading motion reference…</p>}>
        <MascotMotionDocument mode={mode} />
      </Suspense>
    );
  }
  params.set("mascot-motion-document", "");
  return (
    <iframe
      title="Chapchoo mascot motion reference"
      src={`./iframe.html?${params.toString()}`}
      style={{ display: "block", width: "100%", height: "100dvh", border: 0 }}
    />
  );
}

const meta = {
  title: "Chapchoo/Mascot motion",
  component: MascotMotionStory,
  parameters: {
    layout: "fullscreen",
    docs: {
      description: {
        component:
          "Reviewed motion: 5% extension, 1200ms, 30° clockwise, grounded with one small return bounce. Animate the Agent as interaction subject; accessory art stays static. Reference-local React components, not a rollout to General/Admin pages or a native implementation.",
      },
    },
  },
} satisfies Meta<typeof MascotMotionStory>;

export default meta;
type Story = StoryObj<typeof meta>;
export const ActorAndAccessory: Story = { args: { mode: "comparison" } };
export const Family: Story = { args: { mode: "family" } };
