import {
  useDocumentVisibility,
  useIntersection,
  useReducedMotion,
} from "@mantine/hooks";
import type { RefCallback } from "react";

/** Restart locally when visible; never catch up missed background cycles. */
export function useMascotPlayback(playing: boolean): {
  ref: RefCallback<HTMLSpanElement>;
  running: boolean;
} {
  const { ref, entry } = useIntersection<HTMLSpanElement>({ threshold: 0.1 });
  const visibility = useDocumentVisibility();
  const reduced = useReducedMotion();
  return {
    ref,
    running:
      playing &&
      visibility === "visible" &&
      !reduced &&
      entry?.isIntersecting === true,
  };
}
