"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { AdminGrants } from "./components/AdminGrants";
import { useAdminGrantsContainer } from "./containers/useAdminGrantsContainer";

export const AdminGrantsPage = createReactContainer(
  "AdminGrantsPage",
  useAdminGrantsContainer,
  AdminGrants,
);
