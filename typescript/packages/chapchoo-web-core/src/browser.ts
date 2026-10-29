// Type-only exports keep grpc-js and Node configuration out of browser bundles.
import type {
  DomainReason,
  TeamKind,
  TeamRole,
  InvitationStatus,
  InvitationAction,
  MembershipRole,
  GrantKind,
} from "@chapchoo/rpc-client/accounts";
export type {
  User,
  Team,
  Member,
  Invitation,
  Grant,
  IssuedGrant,
  ProofPreview,
  LoginMethods,
  CodeChallenge,
} from "@chapchoo/rpc-client/accounts";
export type WireReason = Exclude<
  keyof typeof DomainReason,
  "UNRECOGNIZED" | "DOMAIN_REASON_UNSPECIFIED"
>;

// Numeric values are checked against the generated enum member types, while
// emitted browser JavaScript imports no grpc-js module or server configuration.
export const teamKind: {
  personal: TeamKind.TEAM_KIND_PERSONAL;
  collaborative: TeamKind.TEAM_KIND_COLLABORATIVE;
} = { personal: 1, collaborative: 2 };
export const teamRole: {
  owner: TeamRole.TEAM_ROLE_OWNER;
  manager: TeamRole.TEAM_ROLE_MANAGER;
  member: TeamRole.TEAM_ROLE_MEMBER;
} = { owner: 1, manager: 2, member: 3 };
export const invitationStatus: {
  pending: InvitationStatus.INVITATION_STATUS_PENDING;
  accepted: InvitationStatus.INVITATION_STATUS_ACCEPTED;
  declined: InvitationStatus.INVITATION_STATUS_DECLINED;
} = { pending: 1, accepted: 2, declined: 3 };
export const invitationAction: {
  accept: InvitationAction.INVITATION_ACTION_ACCEPT;
  decline: InvitationAction.INVITATION_ACTION_DECLINE;
} = { accept: 1, decline: 2 };
export const membershipRole: {
  manager: MembershipRole.MEMBERSHIP_ROLE_MANAGER;
  member: MembershipRole.MEMBERSHIP_ROLE_MEMBER;
} = { manager: 1, member: 2 };
export const grantKind: {
  signup: GrantKind.GRANT_KIND_SIGNUP;
  passwordReset: GrantKind.GRANT_KIND_PASSWORD_RESET;
} = { signup: 1, passwordReset: 2 };
