import { lazy, Suspense } from "react";
import { ReferenceFrame } from "./ReferenceFrame";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { ReactElement } from "react";

const SheetPreview = lazy(
  (): Promise<typeof import("./SheetPreview")> => import("./SheetPreview"),
);
const meta = { title: "Ownership examples/Shared Sheet" } satisfies Meta;
export default meta;
type Story = StoryObj<typeof meta>;
export const NativeBody: Story = {
  render: (): ReactElement =>
    new URLSearchParams(window.location.search).has("sheet-document") ? (
      <Suspense fallback={<p>Loading Sheet…</p>}>
        <SheetPreview />
      </Suspense>
    ) : (
      <ReferenceFrame
        title="Shared native Sheet"
        note="Production Sheet and SheetViewport own chrome, clipping and scroll boundaries; the reference does not redraw them."
      >
        <iframe
          className="reference-common-preview"
          title="Shared Sheet reference"
          src="./iframe.html?id=ownership-examples-shared-sheet--native-body&viewMode=story&sheet-document"
        />
      </ReferenceFrame>
    ),
};
