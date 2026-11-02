"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Resources } from "./components/Resources";
import { useResourcesContainer } from "./containers/useResourcesContainer";
export const ResourcesPage = createReactContainer(
  "ResourcesPage",
  useResourcesContainer,
  Resources,
);
