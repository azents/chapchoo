import { RpcFailure } from "@chapchoo/web-core/errors";

export async function readConnectionPage<T>(
  operation: () => Promise<T>,
): Promise<T | null> {
  try {
    return await operation();
  } catch (error) {
    if (
      error instanceof RpcFailure &&
      (error.reason === "DOMAIN_REASON_INVALID_INPUT" ||
        error.reason === "DOMAIN_REASON_NOT_FOUND" ||
        error.reason === "DOMAIN_REASON_PERMISSION_DENIED")
    ) {
      return null;
    }
    throw error;
  }
}
