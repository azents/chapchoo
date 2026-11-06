"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { AgentAuthentication } from "./AgentAuthentication";
import { useAgentAuthentication } from "./useAgentAuthentication";
export const AgentAuthenticationPage = createReactContainer(
  "AgentAuthenticationPage",
  useAgentAuthentication,
  AgentAuthentication,
);
