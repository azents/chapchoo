export interface NativeComputer {
  container: string;
  volume: string;
  daemon: string;
  user: string;
  noCopy: boolean;
}
type NativeStatus = "created" | "running" | "exited" | "removing";
interface NativeReadinessState {
  Status: string;
  Running: boolean;
  Dead: boolean;
  Paused: boolean;
  Restarting: boolean;
  Error: string;
}
/** Docker marks Dead during removal; that transition is not running readiness. */
export function nativeReadinessStatus(
  state: NativeReadinessState,
): NativeStatus {
  const status = state.Status;
  const admitted =
    status === "created" ||
    status === "running" ||
    status === "exited" ||
    status === "removing";
  if (
    !admitted ||
    state.Running !== (status === "running") ||
    (state.Dead && status !== "removing") ||
    state.Paused ||
    state.Restarting ||
    state.Error
  ) {
    throw new Error(
      `Native execution is not in an admitted readiness state: status=${admitted ? status : "unsupported"}, running=${state.Running}, dead=${state.Dead}, paused=${state.Paused}, restarting=${state.Restarting}, error=${Boolean(state.Error)}`,
    );
  }
  return status;
}
export type NativeObservation =
  | { type: "absent"; daemon: string; volume: string }
  | {
      type: "present";
      native: NativeComputer;
      status: NativeStatus;
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
