import type messages from "../../../messages/en-US.json";
import type { ColorModeSelectorProps } from "@chapchoo/ui/ColorModeSelector";

export type Copy = Record<keyof typeof messages.accounts, string>;
export type LoadState<T> =
  | { type: "LOADING" }
  | { type: "ERROR"; message: string }
  | { type: "LOADED"; data: T };
export type Submission =
  | { type: "READY" }
  | { type: "PENDING" }
  | { type: "ERROR"; message: string }
  | { type: "DONE"; message: string };
export interface AuthInput {
  basePath: string;
  mode: "LOGIN" | "SIGNUP" | "RESET";
  token: string;
}
export interface AuthProps extends AuthInput {
  appearance?: ColorModeSelectorProps;
  copy: Copy;
  state: LoadState<{
    password: boolean;
    emailCode: boolean;
    signupEmail: boolean;
    email: string;
  }>;
  submission: Submission;
  email: string;
  password: string;
  code: string;
  challenge: boolean;
  onEmail: (value: string) => void;
  onPassword: (value: string) => void;
  onCode: (value: string) => void;
  onSubmit: () => void;
  onSendCode: () => void;
  onUsePassword: () => void;
  onRetry: () => void;
}
export interface ContextView {
  id: string;
  name: string;
  personal: boolean;
  role: string;
}
export interface InvitationView {
  id: string;
  teamId: string;
  email: string;
  label: string;
}
export interface MemberView {
  id: string;
  email: string;
  role: "OWNER" | "MANAGER" | "MEMBER";
}
export interface WorkspaceData {
  identity: string;
  contexts: ContextView[];
  received: InvitationView[];
  team: null | {
    id: string;
    name: string;
    handle: string;
    personal: boolean;
    role: "OWNER" | "MANAGER" | "MEMBER";
    members: MemberView[];
    invitations: InvitationView[];
  };
}
export type TeamAction =
  | { type: "REMOVE"; userId: string; label: string }
  | { type: "ROLE"; userId: string; role: "MANAGER" | "MEMBER"; label: string }
  | { type: "TRANSFER"; userId: string; label: string }
  | { type: "LEAVE"; label: string }
  | { type: "DELETE"; label: string };
export interface WorkspaceProps {
  appearance?: ColorModeSelectorProps;
  basePath: string;
  copy: Copy;
  state: LoadState<WorkspaceData>;
  submission: Submission;
  name: string;
  handle: string;
  inviteEmail: string;
  confirmation: TeamAction | null;
  onName: (value: string) => void;
  onHandle: (value: string) => void;
  onInviteEmail: (value: string) => void;
  onCreate: () => void;
  onRename: () => void;
  onInvite: () => void;
  onInvitation: (id: string, accept: boolean) => void;
  onAction: (action: TeamAction) => void;
  onConfirm: () => void;
  onCancel: () => void;
  onRetry: () => void;
  onLogout: () => void;
}
