import type * as Wire from "@chapchoo/rpc-client/computers";
import type { Timestamp } from "@chapchoo/rpc-client/timestamp";
import { invalidResponse } from "./errors.js";
import { timestampMilliseconds, validTimestamp } from "./validation.js";
import { validAgentPalette } from "./agent-profile.js";
import type { AgentPaletteKey, AgentProfileImage } from "./agent-profile.js";
import { validHandle } from "./handle.js";
import { authenticationSelection } from "./external-authentication.js";
import type { AuthenticationSelection } from "./external-authentication.js";
export type { AgentPaletteKey, AgentProfileImage } from "./agent-profile.js";

export type Power = "running" | "stopped" | "unknown";
export type Connection = "connected" | "disconnected" | "unknown";
export type Phase =
  | "accepted"
  | "retiring"
  | "starting"
  | "deleting"
  | "completed"
  | "blocked"
  | "uncertain";
export type LifecycleAction = "start" | "stop" | "restart" | "delete";
export type OwnerAction = LifecycleAction | "retryPreparation";
export type BlockReason =
  | "unsupported"
  | "deleting"
  | "conflicting"
  | "managementUnavailable"
  | "powerUnknown"
  | "requiresStopped"
  | "requiresRunning"
  | "savedUnverified";
export interface ComputerStatus {
  operation: "create" | LifecycleAction;
  intent: "running" | "stopped" | "deleting";
  power: Power;
  lastKnownPower: Power;
  phase: Phase;
  management: Connection;
  connector: Connection;
  saved: "verified" | "missing" | "unknown";
  observedAt: number | null;
}
export type ResourceCursor =
  | {
      kind: "POD" | "PVC" | "BOOTSTRAP_SECRET";
      namespace: string;
      name: string;
      nativeScopeKind: "";
      nativeScope: "";
    }
  | {
      kind: "DOCKER_CONTAINER" | "DOCKER_VOLUME";
      namespace: "";
      name: string;
      nativeScopeKind: "DOCKER_DAEMON";
      nativeScope: string;
    };
