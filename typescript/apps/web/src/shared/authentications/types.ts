import type { AuthenticationCopy } from "@chapchoo/web-core/authentication-messages";
import type {
  AuthenticationScope,
  AuthenticationView,
  DeviceProgress,
} from "@chapchoo/web-core/external-authentication";
export interface CatalogueSnapshot {
  ownerUserId: string;
  scope: AuthenticationScope;
  canManage: boolean;
  page: { items: AuthenticationView[]; nextAfter: string | null } | null;
  renderedAt: number;
}
export type ConnectionDialog =
  | { type: "closed" }
  | { type: "recordUnavailable"; authenticationId: string }
  | { type: "connect" }
  | { type: "key"; target: AuthenticationView | null }
  | { type: "deviceName"; target: AuthenticationView | null }
  | { type: "device"; progress: DeviceProgress }
  | { type: "manage"; target: AuthenticationView }
  | { type: "rename"; target: AuthenticationView }
  | { type: "delete"; target: AuthenticationView }
  | {
      type: "uncertain";
      requestId: string;
      operation: "record" | "deleted" | "device";
      attemptId: string | null;
    };
export interface ConnectionPanelInput {
  enabled?: boolean;
  snapshots: CatalogueSnapshot[];
  selectedId?: string | null;
  onSelect?: (value: AuthenticationView) => void;
  managed?: AuthenticationView | null;
  onManage?: (value: AuthenticationView) => void;
  onCloseManage?: () => void;
  after?: string | null;
  onPage?: (value: string | null) => void;
}
export interface ConnectionPanelProps {
  mobile: boolean;
  basePath?: string;
  showConnectAction?: boolean;
  copy: AuthenticationCopy;
  scope: AuthenticationScope;
  scopes: AuthenticationScope[];
  onScope: (value: string) => void;
  selectedId: string | null;
  selectable: boolean;
  canManage: boolean;
  items: AuthenticationView[];
  nextAfter: string | null;
  after: string | null;
  unavailable: boolean;
  loading: boolean;
  dialog: ConnectionDialog;
  name: string;
  apiKey: string;
  busy: boolean;
  error: string | null;
  remainingSeconds: number | null;
  onName: (value: string) => void;
  onKey: (value: string) => void;
  onPage: (value: string | null) => void;
  onReload: () => void;
  onSelect: (value: AuthenticationView) => void;
  onManage: (value: AuthenticationView) => void;
  onDialog: (value: ConnectionDialog) => void;
  onClose: () => void;
  onSubmit: () => void;
  onDeviceCheck: () => void;
  onDeviceStatus: () => void;
  onDeviceCancel: () => void;
  onDeviceRestart: () => void;
  onRecover: () => void;
}
export function scopeKey(value: AuthenticationScope): string {
  return value.type === "personal" ? "personal" : value.teamId;
}
export function catalogueOwnerMatches(
  snapshot: CatalogueSnapshot,
  userId: string | null,
): boolean {
  return userId !== null && snapshot.ownerUserId === userId;
}
