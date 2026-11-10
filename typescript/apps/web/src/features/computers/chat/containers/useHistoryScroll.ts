"use client";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import type { RefObject } from "react";

export interface HistoryScroll {
  initialFocus: string | null;
  viewportRef: RefObject<HTMLDivElement | null>;
  contentRef: RefObject<HTMLDivElement | null>;
  detached: boolean;
  preserveHistory: () => void;
  followLatest: () => void;
}

/** Keep the independent history viewport anchored while older records arrive. */
export function useHistoryScroll(initialFocus: string | null): HistoryScroll {
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const following = useRef(initialFocus === null);
  const focus = useRef(initialFocus);
  const anchor = useRef<{ element: HTMLElement; top: number } | null>(null);
  const frame = useRef(0);
  const [detached, setDetached] = useState(initialFocus !== null);
  const pin = useCallback((): void => {
    cancelAnimationFrame(frame.current);
    frame.current = requestAnimationFrame(() => {
      const viewport = viewportRef.current;
      if (following.current && viewport) {
        viewport.scrollTop = viewport.scrollHeight;
      }
    });
  }, []);
  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    const content = contentRef.current;
    if (!viewport || !content) {
      return;
    }
    const measure = (): void => {
      if (focus.current !== null) {
        const element = document.getElementById(
          `history-${encodeURIComponent(focus.current)}`,
        );
        if (element && content.contains(element)) {
          focus.current = null;
          element.scrollIntoView({ block: "center" });
          element.focus({ preventScroll: true });
          return;
        }
      }
      const retained = anchor.current;
      if (retained?.element.isConnected) {
        viewport.scrollTop +=
          retained.element.getBoundingClientRect().top -
          viewport.getBoundingClientRect().top -
          retained.top;
      }
      if (following.current) {
        pin();
      }
    };
    const onScroll = (): void => {
      following.current =
        viewport.scrollHeight - viewport.clientHeight - viewport.scrollTop <=
        48;
      setDetached(!following.current);
    };
    const onIntent = (): void => {
      anchor.current = null;
      focus.current = null;
      following.current = false;
      cancelAnimationFrame(frame.current);
    };
    const observer = new ResizeObserver(measure);
    observer.observe(viewport);
    observer.observe(content);
    viewport.addEventListener("scroll", onScroll, { passive: true });
    viewport.addEventListener("wheel", onIntent, { passive: true });
    viewport.addEventListener("touchstart", onIntent, { passive: true });
    viewport.addEventListener("keydown", onIntent);
    viewport.addEventListener("pointerdown", onIntent, { passive: true });
    measure();
    return (): void => {
      observer.disconnect();
      cancelAnimationFrame(frame.current);
      viewport.removeEventListener("scroll", onScroll);
      viewport.removeEventListener("wheel", onIntent);
      viewport.removeEventListener("touchstart", onIntent);
      viewport.removeEventListener("keydown", onIntent);
      viewport.removeEventListener("pointerdown", onIntent);
    };
  }, [pin]);
  return {
    initialFocus,
    viewportRef,
    contentRef,
    detached,
    followLatest: () => {
      anchor.current = null;
      focus.current = null;
      following.current = true;
      setDetached(false);
      pin();
    },
    preserveHistory: () => {
      const viewport = viewportRef.current;
      if (!viewport) {
        return;
      }
      following.current = false;
      const top = viewport.getBoundingClientRect().top;
      const element = [
        ...viewport.querySelectorAll<HTMLElement>("[data-kind]"),
      ].find((record) => record.getBoundingClientRect().bottom > top);
      if (element) {
        anchor.current = {
          element,
          top: element.getBoundingClientRect().top - top,
        };
      }
    },
  };
}
