import { rem } from "@mantine/core";
import classes from "./NavigationIcon.module.css";
import type { ReactElement } from "react";

export interface NavigationIconProps {
  name:
    | "personal"
    | "team"
    | "admin"
    | "down"
    | "back"
    | "forward"
    | "add"
    | "edit"
    | "shuffle"
    | "start"
    | "stop"
    | "restart"
    | "info"
    | "check"
    | "agents"
    | "mail";
  size?: "sm" | "md";
  className?: string;
}
/** Decorative navigation geometry; the containing control supplies its accessible name. */
export function NavigationIcon({
  name,
  size = "md",
  className,
}: NavigationIconProps): ReactElement {
  const extent = rem(size === "sm" ? 16 : 20);
  return (
    <svg
      className={[classes.icon, className].filter(Boolean).join(" ")}
      width={extent}
      height={extent}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
    >
      {name === "personal" ? (
        <path d="m3 11 9-8 9 8M5 10v11h14V10M9 21v-7h6v7" />
      ) : name === "team" ? (
        <>
          <circle cx="9" cy="8" r="3" />
          <path d="M2 21v-2a7 7 0 0 1 14 0v2M16 5a3 3 0 0 1 0 6M22 21v-2a7 7 0 0 0-4-6" />
        </>
      ) : name === "admin" ? (
        <>
          <path d="M12 3 4.5 6v5c0 5 3 8 7.5 10 4.5-2 7.5-5 7.5-10V6Z" />
          <path d="m8.5 12 2.5 2.5 4.5-5" />
        </>
      ) : name === "down" ? (
        <path d="m6 9 6 6 6-6" />
      ) : name === "back" ? (
        <path d="m14 6-6 6 6 6" />
      ) : name === "forward" ? (
        <path d="m10 6 6 6-6 6" />
      ) : name === "edit" ? (
        <path d="m16 3 5 5-12 12-6 1 1-6zM14 5l5 5" />
      ) : name === "shuffle" ? (
        <path d="M3 6h3l12 12h3M17 14l4 4-4 4M3 18h3l4-4M14 10l4-4h3M17 2l4 4-4 4" />
      ) : name === "start" ? (
        <path d="m8 5 11 7-11 7z" />
      ) : name === "stop" ? (
        <rect x="6" y="6" width="12" height="12" rx="2" />
      ) : name === "restart" ? (
        <path d="M20 11a8 8 0 1 0-2 6M20 4v7h-7" />
      ) : name === "info" ? (
        <>
          <circle cx="12" cy="12" r="9" />
          <path d="M12 11v6M12 7h.01" />
        </>
      ) : name === "check" ? (
        <path d="m5 12 4 4L19 6" />
      ) : name === "agents" ? (
        <>
          <rect x="3" y="3" width="7" height="7" rx="2" />
          <rect x="14" y="3" width="7" height="7" rx="2" />
          <rect x="3" y="14" width="7" height="7" rx="2" />
          <rect x="14" y="14" width="7" height="7" rx="2" />
        </>
      ) : name === "mail" ? (
        <>
          <rect x="3" y="5" width="18" height="14" rx="3" />
          <path d="m3 7 9 6 9-6" />
        </>
      ) : (
        <path d="M12 5v14M5 12h14" />
      )}
    </svg>
  );
}
