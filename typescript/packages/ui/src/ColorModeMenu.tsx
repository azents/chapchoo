"use client";

import { ActionIcon, Menu, rem } from "@mantine/core";
import { useRef } from "react";
import classes from "./ColorModeMenu.module.css";
import type { ReactElement } from "react";

export interface ColorModeMenuProps {
  value: "system" | "light" | "dark";
  label: string;
  labels: { system: string; light: string; dark: string };
  onChange: (value: "system" | "light" | "dark") => void;
}

function ModeIcon({
  mode,
}: {
  mode: ColorModeMenuProps["value"];
}): ReactElement {
  return (
    <svg
      width={rem(18)}
      height={rem(18)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {mode === "system" ? (
        <>
          <rect x="3" y="4" width="18" height="13" rx="2" />
          <path d="M8 21h8M12 17v4" />
        </>
      ) : mode === "light" ? (
        <>
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5 19 19M5 19l1.5-1.5M17.5 6.5 19 5" />
        </>
      ) : (
        <path d="M20.5 13A8.5 8.5 0 0 1 11 3.5 8.5 8.5 0 1 0 20.5 13Z" />
      )}
    </svg>
  );
}
function Check(): ReactElement {
  return (
    <svg
      width={rem(16)}
      height={rem(16)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      aria-hidden="true"
    >
      <path d="m5 12 4 4L19 6" />
    </svg>
  );
}

/** Presentation entry point; apps own preference and persistence. */
export function ColorModeMenu({
  value,
  label,
  labels,
  onChange,
}: ColorModeMenuProps): ReactElement {
  const trigger = useRef<HTMLButtonElement>(null);
  const dropdown = useRef<HTMLDivElement>(null);
  function restoreMenuFocus(): void {
    // Restore menu-owned focus; an outside target retains its own focus.
    if (dropdown.current?.contains(document.activeElement)) {
      trigger.current?.focus({ preventScroll: true });
    }
  }
  return (
    <Menu
      position="bottom-end"
      offset={8}
      width={192}
      shadow="sm"
      returnFocus={false}
      onClose={restoreMenuFocus}
    >
      <Menu.Target>
        <ActionIcon
          ref={trigger}
          variant="subtle"
          color="gray"
          size="xl"
          className={classes.trigger}
          aria-label={label}
          aria-description={labels[value]}
          title={label}
          data-mode={value}
        >
          <ModeIcon mode={value} />
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown ref={dropdown}>
        {(["system", "light", "dark"] as const).map((choice) => (
          <Menu.Item
            key={choice}
            leftSection={<ModeIcon mode={choice} />}
            rightSection={value === choice ? <Check /> : null}
            aria-current={value === choice ? "true" : undefined}
            onClick={() => onChange(choice)}
          >
            {labels[choice]}
          </Menu.Item>
        ))}
      </Menu.Dropdown>
    </Menu>
  );
}
