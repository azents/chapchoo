"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { WorkspaceChrome } from "@chapchoo/ui/WorkspaceChrome";
import { useNavigation } from "./useNavigation";
export const NavigationProvider = createReactContainer(
  "NavigationProvider",
  useNavigation,
  WorkspaceChrome,
);
