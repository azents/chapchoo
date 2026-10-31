"use client";

import { AuthenticationContext } from "@chapchoo/web-core/auth-context";
import { trpc } from "@/shared/rpc/client";
import type {
  IdentityState,
  InitialIdentity,
} from "@chapchoo/web-core/auth-context";
import type { ReactNode } from "react";

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
    logout: () => void;
  };
  children: ReactNode;
}): ReactNode {
  const utils = trpc.useUtils();
  const identity = trpc.auth.identity.useQuery(void 0, {
    enabled: initial.type === "LOADED" && initial.data !== null,
    ...(initial.type === "LOADED" ? { initialData: initial.data } : {}),
    retry: false,
    refetchOnMount: false,
    refetchOnReconnect: false,
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
        logout: navigation.logout,
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
