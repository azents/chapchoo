import type * as Wire from "@chapchoo/rpc-client/authentication";
import { Kind, Operation } from "@chapchoo/rpc-client/authentication";
import type { Timestamp } from "@chapchoo/rpc-client/timestamp";
import { invalidResponse } from "./errors.js";
import { timestampMilliseconds } from "./validation.js";

export type AuthenticationScope =
  { type: "personal" } | { type: "team"; teamId: string };
export type AuthenticationKind = "chatgpt" | "openaiApiKey";
export interface AuthenticationView {
  id: string;
  scope: AuthenticationScope;
  kind: AuthenticationKind;
  name: string;
  accountId: string | null;
  email: string | null;
  planType: string | null;
  expiresAt: number | null;
  currentRequestId: string;
  createdAt: number;
  updatedAt: number;
}
export interface AuthenticationSelection {
  authenticationId: string;
  name: string;
  kind: AuthenticationKind;
  requestId: string;
  selectedAt: number;
}
export type AuthenticationAccepted =
  | { type: "record"; authenticationId: string }
  | { type: "deleted"; authenticationId: string }
  | { type: "selection"; agentId: string; authenticationId: string }
  | { type: "device"; attemptId: string };
export type AuthenticationOperation =
  | "registerKey"
  | "replaceKey"
  | "rename"
  | "delete"
  | "select"
  | "startDevice"
  | "cancelDevice";
export type AuthenticationHistory =
  | { type: "notObserved" }
  | {
      type: "accepted";
      operation: AuthenticationOperation;
      result: AuthenticationAccepted;
      acceptedAt: number;
    };
export type DeviceState =
  | {
      type:
        | "starting"
        | "checking"
        | "exchanging"
        | "cancelled"
        | "expired"
        | "failed"
        | "superseded"
        | "uncertain";
    }
  | {
      type: "pending";
      userCode: string;
      verificationUri: string;
      nextCheckAt: number;
    }
  | { type: "registered"; authenticationId: string };
