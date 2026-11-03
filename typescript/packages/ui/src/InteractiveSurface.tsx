import classes from "./InteractiveSurface.module.css";
import type { ComponentPropsWithRef, ReactElement } from "react";

export type InteractiveSurfaceProps =
  | (ComponentPropsWithRef<"a"> & { href: string })
  | (Omit<ComponentPropsWithRef<"button">, "type"> & { href?: never });

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
