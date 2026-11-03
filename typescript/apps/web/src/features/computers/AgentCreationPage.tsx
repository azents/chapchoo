"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { AgentCreation } from "./components/AgentCreation";
import { useAgentCreation } from "./containers/useAgentCreation";
export const AgentCreationPage = createReactContainer(
  "AgentCreationPage",
  useAgentCreation,
  AgentCreation,
);
