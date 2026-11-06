import type {
  CatalogueSnapshot,
  ConnectionPanelProps,
} from "@/shared/authentications/types";
import type { AuthenticationView } from "@chapchoo/web-core/external-authentication";
export interface ConnectionsInput {
  basePath: string;
  teamHandle: string;
  identity: string;
  snapshot: CatalogueSnapshot;
  managed: AuthenticationView | null;
  after: string | null;
}
export interface ConnectionsProps {
  input: ConnectionsInput;
  panel: ConnectionPanelProps;
  access: boolean;
  onRetry: () => void;
}
