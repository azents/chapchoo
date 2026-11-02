"use client";
import { formatDisplayDate } from "@chapchoo/ui/displayFormatting";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { useDisplay } from "@chapchoo/web-core/display-context";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useDocumentVisibility } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import type { AgentsInput, AgentsProps, InitialAgents } from "../types";
import type { AgentView, OwnerAction } from "@chapchoo/web-core/computer";

function pending(agent: AgentView): boolean {
  return agent.computer.phase !== "completed";
}
export function useAgentsContainer({
  initial,
  basePath,
}: AgentsInput): AgentsProps {
  const auth = useAuth();
  const visibility = useDocumentVisibility();
  const enabled =
    visibility === "visible" && auth.state.type === "AUTHENTICATED";
  const copy = computerCopy(useLocale());
  const { value: display } = useDisplay();
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const [name, setName] = useState("");
  const [providerId, setProviderId] = useState("");
  const [providerAfter, setProviderAfter] = useState<string | null>(null);
  const [confirmation, setConfirmation] = useState<{
    action: OwnerAction | "force";
    requestId: string;
  } | null>(null);
  const [submission, setSubmission] = useState<
    { type: "ready" } | { type: "pending" } | { type: "error"; message: string }
  >({ type: "ready" });
  const busy = useRef(false);
  const createRequest = useRef<{
    requestId: string;
    name: string;
    providerId: string;
  } | null>(null);
  const listInput = {
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
  const providerChoices = trpc.computers.providers.useQuery(
    providerAfter ? { after: providerAfter } : {},
    {
      enabled: enabled && initial.type === "list",
      retry: false,
      refetchInterval: 15_000,
      refetchIntervalInBackground: false,
      refetchOnWindowFocus: true,
    },
  );
  const create = trpc.computers.create.useMutation({
    retry: false,
    onError: (error) => {
      if (
        error.data?.reason === "DOMAIN_REASON_INVALID_INPUT" ||
        error.data?.reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
        error.data?.reason === "DOMAIN_REASON_PERMISSION_DENIED"
      ) {
        createRequest.current = null;
      }
    },
  });
  const lifecycle = trpc.computers.lifecycle.useMutation({ retry: false });
  const force = trpc.computers.forceRemove.useMutation({ retry: false });
  const data: InitialAgents =
    initial.type === "list"
      ? { ...initial, page: list.data ?? initial.page }
      : { ...initial, agent: detail.data ?? initial.agent };
  useEffect(() => {
    if (detail.error?.data?.reason === "DOMAIN_REASON_NOT_FOUND") {
      window.location.replace(`${basePath}/agents`);
    }
  }, [detail.error, basePath]);
  async function refresh(): Promise<void> {
    await utils.computers.invalidate();
  }
  async function submitCreate(): Promise<void> {
    if (busy.current) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "pending" });
    const payload = createRequest.current ?? {
      requestId: crypto.randomUUID(),
      name: name.trim(),
      providerId,
    };
    createRequest.current = payload;
    try {
      const result = await create.mutateAsync(payload);
      if (result.type === "removed") {
        setSubmission({ type: "error", message: copy.removed });
        createRequest.current = null;
        return;
      }
      void utils.computers.list.invalidate();
      window.location.assign(
        `${basePath}/agents/${encodeURIComponent(result.agent.id)}`,
      );
    } catch {
      setSubmission({ type: "error", message: copy.lostSubmission });
    } finally {
      busy.current = false;
    }
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
        window.location.assign(`${basePath}/agents`);
        return;
      }
      const updated = await lifecycle.mutateAsync({
        agentId,
        expectedRequestId,
        requestId: crypto.randomUUID(),
        action,
      });
      utils.computers.get.setData({ agentId }, updated);
      setConfirmation(null);
      setSubmission({ type: "ready" });
      void refresh();
    } catch {
      setSubmission({ type: "error", message: copy.lostSubmission });
      await refresh();
    } finally {
      busy.current = false;
    }
  }
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
    stale: initial.type === "list" ? list.isError : detail.isError,
    pending: submission.type === "pending",
    error: submission.type === "error" ? submission.message : null,
    creationLocked: createRequest.current !== null,
    name,
    providerId,
    providers: providerChoices.data
      ? {
          type: "loaded",
          stale: providerChoices.isError,
          items: providerChoices.data.items,
          after: providerAfter,
          nextAfter: providerChoices.data.nextAfter,
        }
      : providerChoices.isError
        ? { type: "error" }
        : { type: "loading" },
    confirmation: confirmation?.action ?? null,
    onName: (value) => {
      setName(value);
      createRequest.current = null;
    },
    onProvider: (value) => {
      setProviderId(value);
      createRequest.current = null;
    },
    onProviderPage: (after) => {
      if (
        busy.current ||
        (submission.type === "error" && createRequest.current !== null)
      ) {
        return;
      }
      setProviderAfter(after);
      setProviderId("");
    },
    onCreate: () => {
      void submitCreate();
    },
    onAction: (value) => {
      if (data.type !== "detail") {
        return;
      }
      const requestId = data.agent.computer.requestId;
      if (value === "start") {
        void submitAction(value, requestId);
      } else {
        setSubmission({ type: "ready" });
        setConfirmation({ action: value, requestId });
      }
    },
    onConfirm: () => {
      if (confirmation) {
        void submitAction(confirmation.action, confirmation.requestId);
      }
    },
    onCancel: () => {
      if (!busy.current) {
        setConfirmation(null);
      }
    },
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
