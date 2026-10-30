import { status } from "@grpc/grpc-js";
import { TRPCError } from "@trpc/server";
import type { WebConfig } from "./config.js";
import { RpcFailure, reportRpcFailure, rpcFailure } from "./errors.js";
import type { RequestSession } from "./session.js";

export function writableRejection(
  request: Request,
  config: WebConfig,
): Response | null {
  if (request.method !== "POST") {
    return Response.json(
      { error: "Method not allowed" },
      { status: 405, headers: { Allow: "POST", "Cache-Control": "no-store" } },
    );
  }
  if (request.headers.get("origin") !== config.origin) {
    return Response.json(
      { error: "Origin is not allowed" },
      { status: 403, headers: { "Cache-Control": "no-store" } },
    );
  }
  // Cookie headers are finalized after ordinary request/response execution.
  // Streaming responses would commit headers before asynchronous auth finishes.
  const accept = request.headers.get("trpc-accept");
  if (accept != null && accept !== "application/json") {
    return Response.json(
      { error: "Response format is not supported" },
      { status: 406, headers: { "Cache-Control": "no-store" } },
    );
  }
  // Authentication creation and protected work own distinct cookie outcomes.
  // Keep one public procedure per HTTP request instead of combining identities.
  if (new URL(request.url).searchParams.has("batch")) {
    return Response.json(
      { error: "Batched requests are not supported" },
      { status: 400, headers: { "Cache-Control": "no-store" } },
    );
  }
  return null;
}

export function privateResponse(
  response: Response,
  session: RequestSession,
): Response {
  response.headers.set("Cache-Control", "private, no-store");
  response.headers.set("Pragma", "no-cache");
  response.headers.append("Vary", "Cookie");
  const cookie = session.outgoingCookie();
  if (cookie != null) {
    response.headers.append("Set-Cookie", cookie);
  }
  return response;
}

export async function attempt<Output>(
  operation: () => Promise<Output>,
): Promise<Output> {
  try {
    return await operation();
  } catch (error) {
    const failure = rpcFailure(error);
    let code: TRPCError["code"];
    switch (failure.code) {
      case status.UNAUTHENTICATED:
        code = "UNAUTHORIZED";
        break;
      case status.PERMISSION_DENIED:
        code = "FORBIDDEN";
        break;
      case status.NOT_FOUND:
        code = "NOT_FOUND";
        break;
      case status.INVALID_ARGUMENT:
        code = "BAD_REQUEST";
        break;
      case status.ALREADY_EXISTS:
      case status.ABORTED:
        code = "CONFLICT";
        break;
      case status.FAILED_PRECONDITION:
        code = "PRECONDITION_FAILED";
        break;
      case status.RESOURCE_EXHAUSTED:
        code = "TOO_MANY_REQUESTS";
        break;
      case status.DEADLINE_EXCEEDED:
        code = "TIMEOUT";
        break;
      case status.UNAVAILABLE:
        code = "SERVICE_UNAVAILABLE";
        break;
      default:
        code = "INTERNAL_SERVER_ERROR";
    }
    throw new TRPCError({
      code,
      message: "The server operation could not be completed",
      cause: failure,
    });
  }
}

/** Called once by the owning fetch adapter's onError handling boundary. */
export function reportTransportFailure(error: TRPCError): void {
  const cause = error.cause;
  if (cause instanceof RpcFailure) {
    reportRpcFailure(cause);
  } else if (error.code === "INTERNAL_SERVER_ERROR") {
    console.error(
      JSON.stringify({
        event: "web_request_processing_failure",
        code: error.code,
        stack: new Error("Web request processing failed").stack,
      }),
    );
  }
}