export interface DeviceProgress {
  attemptId: string;
  expiresAt: number;
  observedAt: number;
  state: DeviceState;
  providerFailure:
    "rejected" | "unavailable" | "invalidResponse" | "uncertain" | null;
}
function text(value: string, max = 256): string {
  return value.length > 0 && value.length <= max && !/[\p{Cc}]/u.test(value)
    ? value
    : invalidResponse();
}
function instant(value: Timestamp | undefined): number {
  return value === undefined ? invalidResponse() : timestampMilliseconds(value);
}
function optional(value: string | undefined): string | null {
  if (value === undefined) return null;
  if (new TextEncoder().encode(value).length > 4096) return invalidResponse();
  return text(value, 4096);
}
function alias(value: string): string {
  if (value !== "" && value.trim() === "") return invalidResponse();
  if ([...value].length > 256 || new TextEncoder().encode(value).length > 1024)
    return invalidResponse();
  return value === "" ? "" : text(value, 1024);
}
export function authenticationKind(value: Wire.Kind): AuthenticationKind {
  switch (value) {
    case Kind.KIND_CHATGPT:
      return "chatgpt";
    case Kind.KIND_OPENAI_API_KEY:
      return "openaiApiKey";
    default:
      return invalidResponse();
  }
}
export function authenticationScope(
  value: Wire.Scope | undefined,
): AuthenticationScope {
  if (value?.personal !== undefined && value.teamId === undefined)
    return { type: "personal" };
  if (value?.teamId !== undefined && value.personal === undefined)
    return { type: "team", teamId: text(value.teamId) };
  return invalidResponse();
}
export function wireAuthenticationScope(
  value: AuthenticationScope,
): Wire.Scope {
  return value.type === "personal"
    ? { personal: {} }
    : { teamId: value.teamId };
}
export function authentication(value: Wire.Authentication): AuthenticationView {
  const kind = authenticationKind(value.kind);
  if (
    kind === "openaiApiKey" &&
    (value.accountId !== undefined ||
      value.email !== undefined ||
      value.planType !== undefined ||
      value.expiresAt !== undefined)
  )
    return invalidResponse();
  return {
    id: text(value.id),
    scope: authenticationScope(value.scope),
    kind,
    name: alias(value.name),
    accountId: optional(value.accountId),
    email: optional(value.email),
    planType: optional(value.planType),
    expiresAt: value.expiresAt === undefined ? null : instant(value.expiresAt),
    currentRequestId: text(value.currentRequestId),
    createdAt: instant(value.createdAt),
    updatedAt: instant(value.updatedAt),
  };
}
export function authenticationList(value: Wire.AuthenticationList): {
  items: AuthenticationView[];
  nextAfter: string | null;
} {
  if (value.items.length > 20) return invalidResponse();
  return {
    items: value.items.map(authentication),
    nextAfter: value.nextAfter === undefined ? null : text(value.nextAfter),
  };
}
export function authenticationSelection(
  value: Wire.AgentSelection,
): AuthenticationSelection {
  return {
    authenticationId: text(value.authenticationId),
    name: alias(value.name),
    kind: authenticationKind(value.kind),
    requestId: text(value.requestId),
    selectedAt: instant(value.selectedAt),
  };
}
export function authenticationAccepted(
  value: Wire.AcceptedResult,
): AuthenticationAccepted {
  if (
    [value.record, value.deleted, value.selection, value.device].filter(
      (entry) => entry !== undefined,
    ).length !== 1
  )
    return invalidResponse();
  if (value.record)
    return {
      type: "record",
      authenticationId: text(value.record.authenticationId),
    };
  if (value.deleted)
    return {
      type: "deleted",
      authenticationId: text(value.deleted.authenticationId),
    };
  if (value.selection)
    return {
      type: "selection",
      agentId: text(value.selection.agentId),
      authenticationId: text(value.selection.authenticationId),
    };
  if (value.device)
    return { type: "device", attemptId: text(value.device.attemptId) };
  return invalidResponse();
}
export function authenticationRecordResult(
  value: Wire.AcceptedResult,
): Extract<AuthenticationAccepted, { type: "record" }> {
  const result = authenticationAccepted(value);
  return result.type === "record" ? result : invalidResponse();
}
export function authenticationSelectionResult(
  value: Wire.AcceptedResult,
): Extract<AuthenticationAccepted, { type: "selection" }> {
  const result = authenticationAccepted(value);
  return result.type === "selection" ? result : invalidResponse();
}
export function authenticationDeletion(
  value: Wire.DeleteResult,
): { type: "inUse" } | { type: "accepted"; result: AuthenticationAccepted } {
  if (value.inUse !== undefined && value.accepted === undefined)
    return { type: "inUse" };
  if (value.accepted !== undefined && value.inUse === undefined) {
    const result = authenticationAccepted(value.accepted);
    if (result.type !== "deleted") return invalidResponse();
    return { type: "accepted", result };
  }
  return invalidResponse();
}
export function authenticationHistory(
  value: Wire.HistoricalResult,
): AuthenticationHistory {
  if (
    value.accepted === undefined &&
    value.acceptedAt === undefined &&
    value.operation === Operation.OPERATION_UNSPECIFIED
  )
    return { type: "notObserved" };
  if (value.accepted === undefined) return invalidResponse();
  const operations: readonly AuthenticationOperation[] = [
    "registerKey",
    "replaceKey",
    "rename",
    "delete",
    "select",
    "startDevice",
    "cancelDevice",
  ];
  const operation = operations[value.operation - 1] ?? invalidResponse();
  const result = authenticationAccepted(value.accepted);
  const expected =
    operation === "delete"
      ? "deleted"
      : operation === "select"
        ? "selection"
        : operation === "startDevice" || operation === "cancelDevice"
          ? "device"
          : "record";
  if (result.type !== expected) return invalidResponse();
  return {
    type: "accepted",
    operation,
    result,
    acceptedAt: instant(value.acceptedAt),
  };
}
export function deviceProgress(
  value: Wire.DeviceProgress,
  observedAt: number,
): DeviceProgress {
  const states = [
    value.starting,
    value.pending,
    value.checking,
    value.exchanging,
    value.registered,
    value.cancelled,
    value.expired,
    value.failed,
    value.superseded,
    value.uncertain,
  ];
  if (states.filter((entry) => entry !== undefined).length !== 1)
    return invalidResponse();
  let state: DeviceState;
  if (value.pending) {
    // Never render an arbitrary provider URL as a credential-entry destination.
    if (
      value.pending.verificationUri !== "https://auth.openai.com/codex/device"
    )
      return invalidResponse();
    state = {
      type: "pending",
      userCode: text(value.pending.userCode, 128),
      verificationUri: value.pending.verificationUri,
      nextCheckAt: instant(value.pending.nextCheckAt),
    };
  } else if (value.registered)
    state = {
      type: "registered",
      authenticationId: text(value.registered.authenticationId),
    };
  else if (value.starting !== undefined) state = { type: "starting" };
  else if (value.checking !== undefined) state = { type: "checking" };
  else if (value.exchanging !== undefined) state = { type: "exchanging" };
  else if (value.cancelled !== undefined) state = { type: "cancelled" };
  else if (value.expired !== undefined) state = { type: "expired" };
  else if (value.failed !== undefined) state = { type: "failed" };
  else if (value.superseded !== undefined) state = { type: "superseded" };
  else state = { type: "uncertain" };
  const failures: readonly NonNullable<DeviceProgress["providerFailure"]>[] = [
    "rejected",
    "unavailable",
    "invalidResponse",
    "uncertain",
  ];
  return {
    attemptId: text(value.attemptId),
    expiresAt: instant(value.expiresAt),
    observedAt,
    state,
    providerFailure:
      value.providerFailure === undefined
        ? null
        : (failures[value.providerFailure - 1] ?? invalidResponse()),
  };
}
