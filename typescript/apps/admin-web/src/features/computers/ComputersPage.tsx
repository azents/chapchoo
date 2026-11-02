"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Computers } from "./components/Computers";
import { useComputersContainer } from "./containers/useComputersContainer";
export const ComputersPage = createReactContainer(
  "ComputersPage",
  useComputersContainer,
  Computers,
);
