"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Agents } from "./components/Agents";
import { useAgentsContainer } from "./containers/useAgentsContainer";
export const AgentsPage = createReactContainer(
  "AgentsPage",
  useAgentsContainer,
  Agents,
);
