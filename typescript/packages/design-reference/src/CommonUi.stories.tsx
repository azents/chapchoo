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

export const ProvisionalReuse: Story = {
  render: (): ReactElement => {
    if (new URLSearchParams(window.location.search).has("common-ui-document")) {
      return (
        <Suspense fallback={<p>Loading provisional common UI…</p>}>
          <CommonUiPreview />
        </Suspense>
      );
    }
    return (
      <ReferenceFrame
        title="Existing common UI reuse"
        note="Provisional Azents-derived presentation. This example demonstrates an import boundary, not approved Chapchoo styling."
      >
        <iframe
          className="reference-common-preview"
          title="Isolated provisional common UI example"
          src="./iframe.html?id=ownership-examples-common-ui-reuse--provisional-reuse&viewMode=story&common-ui-document"
        />
      </ReferenceFrame>
    );
  },
};
