import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type {
  InventoryView,
  Page,
  ProviderView,
} from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
export type InitialComputers = (
  | { type: "providers"; page: Page<ProviderView> }
  | { type: "computers"; providerId: string; page: Page<InventoryView> }
) & { after: string; renderedAt: number; identity: string };
export interface ComputersInput {
  initial: InitialComputers;
  basePath: string;
}
export interface ComputersProps extends ComputersInput {
  access: "authorized" | "signedOut" | "unavailable";
  copy: ComputerCopy;
  appearance?: ColorModeMenuProps;
  data: InitialComputers;
  stale: boolean;
  formatTime: (value: number) => string;
  onRefresh: () => void;
  onLogout: () => void;
}
