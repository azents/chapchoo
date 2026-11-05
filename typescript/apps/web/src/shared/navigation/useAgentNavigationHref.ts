"use client";
import { readSelection } from "@chapchoo/web-core/page-state";
import { useSearchParams } from "next/navigation";
import {
  agentCursor,
  agentCursorKey,
  agentNavigationHref,
} from "./agentSelection";

/** Feature-owned links continue the same URL-owned sidebar selection. */
export function useAgentNavigationHref(): (href: string) => string {
  const after = readSelection(useSearchParams(), agentCursorKey, agentCursor);
  return (href) => agentNavigationHref(href, after);
}
