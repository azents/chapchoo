import type { User, WireReason } from "@chapchoo/web-core/browser";

export type InitialAdminIdentity =
  { type: "LOADED"; user: User } | { type: "ERROR"; reason: WireReason | null };
