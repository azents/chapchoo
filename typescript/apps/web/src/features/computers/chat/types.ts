import type { AgentsInput, AgentsProps } from "../types";
import type { ChatFocus } from "./containers/useChatFocus";
import type { FloatingChat } from "./containers/useFloatingChat";
import type { ChatCopy } from "./copy";
import type { AppNavigation } from "@chapchoo/ui/AppNavigation";
import type { AgentView } from "@chapchoo/web-core/computer";
import type {
  DraftValue,
  QuestionDraft,
} from "@chapchoo/web-core/conversation-form";
import type {
  ConversationAnswer,
  ConversationRecord,
  ConversationRegion,
  InputObservation,
} from "@chapchoo/web-core/conversation-types";
import type { ReactNode } from "react";

export type ChatInput = AgentsInput & {
  initialConversation: ConversationRegion;
};
export interface ConversationControls {
  region: ConversationRegion;
  read: "ready" | "loading" | "error";
  readingOlder: boolean;
  inputs: {
    text: string;
    observation: InputObservation | "sending";
    requestId: string;
    snapshotId?: string;
    conversationId?: string;
  }[];
  stop: "idle" | "pending" | "published" | "rejected" | "uncertain";
  drafts: Partial<Record<string, QuestionDraft>>;
  onRead: () => void;
  onOlder: () => void;
  onLatest: () => void;
  onHistory: (record: ConversationRecord) => void;
  onQuestionOption: (record: ConversationRecord, optionId: string) => void;
  onQuestionField: (
    record: ConversationRecord,
    name: string,
    value: DraftValue | null,
  ) => void;
  onAnswer: (record: ConversationRecord, answer?: ConversationAnswer) => void;
}
export interface ChatProps {
  product: AgentsProps;
  agent: AgentView;
  copy: ChatCopy;
  navigation: AppNavigation | null;
  accountControls: ReactNode;
  supplyControl: ReactNode;
  secondaryOverlay: boolean;
  agentNavigation: ReactNode;
  conversation: ConversationControls;
  input: string;
  activity: "working" | "requiresAction" | "idle" | "unavailable";
  writable: boolean;
  sendable: boolean;
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
