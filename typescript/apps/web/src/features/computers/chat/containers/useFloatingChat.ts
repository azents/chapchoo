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
  followLatest: () => void;
}
/** Measured overlay clearance and explicit latest-follow intent, independent of status size. */
export function useFloatingChat(
  input: string,
  conversationKey: string,
  active = true,
): FloatingChat {
  const shellRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const overlayRef = useRef<HTMLElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const profileRef = useRef<HTMLDivElement>(null);
  const following = useRef(true);
  const frame = useRef(0);
  const intentFrame = useRef(0);
  const [detached, setDetached] = useState(false);
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
    following.current = true;
    setDetached(false);
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
    };
    const onIntent = (): void => {
      cancelAnimationFrame(frame.current);
      following.current = false;
      cancelAnimationFrame(intentFrame.current);
      intentFrame.current = requestAnimationFrame(onScroll);
    };
    viewport.addEventListener("scroll", onScroll, { passive: true });
    viewport.addEventListener("wheel", onIntent, { passive: true });
    viewport.addEventListener("touchstart", onIntent, { passive: true });
    return (): void => {
      observer.disconnect();
      cancelAnimationFrame(frame.current);
      cancelAnimationFrame(intentFrame.current);
      viewport.removeEventListener("scroll", onScroll);
      viewport.removeEventListener("wheel", onIntent);
      viewport.removeEventListener("touchstart", onIntent);
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
    followLatest,
  };
}