export interface ResourceView {
  identity: ResourceCursor;
  uid: string | null;
  dockerContainerId: string | null;
  dockerVolumeCreatedAt: string | null;
  executionId: string | null;
  presence: "present" | "absent" | "unknown";
  observedAt: number | null;
}
export interface ResourcePage {
  items: ResourceView[];
  nextAfter: ResourceCursor | null;
}
function cursor(value: Wire.ResourceCursor): ResourceCursor {
  switch (value.kind) {
    case "POD":
    case "PVC":
    case "BOOTSTRAP_SECRET":
      if (
        value.nativeScopeKind !== "" ||
        value.nativeScope !== "" ||
        !/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/.test(value.namespace) ||
        value.namespace.length > 63 ||
        !/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/.test(value.name) ||
        value.name.length > 253
      )
        return invalidResponse();
      return {
        kind: value.kind,
        namespace: value.namespace,
        name: value.name,
        nativeScopeKind: "",
        nativeScope: "",
      };
    case "DOCKER_CONTAINER":
    case "DOCKER_VOLUME":
      if (
        value.namespace !== "" ||
        value.nativeScopeKind !== "DOCKER_DAEMON" ||
        !value.nativeScope ||
        new TextEncoder().encode(value.nativeScope).length > 1024 ||
        /[\s\p{Cc}]/u.test(value.nativeScope) ||
        !/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(value.name) ||
        value.name.length > 255
      )
        return invalidResponse();
      return {
        kind: value.kind,
        namespace: "",
        name: value.name,
        nativeScopeKind: "DOCKER_DAEMON",
        nativeScope: value.nativeScope,
      };
    default:
      return invalidResponse();
  }
}
export function resources(value: Wire.ResourceInventory): ResourcePage {
  if (value.items.length > 20) return invalidResponse();
  return {
    items: value.items.map((item) => {
      if (!item.identity) return invalidResponse();
      const identity = cursor(item.identity);
      const kubernetes = identity.nativeScopeKind === "";
      if (
        (!kubernetes && item.uid !== undefined) ||
        (identity.kind !== "DOCKER_CONTAINER" &&
          item.dockerContainerId !== undefined) ||
        (identity.kind !== "DOCKER_VOLUME" &&
          item.dockerVolumeCreatedAt !== undefined) ||
        (item.dockerContainerId !== undefined &&
          !/^[0-9a-f]{64}$/.test(item.dockerContainerId)) ||
        (item.dockerVolumeCreatedAt !== undefined &&
          (!item.dockerVolumeCreatedAt ||
            item.dockerVolumeCreatedAt.length > 128 ||
            /[\s\p{Cc}]/u.test(item.dockerVolumeCreatedAt)))
      )
        return invalidResponse();
      const presence =
        item.presence === "PRESENT"
          ? "present"
          : item.presence === "ABSENT"
            ? "absent"
            : item.presence === "UNKNOWN"
              ? "unknown"
              : invalidResponse();
      return {
        identity,
        uid: item.uid ? text(item.uid) : null,
        dockerContainerId: item.dockerContainerId ?? null,
        dockerVolumeCreatedAt: item.dockerVolumeCreatedAt ?? null,
        executionId: item.executionId ? text(item.executionId) : null,
        presence,
        observedAt: instant(item.observedAt),
      };
    }),
    nextAfter: value.nextAfter ? cursor(value.nextAfter) : null,
  };
}
export interface ComputerView extends ComputerStatus {
  requestId: string;
  actions: {
    action: LifecycleAction;
    available: boolean;
    reason: BlockReason | null;
  }[];
  forceRemoveAvailable: boolean;
  startDelayed: boolean;
  preparationRetryAvailable: boolean;
}
export interface AgentView {
  id: string;
  teamId: string;
  name: string;
  handle: string;
  profileImage: Pick<AgentProfileImage, "id" | "assetKey">;
  paletteKey: AgentPaletteKey;
  authentication: AuthenticationSelection | null;
  computer: ComputerView;
}
export interface ProviderChoice {
  id: string;
  name: string;
  kind: "kubernetes" | "docker";
}
export function profileImages(
  value: Wire.AgentProfileImages,
): Page<AgentProfileImage> {
  return page(
    value.items.map((image) => {
      if (!/^[a-z0-9][a-z0-9-]{0,127}(?![\s\S])/.test(image.assetKey))
        return invalidResponse();
      return {
        id: text(image.id),
        internalName: text(image.internalName),
        assetKey: image.assetKey,
      };
    }),
    value.nextAfter,
  );
}
export interface ProviderView extends ProviderChoice {
  kind: "kubernetes" | "docker";
  management: Connection;
  canCreate: boolean;
}
export interface InventoryView extends ComputerStatus {
  id: string;
  residual: boolean;
  cleanupCompletedAt: number | null;
}
export interface Page<T> {
  items: T[];
  nextAfter: string | null;
}

