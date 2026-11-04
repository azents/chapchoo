"use client";
import { useCallback, useEffect, useRef, useState } from "react";
import { editHandle, renameHandleDraft, resolveHandle } from "./handle.js";
import type { HandleDraft, HandleKind } from "./handle.js";

export interface HandleProposal {
  draft: HandleDraft;
  status:
    | "idle"
    | "checking"
    | "available"
    | "invalid"
    | "occupied"
    | "exhausted"
    | "unavailable";
}
export interface HandleDraftControl extends HandleProposal {
  rename: (name: string) => void;
  edit: (candidate: string) => void;
  recheck: () => void;
}
/** Availability is advisory. The final create payload is never rewritten here. */
export function useHandleDraft(
  kind: HandleKind,
  activeIdentity: string | null,
  checker: (candidate: string, signal: AbortSignal) => Promise<boolean>,
): HandleDraftControl {
  const draft = useRef<HandleDraft>({
    source: "automatic",
    kind,
    base: kind,
    candidate: "",
  });
  const [state, setState] = useState<HandleProposal>({
    draft: draft.current,
    status: "idle",
  });
  const checking = useRef<AbortController | null>(null);
  const generation = useRef(0);
  const latest = useRef(checker);
  latest.current = checker;
  const check = useCallback((value: HandleDraft): void => {
    checking.current?.abort();
    const abort = new AbortController();
    checking.current = abort;
    const revision = ++generation.current;
    draft.current = value;
    setState({ draft: value, status: "checking" });
    void resolveHandle(value, {
      signal: abort.signal,
      check: (candidate, signal) => latest.current(candidate, signal),
    })
      .then((result) => {
        if (
          abort.signal.aborted ||
          revision !== generation.current ||
          result.status === "cancelled"
        )
          return;
        draft.current = result.draft;
        setState({ draft: result.draft, status: result.status });
      })
      .catch(() => {
        if (!abort.signal.aborted && revision === generation.current)
          setState({ draft: draft.current, status: "unavailable" });
      });
  }, []);
  useEffect(() => {
    if (activeIdentity && draft.current.candidate) check(draft.current);
    return () => {
      generation.current += 1;
      checking.current?.abort();
    };
  }, [activeIdentity, check]);
  return {
    ...state,
    rename: (name) => {
      if (activeIdentity && draft.current.source === "automatic") {
        try {
          check(renameHandleDraft(draft.current, name));
        } catch {
          checking.current?.abort();
          generation.current += 1;
          setState({ draft: draft.current, status: "unavailable" });
        }
      }
    },
    edit: (candidate) => {
      if (activeIdentity) check(editHandle(candidate));
    },
    recheck: () => {
      if (activeIdentity) check(draft.current);
    },
  };
}
