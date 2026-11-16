"use client";
import { useDocumentVisibility, useReducedMotion } from "@mantine/hooks";
import {
  Children,
  isValidElement,
  useCallback,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { reconcilePresence } from "../chatPresence";
import type { PresenceEntry } from "../chatPresence";
import type { CSSProperties, ReactElement, ReactNode, RefObject } from "react";

export interface ChatPresenceInput {
  children: ReactNode;
  spaced?: boolean;
}
export interface ChatPresenceView {
  entries: PresenceEntry[];
  spaced: boolean;
  onExited: (key: string) => void;
}
export function useChatPresence({
  children,
  spaced = false,
}: ChatPresenceInput): ChatPresenceView {
  const next: { key: string; node: ReactElement; order?: bigint }[] = [];
  Children.forEach(children, (node) => {
    if (isValidElement<{ "data-presence-order"?: bigint }>(node)) {
      next.push({
        key: String(node.key),
        node,
        order: node.props["data-presence-order"],
      });
    }
  });
  const [snapshot, setSnapshot] = useState(() => ({
    children,
    entries: next.map((entry) => ({ ...entry, present: true, initial: true })),
  }));
  let entries = snapshot.entries;
  if (children !== snapshot.children) {
    entries = reconcilePresence(snapshot.entries, next);
    setSnapshot({ children, entries });
  }
  const current = new Map(next.map((entry) => [entry.key, entry.node]));
  return {
    entries: entries.map((entry) => ({
      ...entry,
      node: current.get(entry.key) ?? entry.node,
    })),
    spaced,
    onExited: (key) => {
      setSnapshot((state) => ({
        ...state,
        entries: state.entries.filter(
          (entry) => entry.key !== key || entry.present,
        ),
      }));
    },
  };
}

export interface ChatPresenceItemInput {
  entry: PresenceEntry;
  gap: boolean;
  onExited: (key: string) => void;
}
export interface ChatPresenceItemView {
  outerRef: RefObject<HTMLDivElement | null>;
  innerRef: RefObject<HTMLDivElement | null>;
  style: CSSProperties;
  present: boolean;
  gap: boolean;
  children: ReactElement;
}
export function useChatPresenceItem({
  entry,
  gap,
  onExited,
}: ChatPresenceItemInput): ChatPresenceItemView {
  const outerRef = useRef<HTMLDivElement>(null);
  const innerRef = useRef<HTMLDivElement>(null);
  const running = useRef<Animation[]>([]);
  const settled = useRef(entry.initial);
  const height = useRef<number | null>(null);
  const exit = useRef(onExited);
  useLayoutEffect(() => {
    exit.current = onExited;
  }, [onExited]);
  const reduce = useReducedMotion();
  const visibility = useDocumentVisibility();
  const animate = useCallback((): (() => void) => {
    const box = outerRef.current;
    const body = innerRef.current;
    if (!box || !body) {
      return () => {};
    }
    const target = entry.present ? body.getBoundingClientRect().height : 0;
    const interrupted = running.current.length > 0;
    const start = interrupted
      ? box.getBoundingClientRect().height
      : (height.current ?? box.getBoundingClientRect().height);
    const computed = getComputedStyle(body);
    const opacity = computed.opacity;
    const transform = computed.transform;
    running.current.forEach((animation) => animation.cancel());
    running.current = [];
    height.current = target;
    const finish = (): void => {
      box.style.height = entry.present ? "auto" : "0px";
      settled.current = true;
      if (!entry.present) {
        exit.current(entry.key);
      }
    };
    if (
      reduce ||
      visibility === "hidden" ||
      (settled.current && !interrupted && start === target)
    ) {
      finish();
      return () => {};
    }
    const timing: KeyframeAnimationOptions = {
      duration: 320,
      easing: "cubic-bezier(.4,0,.4,1)",
      fill: "both",
    };
    const size = box.animate(
      [{ height: `${start}px` }, { height: `${target}px` }],
      timing,
    );
    const content = body.animate(
      [
        {
          opacity: settled.current || interrupted ? opacity : 0,
          transform:
            settled.current || interrupted ? transform : "translateY(0.625rem)",
        },
        {
          opacity: entry.present ? 1 : 0,
          transform: entry.present ? "none" : "translateY(-0.3125rem)",
        },
      ],
      timing,
    );
    const animations = [size, content];
    running.current = animations;
    let cancelled = false;
    void size.finished.then(
      () => {
        if (cancelled || running.current !== animations) {
          return;
        }
        finish();
        animations.forEach((animation) => animation.cancel());
        running.current = [];
      },
      () => {},
    );
    return () => {
      cancelled = true;
    };
  }, [entry.key, entry.present, reduce, visibility]);
  useLayoutEffect(() => {
    let cancel = animate();
    const body = innerRef.current;
    if (!body) {
      return cancel;
    }
    const observer = new ResizeObserver(() => {
      const target = entry.present ? body.getBoundingClientRect().height : 0;
      if (target !== height.current) {
        cancel();
        cancel = animate();
      }
    });
    observer.observe(body);
    return () => {
      cancel();
      observer.disconnect();
    };
  }, [animate, entry.present]);
  useLayoutEffect(
    () => () => {
      running.current.forEach((animation) => animation.cancel());
      running.current = [];
    },
    [],
  );
  return {
    outerRef,
    innerRef,
    style: { height: entry.initial ? void 0 : 0 },
    present: entry.present,
    gap,
    children: entry.node,
  };
}
