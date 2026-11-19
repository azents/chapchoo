import { useLayoutEffect, useRef } from "react";
import type { RefObject } from "react";

interface Size {
  width: number;
  height: number;
}

function dimensions({ width, height }: Size): Keyframe {
  return { width: `${width}px`, height: `${height}px` };
}

function spring(from: Size, to: Size, available: number): Keyframe[] {
  return Array.from({ length: 49 }, (_, index) => {
    const t = index / 48;
    const progress =
      index === 48
        ? 1
        : 1 -
          Math.exp(-9.5 * t) *
            (Math.cos(8.8 * t) + (9.5 / 8.8) * Math.sin(8.8 * t));
    return {
      ...dimensions({
        width: Math.max(
          0,
          Math.min(available, from.width + (to.width - from.width) * progress),
        ),
        height: Math.max(0, from.height + (to.height - from.height) * progress),
      }),
      offset: t,
    };
  });
}

/** Size the one floating surface around its content while the composer stays anchored. */
export function useWorkSummarySize({
  expanded,
  exiting,
  animateOnMount,
  onExited,
}: {
  expanded: boolean;
  exiting: boolean;
  animateOnMount: boolean;
  onExited: () => void;
}): {
  surface: RefObject<HTMLElement | null>;
  content: RefObject<HTMLDivElement | null>;
} {
  const surface = useRef<HTMLElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const animation = useRef<Animation | null>(null);
  const target = useRef<Size | null>(null);
  const exit = useRef(onExited);
  useLayoutEffect(() => {
    exit.current = onExited;
  }, [onExited]);
  useLayoutEffect(
    () => () => {
      animation.current?.cancel();
      animation.current = null;
      target.current = null;
    },
    [],
  );
  useLayoutEffect(() => {
    const panel = surface.current;
    const body = content.current;
    const parent = panel?.parentElement;
    if (!panel || !body || !parent) {
      return;
    }
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    let active = true;
    function styleSize(element: HTMLElement, size: Size): void {
      element.style.width = `${size.width}px`;
      element.style.height = `${size.height}px`;
    }
    const sync = (reason: "transition" | "layout" | "preference"): void => {
      const display = panel.getBoundingClientRect();
      const panelStyle = getComputedStyle(panel);
      const parentStyle = getComputedStyle(parent);
      const available = Math.max(
        0,
        parent.clientWidth -
          parseFloat(parentStyle.paddingLeft) -
          parseFloat(parentStyle.paddingRight),
      );
      const stop = (): void => {
        animation.current?.cancel();
        animation.current = null;
      };
      if (exiting) {
        const margin = panelStyle.marginBottom;
        stop();
        target.current = null;
        styleSize(panel, { width: 0, height: 0 });
        panel.style.marginBottom = "0px";
        if (media.matches || display.height <= 2) {
          queueMicrotask(() => {
            if (active) {
              exit.current();
            }
          });
          return;
        }
        const motion = panel.animate(
          [
            {
              ...dimensions(display),
              marginBottom: margin,
              offset: 0,
              easing: "cubic-bezier(.2,.75,.3,1)",
            },
            {
              ...dimensions({
                width: Math.max(
                  display.width,
                  Math.min(display.width + 10, available + 12),
                ),
                height:
                  display.height +
                  Math.min(8, Math.max(4, display.height * 0.09)),
              }),
              marginBottom: margin,
              offset: 0.22,
              easing: "cubic-bezier(.58,.08,.66,1)",
            },
            { width: "0px", height: "0px", marginBottom: "0px", offset: 1 },
          ],
          { duration: 420, fill: "both" },
        );
        animation.current = motion;
        void motion.finished.then(
          () => {
            if (active && animation.current === motion) {
              exit.current();
            }
          },
          () => {},
        );
        return;
      }
      const rootRem = parseFloat(
        getComputedStyle(document.documentElement).fontSize,
      );
      const width = Math.min(available, rootRem * (expanded ? 49.375 : 22));
      const borderWidth =
        parseFloat(panelStyle.borderLeftWidth) +
        parseFloat(panelStyle.borderRightWidth);
      const borderHeight =
        parseFloat(panelStyle.borderTopWidth) +
        parseFloat(panelStyle.borderBottomWidth);
      body.style.width = `${Math.max(0, width - borderWidth)}px`;
      const to = {
        width,
        height: body.getBoundingClientRect().height + borderHeight,
      };
      const previous = target.current;
      if (
        reason !== "preference" &&
        previous?.width === to.width &&
        previous.height === to.height
      ) {
        return;
      }
      const from =
        !previous && animateOnMount && !animation.current
          ? { width: 0, height: 0 }
          : display;
      stop();
      target.current = to;
      panel.style.marginBottom = "";
      styleSize(panel, to);
      if (
        (!previous && !animateOnMount) ||
        reason === "layout" ||
        media.matches ||
        (Math.abs(from.width - to.width) < 1 &&
          Math.abs(from.height - to.height) < 1)
      ) {
        return;
      }
      const motion = panel.animate(spring(from, to, available), {
        duration: to.height >= from.height ? 580 : 500,
        easing: "linear",
      });
      animation.current = motion;
      void motion.finished.then(
        () => {
          if (animation.current === motion) {
            animation.current = null;
          }
        },
        () => {},
      );
    };
    sync("transition");
    const resize = new ResizeObserver(() => sync("layout"));
    if (!exiting) {
      resize.observe(parent, { box: "content-box" });
      resize.observe(body, { box: "border-box" });
    }
    const preference = (): void => sync("preference");
    media.addEventListener("change", preference);
    return () => {
      active = false;
      resize.disconnect();
      media.removeEventListener("change", preference);
    };
  }, [animateOnMount, exiting, expanded]);
  return { surface, content };
}
