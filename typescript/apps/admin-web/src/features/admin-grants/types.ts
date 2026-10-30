import type { WireReason } from "@chapchoo/web-core/browser";

export type InitialAdminIdentity =
  | { type: "LOADED"; email: string }
  | { type: "ERROR"; reason: WireReason | null };
