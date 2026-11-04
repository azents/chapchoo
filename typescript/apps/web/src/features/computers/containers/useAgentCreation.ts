"use client";
import { validAgentPalette } from "@chapchoo/web-core/agent-profile";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { computerCopy } from "@chapchoo/web-core/computer-messages";
import {
  editHandle,
  renameHandleDraft,
  resolveHandle,
} from "@chapchoo/web-core/handle";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useDocumentVisibility } from "@mantine/hooks";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { useAppearanceControl } from "@/shared/appearance/useAppearanceControl";
import { trpc } from "@/shared/rpc/client";
import { creationCopy } from "../creationCopy";
import { CreationRequest } from "../creationRequest";
import type { AgentCreationInput, AgentCreationProps } from "../creationTypes";
import type { HandleDraft } from "@chapchoo/web-core/handle";

export function useAgentCreation({
  initial,
  basePath,
}: AgentCreationInput): AgentCreationProps {
  const auth = useAuth();
  const visibility = useDocumentVisibility();
  const locale = useLocale();
  const copy = computerCopy(locale);
  const profileCopy = creationCopy(locale);
  const appearance = useAppearanceControl();
  const utils = trpc.useUtils();
  const client = utils.client;
  const [name, setName] = useState("");
  const [providerId, setProviderId] = useState("");
  const [after, setAfter] = useState<string | null>(null);
  const [imageAfter, setImageAfter] = useState<string | null>(null);
  const [image, setImage] = useState(initial.initialImage);
  const [paletteKey, setPalette] = useState(initial.initialPalette);
  const draft = useRef<HandleDraft>({
    source: "automatic",
    kind: "agent",
    base: "agent",
    candidate: "",
  });
  const [handle, setHandle] = useState<AgentCreationProps["handle"]>({
    draft: draft.current,
    status: "idle",
  });
  const checking = useRef<AbortController | null>(null);
  const generation = useRef(0);
  const request = useRef(new CreationRequest());
  const busy = useRef(false);
  const [submission, setSubmission] = useState<
    { type: "ready" } | { type: "pending" } | { type: "error"; message: string }
  >({ type: "ready" });
  useEffect(
    () => () => {
      generation.current += 1;
      checking.current?.abort();
    },
    [auth.state.type],
  );

  const providers = trpc.computers.providers.useQuery(after ? { after } : {}, {
    ...(!after ? initialQuery(initial.providers, initial.renderedAt) : {}),
    enabled: auth.state.type === "AUTHENTICATED" && visibility === "visible",
    retry: false,
    refetchInterval: 15_000,
    refetchIntervalInBackground: false,
    refetchOnWindowFocus: true,
  });
  const images = trpc.computers.images.useQuery(
    imageAfter ? { after: imageAfter } : {},
    {
      ...(!imageAfter ? initialQuery(initial.images, initial.renderedAt) : {}),
      enabled: auth.state.type === "AUTHENTICATED" && visibility === "visible",
      retry: false,
    },
  );

  function checkDraft(value: HandleDraft): void {
    if (request.current.locked) {
      return;
    }
    checking.current?.abort();
    const abort = new AbortController();
    checking.current = abort;
    const revision = ++generation.current;
    draft.current = value;
    setHandle({ draft: value, status: "checking" });
    void resolveHandle(value, {
      signal: abort.signal,
      check: (candidate, signal) =>
        client.computers.handleAvailable.query(
          { teamId: initial.teamId, candidate },
          { signal },
        ),
    })
      .then((result) => {
        if (
          abort.signal.aborted ||
          revision !== generation.current ||
          request.current.locked ||
          result.status === "cancelled"
        ) {
          return;
        }
        draft.current = result.draft;
        setHandle({ draft: result.draft, status: result.status });
      })
      .catch(() => {
        if (
          !abort.signal.aborted &&
          revision === generation.current &&
          !request.current.locked
        ) {
          setHandle({ draft: draft.current, status: "unavailable" });
        }
      });
  }

  const create = trpc.computers.create.useMutation({
    retry: false,
    onError: (error) => {
      request.current.reject(error.data?.reason);
      if (error.data?.reason === "DOMAIN_REASON_HANDLE_CONFLICT") {
        checkDraft(draft.current);
      }
    },
  });
  async function submit(): Promise<void> {
    if (
      busy.current ||
      (!request.current.locked &&
        (!name.trim() ||
          !providerId ||
          handle.status !== "available" ||
          !images.data ||
          images.isError ||
          providers.isError ||
          !providers.data?.items.some(
            (provider) => provider.id === providerId,
          )))
    ) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "pending" });
    checking.current?.abort();
    try {
      const payload = request.current.capture(
        {
          teamId: initial.teamId,
          name: name.trim(),
          providerId,
          handle: draft.current.candidate,
          profileImageId: image.id,
          paletteKey,
        },
        () => crypto.randomUUID(),
      );
      const result = await create.mutateAsync(payload);
      if (result.type === "removed") {
        request.current.removed();
        setSubmission({ type: "error", message: copy.removed });
        return;
      }
      void utils.computers.list.invalidate();
      window.location.assign(
        `${basePath}/agents/${encodeURIComponent(result.agent.id)}?team=${encodeURIComponent(result.agent.teamId)}`,
      );
    } catch {
      setSubmission({
        type: "error",
        message: request.current.locked
          ? copy.lostSubmission
          : copy.unavailable,
      });
    } finally {
      busy.current = false;
    }
  }
  function editable(): boolean {
    return !busy.current && !request.current.locked;
  }
  return {
    initial,
    basePath,
    copy,
    profileCopy,
    appearance,
    access:
      auth.state.type === "AUTHENTICATED"
        ? "authorized"
        : auth.state.type === "SIGNED_OUT"
          ? "signedOut"
          : "unavailable",
    pending: submission.type === "pending",
    error: submission.type === "error" ? submission.message : null,
    creationLocked: request.current.locked,
    name,
    providerId,
    handle,
    image,
    paletteKey,
    images: images.data ?? initial.images,
    imageAfter,
    imagesUnavailable: images.isError || !images.data,
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
      if (!editable()) {
        return;
      }
      setName(value);
      setSubmission({ type: "ready" });
      if (draft.current.source === "automatic") {
        checking.current?.abort();
        generation.current += 1;
        try {
          checkDraft(renameHandleDraft(draft.current, value));
        } catch {
          setHandle({ draft: draft.current, status: "unavailable" });
        }
      }
    },
    onHandle: (value) => {
      if (!editable()) {
        return;
      }
      setSubmission({ type: "ready" });
      checkDraft(editHandle(value));
    },
    onProvider: (value) => {
      if (editable()) {
        setProviderId(value);
      }
    },
    onProviderPage: (value) => {
      if (editable()) {
        setAfter(value);
        setProviderId("");
      }
    },
    onImage: (value) => {
      const selected = images.data?.items.find((item) => item.id === value);
      if (editable() && selected) {
        setImage(selected);
      }
    },
    onPalette: (value) => {
      if (editable() && validAgentPalette(value)) {
        setPalette(value);
      }
    },
    onImagePage: (value) => {
      if (editable()) {
        setImageAfter(value);
      }
    },
    onCreate: () => void submit(),
    onRefresh: () => {
      if (auth.state.type === "SIGNED_OUT") {
        window.location.assign(`${basePath}/login`);
      } else if (auth.state.type === "ERROR") {
        void auth.recheck();
      } else {
        void utils.computers.providers.invalidate();
        void utils.computers.images.invalidate();
        if (editable()) {
          checkDraft(draft.current);
        }
      }
    },
    onLogout: auth.logout,
  };
}
