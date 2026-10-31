import { MascotArtwork } from "./MascotArtwork";
import { useMascotPlayback } from "./useMascotPlayback";
import classes from "./AgentMascotMotion.module.css";
import type { MascotArtworkProps } from "./MascotArtwork";
import type { ReactElement } from "react";

export interface AgentMascotMotionProps extends MascotArtworkProps {
  /** Explicit opt-in for an Agent acting as the subject of interaction. */
  playing: boolean;
  /** Stable initial stagger; does not change the 1200ms motion clip. */
  delayMs?: number;
}

/** Reference-local motion primitive, not wired into any product page. */
export function AgentMascotMotion({
  playing,
  delayMs = 0,
  ...artwork
}: AgentMascotMotionProps): ReactElement {
  const { ref, running } = useMascotPlayback(playing);
  return (
    <span
      ref={ref}
      className={classes.motion}
      data-playing={running}
      aria-hidden="true"
      style={{ animationDelay: `${delayMs}ms` }}
    >
      <MascotArtwork {...artwork} />
    </span>
  );
}
