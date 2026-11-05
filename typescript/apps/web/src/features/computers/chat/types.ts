import type { AgentsInput, AgentsProps } from "../types";
import type { ChatFocus } from "./containers/useChatFocus";
import type { FloatingChat } from "./containers/useFloatingChat";
import type { ChatCopy } from "./copy";
import type { AppNavigation } from "@chapchoo/ui/AppNavigation";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ReactNode } from "react";

export type ChatInput = AgentsInput;
export interface ChatProps {
  product: AgentsProps;
  agent: AgentView;
  copy: ChatCopy;
  navigation: AppNavigation | null;
  accountControls: ReactNode;
  secondaryOverlay: boolean;
  choices: readonly AgentView[];
  choicesUnavailable: boolean;
  onRetryChoices: () => void;
  showSample: boolean;
  input: string;
  messages: readonly { id: number; text: string }[];
  activity: "working" | "stopped" | "idle";
  writable: boolean;
  status: string;
  opened: boolean;
  navOpened: boolean;
  onOpen: () => void;
  onClose: () => void;
  onOpenNav: () => void;
  onCloseNav: () => void;
  onInput: (value: string) => void;
  onSend: () => void;
  onStop: () => void;
  floating: FloatingChat;
  focus: ChatFocus;
}
