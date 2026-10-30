import { lazy, Suspense } from "react";
import { ReferenceFrame } from "./ReferenceFrame";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { ReactElement } from "react";

// Load global Mantine styles/provider effects only in the nested document.
const CommonUiPreview = lazy(
  (): Promise<typeof import("./CommonUiPreview")> =>
    import("./CommonUiPreview"),
);

const meta = {
  title: "Ownership examples/Common UI reuse",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const SharedReuse: Story = {
  render: (): ReactElement => {
    if (new URLSearchParams(window.location.search).has("common-ui-document")) {
      return (
        <Suspense fallback={<p>Loading common UI…</p>}>
          <CommonUiPreview />
        </Suspense>
      );
    }
    return (
      <ReferenceFrame
        title="Existing common UI reuse"
        note="The presentation is imported from common UI. Visual principles live in root DESIGN.md."
      >
        <iframe
          className="reference-common-preview"
          title="Isolated common UI example"
          src="./iframe.html?id=ownership-examples-common-ui-reuse--shared-reuse&viewMode=story&common-ui-document"
        />
      </ReferenceFrame>
    );
  },
};

export const DarkReuse: Story = {
  render: (): ReactElement => {
    if (new URLSearchParams(window.location.search).has("common-ui-document")) {
      return (
        <Suspense fallback={<p>Loading common UI…</p>}>
          <CommonUiPreview />
        </Suspense>
      );
    }
    return (
      <ReferenceFrame
        title="Adaptive common UI reuse"
        note="Real shared web components, dark semantic colors and transparent identity art."
      >
        <iframe
          className="reference-common-preview"
          title="Dark common UI example"
          src="./iframe.html?id=ownership-examples-common-ui-reuse--dark-reuse&viewMode=story&common-ui-document&dark"
        />
      </ReferenceFrame>
    );
  },
};
