import { rpcFailure } from "@chapchoo/web-core/errors";
import { preview } from "@chapchoo/web-core/validation";
import type { ProofPreview } from "@chapchoo/web-core/browser";

/** Invalid bearer syntax is an unusable link, not an infrastructure failure. */
export async function readProofPreview(
  read: () => Promise<ProofPreview>,
): Promise<ProofPreview> {
  try {
    return preview(await read());
  } catch (error) {
    if (rpcFailure(error).reason === "DOMAIN_REASON_INVALID_PROOF") {
      return { usable: false, expiresAt: void 0 };
    }
    throw error;
  }
}
