"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Connections } from "./Connections";
import { useConnections } from "./useConnections";
export const ConnectionsPage = createReactContainer(
  "ConnectionsPage",
  useConnections,
  Connections,
);
