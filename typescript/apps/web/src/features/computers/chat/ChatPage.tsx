"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Chat } from "./components/Chat";
import { useChatContainer } from "./containers/useChatContainer";

export const ChatPage = createReactContainer(
  "ChatPage",
  useChatContainer,
  Chat,
);
