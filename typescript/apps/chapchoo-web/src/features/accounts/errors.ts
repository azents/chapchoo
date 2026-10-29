import type { Copy } from "./types";

export function accountError(reason: string | null, c: Copy): string {
  switch (reason) {
    case "DOMAIN_REASON_INVALID_INPUT":
      return c.invalidInput;
    case "DOMAIN_REASON_INVALID_CREDENTIALS":
      return c.invalidCredentials;
    case "DOMAIN_REASON_INVALID_SESSION":
      return c.invalidSession;
    case "DOMAIN_REASON_PERMISSION_DENIED":
      return c.permissionDenied;
    case "DOMAIN_REASON_NOT_FOUND":
    case "DOMAIN_REASON_CONFLICT":
    case "DOMAIN_REASON_INVALID_TRANSITION":
    case "DOMAIN_REASON_HANDLE_CONFLICT":
    case "DOMAIN_REASON_ALREADY_MEMBER":
    case "DOMAIN_REASON_ALREADY_PROCESSED":
      return c.conflict;
    case "DOMAIN_REASON_INVALID_PROOF":
      return c.invalidProof;
    case "DOMAIN_REASON_EMAIL_MISMATCH":
      return c.emailMismatch;
    case "DOMAIN_REASON_EMAIL_REGISTERED":
      return c.registered;
    case "DOMAIN_REASON_WEAK_PASSWORD":
      return c.weakPassword;
    case "DOMAIN_REASON_INVALID_CODE":
      return c.invalidCode;
    case "DOMAIN_REASON_REGISTRATION_REQUIRED":
      return c.registrationRequired;
    case "DOMAIN_REASON_OWNER_TRANSFER_REQUIRED":
      return c.ownerLeaveDescription;
    case "DOMAIN_REASON_MAIL_UNAVAILABLE":
      return c.mailUncertain;
    default:
      return c.genericError;
  }
}
