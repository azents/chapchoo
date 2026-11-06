import type { AgentPaletteKey } from "@chapchoo/web-core/computer";

export interface CreationDraft {
  teamId: string;
  providerId: string;
  authenticationId: string;
  name: string;
  handle: string;
  profileImageId: string;
  paletteKey: AgentPaletteKey;
}
export type CreationPayload = Readonly<CreationDraft & { requestId: string }>;

/** Own one exact payload until definitive rejection or removed-result recovery. */
export class CreationRequest {
  private payload: CreationPayload | null = null;

  get locked(): boolean {
    return this.payload !== null;
  }

  capture(draft: CreationDraft, requestId: () => string): CreationPayload {
    this.payload ??= Object.freeze({ ...draft, requestId: requestId() });
    return this.payload;
  }

  reject(reason?: string | null): void {
    if (
      reason === "DOMAIN_REASON_INVALID_INPUT" ||
      reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
      reason === "DOMAIN_REASON_PERMISSION_DENIED" ||
      reason === "DOMAIN_REASON_NOT_FOUND" ||
      reason === "DOMAIN_REASON_HANDLE_CONFLICT"
    ) {
      this.payload = null;
    }
  }

  removed(): void {
    this.payload = null;
  }
}
