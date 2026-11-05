import type { CreationCopy } from "./creationCopy";
import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type {
  AgentPaletteKey,
  AgentProfileImage,
} from "@chapchoo/web-core/agent-profile";
import type { AgentView, OwnerAction, Page } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
export interface ProfileDraft {
  name: string;
  image: Pick<AgentProfileImage, "id" | "assetKey">;
  paletteKey: AgentPaletteKey;
}
export type ProfileEditor =
  | { type: "closed" }
  | { type: "editing"; draft: ProfileDraft; error: string | null }
  | { type: "saving"; draft: ProfileDraft };
export type InitialAgents = (
  | { type: "list"; page: Page<AgentView>; after: string }
  | { type: "detail"; agent: AgentView }
) & {
  renderedAt: number;
  identity: string;
  teamId: string;
  teamHandle: string;
};
export interface AgentsInput {
  initial: InitialAgents;
  basePath: string;
}
export interface AgentsProps extends AgentsInput {
  access: "authorized" | "signedOut" | "unavailable";
  copy: ComputerCopy;
  profileCopy: CreationCopy;
  mobile: boolean;
  editor: ProfileEditor;
  images: Page<AgentProfileImage>;
  imageAfter: string | null;
  imagesUnavailable: boolean;
  onEditProfile: () => void;
  onProfileName: (value: string) => void;
  onProfileImage: (value: string) => void;
  onProfilePalette: (value: string) => void;
  onRandomProfilePalette: () => void;
  onImagePage: (after: string | null) => void;
  onSaveProfile: () => void;
  onCancelProfile: () => void;
  appearance?: ColorModeMenuProps;
  data: InitialAgents;
  stale: boolean;
  pending: boolean;
  error: string | null;
  confirmation: OwnerAction | "force" | null;
  onAction: (value: OwnerAction | "force") => void;
  onConfirm: () => void;
  onCancel: () => void;
  onRefresh: () => void;
  onLogout: () => void;
  formatTime: (value: number) => string;
}
