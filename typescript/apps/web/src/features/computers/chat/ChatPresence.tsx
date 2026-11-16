"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import {
  ChatPresenceItemView,
  ChatPresenceView,
} from "./components/ChatPresenceView";
import {
  useChatPresence,
  useChatPresenceItem,
} from "./containers/useChatPresence";

export const ChatPresenceItem = createReactContainer(
  "ChatPresenceItem",
  useChatPresenceItem,
  ChatPresenceItemView,
);
export const ChatPresence = createReactContainer(
  "ChatPresence",
  useChatPresence,
  ChatPresenceView,
);
