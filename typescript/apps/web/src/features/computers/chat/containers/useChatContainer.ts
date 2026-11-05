"use client";
import { HeaderControlsContext } from "@chapchoo/ui/AppHeader";
import { AppNavigationContext } from "@chapchoo/ui/AppNavigation";
import { computerOperation } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { useLocale } from "next-intl";
import { useContext, useRef, useState } from "react";
import { useAgentsContainer } from "../../containers/useAgentsContainer";
import { chatCopy } from "../copy";
import { useChatFocus } from "./useChatFocus";
import { useFloatingChat } from "./useFloatingChat";
import type { ChatInput, ChatProps } from "../types";

export function useChatContainer(props: ChatInput): ChatProps {
  const baseProduct = useAgentsContainer(props);
  const navigation = useContext(AppNavigationContext);
  const accountControls = useContext(HeaderControlsContext);
  const display = useDisplay();
  const secondaryOverlay = display.opened || Boolean(navigation?.overlayActive);
  const controlled = useChatFocus(baseProduct, secondaryOverlay);
  const product = controlled.product;
  const copy = chatCopy(useLocale());
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<{ id: number; text: string }[]>([]);
  const messageId = useRef(0);
  const [activity, setActivity] = useState<ChatProps["activity"]>("working");
  const [opened, setOpened] = useState(false);
  const [navOpened, setNavOpened] = useState(false);
  const floating = useFloatingChat(
    input,
    props.initial.type === "detail" ? props.initial.agent.id : "",
    product.access === "authorized",
  );
  if (product.data.type !== "detail") {
    throw new Error("Chat requires an admitted Agent detail snapshot");
  }
  const agent = product.data.agent;
  const computer = agent.computer;
  const writable =
    product.access === "authorized" &&
    !product.stale &&
    !product.pending &&
    computer.power === "running" &&
    computer.connector === "connected" &&
    computer.phase === "completed";
  const operation = computerOperation(computer, product.copy);
  const status =
    product.error ??
    (product.stale
      ? product.copy.stale
      : computer.phase !== "completed"
        ? operation.label
        : computer.power === "stopped"
          ? `${product.detailCopy.computer} · ${product.copy.stopped}`
          : computer.power === "unknown"
            ? `${product.detailCopy.computer} · ${product.copy.unknown}`
            : computer.connector !== "connected"
              ? product.copy[computer.connector]
              : activity === "working"
                ? copy.working
                : activity === "stopped"
                  ? copy.stopped
                  : copy.idle);
  return {
    product,
    agent,
    copy,
    navigation,
    accountControls,
    secondaryOverlay,
    agentNavigation: navigation?.content,
    showSample: true,
    input,
    messages,
    activity,
    writable,
    status,
    opened,
    navOpened,
    floating,
    focus: controlled.focus,
    onOpen: () => setOpened(true),
    onClose: () => setOpened(false),
    onOpenNav: () => setNavOpened(true),
    onCloseNav: () => setNavOpened(false),
    onInput: setInput,
    onSend: () => {
      if (!writable || !input.trim()) {
        return;
      }
      const text = input.trim();
      const id = ++messageId.current;
      floating.followLatest();
      setMessages((previous) => [...previous, { id, text }]);
      setInput("");
      setActivity("idle");
    },
    onStop: () => setActivity("stopped"),
  };
}
