"use client";

import { Image } from "@mantine/core";
import { mascotArtwork } from "./mascotArtwork";
import { useMascotPlayback } from "./useMascotPlayback";
import classes from "./AgentMascotMotion.module.css";
import type { CSSProperties, ReactElement } from "react";

/** Decorative automatic motion; containing links and form controls keep input ownership. */
export function CharacterArtwork({
  assetKey,
  basePath = "",
  className,
  style,
  delayMs = 0,
}: {
  assetKey: string;
  basePath?: string;
  className?: string;
  style?: CSSProperties;
  delayMs?: number;
}): ReactElement {
  const { ref, running, finish } = useMascotPlayback<HTMLSpanElement>(
    true,
    delayMs,
  );
  const contact = Object.values(mascotArtwork).find(
    (artwork) => artwork.file === `transparent/${assetKey}.png`,
  );
  return (
    <span
      ref={ref}
      className={[classes.artwork, className].filter(Boolean).join(" ")}
      style={style}
      aria-hidden="true"
    >
      <span
        className={`${classes.motion} ${classes.artworkMotion}`}
        data-playing={running}
        onAnimationEnd={finish}
        style={{
          transformOrigin: `50% ${((contact?.bottom ?? 1254) / 1254) * 100}%`,
        }}
      >
        <Image
          src={`${basePath}/design-assets/${assetKey}-216.webp`}
          alt=""
          w="100%"
          h="100%"
          fit="contain"
        />
      </span>
    </span>
  );
}
