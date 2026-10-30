"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Workspace } from "./components/Workspace";
import { useWorkspaceContainer } from "./containers/useWorkspaceContainer";

export const WorkspacePage = createReactContainer(
  "WorkspacePage",
  useWorkspaceContainer,
  Workspace,
);
