import { reportRpcFailure, rpcFailure } from "./errors.js";
import type { InitialRead } from "./browser.js";

/** Resolve public initial state without a writable session or browser retry. */
export async function loadInitialRead<T>(
  read: () => Promise<T>,
): Promise<InitialRead<T>> {
  try {
    return { type: "LOADED", data: await read() };
  } catch (error) {
    const failure = rpcFailure(error);
    reportRpcFailure(failure);
    return { type: "ERROR", reason: failure.reason };
  }
}
