"use client";

import { rem } from "@mantine/core";
import { Mascot } from "./Mascot";
import { defaultMascot } from "./designTokens";
import { mascotArtwork } from "./mascotArtwork";
import { useMascotPlayback } from "./useMascotPlayback";
import classes from "./AgentMascotMotion.module.css";
import type { MascotId } from "./designTokens";
import type { ReactElement } from "react";

export interface AgentMascotMotionProps {
  character?: MascotId;
  basePath?: string;
  size?: number;
  /** Explicit opt-in for selected animated character placements. */
  playing: boolean;
  /** Localized accessible action name for click, touch and keyboard replay. */
  replayLabel: string;
  /** Stable initial stagger; does not change the 1200ms motion clip. */
  delayMs?: number;
}

/** Shared motion preserving the static artwork's square layout and WebP URL. */
export function AgentMascotMotion({
  playing,
  replayLabel,
  delayMs = 0,
  character = defaultMascot,
  basePath = "",
  size = 64,
}: AgentMascotMotionProps): ReactElement {
  const { ref, running, reducedMotion, replay, finish } = useMascotPlayback(
    playing,
    delayMs,
  );
  const artwork = mascotArtwork[character];
  const contactY = artwork.bottom;
  return (
    <button
      ref={ref}
      type="button"
      className={classes.trigger}
      aria-label={replayLabel}
      disabled={reducedMotion}
      onClick={replay}
      style={{ width: rem(size), height: rem(size), flexShrink: 0 }}
    >
      <span
        className={classes.motion}
        data-playing={running}
        aria-hidden="true"
        onAnimationEnd={finish}
        style={{ transformOrigin: `50% ${(contactY / 1254) * 100}%` }}
      >
        <Mascot character={character} basePath={basePath} size={size} />
      </span>
    </button>
  );
}
