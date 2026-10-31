"use client";

import { AuthenticationContext } from "@chapchoo/web-core/auth-context";
import { useRef } from "react";
import { trpc } from "@/shared/rpc/client";
import type {
  IdentityState,
  InitialIdentity,
} from "@chapchoo/web-core/auth-context";
import type { ReactNode } from "react";

/** Adapts the independent Admin tRPC contract to the shared authentication API. */
export function AuthProvider({
  initial,
  navigation,
  children,
}: {
  initial: InitialIdentity;
  navigation: {
    signedOut: () => void;
    authenticated: () => void;
    clearIdentity: () => void;
  };
  children: ReactNode;
}): ReactNode {
  const utils = trpc.useUtils();
  const loggingOut = useRef(false);
  const identity = trpc.auth.identity.useQuery(void 0, {
    enabled: initial.type === "LOADED" && initial.data !== null,
    ...(initial.type === "LOADED" ? { initialData: initial.data } : {}),
    retry: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
  });
  const logout = trpc.auth.logout.useMutation({
    onSuccess: navigation.signedOut,
    onSettled: () => {
      loggingOut.current = false;
    },
  });
  let state: IdentityState;
  if (identity.isError) {
    state = { type: "ERROR", reason: identity.error.data?.reason ?? null };
  } else if (identity.data) {
    state = { type: "AUTHENTICATED", user: identity.data };
  } else if (identity.data === null || initial.type === "LOADED") {
    state = { type: "SIGNED_OUT" };
  } else {
    state = { type: "ERROR", reason: initial.reason };
  }
  return (
    <AuthenticationContext.Provider
      value={{
        state,
        checking: identity.isFetching,
        logoutPending: logout.isPending,
        logoutError: logout.isError
          ? (logout.error.data?.reason ?? "DOMAIN_REASON_INTERNAL")
          : null,
        logout: () => {
          if (!loggingOut.current) {
            loggingOut.current = true;
            logout.mutate();
          }
        },
        recheck: async () => {
          const user = await utils.auth.identity.fetch();
          if (user === null) {
            navigation.signedOut();
          }
        },
        authenticated: navigation.authenticated,
        clearIdentity: navigation.clearIdentity,
      }}
    >
      {children}
    </AuthenticationContext.Provider>
  );
}
