import {
  useDocumentVisibility,
  useIntersection,
  useReducedMotion,
} from "@mantine/hooks";
import { useEffect, useState } from "react";
import { createMascotPlayback } from "./mascotPlayback";
import type { RefCallback } from "react";

/** Restart locally when visible; never catch up missed background cycles. */
export function useMascotPlayback(
  playing: boolean,
  delayMs: number,
): {
  ref: RefCallback<HTMLButtonElement>;
  running: boolean;
  reducedMotion: boolean;
  replay: () => void;
  finish: () => void;
} {
  const { ref, entry } = useIntersection<HTMLButtonElement>({ threshold: 0.1 });
  const visibility = useDocumentVisibility();
  const reduced = useReducedMotion();
  const enabled =
    visibility === "visible" && !reduced && entry?.isIntersecting === true;
  const [running, setRunning] = useState(false);
  const [playback] = useState(() =>
    createMascotPlayback({
      schedule(callback, delay): () => void {
        const timer = window.setTimeout(callback, delay);
        return (): void => window.clearTimeout(timer);
      },
      onActiveChange: setRunning,
    }),
  );
  useEffect(() => {
    playback.configure({ enabled, automatic: playing, delayMs });
    return (): void => playback.dispose();
  }, [playback, enabled, playing, delayMs]);
  return {
    ref,
    running,
    reducedMotion: reduced === true,
    replay: playback.replay,
    finish: playback.finish,
  };
}
