"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { Supply } from "./components/Supply";
import { useSupplyContainer } from "./containers/useSupplyContainer";
export const SupplyPage = createReactContainer(
  "SupplyPage",
  useSupplyContainer,
  Supply,
);
