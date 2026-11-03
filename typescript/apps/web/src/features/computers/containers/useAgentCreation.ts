"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useDocumentVisibility } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useRef, useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import type { AgentCreationInput, AgentCreationProps } from "../creationTypes";

export function useAgentCreation({
  initial,
  basePath,
}: AgentCreationInput): AgentCreationProps {
  const auth = useAuth();
  const visibility = useDocumentVisibility();
  const copy = computerCopy(useLocale());
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const [name, setName] = useState("");
  const [providerId, setProviderId] = useState("");
  const [after, setAfter] = useState<string | null>(null);
  const [submission, setSubmission] = useState<
    { type: "ready" } | { type: "pending" } | { type: "error"; message: string }
  >({ type: "ready" });
  const busy = useRef(false);
  const request = useRef<{
    requestId: string;
    teamId: string;
    name: string;
    providerId: string;
  } | null>(null);
  const providers = trpc.computers.providers.useQuery(after ? { after } : {}, {
    ...(!after ? initialQuery(initial.providers, initial.renderedAt) : {}),
    enabled: auth.state.type === "AUTHENTICATED" && visibility === "visible",
    retry: false,
    refetchInterval: 15_000,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
  });
  const create = trpc.computers.create.useMutation({
    retry: false,
    onError: (error) => {
      if (
        error.data?.reason === "DOMAIN_REASON_INVALID_INPUT" ||
        error.data?.reason === "DOMAIN_REASON_INVALID_TRANSITION" ||
        error.data?.reason === "DOMAIN_REASON_PERMISSION_DENIED"
      ) {
        request.current = null;
      }
    },
  });
  async function submit(): Promise<void> {
    if (busy.current) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "pending" });
    const payload = request.current ?? {
      requestId: crypto.randomUUID(),
      teamId: initial.teamId,
      name: name.trim(),
      providerId,
    };
    request.current = payload;
    try {
      const result = await create.mutateAsync(payload);
      if (result.type === "removed") {
        setSubmission({ type: "error", message: copy.removed });
        request.current = null;
        return;
      }
      void utils.computers.list.invalidate();
      window.location.assign(
        `${basePath}/agents/${encodeURIComponent(result.agent.id)}?team=${encodeURIComponent(result.agent.teamId)}`,
      );
    } catch {
      setSubmission({ type: "error", message: copy.lostSubmission });
    } finally {
      busy.current = false;
    }
  }
  return {
    initial,
    basePath,
    copy,
    appearance,
    access:
      auth.state.type === "AUTHENTICATED"
        ? "authorized"
        : auth.state.type === "SIGNED_OUT"
          ? "signedOut"
          : "unavailable",
    pending: submission.type === "pending",
    error: submission.type === "error" ? submission.message : null,
    creationLocked: request.current !== null,
    name,
    providerId,
    providers: providers.data
      ? {
          type: "loaded",
          items: providers.data.items,
          nextAfter: providers.data.nextAfter,
          after,
          stale: providers.isError,
        }
      : providers.isError
        ? { type: "error" }
        : { type: "loading" },
    onName: (value) => {
      setName(value);
      request.current = null;
    },
    onProvider: (value) => {
      setProviderId(value);
      request.current = null;
    },
    onProviderPage: (value) => {
      if (busy.current || request.current !== null) {
        return;
      }
      setAfter(value);
      setProviderId("");
    },
    onCreate: () => void submit(),
    onRefresh: () => {
      if (auth.state.type === "SIGNED_OUT") {
        window.location.assign(`${basePath}/login`);
      } else if (auth.state.type === "ERROR") {
        void auth.recheck();
      } else {
        void utils.computers.providers.invalidate();
      }
    },
    onLogout: auth.logout,
  };
}
