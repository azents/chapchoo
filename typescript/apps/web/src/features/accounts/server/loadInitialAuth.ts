import "server-only";
import { loadInitialRead } from "@chapchoo/web-core/initial-read";
import { call } from "@chapchoo/web-core/rpc";
import { preview } from "@chapchoo/web-core/validation";
import { connection } from "next/server";
import { getConfig } from "@/config";
import { getBackend } from "@/shared/server/backend";
import type { InitialAuth } from "../initialAuth";
import type { AuthInput } from "../types";

export async function loadInitialAuth(
  mode: AuthInput["mode"],
  token: string,
): Promise<InitialAuth> {
  await connection();
  const backend = getBackend(getConfig());
  if (mode !== "LOGIN" && token) {
    return {
      type: "PROOF",
      result: await loadInitialRead(async () =>
        preview(
          await call(
            mode === "SIGNUP"
              ? backend.general.previewSignup.bind(backend.general)
              : backend.general.previewPasswordReset.bind(backend.general),
            { proof: token },
          ),
        ),
      ),
    };
  }
  if (mode === "RESET") {
    return { type: "NO_PROOF" };
  }
  return {
    type: "METHODS",
    result: await loadInitialRead(() =>
      call(backend.auth.methods.bind(backend.auth), {}),
    ),
  };
}
