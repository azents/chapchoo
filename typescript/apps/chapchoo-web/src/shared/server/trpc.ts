import "server-only";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { initTRPC } from "@trpc/server";
import superjson from "superjson";
import type { RpcContext } from "@/shared/server/context";

const t = initTRPC.context<RpcContext>().create({
  transformer: superjson,
  errorFormatter({ shape, error }) {
    const reason =
      error.cause instanceof RpcFailure
        ? error.cause.reason
        : error.code === "BAD_REQUEST"
          ? "DOMAIN_REASON_INVALID_INPUT"
          : null;
    return {
      ...shape,
      message: "The request could not be completed",
      data: {
        code: shape.data.code,
        httpStatus: shape.data.httpStatus,
        path: shape.data.path,
        reason,
      },
    };
  },
});
export const router = t.router;
export const procedure = t.procedure;
