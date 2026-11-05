"use client";
import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import {
  randomAgentPalette,
  validAgentPalette,
} from "@chapchoo/web-core/agent-profile";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { agentsPath } from "@chapchoo/web-core/workspace";
import { useDocumentVisibility, useMediaQuery } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { useAgentNavigationHref } from "@/shared/navigation/useAgentNavigationHref";
import { trpc } from "@/shared/rpc/client";
import { creationCopy } from "../creationCopy";
import { computerDetailCopy } from "../detailCopy";
import { useComputerActions } from "./useComputerActions";
import type {
  AgentsInput,
  AgentsProps,
  InitialAgents,
  ProfileDraft,
  ProfileEditor,
} from "../types";
import type { AgentView, OwnerAction } from "@chapchoo/web-core/computer";

function pending(agent: AgentView): boolean {
  return agent.computer.phase !== "completed";
}
export function useAgentsContainer({
  initial,
  basePath,
}: AgentsInput): AgentsProps {
  const navigationHref = useAgentNavigationHref();
  const agentsHref = navigationHref(agentsPath(basePath, initial.teamHandle));
  const auth = useAuth();
  const mobile = useMediaQuery("(max-width: 48em)");
  const visibility = useDocumentVisibility();
  const enabled =
    visibility === "visible" && auth.state.type === "AUTHENTICATED";
  const copy = computerCopy(useLocale());
  const profileCopy = creationCopy(useLocale());
  const detailCopy = computerDetailCopy(useLocale());
  const [detailsOpened, setDetailsOpened] = useState(false);
  const { value: display } = useDisplay();
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const [editor, setEditor] = useState<ProfileEditor>({ type: "closed" });
  const [imageAfter, setImageAfter] = useState<string | null>(null);
  const saving = useRef(false);
  const images = trpc.computers.images.useQuery(
    imageAfter ? { after: imageAfter } : {},
    {
      enabled: enabled && editor.type !== "closed",
      retry: false,
    },
  );
  const updateProfile = trpc.computers.updateProfile.useMutation({
    retry: false,
  });
  const [submission, setSubmission] = useState<
    { type: "ready" } | { type: "pending" } | { type: "error"; message: string }
  >({ type: "ready" });
  const busy = useRef(false);
  const listInput = {
    teamId: initial.teamId,
    ...(initial.type === "list" && initial.after
      ? { after: initial.after }
      : {}),
  };
  const list = trpc.computers.list.useQuery(listInput, {
    ...(initial.type === "list"
      ? initialQuery(initial.page, initial.renderedAt)
      : {}),
    enabled: enabled && initial.type === "list",
    retry: false,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
    refetchInterval: (query) =>
      query.state.data?.items.some(pending) ? 5_000 : 15_000,
  });
  const agentId = initial.type === "detail" ? initial.agent.id : "";
  const detail = trpc.computers.get.useQuery(
    { agentId },
    {
      ...(initial.type === "detail"
        ? initialQuery(initial.agent, initial.renderedAt)
        : {}),
      enabled: enabled && initial.type === "detail",
      retry: false,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: true,
      refetchInterval: (query) =>
        query.state.data && pending(query.state.data) ? 5_000 : 15_000,
    },
  );
  const lifecycle = trpc.computers.lifecycle.useMutation({ retry: false });
  const force = trpc.computers.forceRemove.useMutation({ retry: false });
  const data: InitialAgents =
    initial.type === "list"
      ? { ...initial, page: list.data ?? initial.page }
      : { ...initial, agent: detail.data ?? initial.agent };
  function editDraft(patch: Partial<ProfileDraft>): void {
    if (saving.current) {
      return;
    }
    setEditor((current) =>
      current.type === "editing"
        ? {
            type: "editing",
            draft: { ...current.draft, ...patch },
            error: null,
          }
        : current,
    );
  }
  async function saveProfile(): Promise<void> {
    if (
      saving.current ||
      editor.type !== "editing" ||
      !editor.draft.name.trim()
    ) {
      return;
    }
    saving.current = true;
    const draft = editor.draft;
    setEditor({ type: "saving", draft });
    try {
      const saved = await updateProfile.mutateAsync({
        agentId,
        name: draft.name.trim(),
        profileImageId: draft.image.id,
        paletteKey: draft.paletteKey,
      });
      utils.computers.get.setData({ agentId }, saved);
      setEditor({ type: "closed" });
      void utils.computers.invalidate();
    } catch {
      setEditor({ type: "editing", draft, error: profileCopy.saveFailed });
    } finally {
      saving.current = false;
    }
  }
  useEffect(() => {
    if (detail.error?.data?.reason === "DOMAIN_REASON_NOT_FOUND") {
      window.location.replace(agentsHref);
    }
  }, [detail.error, agentsHref]);
  async function refresh(): Promise<void> {
    await utils.computers.invalidate();
  }
  async function submitAction(
    action: OwnerAction | "force",
    expectedRequestId: string,
  ): Promise<void> {
    if (busy.current || data.type !== "detail") {
      return;
    }
    busy.current = true;
    setSubmission({ type: "pending" });
    try {
      if (action === "force") {
        await force.mutateAsync({
          agentId,
          deletionRequestId: expectedRequestId,
        });
        void utils.computers.invalidate();
        window.location.assign(agentsHref);
        return;
      }
      const updated = await lifecycle.mutateAsync({
        agentId,
        expectedRequestId,
        requestId: crypto.randomUUID(),
        action,
      });
      utils.computers.get.setData({ agentId }, updated);
      actions.complete();
      setSubmission({ type: "ready" });
      void refresh();
    } catch {
      setSubmission({ type: "error", message: copy.lostSubmission });
      await refresh();
    } finally {
      busy.current = false;
    }
  }
  const actions = useComputerActions({
    pending: submission.type === "pending",
    submit: (action, requestId) => void submitAction(action, requestId),
    reset: () => setSubmission({ type: "ready" }),
  });
  return {
    initial,
    navigationHref,
    access:
      auth.state.type === "AUTHENTICATED"
        ? "authorized"
        : auth.state.type === "SIGNED_OUT"
          ? "signedOut"
          : "unavailable",
    basePath,
    data,
    copy,
    profileCopy,
    detailCopy,
    detailsOpened,
    onOpenDetails: () => setDetailsOpened(true),
    onCloseDetails: () => setDetailsOpened(false),
    mobile,
    editor,
    images: images.data ?? { items: [], nextAfter: null },
    imageAfter,
    imagesUnavailable: images.isError || !images.data,
    onEditProfile: () => {
      if (data.type === "detail" && !saving.current) {
        setEditor({
          type: "editing",
          draft: {
            name: data.agent.name,
            image: data.agent.profileImage,
            paletteKey: data.agent.paletteKey,
          },
          error: null,
        });
      }
    },
    onProfileName: (name) => editDraft({ name }),
    onProfileImage: (id) => {
      const image = images.data?.items.find((item) => item.id === id);
      if (image) {
        editDraft({ image });
      }
    },
    onProfilePalette: (paletteKey) => {
      if (validAgentPalette(paletteKey)) {
        editDraft({ paletteKey });
      }
    },
    onRandomProfilePalette: () => {
      if (editor.type !== "editing" || saving.current) {
        return;
      }
      try {
        editDraft({ paletteKey: randomAgentPalette(editor.draft.paletteKey) });
      } catch {
        setEditor((current) =>
          current.type === "editing"
            ? { ...current, error: profileCopy.randomFailed }
            : current,
        );
      }
    },
    onImagePage: (after) => {
      if (!saving.current) {
        setImageAfter(after);
      }
    },
    onSaveProfile: () => void saveProfile(),
    onCancelProfile: () => {
      if (!saving.current) {
        setEditor({ type: "closed" });
      }
    },
    appearance,
    stale: initial.type === "list" ? list.isError : detail.isError,
    pending: submission.type === "pending",
    error: submission.type === "error" ? submission.message : null,
    confirmation: actions.confirmation?.action ?? null,
    onAction: (value) => {
      if (data.type !== "detail") {
        return;
      }
      const requestId = data.agent.computer.requestId;
      actions.select(value, requestId);
    },
    onConfirm: actions.confirm,
    onCancel: actions.cancel,
    onRefresh: () => {
      if (auth.state.type === "SIGNED_OUT") {
        window.location.assign(`${basePath}/login`);
      } else if (auth.state.type === "ERROR") {
        void auth.recheck();
      } else {
        void refresh();
      }
    },
    onLogout: auth.logout,
    formatTime: (value) =>
      formatDisplayDate(value, display.region, display.timeZone, "medium") ??
      copy.unknown,
  };
}
