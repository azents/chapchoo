import type { CreationCopy } from "./creationCopy";
import type {
  CatalogueSnapshot,
  ConnectionPanelProps,
} from "@/shared/authentications/types";
import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type { AuthenticationCopy } from "@chapchoo/web-core/authentication-messages";
import type {
  AgentPaletteKey,
  AgentProfileImage,
  Page,
  ProviderChoice,
} from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
import type { AuthenticationView } from "@chapchoo/web-core/external-authentication";
import type { HandleDraft } from "@chapchoo/web-core/handle";

export interface InitialAgentCreation {
  teamId: string;
  teamHandle: string;
  identity: string;
  providers: Page<ProviderChoice>;
  authentications: CatalogueSnapshot[];
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
  step: "environment" | "authentication" | "profile";
  authentication: AuthenticationView | null;
  authenticationCopy: AuthenticationCopy;
  authenticationPanel: ConnectionPanelProps;
  onNext: () => void;
  onBack: () => void;
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
  onRandomPalette: () => void;
  onImagePage: (after: string | null) => void;
  onCreate: () => void;
  onRefresh: () => void;
  onLogout: () => void;
}
