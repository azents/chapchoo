import type en from "../../../messages/en-US.json";

export type ErrorKey = keyof typeof en.Errors;

export function errorKey(reason: string | null): ErrorKey {
  switch (reason) {
    case "DOMAIN_REASON_INVALID_INPUT":
      return "invalidInput";
    case "DOMAIN_REASON_INVALID_CREDENTIALS":
      return "invalidCredentials";
    case "DOMAIN_REASON_INVALID_SESSION":
      return "invalidSession";
    case "DOMAIN_REASON_PERMISSION_DENIED":
      return "permissionDenied";
    case "DOMAIN_REASON_NOT_FOUND":
      return "notFound";
    case "DOMAIN_REASON_CONFLICT":
      return "conflict";
    case "DOMAIN_REASON_INVALID_TRANSITION":
      return "invalidTransition";
    case "DOMAIN_REASON_INVALID_PROOF":
      return "invalidProof";
    case "DOMAIN_REASON_EMAIL_MISMATCH":
      return "emailMismatch";
    case "DOMAIN_REASON_EMAIL_REGISTERED":
      return "emailRegistered";
    case "DOMAIN_REASON_WEAK_PASSWORD":
      return "weakPassword";
    case "DOMAIN_REASON_INVALID_CODE":
      return "invalidCode";
    case "DOMAIN_REASON_REGISTRATION_REQUIRED":
      return "registrationRequired";
    case "DOMAIN_REASON_ALREADY_MEMBER":
      return "alreadyMember";
    case "DOMAIN_REASON_ALREADY_PROCESSED":
      return "alreadyProcessed";
    case "DOMAIN_REASON_OWNER_TRANSFER_REQUIRED":
      return "ownerTransferRequired";
    case "DOMAIN_REASON_HANDLE_CONFLICT":
      return "handleConflict";
    case "DOMAIN_REASON_MAIL_UNAVAILABLE":
      return "mailUnavailable";
    case "DOMAIN_REASON_CAPACITY_EXHAUSTED":
      return "capacityExhausted";
    default:
      return "generic";
  }
}
