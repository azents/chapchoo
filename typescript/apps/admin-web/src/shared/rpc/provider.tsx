"use client";
import { createAuthenticationClient } from "@chapchoo/web-core/auth-client";
import { QueryClientProvider } from "@tanstack/react-query";
import { httpLink } from "@trpc/client";
import { getQueryKey } from "@trpc/react-query";
import { useState } from "react";
import superjson from "superjson";
import { AuthProvider } from "@/shared/auth/provider";
import { trpc } from "@/shared/rpc/client";
import type { InitialIdentity } from "@chapchoo/web-core/auth-context";
import type { ReactNode } from "react";

export function RpcProvider({
  children,
  basePath = "",
  initialIdentity,
}: {
  children: ReactNode;
  basePath?: string;
  initialIdentity: InitialIdentity;
}): ReactNode {
  const [{ queryClient, navigation }] = useState(() =>
    createAuthenticationClient(
      getQueryKey(trpc.auth.identity, void 0, "query"),
      initialIdentity.type === "LOADED" && initialIdentity.data !== null,
      basePath,
      (path) => window.location.replace(path),
      "admin",
    ),
  );
  const [client] = useState(() =>
    trpc.createClient({
      links: [
        httpLink({
          url: `${basePath}/api/trpc`,
          transformer: superjson,
          methodOverride: "POST",
        }),
      ],
    }),
  );
  return (
    <trpc.Provider client={client} queryClient={queryClient}>
      <QueryClientProvider client={queryClient}>
        <AuthProvider initial={initialIdentity} navigation={navigation}>
          {children}
        </AuthProvider>
      </QueryClientProvider>
    </trpc.Provider>
  );
}
