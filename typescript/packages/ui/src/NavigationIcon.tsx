import { rem } from "@mantine/core";
import classes from "./NavigationIcon.module.css";
import type { ReactElement } from "react";

export interface NavigationIconProps {
  name: "personal" | "team" | "admin" | "down" | "back" | "forward" | "add";
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
      ) : (
        <path d="M12 5v14M5 12h14" />
      )}
    </svg>
  );
}
