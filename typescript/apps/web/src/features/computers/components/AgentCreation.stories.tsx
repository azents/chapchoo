import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { creationCopy } from "../creationCopy";
import { AgentCreation } from "./AgentCreation";
import type { InitialAgentCreation } from "../creationTypes";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

const provider = { id: "provider", name: "Community Kubernetes" };
const image = { id: "image", internalName: "Corgi", assetKey: "03-corgi" };
const initial: InitialAgentCreation = {
  teamId: "team",
  teamHandle: "my",
  identity: "owner@example.invalid",
  providers: { items: [provider], nextAfter: null },
  images: { items: [image], nextAfter: null },
  initialImage: image,
  initialPalette: "sage",
  renderedAt: 1793577600000,
};
const meta = {
  title: "Computers/Agent creation",
  component: AgentCreation,
  args: {
    initial,
    basePath: "",
    access: "authorized",
    copy: computerCopy("en-US"),
    profileCopy: creationCopy("en-US"),
    pending: false,
    error: null,
    creationLocked: false,
    name: "Research companion",
    providerId: "provider",
    handle: {
      draft: {
        source: "automatic",
        kind: "agent",
        base: "research-companion",
        candidate: "research-compani",
      },
      status: "available",
    },
    image,
    paletteKey: "sage",
    images: initial.images,
    imageAfter: null,
    imagesUnavailable: false,
    providers: {
      type: "loaded",
      items: [provider],
      after: null,
      nextAfter: null,
      stale: false,
    },
    onName: (): void => {},
    onProvider: (): void => {},
    onProviderPage: (): void => {},
    onHandle: (): void => {},
    onImage: (): void => {},
    onPalette: (): void => {},
    onRandomPalette: (): void => {},
    onImagePage: (): void => {},
    onCreate: (): void => {},
    onRefresh: (): void => {},
    onLogout: (): void => {},
  },
} satisfies Meta<typeof AgentCreation>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Ready: Story = {};
export const Empty: Story = {
  args: {
    providers: {
      type: "loaded",
      items: [],
      after: null,
      nextAfter: null,
      stale: false,
    },
    name: "",
    providerId: "",
  },
};
export const ProviderReadFailure: Story = {
  args: { providers: { type: "error" } },
};
export const RetainedCreationForm: Story = {
  args: {
    providers: {
      type: "loaded",
      items: [provider],
      after: null,
      nextAfter: null,
      stale: true,
    },
  },
};
export const UnresolvedCreationWithoutProvider: Story = {
  args: {
    creationLocked: true,
    error: computerCopy("en-US").lostSubmission,
    providers: {
      type: "loaded",
      items: [],
      after: null,
      nextAfter: null,
      stale: false,
    },
  },
};
export const Pending: Story = { args: { pending: true, creationLocked: true } };
export const CheckingHandle: Story = {
  args: {
    handle: {
      draft: { source: "manual", candidate: "my-agent" },
      status: "checking",
    },
  },
};
export const OccupiedHandle: Story = {
  args: {
    handle: {
      draft: { source: "manual", candidate: "my-agent" },
      status: "occupied",
    },
  },
};
export const CatalogueUnavailable: Story = {
  args: { imagesUnavailable: true },
};
export const Japanese: Story = {
  args: {
    copy: computerCopy("ja-JP"),
    profileCopy: creationCopy("ja-JP"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
export const French: Story = {
  args: {
    copy: computerCopy("fr-FR"),
    profileCopy: creationCopy("fr-FR"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
export const Korean: Story = {
  args: {
    copy: computerCopy("ko-KR"),
    profileCopy: creationCopy("ko-KR"),
    providers: Empty.args?.providers,
    name: "",
    providerId: "",
  },
};
