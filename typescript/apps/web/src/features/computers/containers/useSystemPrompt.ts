"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useMediaQuery } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { systemPromptCopy } from "../systemPromptCopy";
import type {
  SystemPromptEditor,
  SystemPromptInput,
  SystemPromptProps,
} from "../systemPromptTypes";

export function useSystemPrompt(input: SystemPromptInput): SystemPromptProps {
  const auth = useAuth();
  const copy = systemPromptCopy(useLocale());
  const mobile = useMediaQuery("(max-width: 48em)");
  const utils = trpc.useUtils();
  const [editor, setEditor] = useState<SystemPromptEditor>({ type: "closed" });
  const fence = useRef(0);
  const saving = useRef(false);
  const mutation = trpc.computers.saveSystemPrompt.useMutation({
    retry: false,
  });
  const authorized = auth.state.type === "AUTHENTICATED";
  useEffect(
    () => () => {
      fence.current += 1;
    },
    [input.agentId, authorized],
  );

  async function read(): Promise<void> {
    if (saving.current || input.unavailable || !authorized) {
      return;
    }
    const operation = ++fence.current;
    setEditor({ type: "loading" });
    try {
      const saved = await utils.computers.systemPrompt.fetch({
        agentId: input.agentId,
      });
      if (operation === fence.current) {
        setEditor({ type: "editing", content: saved.content, error: null });
      }
    } catch {
      if (operation === fence.current) {
        setEditor({ type: "readError" });
      }
    }
  }
  async function save(): Promise<void> {
    if (
      saving.current ||
      editor.type !== "editing" ||
      input.unavailable ||
      !authorized
    ) {
      return;
    }
    const content = editor.content;
    if (new TextEncoder().encode(content).length > 16 * 1024) {
      setEditor({ type: "editing", content, error: copy.tooLong });
      return;
    }
    saving.current = true;
    const operation = fence.current;
    setEditor({ type: "saving", content });
    try {
      await mutation.mutateAsync({ agentId: input.agentId, content });
      if (operation === fence.current) {
        setEditor({ type: "closed" });
      }
      void utils.computers.systemPrompt.invalidate({ agentId: input.agentId });
    } catch {
      if (operation === fence.current) {
        setEditor({ type: "editing", content, error: copy.saveFailed });
      }
    } finally {
      saving.current = false;
    }
  }
  return {
    copy,
    mobile,
    unavailable: input.unavailable || !authorized,
    editor,
    onOpen: () => {
      void read();
    },
    onRead: () => {
      void read();
    },
    onClose: () => {
      if (!saving.current) {
        fence.current += 1;
        setEditor({ type: "closed" });
      }
    },
    onChange: (content) =>
      setEditor((current) =>
        current.type === "editing"
          ? { type: "editing", content, error: null }
          : current,
      ),
    onSave: () => {
      void save();
    },
  };
}
