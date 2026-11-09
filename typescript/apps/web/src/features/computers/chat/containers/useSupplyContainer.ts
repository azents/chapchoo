"use client";
import { sameHarnessSupply } from "@chapchoo/web-core/conversation-supply";
import { harnessSupplyChoice } from "@chapchoo/web-core/conversation-types";
import { TRPCClientError } from "@trpc/client";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { supplyCopy } from "../supplyCopy";
import type { SupplyInput, SupplyProps } from "../supplyTypes";
import type { HarnessSupplyChoice } from "@chapchoo/web-core/conversation-types";

type Submission =
  | { type: "ready" | "saved" | "rejected" }
  | {
      type: "saving" | "uncertain";
      requestId: string;
      choice: HarnessSupplyChoice;
    };
function rejected(error: unknown): boolean {
  if (!(error instanceof TRPCClientError)) {
    return false;
  }
  const data: unknown = error.data;
  return (
    typeof data === "object" &&
    data !== null &&
    "code" in data &&
    typeof data.code === "string" &&
    [
      "BAD_REQUEST",
      "CONFLICT",
      "NOT_FOUND",
      "FORBIDDEN",
      "UNAUTHORIZED",
      "PRECONDITION_FAILED",
    ].includes(data.code)
  );
}
export function useSupplyContainer({
  agentId,
  applied,
  unavailable,
}: SupplyInput): SupplyProps {
  const copy = supplyCopy(useLocale());
  const [opened, setOpened] = useState(false);
  const [draft, setDraft] = useState<HarnessSupplyChoice | null>(null);
  const [submission, setSubmission] = useState<Submission>({ type: "ready" });
  const busy = useRef(false);
  const utils = trpc.useUtils();
  const query = trpc.conversations.supply.useQuery(
    { agentId },
    { enabled: opened, retry: false },
  );
  const mutation = trpc.conversations.saveSupply.useMutation({ retry: false });
  const saved = query.data;
  const current = draft ?? saved?.choice ?? { type: "managed" };
  const locked =
    unavailable ||
    query.isError ||
    query.isFetching ||
    submission.type === "saving" ||
    submission.type === "uncertain";
  useEffect(() => {
    if (
      submission.type === "uncertain" &&
      saved?.requestId === submission.requestId
    ) {
      setDraft(null);
      setSubmission({ type: "saved" });
    }
  }, [saved, submission]);
  async function save(): Promise<void> {
    if (
      busy.current ||
      locked ||
      !saved ||
      !harnessSupplyChoice.safeParse(current).success ||
      sameHarnessSupply(current, saved.choice)
    ) {
      return;
    }
    busy.current = true;
    const requestId = crypto.randomUUID();
    setSubmission({ type: "saving", requestId, choice: current });
    let reread = false;
    try {
      try {
        await mutation.mutateAsync({
          agentId,
          requestId,
          expectedRequestId: saved.requestId,
          choice: current,
        });
        setDraft(current);
        setSubmission({ type: "saved" });
        reread = true;
      } catch (error) {
        const knownRejection = rejected(error);
        setSubmission(
          knownRejection
            ? { type: "rejected" }
            : { type: "uncertain", requestId, choice: current },
        );
        reread = knownRejection;
      }
      // A subsequent read failure cannot erase an acknowledged save.
      if (reread) {
        await utils.conversations.supply
          .invalidate({ agentId })
          .catch(() => void 0);
      }
    } finally {
      busy.current = false;
    }
  }
  return {
    copy,
    opened,
    state: saved
      ? {
          type: "ready",
          saved,
          applied,
          draft: current,
          submission: submission.type,
          stale: query.isError,
          writable: !locked,
          saveable:
            !locked &&
            harnessSupplyChoice.safeParse(current).success &&
            !sameHarnessSupply(current, saved.choice),
        }
      : query.isError
        ? { type: "error" }
        : { type: "loading" },
    onToggle: () => setOpened((value) => !value),
    onMode: (mode) => {
      if (!locked) {
        setDraft(
          mode === "managed"
            ? { type: "managed" }
            : {
                type: "existing",
                path: current.type === "existing" ? current.path : "",
              },
        );
        setSubmission({ type: "ready" });
      }
    },
    onPath: (path) => {
      if (!locked) {
        setDraft({ type: "existing", path });
        setSubmission({ type: "ready" });
      }
    },
    onSave: () => {
      void save();
    },
    onRead: () => {
      void utils.conversations.supply.invalidate({ agentId });
    },
  };
}
