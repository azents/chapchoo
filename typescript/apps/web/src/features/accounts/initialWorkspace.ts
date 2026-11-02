import type { Invitation, Member, Team } from "@chapchoo/web-core/browser";
export type InitialWorkspace =
  | { type: "INVITATIONS"; received: { items: Invitation[] } }
  | { type: "TEAM"; team: Team; members: { items: Member[] } };
