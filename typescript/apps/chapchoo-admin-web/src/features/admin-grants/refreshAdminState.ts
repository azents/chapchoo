interface InvalidatableQuery {
  invalidate: () => Promise<unknown>;
}

interface AdminStateQueries {
  auth: { identity: InvalidatableQuery };
  signupGrants: { list: InvalidatableQuery };
  resetGrants: { list: InvalidatableQuery };
}

/** Explicit recovery must refresh every query that can own the error screen. */
export async function refreshAdminState(
  queries: AdminStateQueries,
): Promise<void> {
  await Promise.all([
    queries.auth.identity.invalidate(),
    queries.signupGrants.list.invalidate(),
    queries.resetGrants.list.invalidate(),
  ]);
}
