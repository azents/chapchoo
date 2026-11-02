"use client";
import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useDocumentVisibility } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import type {
  ComputersInput,
  ComputersProps,
  InitialComputers,
} from "../types";
export function useComputersContainer({
  initial,
  basePath,
}: ComputersInput): ComputersProps {
  const auth = useAuth();
  const visible = useDocumentVisibility();
  const enabled = visible === "visible" && auth.state.type === "AUTHENTICATED";
  const copy = computerCopy(useLocale());
  const { value: display } = useDisplay();
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const page = { ...(initial.after ? { after: initial.after } : {}) };
  const providers = trpc.computers.providers.useQuery(page, {
    ...(initial.type === "providers"
      ? initialQuery(initial.page, initial.renderedAt)
      : {}),
    enabled: enabled && initial.type === "providers",
    retry: false,
    refetchInterval: 15_000,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
  });
  const computers = trpc.computers.list.useQuery(
    {
      ...page,
      providerId: initial.type === "computers" ? initial.providerId : "",
    },
    {
      ...(initial.type === "computers"
        ? initialQuery(initial.page, initial.renderedAt)
        : {}),
      enabled: enabled && initial.type === "computers",
      retry: false,
      refetchInterval: (query) =>
        query.state.data?.items.some((item) => item.phase !== "completed")
          ? 5_000
          : 15_000,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: true,
    },
  );
  const data: InitialComputers =
    initial.type === "providers"
      ? { ...initial, page: providers.data ?? initial.page }
      : { ...initial, page: computers.data ?? initial.page };
  return {
    initial,
    access:
      auth.state.type === "AUTHENTICATED"
        ? "authorized"
        : auth.state.type === "SIGNED_OUT"
          ? "signedOut"
          : "unavailable",
    basePath,
    data,
    copy,
    appearance,
    stale: initial.type === "providers" ? providers.isError : computers.isError,
    formatTime: (value) =>
      formatDisplayDate(value, display.region, display.timeZone, "medium") ??
      copy.unknown,
    onRefresh: () => {
      if (auth.state.type === "SIGNED_OUT") {
        window.location.assign(`${basePath}/login`);
      } else if (auth.state.type === "ERROR") {
        void auth.recheck();
      } else {
        void utils.computers.invalidate();
      }
    },
    onLogout: auth.logout,
  };
}
