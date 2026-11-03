import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type { Page, ProviderChoice } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";

export interface InitialAgentCreation {
  teamId: string;
  identity: string;
  providers: Page<ProviderChoice>;
  renderedAt: number;
}
export interface AgentCreationInput {
  initial: InitialAgentCreation;
  basePath: string;
}
export interface AgentCreationProps extends AgentCreationInput {
  access: "authorized" | "signedOut" | "unavailable";
  copy: ComputerCopy;
  appearance?: ColorModeMenuProps;
  pending: boolean;
  error: string | null;
  creationLocked: boolean;
  name: string;
  providerId: string;
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
  onCreate: () => void;
  onRefresh: () => void;
  onLogout: () => void;
}
