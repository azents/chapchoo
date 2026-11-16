import classes from "./FloatingSurface.module.css";
import type { ComponentPropsWithRef, ReactElement } from "react";

/** Apply to native/Mantine controls when a wrapping surface would change semantics. */
export const floatingSurfaceClassName = classes.surface;

/** Shared material only; placement, radius and interaction belong to the caller. */
export function FloatingSurface({
  className,
  ...props
}: ComponentPropsWithRef<"div">): ReactElement {
  return (
    <div
      {...props}
      className={[classes.surface, className].filter(Boolean).join(" ")}
    />
  );
}
