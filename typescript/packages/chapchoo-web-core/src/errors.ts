import { DomainReason } from "@chapchoo/rpc-client/accounts";
import { Metadata, status } from "@grpc/grpc-js";
import type { WireReason } from "./browser.js";

export const reasonMetadataKey = "chapchoo-domain-reason";

export function domainReason(metadata: Metadata): WireReason | null {
  const values = metadata.get(reasonMetadataKey);
  if (values.length !== 1 || typeof values[0] !== "string") {
    return null;
  }
  const value = values[0];
  for (const [name, number] of Object.entries(DomainReason)) {
    if (
      name === value &&
      name !== "UNRECOGNIZED" &&
      name !== "DOMAIN_REASON_UNSPECIFIED" &&
      typeof number === "number" &&
      number > DomainReason.DOMAIN_REASON_UNSPECIFIED
    ) {
      return recognizedReason(name);
    }
  }
  return null;
}

function recognizedReason(value: string): WireReason | null {
  // Membership is narrowed without an unchecked assertion or accepting numeric
  // enum reverse-map keys. The generated source remains the allowed inventory.
  switch (value) {
    case "DOMAIN_REASON_INVALID_INPUT":
    case "DOMAIN_REASON_INVALID_CREDENTIALS":
    case "DOMAIN_REASON_INVALID_SESSION":
    case "DOMAIN_REASON_PERMISSION_DENIED":
    case "DOMAIN_REASON_NOT_FOUND":
    case "DOMAIN_REASON_CONFLICT":
    case "DOMAIN_REASON_INVALID_TRANSITION":
    case "DOMAIN_REASON_INVALID_PROOF":
    case "DOMAIN_REASON_EMAIL_MISMATCH":
    case "DOMAIN_REASON_EMAIL_REGISTERED":
    case "DOMAIN_REASON_WEAK_PASSWORD":
    case "DOMAIN_REASON_INVALID_CODE":
    case "DOMAIN_REASON_REGISTRATION_REQUIRED":
    case "DOMAIN_REASON_ALREADY_MEMBER":
    case "DOMAIN_REASON_ALREADY_PROCESSED":
    case "DOMAIN_REASON_OWNER_TRANSFER_REQUIRED":
    case "DOMAIN_REASON_HANDLE_CONFLICT":
    case "DOMAIN_REASON_MAIL_UNAVAILABLE":
    case "DOMAIN_REASON_CAPACITY_EXHAUSTED":
    case "DOMAIN_REASON_INTERNAL":
      return value;
    default:
      return null;
  }
}

/** This wrapper preserves the native cause internally, with sanitized public text. */
export class RpcFailure extends Error {
  constructor(
    readonly code: status,
    readonly reason: WireReason | null,
    cause?: unknown,
  ) {
    super("The server operation could not be completed", { cause });
    this.name = "RpcFailure";
  }
}

/** Source retention that framework-default Error inspection cannot traverse. */
export class SafeRenderFailure extends RpcFailure {
  #source: RpcFailure;

  constructor(source: RpcFailure) {
    super(source.code, source.reason);
    this.name = "SafeRenderFailure";
    this.#source = source;
  }

  /** Explicit diagnostics boundary only; never return it as page data. */
  sourceFailure(): RpcFailure {
    return this.#source;
  }
}

export function reportRpcFailure(
  failure: RpcFailure,
  event:
    | "web_rpc_transport_failure"
    | "web_ssr_authentication_failure" = "web_rpc_transport_failure",
): void {
  // Known backend reasons already have their Rust reporting boundary. The
  // wrapper's own stack is safe; native cause messages/stacks stay internal.
  if (failure.reason == null) {
    console.error(
      JSON.stringify({
        event,
        code: failure.code,
        reason: failure.reason,
        stack: failure.stack,
      }),
    );
  }
}

export function rpcFailure(source: unknown): RpcFailure {
  if (source instanceof RpcFailure) {
    return source;
  }
  if (
    source instanceof Error &&
    "code" in source &&
    typeof source.code === "number" &&
    "metadata" in source &&
    source.metadata instanceof Metadata
  ) {
    const code = Object.values(status).find(
      (value) => typeof value === "number" && value === source.code,
    );
    if (typeof code === "number") {
      return new RpcFailure(code, domainReason(source.metadata), source);
    }
  }
  return new RpcFailure(status.INTERNAL, null, source);
}

export function invalidSession(): RpcFailure {
  return new RpcFailure(
    status.UNAUTHENTICATED,
    "DOMAIN_REASON_INVALID_SESSION",
  );
}
export function permissionDenied(): RpcFailure {
  return new RpcFailure(
    status.PERMISSION_DENIED,
    "DOMAIN_REASON_PERMISSION_DENIED",
  );
}
export function invalidResponse(): never {
  throw new RpcFailure(status.INTERNAL, null);
}
