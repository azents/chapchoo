"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { NavigationChrome } from "./NavigationChrome";
import { useNavigation } from "./useNavigation";
export const NavigationProvider = createReactContainer(
  "NavigationProvider",
  useNavigation,
  NavigationChrome,
);
