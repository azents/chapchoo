import classes from "./InteractiveSurface.module.css";
import type { ComponentPropsWithoutRef, ReactElement } from "react";

export type InteractiveSurfaceProps =
  | (ComponentPropsWithoutRef<"a"> & { href: string })
  | (Omit<ComponentPropsWithoutRef<"button">, "type"> & { href?: never });

/** Native navigation/action semantics with the shared calm surface-state treatment. */
export function InteractiveSurface(
  props: InteractiveSurfaceProps,
): ReactElement {
  if (typeof props.href === "string") {
    const { className, ...link } = props;
    return (
      <a
        {...link}
        className={[classes.surface, className].filter(Boolean).join(" ")}
      />
    );
  }
  const { className, ...button } = props;
  return (
    <button
      {...button}
      type="button"
      className={[classes.surface, className].filter(Boolean).join(" ")}
    />
  );
}
