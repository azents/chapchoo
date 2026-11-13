"use client";

import { Drawer } from "@mantine/core";
import classes from "./Sheet.module.css";
import type { DrawerProps } from "@mantine/core";
import type { ComponentPropsWithRef, ReactElement } from "react";

export type SheetProps = Pick<
  DrawerProps,
  | "opened"
  | "onClose"
  | "children"
  | "title"
  | "size"
  | "returnFocus"
  | "trapFocus"
  | "closeOnEscape"
  | "closeOnClickOutside"
  | "onExitTransitionEnd"
  | "closeButtonProps"
  | "overlayProps"
>;

/** Fixed chrome and a clipped body frame; compose one SheetViewport inside. */
export function Sheet(props: SheetProps): ReactElement {
  return (
    <Drawer
      {...props}
      position="bottom"
      lockScroll={false}
      classNames={{
        inner: classes.inner,
        content: classes.content,
        header: classes.header,
        body: classes.body,
      }}
    />
  );
}

export type SheetViewportProps = ComponentPropsWithRef<"div">;

/** The only vertical scroll owner below Sheet's header. Edge gestures stay native. */
export function SheetViewport({
  className,
  ...props
}: SheetViewportProps): ReactElement {
  return (
    <div
      {...props}
      data-sheet-viewport
      className={[classes.viewport, className].filter(Boolean).join(" ")}
    />
  );
}
