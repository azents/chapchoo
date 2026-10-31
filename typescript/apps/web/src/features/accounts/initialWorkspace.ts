import type { Member, Team } from "@chapchoo/web-core/browser";
export type InitialWorkspace =
  | { type: "HOME"; contexts: { items: Team[] } }
  | { type: "TEAM"; team: Team; members: { items: Member[] } };
