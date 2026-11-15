"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { SystemPrompt } from "./components/SystemPrompt";
import { useSystemPrompt } from "./containers/useSystemPrompt";

export const SystemPromptPage = createReactContainer(
  "SystemPromptPage",
  useSystemPrompt,
  SystemPrompt,
);
