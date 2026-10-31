import "server-only";
import { RpcFailure } from "@chapchoo/web-core/errors";
import { readIdentity } from "@/shared/server/identity";
import type { InitialIdentity } from "@chapchoo/web-core/auth-context";

export async function loadIdentity(): Promise<InitialIdentity> {
  try {
    return { type: "LOADED", data: await readIdentity() };
  } catch (error) {
    if (error instanceof RpcFailure) {
      return { type: "ERROR", reason: error.reason };
    }
    throw error;
  }
}
