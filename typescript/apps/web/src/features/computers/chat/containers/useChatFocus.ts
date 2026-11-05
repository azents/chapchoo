"use client";
import { useCallback, useLayoutEffect, useRef } from "react";
import type { AgentsProps } from "../../types";
import type { RefObject } from "react";

export interface ChatFocus {
  statusRef: RefObject<HTMLButtonElement | null>;
  navigationRef: RefObject<HTMLButtonElement | null>;
  activityClosed: () => void;
  navigationClosed: () => void;
  nestedClosed: () => void;
}
/** Preserve original openers across nested overlays and parent trap reactivation. */
export function useChatFocus(
  product: AgentsProps,
  secondaryOverlay = false,
): {
  product: AgentsProps;
  focus: ChatFocus;
} {
  const statusRef = useRef<HTMLButtonElement>(null);
  const navigationRef = useRef<HTMLButtonElement>(null);
  const nestedOpener = useRef<HTMLElement | null>(null);
  const navigationOpener = useRef<HTMLElement | null>(null);
  const originalAutofocus = useRef<string | null>(null);
  const secondaryWasOpen = useRef(false);
  const restoreAutofocus = useCallback((): void => {
    const opener = navigationOpener.current;
    if (opener) {
      if (originalAutofocus.current === null) {
        opener.removeAttribute("data-autofocus");
      } else {
        opener.setAttribute("data-autofocus", originalAutofocus.current);
      }
    }
    navigationOpener.current = null;
  }, []);
  useLayoutEffect(() => {
    if (secondaryOverlay && !secondaryWasOpen.current) {
      restoreAutofocus();
      navigationOpener.current =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      originalAutofocus.current =
        navigationOpener.current?.getAttribute("data-autofocus") ?? null;
      // Mantine's public autofocus marker makes trap reactivation preserve this opener.
      navigationOpener.current?.setAttribute("data-autofocus", "true");
    }
    secondaryWasOpen.current = secondaryOverlay;
  }, [secondaryOverlay, restoreAutofocus]);
  useLayoutEffect(() => restoreAutofocus, [restoreAutofocus]);
  function capture(): void {
    nestedOpener.current =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
  }
  return {
    product: {
      ...product,
      onAction: (value) => {
        capture();
        product.onAction(value);
      },
      onOpenDetails: () => {
        capture();
        product.onOpenDetails();
      },
    },
    focus: {
      statusRef,
      navigationRef,
      activityClosed: () => statusRef.current?.focus({ preventScroll: true }),
      navigationClosed: () => {
        restoreAutofocus();
        navigationRef.current?.focus({ preventScroll: true });
      },
      nestedClosed: () => nestedOpener.current?.focus({ preventScroll: true }),
    },
  };
}
