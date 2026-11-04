"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { teamKind } from "@chapchoo/web-core/browser";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { shellCopy } from "@chapchoo/web-core/shell-messages";
import { workspacePreference } from "@chapchoo/web-core/workspace";
import { useMediaQuery } from "@mantine/hooks";
import { usePathname, useSearchParams } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import type { InitialNavigation } from "./types";
import type { NavigationItem } from "@chapchoo/ui/AppNavigation";
import type { WorkspaceChoice } from "@chapchoo/ui/WorkspaceSelector";
import type { ReactNode } from "react";

export interface NavigationInput {
  initial: InitialNavigation;
  basePath: string;
  children: ReactNode;
  adminUrl?: string | null;
}
export function useNavigation({
  initial,
  basePath,
  children,
  adminUrl,
}: NavigationInput): NavigationModel {
  const auth = useAuth();
  const display = useDisplay();
  const copy = shellCopy(display.value.language);
  const pathname = usePathname();
  const search = useSearchParams();
  const mobile = useMediaQuery("(max-width: 48em)");
  const [drawer, setDrawer] = useState(false);
  const [picker, setPicker] = useState(false);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [handle, setHandle] = useState("");
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const utils = trpc.useUtils();
  const identity = auth.state.type === "AUTHENTICATED" ? auth.state.user : null;
  const query = trpc.teams.list.useQuery(void 0, {
    ...(initial.type === "ready" && initial.userId === identity?.id
      ? initialQuery({ items: initial.teams }, initial.renderedAt)
      : {}),
    enabled: identity !== null,
    retry: false,
  });
  const route = pathname.startsWith(basePath + "/")
    ? pathname.slice(basePath.length)
    : pathname;
  const routeTeam = /^\/teams\/([^/]+)$/.exec(route)?.[1];
  const selectedId = routeTeam
    ? decodeURIComponent(routeTeam)
    : (search.get("team") ??
      (initial.type === "ready" ? initial.preferred : null));
  const contexts = query.data?.items ?? [];
  const selected =
    contexts.find((team) => team.id === selectedId) ??
    contexts.find((team) => team.kind === teamKind.personal) ??
    null;
  useEffect(() => {
    if (
      identity &&
      selected &&
      (search.get("team") === selected.id || routeTeam === selected.id)
    ) {
      document.cookie = workspacePreference(
        identity.id,
        selected.id,
        window.location.protocol === "https:",
      );
    }
  }, [identity, selected, routeTeam, search]);
  const choices: WorkspaceChoice[] = contexts.map((team) => ({
    key: team.id,
    kind: team.kind === teamKind.personal ? "personal" : "team",
    label: team.kind === teamKind.personal ? copy.yourSpace : team.name,
    href: `${basePath}/agents?team=${encodeURIComponent(team.id)}`,
    selected: team.id === selected?.id,
  }));
  const current = choices.find((choice) => choice.selected) ?? null;
  if (identity?.isAdmin && adminUrl) {
    choices.push({
      key: "administration",
      kind: "admin",
      label: copy.administration,
      href: adminUrl + "/",
      selected: false,
    });
  }
  const suffix = selected ? `?team=${encodeURIComponent(selected.id)}` : "";
  const items: NavigationItem[] = [
    {
      key: "agents",
      label: copy.agents,
      href: `${basePath}/agents${suffix}`,
      active: route.startsWith("/agents"),
    },
    {
      key: "invitations",
      label: copy.invitations,
      href: `${basePath}/invitations${suffix}`,
      active: route === "/invitations",
    },
    ...(selected?.kind === teamKind.collaborative
      ? [
          {
            key: "team",
            label: copy.teamSettings,
            href: `${basePath}/teams/${encodeURIComponent(selected.id)}`,
            active: route.startsWith("/teams/"),
          },
        ]
      : []),
  ];
  const create = trpc.teams.create.useMutation({ retry: false });
  async function submit(): Promise<void> {
    if (pending.current) {
      return;
    }
    pending.current = true;
    setError(null);
    try {
      const team = await create.mutateAsync({
        name: name.trim(),
        handle,
      });
      void utils.teams.invalidate();
      if (identity) {
        document.cookie = workspacePreference(
          identity.id,
          team.id,
          window.location.protocol === "https:",
        );
      }
      window.location.assign(
        `${basePath}/agents?team=${encodeURIComponent(team.id)}`,
      );
    } catch (cause) {
      void utils.teams.list.invalidate();
      // No automatic retry: the ordinary create endpoint has no receipt contract.
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
    homeHref: `${basePath}/`,
    mobile,
    drawer,
    picker,
    creating,
    name,
    handle,
    error,
    choices,
    current,
    items,
    status: query.isError ? "error" : query.data ? "ready" : "loading",
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
    retry: () => void utils.teams.list.invalidate(),
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
export interface NavigationModel {
  children: ReactNode;
  identity: boolean;
  copy: ReturnType<typeof shellCopy>;
  basePath: string;
  homeHref: string;
  mobile: boolean;
  drawer: boolean;
  picker: boolean;
  creating: boolean;
  name: string;
  handle: string;
  error: string | null;
  choices: WorkspaceChoice[];
  current: WorkspaceChoice | null;
  items: NavigationItem[];
  status: "ready" | "loading" | "error";
  busy: boolean;
  mascotReplayLabel: string;
  openDrawer: () => void;
  closeDrawer: () => void;
  openPicker: () => void;
  closePicker: () => void;
  openCreation: () => void;
  closeCreation: () => void;
  onName: (value: string) => void;
  onHandle: (value: string) => void;
  onSubmit: () => void;
  retry: () => void;
  select: (choice: WorkspaceChoice) => void;
}
