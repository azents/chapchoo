export interface RevokeSelection {
  id: string;
  offset: bigint;
}

/** Transient confirmation authority belongs to the currently displayed page. */
export function currentRevokeId(
  selection: RevokeSelection | null,
  offset: bigint,
): string | null {
  return selection?.offset === offset ? selection.id : null;
}
