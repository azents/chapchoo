import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type {
  AgentView,
  LifecycleAction,
  Page,
  ProviderChoice,
} from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
export type InitialAgents = (
  | { type: "list"; page: Page<AgentView>; after: string }
  | { type: "detail"; agent: AgentView }
) & { renderedAt: number; identity: string };
export interface AgentsInput {
  initial: InitialAgents;
  basePath: string;
}
export interface AgentsProps extends AgentsInput {
  access: "authorized" | "signedOut" | "unavailable";
  copy: ComputerCopy;
  appearance?: ColorModeMenuProps;
  data: InitialAgents;
  stale: boolean;
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
        stale: boolean;
        items: ProviderChoice[];
        after: string | null;
        nextAfter: string | null;
      };
  confirmation: LifecycleAction | "force" | null;
  onName: (value: string) => void;
  onProvider: (value: string) => void;
  onProviderPage: (after: string | null) => void;
  onCreate: () => void;
  onAction: (value: LifecycleAction | "force") => void;
  onConfirm: () => void;
  onCancel: () => void;
  onRefresh: () => void;
  onLogout: () => void;
  formatTime: (value: number) => string;
}
