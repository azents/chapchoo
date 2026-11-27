"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { McpServers } from "./components/McpServers";
import { useMcpServers } from "./containers/useMcpServers";

export const McpServersPage = createReactContainer(
  "McpServersPage",
  useMcpServers,
  McpServers,
);
