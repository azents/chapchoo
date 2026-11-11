"use client";
import { useOlderRecords } from "./useOlderRecords";
import type { ConversationControls } from "../types";
import type { RefObject } from "react";
export function OlderRecordsContainer({
  controls,
  viewportRef,
  contentRef,
  followingLatest,
}: {
  controls: ConversationControls;
  viewportRef: RefObject<HTMLDivElement | null>;
  contentRef: RefObject<HTMLDivElement | null>;
  followingLatest: boolean;
}): null {
  useOlderRecords(controls, viewportRef, contentRef, followingLatest);
  return null;
}
