import { MascotArtwork } from "./MascotArtwork";
import { useMascotPlayback } from "./useMascotPlayback";
import classes from "./AgentMascotMotion.module.css";
import type { MascotArtworkProps } from "./MascotArtwork";
import type { ReactElement } from "react";

export interface AgentMascotMotionProps extends MascotArtworkProps {
  /** Explicit opt-in for an Agent acting as the subject of interaction. */
  playing: boolean;
  /** Localized accessible action name for click, touch and keyboard replay. */
  replayLabel: string;
  /** Stable initial stagger; does not change the 1200ms motion clip. */
  delayMs?: number;
}

/** Reference-local motion primitive, not wired into any product page. */
export function AgentMascotMotion({
  playing,
  replayLabel,
  delayMs = 0,
  ...artwork
}: AgentMascotMotionProps): ReactElement {
  const { ref, running, reducedMotion, replay, finish } = useMascotPlayback(
    playing,
    delayMs,
  );
  return (
    <button
      ref={ref}
      type="button"
      className={classes.trigger}
      aria-label={replayLabel}
      disabled={reducedMotion}
      onClick={replay}
    >
      <span
        className={classes.motion}
        data-playing={running}
        aria-hidden="true"
        onAnimationEnd={finish}
      >
        <MascotArtwork {...artwork} />
      </span>
    </button>
  );
}
