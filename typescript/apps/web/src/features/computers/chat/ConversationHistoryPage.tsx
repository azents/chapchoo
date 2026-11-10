"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { ConversationHistory } from "./components/ConversationHistory";
import { useConversationHistory } from "./containers/useConversationHistory";

export const ConversationHistoryPage = createReactContainer(
  "ConversationHistoryPage",
  useConversationHistory,
  ConversationHistory,
);
