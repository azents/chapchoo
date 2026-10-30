import "server-only";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { redirect } from "next/navigation";
import { getLoginRedirect } from "@/config/index";
import { readIdentity } from "@/shared/server/identity";
import type { InitialAdminIdentity } from "../types";

/** Preserve a failed read-only precondition as an error, not a signed-out user. */
export async function loadAdminIdentity(): Promise<InitialAdminIdentity> {
  try {
    const identity = await readIdentity();
    if (!identity) {
      redirect(getLoginRedirect());
    }
    return { type: "LOADED", email: identity.email };
  } catch (error) {
    if (error instanceof RpcFailure) {
      return { type: "ERROR", reason: error.reason };
    }
    throw error;
  }
}
