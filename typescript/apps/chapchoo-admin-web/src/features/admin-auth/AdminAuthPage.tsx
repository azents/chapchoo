"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { AdminAuth } from "./components/AdminAuth";
import { useAdminAuthContainer } from "./containers/useAdminAuthContainer";

export const AdminAuthPage = createReactContainer(
  "AdminAuthPage",
  useAdminAuthContainer,
  AdminAuth,
);
