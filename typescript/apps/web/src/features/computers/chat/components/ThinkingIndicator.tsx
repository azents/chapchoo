import classes from "./ThinkingIndicator.module.css";
import type { ReactElement } from "react";

/** Ephemeral execution feedback, independent of stored conversation messages. */
export function ThinkingIndicator({ label }: { label: string }): ReactElement {
  return (
    <div className={classes.root} role="status" aria-live="polite">
      <div className={classes.content}>
        <span className={classes.spark} aria-hidden="true">
          ✦
        </span>
        <span className={classes.label}>{label}</span>
      </div>
    </div>
  );
}
