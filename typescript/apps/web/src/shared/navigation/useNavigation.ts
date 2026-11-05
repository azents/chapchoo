"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { teamKind } from "@chapchoo/web-core/browser";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { validHandle } from "@chapchoo/web-core/handle";
import { useHandleDraft } from "@chapchoo/web-core/handle-draft";
import {
  initialQuery,
  navigateSelection,
  readSelection,
} from "@chapchoo/web-core/page-state";
import { shellCopy, teamHandleCopy } from "@chapchoo/web-core/shell-messages";
import {
  agentChatPath,
  agentsPath,
  newAgentPath,
  routeTeamHandle,
  teamPath,
  teamRouteKey,
  workspacePreference,
} from "@chapchoo/web-core/workspace";
import { useDocumentVisibility, useMediaQuery } from "@mantine/hooks";
import { usePathname, useSearchParams } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import {
  agentCursor,
  agentCursorKey,
  agentNavigationHref,
  agentNavigationItems,
  routeAgentHandle,
} from "./agentSelection";
import { navigationOwnerMatches } from "./navigationOwner";
import type { AgentNavigationProps } from "./AgentNavigation";
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
  const params = useSearchParams();
  const mobile = useMediaQuery("(max-width: 48em)");
  const visibility = useDocumentVisibility();
  const [drawer, setDrawer] = useState(false);
  const [picker, setPicker] = useState(false);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const utils = trpc.useUtils();
  const identity = auth.state.type === "AUTHENTICATED" ? auth.state.user : null;
  const proposal = useHandleDraft(
    "team",
    creating ? (identity?.id ?? null) : null,
    (candidate, signal) =>
      utils.client.teams.handleAvailable.query(
        { handle: candidate },
        { signal },
      ),
  );
  const handle = proposal.draft.candidate;
  const handleCopy = teamHandleCopy(display.value.language);
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
  const routeTeam = routeTeamHandle(route);
  const contexts = query.data?.items ?? [];
  const selected =
    (routeTeam
      ? contexts.find((team) => teamRouteKey(team) === routeTeam)
      : (contexts.find(
          (team) =>
            team.id === (initial.type === "ready" ? initial.preferred : null),
        ) ?? contexts.find((team) => team.kind === teamKind.personal))) ?? null;
  const admittedIdentity = navigationOwnerMatches(
    initial,
    identity?.id ?? null,
  );
  const teamId = admittedIdentity && routeTeam && selected ? selected.id : "";
  const after = readSelection(params, agentCursorKey, agentCursor);
  const agentList = trpc.computers.list.useQuery(
    { teamId, ...(after ? { after } : {}) },
    {
      enabled: identity !== null && Boolean(teamId) && visibility === "visible",
      retry: false,
      refetchInterval: 15_000,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: true,
    },
  );
  const computerText = computerCopy(display.value.language);
  const currentAgent = routeAgentHandle(route);
  const currentDetail = trpc.computers.getInTeam.useQuery(
    { teamId, handle: currentAgent ?? "" },
    {
      enabled:
        Boolean(teamId && currentAgent) &&
        identity !== null &&
        visibility === "visible",
      retry: false,
      refetchInterval: 15_000,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: true,
    },
  );
  useEffect(() => {
    if (identity && selected && routeTeam === teamRouteKey(selected)) {
      document.cookie = workspacePreference(
        identity.id,
        selected.id,
        window.location.protocol === "https:",
      );
    }
  }, [identity, selected, routeTeam]);
  const choices: WorkspaceChoice[] = contexts.map((team): WorkspaceChoice => {
    const common = {
      key: team.id,
      href: agentsPath(basePath, team),
      selected: team.id === selected?.id,
    };
    return team.kind === teamKind.personal
      ? { ...common, kind: "personal", label: copy.yourSpace }
      : { ...common, kind: "team", label: team.name, handle: team.handle };
  });
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
  const items: NavigationItem[] = [
    ...(!teamId
      ? ([
          {
            key: "agents",
            label: copy.agents,
            icon: "agents",
            href: selected ? agentsPath(basePath, selected) : `${basePath}/`,
            active: /^\/teams\/[^/]+\/(?:agents|new-agent)(?:\/|$)/.test(route),
          },
        ] satisfies NavigationItem[])
      : []),
    ...(selected?.kind === teamKind.collaborative
      ? [
          {
            key: "team",
            label: copy.teamSettings,
            icon: "team" as const,
            href: agentNavigationHref(teamPath(basePath, selected), after),
            active: /^\/teams\/[^/]+$/.test(route),
          },
        ]
      : []),
  ];
  const create = trpc.teams.create.useMutation({ retry: false });
  async function submit(): Promise<void> {
    if (pending.current || proposal.status !== "available") {
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
      window.location.assign(agentsPath(basePath, team));
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
      if (reason === "DOMAIN_REASON_HANDLE_CONFLICT") {
        proposal.recheck();
      }
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
    prospectiveAddress: validHandle(handle) ? teamPath(basePath, handle) : null,
    handleStatus: {
      description: handleCopy.permanent,
      message: proposal.status === "idle" ? null : handleCopy[proposal.status],
      available: proposal.status === "available",
    },
    error,
    choices,
    current,
    items,
    agentNavigation:
      teamId && selected && identity
        ? {
            basePath,
            copy: computerText,
            titleHref: agentNavigationHref(
              agentsPath(basePath, selected),
              after,
            ),
            createHref: agentNavigationHref(
              newAgentPath(basePath, selected),
              after,
            ),
            createLabel: computerText.create,
            items: agentNavigationItems(
              agentList.data?.items ?? [],
              currentDetail.data ?? null,
              teamId,
              currentAgent,
            ),
            selectedHandle: currentAgent,
            agentHref: (handle) =>
              agentNavigationHref(
                agentChatPath(basePath, selected, handle),
                after,
              ),
            status:
              agentList.isError || (currentAgent && currentDetail.isError)
                ? "error"
                : agentList.data
                  ? "ready"
                  : "loading",
            after,
            nextAfter: agentList.data?.nextAfter ?? null,
            onPage: (value) =>
              navigateSelection(agentCursorKey, value, agentCursor, "push"),
            onSelect: () => setDrawer(false),
          }
        : null,
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
    onName: (value) => {
      if (!pending.current) {
        setName(value);
        proposal.rename(value);
      }
    },
    onHandle: (value) => {
      if (!pending.current) {
        proposal.edit(value);
      }
    },
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
  prospectiveAddress: string | null;
  handleStatus: {
    message: string | null;
    description: string;
    available: boolean;
  };
  error: string | null;
  choices: WorkspaceChoice[];
  current: WorkspaceChoice | null;
  items: NavigationItem[];
  agentNavigation: AgentNavigationProps | null;
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
