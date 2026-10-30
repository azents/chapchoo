"use client";

import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Auth } from "./components/Auth";
import { useAuthContainer } from "./containers/useAuthContainer";

export const AuthPage = createReactContainer(
  "AuthPage",
  useAuthContainer,
  Auth,
);
