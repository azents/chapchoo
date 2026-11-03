import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type { AgentView, OwnerAction, Page } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
export type InitialAgents = (
  | { type: "list"; page: Page<AgentView>; after: string }
  | { type: "detail"; agent: AgentView }
) & { renderedAt: number; identity: string; teamId: string };
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
  confirmation: OwnerAction | "force" | null;
  onAction: (value: OwnerAction | "force") => void;
  onConfirm: () => void;
  onCancel: () => void;
  onRefresh: () => void;
  onLogout: () => void;
  formatTime: (value: number) => string;
}
