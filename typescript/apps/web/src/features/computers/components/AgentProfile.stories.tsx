import { validAgentPalette } from "@chapchoo/web-core/agent-profile";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useState } from "react";
import { creationCopy } from "../creationCopy";
import { AgentProfile } from "./AgentProfile";
import type { ProfileEditor } from "../types";
import type { AgentProfileProps } from "./AgentProfile";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import type { ReactElement } from "react";

const image = { id: "image", assetKey: "03-corgi", internalName: "Corgi" };
const draft = { name: "Changed companion", image, paletteKey: "rose" as const };
const meta = {
  title: "Computers/Agent profile",
  component: AgentProfile,
  args: {
    basePath: "",
    mobile: false,
    profileCopy: creationCopy("en-US"),
    copy: computerCopy("en-US"),
    agent: {
      id: "opaque-agent",
      teamId: "opaque-team",
      name: "Research companion",
      handle: "research",
      profileImage: image,
      paletteKey: "sage",
      computer: {
        intent: "running",
        power: "running",
        lastKnownPower: "running",
        phase: "completed",
        management: "connected",
        connector: "connected",
        saved: "verified",
        observedAt: null,
        requestId: "create",
        actions: [],
        forceRemoveAvailable: false,
        startDelayed: false,
        preparationRetryAvailable: false,
      },
    },
    editor: { type: "closed" },
    images: { items: [image], nextAfter: null },
    imageAfter: null,
    imagesUnavailable: false,
    onEditProfile: (): void => {},
    onProfileName: (): void => {},
    onProfileImage: (): void => {},
    onProfilePalette: (): void => {},
    onImagePage: (): void => {},
    onRefresh: (): void => {},
    onSaveProfile: (): void => {},
    onCancelProfile: (): void => {},
  },
} satisfies Meta<typeof AgentProfile>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Saved: Story = {};
export const Editing: Story = {
  args: { editor: { type: "editing", draft, error: null } },
};
export const Saving: Story = { args: { editor: { type: "saving", draft } } };
export const SaveFailure: Story = {
  args: {
    editor: { type: "editing", draft, error: creationCopy("en-US").saveFailed },
  },
};
export const CatalogueFailure: Story = {
  args: {
    ...Editing.args,
    imagesUnavailable: true,
    images: { items: [], nextAfter: null },
  },
};
export const Korean: Story = {
  args: {
    ...Editing.args,
    profileCopy: creationCopy("ko"),
    copy: computerCopy("ko-KR"),
    mobile: true,
  },
};
function InteractiveProfile(props: AgentProfileProps): ReactElement {
  const [editor, setEditor] = useState<ProfileEditor>({ type: "closed" });
  return (
    <AgentProfile
      {...props}
      editor={editor}
      onEditProfile={() =>
        setEditor({
          type: "editing",
          draft: {
            name: props.agent.name,
            image: props.agent.profileImage,
            paletteKey: props.agent.paletteKey,
          },
          error: null,
        })
      }
      onProfileName={(name) =>
        setEditor((current) =>
          current.type === "editing"
            ? { ...current, draft: { ...current.draft, name } }
            : current,
        )
      }
      onProfilePalette={(paletteKey) => {
        if (validAgentPalette(paletteKey)) {
          setEditor((current) =>
            current.type === "editing"
              ? { ...current, draft: { ...current.draft, paletteKey } }
              : current,
          );
        }
      }}
      onSaveProfile={() =>
        setEditor((current) =>
          current.type === "editing"
            ? { ...current, error: props.profileCopy.saveFailed }
            : current,
        )
      }
      onCancelProfile={() => setEditor({ type: "closed" })}
    />
  );
}
export const Interactive: Story = {
  render: (args): ReactElement => <InteractiveProfile {...args} />,
};
