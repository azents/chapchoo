import type {
  AuthSession,
  CodeChallenge,
  Grant,
  Grants,
  Invitation,
  Invitations,
  Member,
  Members,
  ProofPreview,
  Team,
  Teams,
  User,
} from "@chapchoo/rpc-client/accounts";
import type { Timestamp } from "@chapchoo/rpc-client/timestamp";
import { invalidResponse } from "./errors.js";

function required(value: string): boolean {
  return value.length > 0;
}
export function validTimestamp(
  value: Timestamp | null | undefined,
): value is Timestamp {
  return (
    value != null &&
    typeof value.seconds === "bigint" &&
    value.seconds >= -62135596800n &&
    value.seconds <= 253402300799n &&
    Number.isInteger(value.nanos) &&
    value.nanos >= 0 &&
    value.nanos < 1_000_000_000
  );
}
export function timestampMilliseconds(value: Timestamp): number {
  if (!validTimestamp(value)) {
    return invalidResponse();
  }
  return Number(
    value.seconds * 1000n + BigInt(Math.floor(value.nanos / 1_000_000)),
  );
}
export function user(value: User): User {
  if (
    !required(value.id) ||
    !required(value.email) ||
    !validTimestamp(value.createdAt)
  ) {
    return invalidResponse();
  }
  return value;
}
export function session(
  value: AuthSession,
): AuthSession & { user: User; expiresAt: Timestamp; createdAt: Timestamp } {
  if (
    value.user == null ||
    !/^[A-Za-z0-9_-]{43}$/.test(value.token) ||
    !validTimestamp(value.expiresAt) ||
    !validTimestamp(value.createdAt) ||
    (value.absoluteExpiresAt != null &&
      !validTimestamp(value.absoluteExpiresAt))
  ) {
    return invalidResponse();
  }
  user(value.user);
  return {
    ...value,
    user: value.user,
    expiresAt: value.expiresAt,
    createdAt: value.createdAt,
  };
}
export function team(value: Team): Team {
  if (
    !required(value.id) ||
    !required(value.name) ||
    !required(value.handle) ||
    !required(value.ownerUserId) ||
    ![1, 2].includes(value.kind) ||
    ![1, 2, 3].includes(value.callerRole)
  ) {
    return invalidResponse();
  }
  return value;
}
export function teams(value: Teams): Teams {
  value.items.forEach(team);
  return value;
}
export function member(value: Member): Member {
  if (
    !required(value.userId) ||
    !required(value.email) ||
    ![1, 2, 3].includes(value.role)
  ) {
    return invalidResponse();
  }
  return value;
}
export function members(value: Members): Members {
  value.items.forEach(member);
  return value;
}
export function invitation(value: Invitation): Invitation {
  if (
    !required(value.id) ||
    !required(value.teamId) ||
    !required(value.teamName) ||
    !required(value.teamHandle) ||
    !required(value.recipientEmail) ||
    ![1, 2, 3].includes(value.status) ||
    !validTimestamp(value.createdAt) ||
    (value.processedAt != null && !validTimestamp(value.processedAt))
  ) {
    return invalidResponse();
  }
  return value;
}
export function invitations(value: Invitations): Invitations {
  value.items.forEach(invitation);
  return value;
}
export function grant(value: Grant): Grant {
  if (
    !required(value.id) ||
    !required(value.email) ||
    ![1, 2].includes(value.kind) ||
    !validTimestamp(value.issuedAt) ||
    !validTimestamp(value.expiresAt) ||
    (value.consumedAt != null && !validTimestamp(value.consumedAt)) ||
    (value.revokedAt != null && !validTimestamp(value.revokedAt))
  ) {
    return invalidResponse();
  }
  return value;
}
export function grants(value: Grants): Grants {
  if (value.total < 0n) {
    return invalidResponse();
  }
  value.items.forEach(grant);
  return value;
}
export function preview(value: ProofPreview): ProofPreview {
  if (
    value.usable
      ? !value.emailHint ||
        !/^[^@\r\n]\*{3}@[^@\r\n]+$/u.test(value.emailHint) ||
        !validTimestamp(value.expiresAt)
      : value.emailHint != null || value.expiresAt != null
  ) {
    return invalidResponse();
  }
  return value;
}
export function challenge(value: CodeChallenge): CodeChallenge {
  if (
    !/^[A-Za-z0-9_-]{43}$/.test(value.binding) ||
    !validTimestamp(value.expiresAt)
  ) {
    return invalidResponse();
  }
  return value;
}
