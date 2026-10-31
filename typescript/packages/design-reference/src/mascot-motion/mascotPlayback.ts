export interface MascotPlaybackOptions {
  enabled: boolean;
  automatic: boolean;
  delayMs: number;
}

export interface MascotPlayback {
  configure: (options: MascotPlaybackOptions) => void;
  replay: () => void;
  finish: () => void;
  dispose: () => void;
}

/** A single active gesture; input during it neither restarts nor queues work. */
export function createMascotPlayback({
  schedule,
  onActiveChange,
}: {
  schedule: (callback: () => void, delayMs: number) => () => void;
  onActiveChange: (active: boolean) => void;
}): MascotPlayback {
  let options: MascotPlaybackOptions = {
    enabled: false,
    automatic: false,
    delayMs: 0,
  };
  let active = false;
  let cancelTimer: (() => void) | undefined;
  let generation = 0;

  function cancelPending(): void {
    generation += 1;
    cancelTimer?.();
    cancelTimer = undefined;
  }

  function replay(): void {
    if (!options.enabled || active) {
      return;
    }
    cancelPending();
    active = true;
    onActiveChange(true);
  }

  function queue(delayMs: number): void {
    const current = generation;
    cancelTimer = schedule(() => {
      if (current === generation) {
        replay();
      }
    }, delayMs);
  }

  return {
    configure(next): void {
      cancelPending();
      options = next;
      active = false;
      onActiveChange(false);
      if (options.enabled && options.automatic) {
        queue(options.delayMs);
      }
    },
    replay,
    finish(): void {
      if (!active) {
        return;
      }
      active = false;
      onActiveChange(false);
      if (options.enabled && options.automatic) {
        queue(6000);
      }
    },
    dispose(): void {
      cancelPending();
      active = false;
      options = { ...options, enabled: false };
    },
  };
}
