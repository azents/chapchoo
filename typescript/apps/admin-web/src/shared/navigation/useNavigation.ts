"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { teamKind } from "@chapchoo/web-core/browser";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { workspacePreference } from "@chapchoo/web-core/workspace";
import { useMediaQuery } from "@mantine/hooks";
import { usePathname } from "next/navigation";
import { useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import type { InitialNavigation } from "./types";
import type { WorkspaceChromeProps } from "@chapchoo/ui/WorkspaceChrome";
import type { WorkspaceChoice } from "@chapchoo/ui/WorkspaceSelector";
import type { ReactNode } from "react";

export interface NavigationInput {
  initial: InitialNavigation;
  basePath: string;
  generalUrl: string | null;
  children: ReactNode;
}
export function useNavigation({
  initial,
  basePath,
  generalUrl,
  children,
}: NavigationInput): WorkspaceChromeProps {
  const auth = useAuth();
  const display = useDisplay();
  const copy = shellCopy(display.value.language);
  const pathname = usePathname();
  const route = pathname.startsWith(basePath + "/")
    ? pathname.slice(basePath.length)
    : pathname;
  const mobile = useMediaQuery("(max-width: 48em)");
  const [drawer, setDrawer] = useState(false),
    [picker, setPicker] = useState(false),
    [creating, setCreating] = useState(false);
  const [name, setName] = useState(""),
    [handle, setHandle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const utils = trpc.useUtils();
  const identity = auth.state.type === "AUTHENTICATED" ? auth.state.user : null;
  const query = trpc.workspaces.list.useQuery(void 0, {
    ...(initial.type === "ready" && initial.userId === identity?.id
      ? initialQuery({ items: initial.teams }, initial.renderedAt)
      : {}),
    enabled: identity !== null,
    retry: false,
  });
  const current: WorkspaceChoice = {
    key: "administration",
    kind: "admin",
    label: copy.administration,
    href: basePath + "/",
    selected: true,
  };
  const choices: WorkspaceChoice[] = generalUrl
    ? (query.data?.items ?? []).map((team) => ({
        key: team.id,
        kind: team.kind === teamKind.personal ? "personal" : "team",
        label: team.kind === teamKind.personal ? copy.yourSpace : team.name,
        href: `${generalUrl}/agents?team=${encodeURIComponent(team.id)}`,
        selected: false,
      }))
    : [];
  choices.push(current);
  const create = trpc.workspaces.create.useMutation({ retry: false });
  async function submit(): Promise<void> {
    if (pending.current || !generalUrl) {
      return;
    }
    pending.current = true;
    setError(null);
    try {
      const team = await create.mutateAsync({
        name: name.trim(),
        handle: handle.trim(),
      });
      void utils.workspaces.invalidate();
      if (identity) {
        document.cookie = workspacePreference(
          identity.id,
          team.id,
          window.location.protocol === "https:",
        );
      }
      window.location.assign(
        `${generalUrl}/agents?team=${encodeURIComponent(team.id)}`,
      );
    } catch (cause) {
      void utils.workspaces.list.invalidate();
      const reason =
        cause instanceof Error &&
        "data" in cause &&
        typeof cause.data === "object" &&
        cause.data !== null &&
        "reason" in cause.data
          ? cause.data.reason
          : null;
      setError(
        reason === "DOMAIN_REASON_HANDLE_CONFLICT"
          ? copy.handleConflict
          : reason === "DOMAIN_REASON_INVALID_INPUT"
            ? copy.invalidTeam
            : copy.creationUncertain,
      );
    } finally {
      pending.current = false;
    }
  }
  return {
    children,
    identity: identity !== null,
    copy,
    basePath,
    homeHref: generalUrl ? generalUrl + "/" : null,
    environment: "admin",
    mobile,
    drawer,
    picker,
    creating,
    name,
    handle,
    error,
    choices,
    current,
    items: [
      {
        key: "signup",
        label: copy.signupLinks,
        href: basePath + "/",
        active: route === "/" || route === basePath,
      },
      {
        key: "reset",
        label: copy.resetLinks,
        href: basePath + "/password-resets",
        active: route === "/password-resets",
      },
      {
        key: "computers",
        label: copy.computerProviders,
        href: basePath + "/computer-providers",
        active:
          route.startsWith("/computer-providers") ||
          route.startsWith("/computers/"),
      },
      {
        key: "settings",
        label: copy.serverSettings,
        href: basePath + "/settings",
        active: route === "/settings",
      },
    ],
    status:
      query.isError || !generalUrl ? "error" : query.data ? "ready" : "loading",
    busy: create.isPending,
    mascotReplayLabel: computerCopy(display.value.language).replay,
    openDrawer: () => {
      display.close();
      setDrawer(true);
      setPicker(false);
    },
    closeDrawer: () => {
      setDrawer(false);
      setPicker(false);
    },
    openPicker: () => setPicker(true),
    closePicker: () => setPicker(false),
    openCreation: () => {
      setDrawer(false);
      setPicker(false);
      setCreating(true);
      setError(null);
    },
    closeCreation: () => {
      if (!create.isPending) {
        setCreating(false);
      }
    },
    onName: setName,
    onHandle: setHandle,
    onSubmit: () => void submit(),
    retry: () => void utils.workspaces.list.invalidate(),
    select: (choice) => {
      if (identity && choice.kind !== "admin") {
        document.cookie = workspacePreference(
          identity.id,
          choice.key,
          window.location.protocol === "https:",
        );
      }
      setDrawer(false);
      setPicker(false);
    },
  };
}
