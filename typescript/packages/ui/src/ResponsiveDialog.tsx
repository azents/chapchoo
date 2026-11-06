"use client";

import { Drawer, Modal } from "@mantine/core";
import classes from "./ResponsiveDialog.module.css";
import type { ReactElement, ReactNode } from "react";

export interface ResponsiveDialogProps {
  mobile: boolean;
  opened: boolean;
  onClose: () => void;
  title: string;
  closeLabel: string;
  locked: boolean;
  size?: "md" | "lg";
  children: ReactNode;
}

/** Shared presentation; callers retain operation state and dismissal authority. */
export function ResponsiveDialog(props: ResponsiveDialogProps): ReactElement {
  const common = {
    opened: props.opened,
    onClose: props.onClose,
    title: props.title,
    withCloseButton: !props.locked,
    closeButtonProps: { "aria-label": props.closeLabel },
    closeOnEscape: !props.locked,
    closeOnClickOutside: !props.locked,
  };
  return props.mobile ? (
    <Drawer
      {...common}
      position="bottom"
      size="auto"
      classNames={{ content: classes.sheet, body: classes.body }}
      overlayProps={{ backgroundOpacity: 0.35, blur: 4 }}
    >
      {props.children}
    </Drawer>
  ) : (
    <Modal {...common} centered size={props.size ?? "md"}>
      {props.children}
    </Modal>
  );
}
