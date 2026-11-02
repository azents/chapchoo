import type { ResourceSelection } from "@/shared/server/resourceInput";
import type { ColorModeMenuProps } from "@chapchoo/ui/ColorModeMenu";
import type { ResourcePage } from "@chapchoo/web-core/computer";
import type { ComputerCopy } from "@chapchoo/web-core/computer-messages";
export interface InitialResources {
  selection: ResourceSelection;
  page: ResourcePage;
  renderedAt: number;
  identity: string;
}
export interface ResourcesInput {
  initial: InitialResources;
  basePath: string;
}
export interface ResourcesProps extends ResourcesInput {
  access: "authorized" | "signedOut" | "unavailable";
  data: ResourcePage;
  copy: ComputerCopy;
  appearance?: ColorModeMenuProps;
  stale: boolean;
  onRefresh: () => void;
  onLogout: () => void;
  formatTime: (value: number) => string;
}
