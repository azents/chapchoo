import type { McpDraft } from "./mcpDraft";
import type { McpServersCopy } from "./mcpServersCopy";
import type { McpServerView, McpTestResult } from "@chapchoo/web-core/computer";

export type McpTestState =
  { type: "idle" | "running" | "unavailable" } | McpTestResult;
export type McpEditor = {
  saved: McpServerView | null;
  draft: McpDraft;
  confirming: boolean;
  rejected: boolean;
  test: McpTestState;
};
export type McpList =
  | { type: "hidden" | "loading" | "unavailable" }
  | {
      type: "loaded";
      items: McpServerView[];
      more: boolean;
      previous: boolean;
    };
export type McpWriteState = "idle" | "saving" | "checking" | "uncertain";

export interface McpServersProps {
  copy: McpServersCopy;
  list: McpList;
  mobile: boolean;
  editor: McpEditor | null;
  write: McpWriteState;
  canTest: boolean;
  canSave: boolean;
  keepsToken: boolean;
  onRetry: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onOpen: (server: McpServerView | null) => void;
  onClose: () => void;
  onDraft: (change: Partial<McpDraft>) => void;
  onTest: () => void;
  onSave: () => void;
  onConfirmDelete: (value: boolean) => void;
  onDelete: () => void;
  onCheck: () => void;
}
