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

export function recoverCanaryId(
  state: { id: string; name: string; labels: Record<string, string> },
  name: string,
  labels: Record<string, string>,
): string {
  if (
    !/^[a-f0-9]{64}$/.test(state.id) ||
    state.name !== `/${name}` ||
    !Object.entries(labels).every(([key, value]) => state.labels[key] === value)
  ) {
    throw new Error("Canary creation identity is unverified");
  }
  return state.id;
}

export interface CanaryVolumeState {
  name: string;
  driver: string;
  scope: string;
  labels: Record<string, string>;
}
export async function cleanupCanaryVolume(
  control: {
    inspect(): Promise<CanaryVolumeState | null>;
    remove(): Promise<void>;
  },
  name: string,
  labels: Record<string, string>,
): Promise<void> {
  const state = await control.inspect();
  if (!state) {
    return;
  }
  if (
    state.name !== name ||
    state.driver !== "local" ||
    state.scope !== "local" ||
    !Object.entries(labels).every(([key, value]) => state.labels[key] === value)
  ) {
    throw new Error("Canary volume identity is unverified");
  }
  await control.remove();
  if (await control.inspect()) {
    throw new Error("Canary volume removal is unconfirmed");
  }
}

/** Reconcile only a verified canary's exact identity, never its Start. */
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
