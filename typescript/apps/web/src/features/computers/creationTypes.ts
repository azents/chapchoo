import type { CreationCopy } from "./creationCopy";
import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type {
  AgentPaletteKey,
  AgentProfileImage,
  Page,
  ProviderChoice,
} from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
import type { HandleDraft } from "@chapchoo/web-core/handle";

export interface InitialAgentCreation {
  teamId: string;
  identity: string;
  providers: Page<ProviderChoice>;
  images: Page<AgentProfileImage>;
  initialImage: AgentProfileImage;
  initialPalette: AgentPaletteKey;
  renderedAt: number;
}
export interface AgentCreationInput {
  initial: InitialAgentCreation;
  basePath: string;
}
export interface AgentCreationProps extends AgentCreationInput {
  access: "authorized" | "signedOut" | "unavailable";
  copy: ComputerCopy;
  profileCopy: CreationCopy;
  appearance?: ColorModeMenuProps;
  pending: boolean;
  error: string | null;
  creationLocked: boolean;
  name: string;
  providerId: string;
  handle: {
    draft: HandleDraft;
    status:
      | "idle"
      | "checking"
      | "available"
      | "invalid"
      | "occupied"
      | "exhausted"
      | "unavailable";
  };
  image: AgentProfileImage;
  paletteKey: AgentPaletteKey;
  images: Page<AgentProfileImage>;
  imageAfter: string | null;
  imagesUnavailable: boolean;
  providers:
    | { type: "loading" }
    | { type: "error" }
    | {
        type: "loaded";
        items: ProviderChoice[];
        after: string | null;
        nextAfter: string | null;
        stale: boolean;
      };
  onName: (value: string) => void;
  onProvider: (value: string) => void;
  onProviderPage: (after: string | null) => void;
  onHandle: (value: string) => void;
  onImage: (value: string) => void;
  onPalette: (value: string) => void;
  onImagePage: (after: string | null) => void;
  onCreate: () => void;
  onRefresh: () => void;
  onLogout: () => void;
}