function text(value: string): string {
  return value.length > 0 && value.length <= 256 ? value : invalidResponse();
}
function label<T extends string>(value: number, labels: readonly T[]): T {
  return labels[value - 1] ?? invalidResponse();
}
function instant(value: Timestamp | undefined): number | null {
  if (value === undefined) return null;
  return validTimestamp(value)
    ? timestampMilliseconds(value)
    : invalidResponse();
}
function status(value: Wire.Computer | Wire.InventoryComputer): ComputerStatus {
  return {
    operation: label<ComputerStatus["operation"]>(value.action, [
      "create",
      "start",
      "stop",
      "restart",
      "delete",
    ]),
    intent: label<ComputerStatus["intent"]>(value.desired, [
      "running",
      "stopped",
      "deleting",
    ]),
    power: label<Power>(value.power, ["running", "stopped", "unknown"]),
    lastKnownPower: label<Power>(value.lastKnownPower, [
      "running",
      "stopped",
      "unknown",
    ]),
    phase: label<Phase>(value.phase, [
      "accepted",
      "retiring",
      "starting",
      "deleting",
      "completed",
      "blocked",
      "uncertain",
    ]),
    management: label<Connection>(value.management, [
      "connected",
      "disconnected",
      "unknown",
    ]),
    connector: label<Connection>(value.connector, [
      "connected",
      "disconnected",
      "unknown",
    ]),
    saved: label<ComputerStatus["saved"]>(value.savedEnvironment, [
      "verified",
      "missing",
      "unknown",
    ]),
    observedAt: instant(value.powerObservedAt),
  };
}
export function actionNumber(action: LifecycleAction): Wire.Action {
  switch (action) {
    case "start":
      return 2;
    case "stop":
      return 3;
    case "restart":
      return 4;
    case "delete":
      return 5;
  }
}
export function agent(value: Wire.Agent): AgentView {
  if (
    !value.computer ||
    value.computer.actions.length > 4 ||
    !validHandle(value.handle) ||
    !validAgentPalette(value.paletteKey) ||
    !/^[a-z0-9][a-z0-9-]{0,127}(?![\s\S])/.test(value.profileImageAssetKey)
  )
    return invalidResponse();
  const computer = value.computer;
  return {
    id: text(value.id),
    teamId: text(value.teamId),
    name: text(value.name),
    handle: value.handle,
    profileImage: {
      id: text(value.profileImageId),
      assetKey: value.profileImageAssetKey,
    },
    paletteKey: value.paletteKey,
    authentication: value.authentication
      ? authenticationSelection(value.authentication)
      : null,
    computer: {
      ...status(computer),
      requestId: text(computer.requestId),
      actions: computer.actions.map((item) => ({
        action: label<LifecycleAction>(item.action - 1, [
          "start",
          "stop",
          "restart",
          "delete",
        ]),
        available: item.available,
        reason:
          item.blockedReason === undefined
            ? null
            : label<BlockReason>(item.blockedReason, [
                "unsupported",
                "deleting",
                "conflicting",
                "managementUnavailable",
                "powerUnknown",
                "requiresStopped",
                "requiresRunning",
                "savedUnverified",
              ]),
      })),
      forceRemoveAvailable: computer.forceRemoveAvailable,
      startDelayed: computer.startDelayed,
      preparationRetryAvailable: computer.preparationRetryAvailable,
    },
  };
}
function page<T>(items: T[], next: string | undefined): Page<T> {
  if (items.length > 20) return invalidResponse();
  return { items, nextAfter: next === undefined ? null : text(next) };
}
export function agents(value: Wire.AgentList): Page<AgentView> {
  return page(value.items.map(agent), value.nextAfter);
}
export function choices(value: Wire.ProviderChoices): Page<ProviderChoice> {
  return page(
    value.items.map((item) => ({
      id: text(item.id),
      name: text(item.name),
      kind: label<ProviderChoice["kind"]>(item.kind, ["kubernetes", "docker"]),
    })),
    value.nextAfter,
  );
}
export function providers(value: Wire.ProviderList): Page<ProviderView> {
  return page(
    value.items.map((item) => {
      if (!item.capabilities) return invalidResponse();
      return {
        id: text(item.id),
        name: text(item.name),
        canCreate: item.capabilities.create,
        kind: label<ProviderView["kind"]>(item.kind, ["kubernetes", "docker"]),
        management: label<Connection>(item.management, [
          "connected",
          "disconnected",
          "unknown",
        ]),
      };
    }),
    value.nextAfter,
  );
}
export function inventory(value: Wire.ComputerInventory): Page<InventoryView> {
  return page(
    value.items.map((item) => ({
      ...status(item),
      id: text(item.id),
      residual: item.residual,
      cleanupCompletedAt: instant(item.cleanupCompletedAt),
    })),
    value.nextAfter,
  );
}
export function creation(
  value: Wire.CreateAgentResult,
): { type: "accepted"; agent: AgentView } | { type: "removed" } {
  if (value.agent && !value.removed)
    return { type: "accepted", agent: agent(value.agent) };
  if (value.removed && !value.agent) return { type: "removed" };
  return invalidResponse();
}

export type HistoricalCreation =
  | { type: "notObserved" }
  | { type: "present"; agentId: string; teamId: string }
  | { type: "removed"; agentId: string };
export function historicalCreation(
  value: Wire.HistoricalCreationResult,
): HistoricalCreation {
  if (value.present && value.removed) return invalidResponse();
  if (value.present)
    return {
      type: "present",
      agentId: text(value.present.assignedAgentId),
      teamId: text(value.present.teamId),
    };
  if (value.removed)
    return { type: "removed", agentId: text(value.removed.assignedAgentId) };
  return { type: "notObserved" };
}
