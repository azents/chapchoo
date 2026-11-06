import { Drawer, Modal } from "@mantine/core";
import classes from "./ConnectionPanel.module.css";
import type { ReactElement, ReactNode } from "react";
export interface AuthenticationDialogProps {
  mobile: boolean;
  opened: boolean;
  onClose: () => void;
  title: string;
  closeLabel: string;
  locked: boolean;
  size?: "md" | "lg";
  children: ReactNode;
}
export function AuthenticationDialog(
  props: AuthenticationDialogProps,
): ReactElement {
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
      classNames={{ content: classes.sheet }}
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
