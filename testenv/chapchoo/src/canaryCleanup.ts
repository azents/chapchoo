export interface CanaryState {
  id: string;
  running: boolean;
  labels: Record<string, string>;
}
export interface CanaryControl {
  inspect(): Promise<CanaryState | null>;
  stop(): Promise<void>;
  remove(): Promise<void>;
}

/** Reconcile only a positively created canary's exact identity, never its Start. */
export async function cleanupCanary(
  control: CanaryControl,
  id: string,
  labels: Record<string, string>,
): Promise<void> {
  const state = await control.inspect();
  if (!state) {
    return;
  }
  if (
    state.id !== id ||
    !Object.entries(labels).every(([key, value]) => state.labels[key] === value)
  ) {
    throw new Error("Canary identity changed");
  }
  if (state.running) {
    await control.stop();
  }
  await control.remove();
  if (await control.inspect()) {
    throw new Error("Canary removal is unconfirmed");
  }
}
