"use client";
import { useDocumentVisibility, useReducedMotion } from "@mantine/hooks";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { RefObject } from "react";

export interface ReturnToLatestInput {
  visible: boolean;
  unread: boolean;
  label: string;
  unreadLabel: string;
  onLatest: () => void;
}
export interface ReturnToLatestProps extends ReturnToLatestInput {
  shownUnread: boolean;
  phase: "idle" | "out" | "in";
  width?: number;
  normalRef: RefObject<HTMLSpanElement | null>;
  unreadRef: RefObject<HTMLSpanElement | null>;
  duration: number;
  onExited: () => void;
  onLabelAnimationEnd: () => void;
}

/** Visual label lifetime is independent of unread acknowledgement at the bottom. */
export function useReturnToLatest(
  input: ReturnToLatestInput,
): ReturnToLatestProps {
  const { visible, unread } = input;
  const [shownUnread, setShownUnread] = useState(unread);
  const [phase, setPhase] = useState<ReturnToLatestProps["phase"]>("idle");
  const [widths, setWidths] = useState({ normal: 0, unread: 0 });
  const normalRef = useRef<HTMLSpanElement>(null);
  const unreadRef = useRef<HTMLSpanElement>(null);
  const reducedMotion = useReducedMotion();
  const visibility = useDocumentVisibility();
  const instant = reducedMotion || visibility === "hidden";
  useLayoutEffect(() => {
    const normal = normalRef.current;
    const incoming = unreadRef.current;
    if (!normal || !incoming) {
      return;
    }
    const measure = (): void =>
      setWidths({
        normal: normal.getBoundingClientRect().width,
        unread: incoming.getBoundingClientRect().width,
      });
    const observer = new ResizeObserver(measure);
    observer.observe(normal);
    observer.observe(incoming);
    measure();
    return () => observer.disconnect();
  }, [input.label, input.unreadLabel]);
  useEffect(() => {
    if (!visible) {
      setPhase("idle");
    } else if (instant) {
      setShownUnread(unread);
      setPhase("idle");
    } else if (unread !== shownUnread) {
      setPhase("out");
    }
  }, [visible, unread, shownUnread, instant]);
  return {
    ...input,
    shownUnread,
    phase,
    width: (shownUnread ? widths.unread : widths.normal) || void 0,
    normalRef,
    unreadRef,
    duration: instant ? 0 : 240,
    onExited: () => {
      if (!visible) {
        setShownUnread(false);
        setPhase("idle");
      }
    },
    onLabelAnimationEnd: () => {
      if (!visible) {
        return;
      }
      if (phase === "out") {
        setShownUnread(unread);
        setPhase("in");
      } else if (phase === "in") {
        setPhase("idle");
      }
    },
  };
}
