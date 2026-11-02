import type * as Wire from "@chapchoo/rpc-client/computers";
import type { Timestamp } from "@chapchoo/rpc-client/timestamp";
import { invalidResponse } from "./errors.js";
import { timestampMilliseconds, validTimestamp } from "./validation.js";

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
  intent: "running" | "stopped" | "deleting";
  power: Power;
  lastKnownPower: Power;
  phase: Phase;
  management: Connection;
  connector: Connection;
  saved: "verified" | "missing" | "unknown";
  observedAt: number | null;
}
export interface ResourceCursor {
  kind: string;
  namespace: string;
  name: string;
}
export interface ResourceView {
  identity: ResourceCursor;
  uid: string | null;
  executionId: string | null;
  presence: "present" | "absent" | "unknown";
  observedAt: number | null;
}
export interface ResourcePage {
  items: ResourceView[];
  nextAfter: ResourceCursor | null;
}
function cursor(value: Wire.ResourceCursor): ResourceCursor {
  if (!["POD", "BOOTSTRAP_SECRET", "PVC"].includes(value.kind))
    return invalidResponse();
  return {
    kind: value.kind,
    namespace: text(value.namespace),
    name: text(value.name),
  };
}
export function resources(value: Wire.ResourceInventory): ResourcePage {
  if (value.items.length > 20) return invalidResponse();
  return {
    items: value.items.map((item) => {
      if (!item.identity) return invalidResponse();
      const presence =
        item.presence === "PRESENT"
          ? "present"
          : item.presence === "ABSENT"
            ? "absent"
            : item.presence === "UNKNOWN"
              ? "unknown"
              : invalidResponse();
      return {
        identity: cursor(item.identity),
        uid: item.uid ? text(item.uid) : null,
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
  name: string;
  computer: ComputerView;
}
export interface ProviderChoice {
  id: string;
  name: string;
}
export interface ProviderView extends ProviderChoice {
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
  if (!value.computer || value.computer.actions.length > 4)
    return invalidResponse();
  const computer = value.computer;
  return {
    id: text(value.id),
    name: text(value.name),
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
    value.items.map((item) => ({ id: text(item.id), name: text(item.name) })),
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
