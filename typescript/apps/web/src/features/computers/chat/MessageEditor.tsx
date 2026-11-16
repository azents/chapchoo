"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { MessageEditorView } from "./components/MessageEditorView";
import { useMessageEditor } from "./containers/useMessageEditor";

export const MessageEditor = createReactContainer(
  "MessageEditor",
  useMessageEditor,
  MessageEditorView,
);
