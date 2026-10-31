/** URL history can switch methods without running the selection click handler. */
export function activeLoginChallenge<T>(
  method: "password" | "code",
  challenge: T | null,
): T | null {
  return method === "code" ? challenge : null;
}
