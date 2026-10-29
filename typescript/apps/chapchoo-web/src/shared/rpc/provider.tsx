"use client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { httpLink } from "@trpc/client";
import { useState } from "react";
import superjson from "superjson";
import { trpc } from "@/shared/rpc/client";
import type { ReactNode } from "react";

export function RpcProvider({
  children,
  basePath = "",
}: {
  children: ReactNode;
  basePath?: string;
}): ReactNode {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { retry: false, refetchOnWindowFocus: false },
          mutations: { retry: false },
        },
      }),
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
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    </trpc.Provider>
  );
}
