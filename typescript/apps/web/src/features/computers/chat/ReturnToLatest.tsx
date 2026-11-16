"use client";
import { createReactContainer } from "@chapchoo/ui/createReactContainer";
import { ReturnToLatestView } from "./components/ReturnToLatestView";
import { useReturnToLatest } from "./containers/useReturnToLatest";

export const ReturnToLatest = createReactContainer(
  "ReturnToLatest",
  useReturnToLatest,
  ReturnToLatestView,
);
