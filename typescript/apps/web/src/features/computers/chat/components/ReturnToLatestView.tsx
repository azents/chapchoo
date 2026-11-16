import { floatingSurfaceClassName } from "@chapchoo/ui/FloatingSurface";
import { Transition } from "@mantine/core";
import classes from "./ReturnToLatestView.module.css";
import type { ReturnToLatestProps } from "../containers/useReturnToLatest";
import type { ReactElement } from "react";

export function ReturnToLatestView({
  visible,
  label,
  unreadLabel,
  onLatest,
  shownUnread,
  phase,
  width,
  normalRef,
  unreadRef,
  duration,
  onExited,
  onLabelAnimationEnd,
}: ReturnToLatestProps): ReactElement {
  return (
    <div className={classes.anchor}>
      <span className={classes.measurements} aria-hidden="true">
        <span ref={normalRef}>{label}</span>
        <span ref={unreadRef}>{unreadLabel}</span>
      </span>
      <Transition
        mounted={visible}
        transition="slide-up"
        duration={duration}
        timingFunction="cubic-bezier(.4,0,.4,1)"
        onExited={onExited}
      >
        {(style) => (
          <button
            type="button"
            className={`${classes.button} ${floatingSurfaceClassName ?? ""}`}
            style={style}
            aria-label={shownUnread ? unreadLabel : label}
            data-return-to-latest={visible ? "visible" : "exiting"}
            inert={!visible}
            onClick={onLatest}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 5v14m-5-5 5 5 5-5" />
            </svg>
            <span className={classes.labels} style={{ width }}>
              <span data-phase={phase} onAnimationEnd={onLabelAnimationEnd}>
                {shownUnread ? unreadLabel : label}
              </span>
            </span>
          </button>
        )}
      </Transition>
    </div>
  );
}
