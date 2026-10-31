import {
  hashKey,
  MutationCache,
  QueryCache,
  QueryClient,
} from "@tanstack/react-query";
import { TRPCClientError } from "@trpc/client";
import type { QueryKey } from "@tanstack/react-query";

export function invalidatesSession(
  reason: string | null,
  path: string | null,
  app: "general" | "admin",
): boolean {
  if (path === null) return false;
  if (
    app === "admin" &&
    path === "auth.identity" &&
    reason === "DOMAIN_REASON_PERMISSION_DENIED"
  )
    return true;
  return (
    reason === "DOMAIN_REASON_INVALID_SESSION" &&
    (path === "auth.identity" ||
      (app === "general"
        ? path.startsWith("teams.") || path.startsWith("invitations.")
        : path.startsWith("signupGrants.") || path.startsWith("resetGrants.")))
  );
}

/** Native POST receives HttpOnly cookie deletion without replaying a mutation. */
export function submitLogout(path: string): void {
  const form = document.createElement("form");
  form.method = "POST";
  form.action = path;
  document.body.append(form);
  form.submit();
}

export function createAuthenticationClient(
  identityKey: QueryKey,
  initiallyAuthenticated: boolean,
  basePath: string,
  navigate: (path: string) => void,
  app: "general" | "admin",
  post: (path: string) => void = submitLogout,
): {
  queryClient: QueryClient;
  navigation: {
    signedOut: () => void;
    authenticated: () => void;
    clearIdentity: () => void;
    logout: () => void;
  };
} {
  let leaving = false;
  let expectsSession = initiallyAuthenticated;
  function clearIdentity(): void {
    expectsSession = false;
    void queryClient.cancelQueries();
    queryClient.setQueryData(identityKey, null);
    queryClient.removeQueries({
      predicate: (query) => query.queryHash !== hashKey(identityKey),
    });
    queryClient.getMutationCache().clear();
  }
  function leave(path: string): void {
    if (leaving) return;
    leaving = true;
    clearIdentity();
    navigate(path);
  }
  const onError = (error: unknown): void => {
    if (!(error instanceof TRPCClientError)) return;
    const data: unknown = error.data;
    if (typeof data !== "object" || data === null) return;
    const reason =
      "reason" in data && typeof data.reason === "string" ? data.reason : null;
    const path =
      "path" in data && typeof data.path === "string" ? data.path : null;
    if (invalidatesSession(reason, path, app)) leave(`${basePath}/login`);
  };
  const queryClient = new QueryClient({
    queryCache: new QueryCache({
      onError,
      onSuccess: (data, query) => {
        if (query.queryHash !== hashKey(identityKey)) return;
        if (data === null) {
          if (expectsSession) leave(`${basePath}/login`);
        } else {
          expectsSession = true;
        }
      },
    }),
    mutationCache: new MutationCache({ onError }),
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });
  return {
    queryClient,
    navigation: {
      signedOut: () => leave(`${basePath}/login`),
      authenticated: () => leave(`${basePath}/`),
      clearIdentity,
      logout: () => {
        if (leaving) return;
        leaving = true;
        clearIdentity();
        post(`${basePath}/api/logout`);
      },
    },
  };
}
