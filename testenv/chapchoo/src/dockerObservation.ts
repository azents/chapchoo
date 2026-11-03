export interface NativeComputer {
  container: string;
  volume: string;
  daemon: string;
  user: string;
  noCopy: boolean;
}
export type NativeObservation =
  | { type: "absent"; daemon: string; volume: string }
  | {
      type: "present";
      native: NativeComputer;
      status: "created" | "running" | "exited" | "removing";
    };

/** A gap is not success; only a different verified running ID is ready. */
export function runningSuccessor(
  previous: NativeComputer,
  observed: NativeObservation,
): NativeComputer | null {
  const identity = observed.type === "present" ? observed.native : observed;
  if (
    identity.daemon !== previous.daemon ||
    identity.volume !== previous.volume
  ) {
    throw new Error(
      "Replacement observation changed the original daemon or volume",
    );
  }
  if (
    observed.type === "absent" ||
    observed.native.container === previous.container ||
    observed.status === "created"
  ) {
    return null;
  }
  if (observed.status !== "running") {
    throw new Error("A successor is terminating before running readiness");
  }
  return observed.native;
}
