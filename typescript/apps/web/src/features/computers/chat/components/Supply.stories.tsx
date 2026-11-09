import { useState } from "react";
import { supplyCopy } from "../supplyCopy";
import { Supply } from "./Supply";
import type { SupplyProps } from "../supplyTypes";
import type { HarnessSupplyChoice } from "@chapchoo/web-core/conversation-types";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

function Fixture({
  mode,
  locale,
}: {
  mode:
    "default" | "different" | "uncertain" | "readonly" | "loading" | "error";
  locale: string;
}): ReactElement {
  const copy = supplyCopy(locale);
  const [draft, setDraft] = useState<HarnessSupplyChoice>({ type: "managed" });
  const [reads, setReads] = useState(0);
  const [saves, setSaves] = useState(0);
  const state: SupplyProps["state"] =
    mode === "loading"
      ? { type: "loading" }
      : mode === "error"
        ? { type: "error" }
        : {
            type: "ready",
            saved: {
              choice:
                mode === "different"
                  ? { type: "existing", path: "/owner/selected codex" }
                  : { type: "managed" },
            },
            applied: mode === "default" ? void 0 : { type: "managed" },
            draft,
            submission: mode === "uncertain" ? "uncertain" : "ready",
            stale: false,
            writable: mode !== "readonly" && mode !== "uncertain",
            saveable: draft.type === "existing" && draft.path.startsWith("/"),
          };
  return (
    <>
      <Supply
        copy={copy}
        opened
        state={state}
        onToggle={() => {}}
        onMode={(type) =>
          setDraft(
            type === "managed"
              ? { type: "managed" }
              : { type: "existing", path: "" },
          )
        }
        onPath={(path) => setDraft({ type: "existing", path })}
        onSave={() => setSaves((value) => value + 1)}
        onRead={() => setReads((value) => value + 1)}
      />
      <output aria-label="Fixture read count">{reads}</output>
      <output aria-label="Fixture save count">{saves}</output>
    </>
  );
}
const meta = {
  title: "Computers/Chat/Supply",
  component: Fixture,
  args: { mode: "default", locale: "en" },
} satisfies Meta<typeof Fixture>;
export default meta;
type Story = StoryObj<typeof meta>;
export const ManagedDefault: Story = {};
export const SavedDiffersFromApplied: Story = { args: { mode: "different" } };
export const UncertainSave: Story = { args: { mode: "uncertain" } };
export const ReadOnly: Story = { args: { mode: "readonly" } };
export const Loading: Story = { args: { mode: "loading" } };
export const Error: Story = { args: { mode: "error" } };
export const Korean: Story = { args: { mode: "different", locale: "ko" } };
export const Japanese: Story = { args: { mode: "different", locale: "ja" } };
export const French: Story = { args: { mode: "different", locale: "fr" } };
