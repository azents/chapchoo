"use client";
import { useEffect, useRef } from "react";
import type { ConversationControls, MessageControls } from "../types";
import type { RefObject } from "react";
/** One request per source-bound cursor; prepend anchoring remains with the caller. */
export function useOlderRecords(
  controls: ConversationControls | MessageControls,
  viewportRef: RefObject<HTMLDivElement | null>,
  contentRef: RefObject<HTMLDivElement | null>,
  followingLatest: boolean,
): void {
  const current = useRef(controls);
  current.current = controls;
  const requested = useRef<string | null>(null);
  useEffect(() => {
    requested.current = null;
  }, [controls.paginationReset]);
  const region = controls.region;
  const cursor = JSON.stringify(
    "header" in region
      ? [
          region.header.conversationId,
          region.header.source?.snapshotId,
          region.older?.ordinal.toString(),
          region.older?.index.toString(),
          region.inputOlder?.acceptedAt.toString(),
          region.inputOlder?.requestId,
        ]
      : [region.conversationId, region.older?.toString()],
  );
  useEffect(() => {
    const viewport = viewportRef.current;
    const content = contentRef.current;
    if (!viewport || !content) {
      return;
    }
    let frame = 0;
    function check(): void {
      const c = current.current;
      if (
        c.read !== "ready" ||
        (c.region.older === void 0 &&
          !("inputOlder" in c.region && c.region.inputOlder)) ||
        requested.current === cursor
      ) {
        return;
      }
      if (
        viewport &&
        ((!followingLatest && viewport.scrollTop <= 64) ||
          (content && content.offsetHeight <= viewport.clientHeight + 1) ||
          viewport.scrollHeight <= viewport.clientHeight + 1)
      ) {
        requested.current = cursor;
        c.onOlder();
      }
    }
    function schedule(): void {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(check);
    }
    const observer = new ResizeObserver(schedule);
    observer.observe(viewport);
    observer.observe(content);
    viewport.addEventListener("scroll", schedule, { passive: true });
    schedule();
    return (): void => {
      observer.disconnect();
      cancelAnimationFrame(frame);
      viewport.removeEventListener("scroll", schedule);
    };
  }, [
    cursor,
    controls.read,
    controls.paginationReset,
    viewportRef,
    contentRef,
    followingLatest,
  ]);
}
