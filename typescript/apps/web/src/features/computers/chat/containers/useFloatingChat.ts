"use client";
import { useCallback, useLayoutEffect, useRef, useState } from "react";
import type { RefObject } from "react";

export interface FloatingChat {
  shellRef: RefObject<HTMLDivElement | null>;
  viewportRef: RefObject<HTMLDivElement | null>;
  contentRef: RefObject<HTMLDivElement | null>;
  overlayRef: RefObject<HTMLElement | null>;
  textareaRef: RefObject<HTMLTextAreaElement | null>;
  profileRef: RefObject<HTMLDivElement | null>;
  detached: boolean;
  unseen: boolean;
  followLatest: () => void;
  preserveHistory: () => void;
}
/** Measured overlay clearance and explicit latest-follow intent, independent of status size. */
export function useFloatingChat(
  input: string,
  conversationKey: string,
  active = true,
  updates: readonly { key: string; updatedOrdinal: bigint }[] = [],
): FloatingChat {
  const shellRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const overlayRef = useRef<HTMLElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const profileRef = useRef<HTMLDivElement>(null);
  const following = useRef(true);
  const historyAnchor = useRef<{ element: HTMLElement; top: number } | null>(
    null,
  );
  const frame = useRef(0);
  const intentFrame = useRef(0);
  const [detached, setDetached] = useState(false);
  const [unseen, setUnseen] = useState(false);
  const previousUpdate = useRef(
    new Map(updates.map((record) => [record.key, record.updatedOrdinal])),
  );
  useLayoutEffect(() => {
    const unseenUpdate = updates.some((record) => {
      const previous = previousUpdate.current.get(record.key);
      return previous === void 0 || record.updatedOrdinal > previous;
    });
    previousUpdate.current = new Map(
      updates.map((record) => [record.key, record.updatedOrdinal]),
    );
    if (unseenUpdate && !following.current) {
      setUnseen(true);
    }
  }, [updates]);
  const schedulePin = useCallback((): void => {
    cancelAnimationFrame(frame.current);
    frame.current = requestAnimationFrame(() => {
      const viewport = viewportRef.current;
      if (following.current && viewport) {
        viewport.scrollTop = viewport.scrollHeight;
      }
    });
  }, []);
  const followLatest = useCallback((): void => {
    historyAnchor.current = null;
    following.current = true;
    setDetached(false);
    setUnseen(false);
    schedulePin();
  }, [schedulePin]);
  useLayoutEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) {
      return;
    }
    textarea.style.height = "auto";
    const style = getComputedStyle(textarea);
    const line = Number.parseFloat(style.lineHeight);
    const padding =
      Number.parseFloat(style.paddingTop) +
      Number.parseFloat(style.paddingBottom);
    const height = textarea.scrollHeight - padding;
    // DOM measurements are already rendered CSS pixels, including enlarged text.
    textarea.style.height = `${Math.min(line * 5, Math.max(line, height))}px`;
    textarea.style.overflowY = height > line * 5 ? "auto" : "hidden";
  }, [input, active]);
  useLayoutEffect(() => {
    const shell = shellRef.current;
    const overlay = overlayRef.current;
    const viewport = viewportRef.current;
    const content = contentRef.current;
    const profile = profileRef.current;
    if (!shell || !overlay || !viewport || !content || !profile) {
      return;
    }
    const measure = (): void => {
      const anchor = historyAnchor.current;
      if (anchor?.element.isConnected) {
        const top =
          anchor.element.getBoundingClientRect().top -
          viewport.getBoundingClientRect().top;
        viewport.scrollTop += top - anchor.top;
      }
      shell.style.setProperty(
        "--composer-space",
        `${overlay.getBoundingClientRect().height}px`,
      );
      shell.style.setProperty(
        "--profile-space",
        `${Math.max(
          0,
          profile.getBoundingClientRect().bottom -
            shell.getBoundingClientRect().top,
        )}px`,
      );
      if (following.current) {
        schedulePin();
      }
    };
    const observer = new ResizeObserver(measure);
    [overlay, viewport, content, profile].forEach((node) =>
      observer.observe(node),
    );
    measure();
    const onScroll = (): void => {
      following.current =
        viewport.scrollHeight - viewport.clientHeight - viewport.scrollTop <=
        48;
      setDetached(!following.current);
      if (following.current) {
        setUnseen(false);
      }
    };
    const onIntent = (): void => {
      historyAnchor.current = null;
      cancelAnimationFrame(frame.current);
      following.current = false;
      cancelAnimationFrame(intentFrame.current);
      intentFrame.current = requestAnimationFrame(onScroll);
    };
    viewport.addEventListener("scroll", onScroll, { passive: true });
    viewport.addEventListener("wheel", onIntent, { passive: true });
    viewport.addEventListener("touchstart", onIntent, { passive: true });
    viewport.addEventListener("keydown", onIntent);
    return (): void => {
      observer.disconnect();
      cancelAnimationFrame(frame.current);
      cancelAnimationFrame(intentFrame.current);
      viewport.removeEventListener("scroll", onScroll);
      viewport.removeEventListener("wheel", onIntent);
      viewport.removeEventListener("touchstart", onIntent);
      viewport.removeEventListener("keydown", onIntent);
    };
  }, [schedulePin, active]);
  useLayoutEffect(followLatest, [conversationKey, followLatest]);
  return {
    shellRef,
    viewportRef,
    contentRef,
    overlayRef,
    textareaRef,
    profileRef,
    detached,
    unseen,
    followLatest,
    preserveHistory: () => {
      const viewport = viewportRef.current;
      if (viewport) {
        following.current = false;
        const top = viewport.getBoundingClientRect().top;
        const element = [
          ...viewport.querySelectorAll<HTMLElement>("[data-kind]"),
        ].find((record) => record.getBoundingClientRect().bottom > top);
        if (element) {
          historyAnchor.current = {
            element,
            top: element.getBoundingClientRect().top - top,
          };
        }
      }
    },
  };
}
