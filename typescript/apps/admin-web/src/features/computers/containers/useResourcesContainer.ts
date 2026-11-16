"use client";
import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { displayText } from "@chapchoo/web-core/display-messages";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useDocumentVisibility } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import type { ResourcesInput, ResourcesProps } from "../resourceTypes";
export function useResourcesContainer({
  initial,
  basePath,
}: ResourcesInput): ResourcesProps {
  const auth = useAuth();
  const visible = useDocumentVisibility();
  const copy = computerCopy(useLocale());
  const { value: display } = useDisplay();
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const query = trpc.computers.resources.useQuery(initial.selection, {
    ...initialQuery(initial.page, initial.renderedAt),
    enabled: visible === "visible" && auth.state.type === "AUTHENTICATED",
    retry: false,
    refetchInterval: 15_000,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
  });
  return {
    initial,
    access:
      auth.state.type === "AUTHENTICATED"
        ? "authorized"
        : auth.state.type === "SIGNED_OUT"
          ? "signedOut"
          : "unavailable",
    basePath,
    copy,
    appearance,
    data: query.data,
    stale: query.isError,
    onRefresh: () => {
      if (auth.state.type === "SIGNED_OUT") {
        window.location.assign(`${basePath}/login`);
      } else if (auth.state.type === "ERROR") {
        void auth.recheck();
      } else {
        void utils.computers.resources.invalidate(initial.selection);
      }
    },
    onLogout: auth.logout,
    formatTime: (value) =>
      formatDisplayDate(value, display.region, display.timeZone, "medium") ??
      displayText(display.language).unavailable,
  };
}
