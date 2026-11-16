import type { SystemPromptCopy } from "./systemPromptCopy";

export type SystemPromptEditor =
  | { type: "closed" }
  | { type: "loading" }
  | { type: "readError" }
  | { type: "editing"; content: string; error: string | null }
  | { type: "saving"; content: string };

export interface SystemPromptProps {
  copy: SystemPromptCopy;
  mobile: boolean;
  unavailable: boolean;
  editor: SystemPromptEditor;
  onOpen: () => void;
  onClose: () => void;
  onChange: (value: string) => void;
  onSave: () => void;
  onRead: () => void;
}
export interface SystemPromptInput {
  agentId: string;
  unavailable: boolean;
}
