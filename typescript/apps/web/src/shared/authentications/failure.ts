function failureReason(error: unknown): unknown {
  if (
    !(error instanceof Error) ||
    !("data" in error) ||
    !error.data ||
    typeof error.data !== "object"
  ) {
    return null;
  }
  return Reflect.get(error.data, "reason");
}
export function authenticationRecordUnavailable(error: unknown): boolean {
  return failureReason(error) === "DOMAIN_REASON_NOT_FOUND";
}
export function authenticationTargetChanged(error: unknown): boolean {
  const reason = failureReason(error);
  return (
    reason === "DOMAIN_REASON_CONFLICT" ||
    reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
    reason === "DOMAIN_REASON_NOT_FOUND"
  );
}
/** Explicit rejection of a freshly issued mutation permits another deliberate action. */
export function definitiveAuthenticationRejection(error: unknown): boolean {
  const reason = failureReason(error);
  return (
    reason === "DOMAIN_REASON_INVALID_INPUT" ||
    reason === "DOMAIN_REASON_PERMISSION_DENIED" ||
    reason === "DOMAIN_REASON_NOT_FOUND" ||
    reason === "DOMAIN_REASON_CONFLICT" ||
    reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
    reason === "DOMAIN_REASON_CAPACITY_EXHAUSTED"
  );
}
/** Imperative secret mutations bypass mutation caches, not identity withdrawal. */
export function withdrawAuthenticationSession(
  error: unknown,
  clearIdentity: () => void,
): boolean {
  if (failureReason(error) !== "DOMAIN_REASON_INVALID_SESSION") {
    return false;
  }
  clearIdentity();
  return true;
}
