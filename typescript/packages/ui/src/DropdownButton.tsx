"use client";

import { Popover } from "@mantine/core";
import { useMergedRef, useMounted } from "@mantine/hooks";
import { useRef } from "react";
import { InteractiveSurface } from "./InteractiveSurface";
import classes from "./DropdownButton.module.css";
import type { PopoverProps } from "@mantine/core";
import type { ReactElement, ReactNode, Ref } from "react";

export interface DropdownButtonProps {
  label: ReactNode;
  accessibleLabel: string;
  children: ReactNode;
  opened: boolean;
  onChange: (opened: boolean) => void;
  withinPortal?: boolean;
  width?: PopoverProps["width"];
  className?: string;
  dropdownClassName?: string;
  triggerRef?: Ref<HTMLButtonElement>;
}

/** One mounted trigger owns a controlled, dismissible overlay. */
export function DropdownButton(props: DropdownButtonProps): ReactElement {
  const mounted = useMounted();
  const trigger = useRef<HTMLButtonElement>(null);
  const triggerRef = useMergedRef(trigger, props.triggerRef);
  return (
    <Popover
      opened={props.opened}
      onChange={props.onChange}
      position="bottom-start"
      width={props.width}
      withinPortal={props.withinPortal ?? true}
      closeOnEscape={false}
      trapFocus
      shadow="sm"
    >
      <Popover.Target>
        <InteractiveSurface
          ref={triggerRef}
          className={[classes.trigger, props.className]
            .filter(Boolean)
            .join(" ")}
          aria-label={props.accessibleLabel}
          aria-expanded={props.opened}
          disabled={!mounted}
          onClick={(event) => {
            event.currentTarget.focus();
            props.onChange(!props.opened);
          }}
          onKeyDown={(event) => {
            if (event.key === "Escape" && props.opened) {
              event.stopPropagation();
              props.onChange(false);
            }
          }}
        >
          {props.label}
        </InteractiveSurface>
      </Popover.Target>
      <Popover.Dropdown
        p="sm"
        className={props.dropdownClassName}
        onKeyDownCapture={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            props.onChange(false);
            trigger.current?.focus({ preventScroll: true });
          }
        }}
      >
        {props.children}
      </Popover.Dropdown>
    </Popover>
  );
}
