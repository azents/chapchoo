"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { teamKind } from "@chapchoo/web-core/browser";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { validHandle } from "@chapchoo/web-core/handle";
import { useHandleDraft } from "@chapchoo/web-core/handle-draft";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { shellCopy, teamHandleCopy } from "@chapchoo/web-core/shell-messages";
import {
  agentsPath,
  profileNavigation,
  profileRoute,
  teamPath,
  workspacePreference,
} from "@chapchoo/web-core/workspace";
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
  const inProfile = profileRoute(route);
  const [drawer, setDrawer] = useState(false),
    [picker, setPicker] = useState(false),
    [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const utils = trpc.useUtils();
  const identity = auth.state.type === "AUTHENTICATED" ? auth.state.user : null;
  const proposal = useHandleDraft(
    "team",
    creating ? (identity?.id ?? null) : null,
    (candidate, signal) =>
      utils.client.workspaces.handleAvailable.query(
        { handle: candidate },
        { signal },
      ),
  );
  const handle = proposal.draft.candidate;
  const handleCopy = teamHandleCopy(display.value.language);
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
    selected: !inProfile,
  };
  const choices: WorkspaceChoice[] = generalUrl
    ? (query.data?.items ?? []).map((team): WorkspaceChoice => {
        const common = {
          key: team.id,
          href: agentsPath(generalUrl, team),
          selected: false,
        };
        return team.kind === teamKind.personal
          ? { ...common, kind: "personal", label: copy.yourSpace }
          : { ...common, kind: "team", label: team.name, handle: team.handle };
      })
    : [];
  choices.push(current);
  const profile: WorkspaceChoice = {
    key: "profile",
    kind: "profile",
    label: copy.profile,
    href: `${basePath}/profile`,
    selected: inProfile,
  };
  choices.unshift(profile);
  const create = trpc.workspaces.create.useMutation({ retry: false });
  async function submit(): Promise<void> {
    if (pending.current || !generalUrl || proposal.status !== "available") {
      return;
    }
    pending.current = true;
    setError(null);
    try {
      const team = await create.mutateAsync({
        name: name.trim(),
        handle,
      });
      void utils.workspaces.invalidate();
      if (identity) {
        document.cookie = workspacePreference(
          identity.id,
          team.id,
          window.location.protocol === "https:",
        );
      }
      window.location.assign(agentsPath(generalUrl, team));
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
    homeHref: inProfile
      ? `${basePath}/profile`
      : generalUrl
        ? generalUrl + "/"
        : null,
    homeTarget: !inProfile && generalUrl ? "_blank" : void 0,
    environment: "admin",
    mobile,
    drawer,
    picker,
    creating,
    name,
    handle,
    prospectiveAddress:
      generalUrl && validHandle(handle) ? teamPath(generalUrl, handle) : null,
    handleStatus: {
      description: handleCopy.permanent,
      message: proposal.status === "idle" ? null : handleCopy[proposal.status],
      available: proposal.status === "available",
    },
    error,
    choices,
    current: inProfile ? profile : current,
    items: inProfile
      ? profileNavigation(basePath, route, copy, false)
      : [
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
    retry: () => void utils.workspaces.list.invalidate(),
    select: (choice) => {
      if (identity && (choice.kind === "personal" || choice.kind === "team")) {
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
