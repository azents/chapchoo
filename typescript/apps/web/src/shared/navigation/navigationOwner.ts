import type { InitialNavigation } from "./types";

/** Private supplementary content belongs to the server-admitted account. */
export function navigationOwnerMatches(
  initial: InitialNavigation,
  currentUserId: string | null,
): boolean {
  return (
    currentUserId !== null &&
    initial.type !== "guest" &&
    initial.userId === currentUserId
  );
}
