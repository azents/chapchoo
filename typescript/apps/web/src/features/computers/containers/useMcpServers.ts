import { useAuth } from "@chapchoo/web-core/auth-context";
import { useMediaQuery } from "@mantine/hooks";
import { TRPCClientError } from "@trpc/client";
import { useLocale } from "next-intl";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { trpc } from "@/shared/rpc/client";
import { draftFor, knownRejection, mcpConnection } from "../mcpDraft";
import {
  decodePendingWrite,
  pendingWriteKey,
  readPendingWrite,
  storePendingWrite,
  subscribePendingWrite,
} from "../mcpPendingWrite";
import { mcpServersCopy } from "../mcpServersCopy";
import type { McpDraft } from "../mcpDraft";
import type { PendingMcpWrite } from "../mcpPendingWrite";
import type {
  McpEditor,
  McpList,
  McpServersProps,
  McpWriteState,
} from "../mcpServerTypes";
import type { AppRouter } from "@/shared/server/router";
import type { IntegrationReceiptView } from "@chapchoo/web-core/computer";

export function useMcpServers({
  agentId,
}: {
  agentId: string;
}): McpServersProps {
  const auth = useAuth();
  const user = auth.state.type === "AUTHENTICATED" ? auth.state.user.id : null;
  const utils = trpc.useUtils();
  const client = utils.client.computers.integrations;
  const [cursors, setCursors] = useState<string[]>([]);
  const after = cursors.at(-1);
  const listInput = { agentId, after };
  const query = trpc.computers.integrations.list.useQuery(listInput, {
    enabled: user !== null,
    retry: false,
  });
  const [editor, setEditor] = useState<McpEditor | null>(null);
  const [progress, setProgress] = useState<"saving" | "checking" | null>(null);
  const storageKey = user === null ? null : pendingWriteKey(user, agentId);
  const raw = useSyncExternalStore(
    subscribePendingWrite,
    () => readPendingWrite(storageKey),
    () => null,
  );
  const pending = decodePendingWrite(raw);
  const write: McpWriteState = progress ?? (pending ? "uncertain" : "idle");
  const busy = useRef(false);
  const session = useRef(0);
  const revision = useRef(0);
  useEffect(
    () => () => {
      session.current += 1;
      revision.current += 1;
    },
    [agentId, user],
  );
  const copy = mcpServersCopy(useLocale());
  const mobile = useMediaQuery("(max-width: 48em)");
  const denied =
    query.error?.data?.code === "NOT_FOUND" ||
    query.error?.data?.code === "FORBIDDEN";
  const list: McpList =
    user === null || denied
      ? { type: "hidden" }
      : query.isError
        ? { type: "unavailable" }
        : query.data
          ? {
              type: "loaded",
              items: query.data.items,
              more: !!query.data.nextAfter,
              previous: !!after,
            }
          : { type: "loading" };
  const connection = editor && mcpConnection(editor.draft, editor.saved);
  const canTest = list.type === "loaded" && write === "idle" && !!connection;
  const canSave =
    canTest &&
    !!editor.draft.displayName.trim() &&
    editor.draft.displayName.trim().length <= 120;

  function changeDraft(delta: Partial<McpDraft>): void {
    if (busy.current || write !== "idle") {
      return;
    }
    revision.current += 1;
    setEditor(
      (current) =>
        current && {
          ...current,
          draft: { ...current.draft, ...delta },
          rejected: false,
          test: { type: "idle" },
        },
    );
  }

  async function testConnection(): Promise<void> {
    if (
      !editor ||
      !connection ||
      !canTest ||
      editor.test.type === "running" ||
      busy.current
    ) {
      return;
    }
    const version = ++revision.current;
    setEditor({ ...editor, test: { type: "running" } });
    try {
      const result = await client.test.mutate({
        agentId,
        integrationId: editor.saved?.id,
        connection,
      });
      if (version === revision.current) {
        setEditor((current) => current && { ...current, test: result });
      }
    } catch {
      if (version === revision.current) {
        setEditor(
          (current) => current && { ...current, test: { type: "unavailable" } },
        );
      }
    }
  }

  function acceptReceipt(
    expected: PendingMcpWrite,
    receipt: IntegrationReceiptView | null,
  ): boolean {
    if (
      !receipt ||
      receipt.action !== expected.action ||
      ("id" in expected && receipt.id !== expected.id)
    ) {
      return false;
    }
    if (receipt.action === "delete") {
      utils.computers.integrations.list.setData(
        listInput,
        (old) =>
          old && {
            ...old,
            items: old.items.filter((item) => item.id !== receipt.id),
          },
      );
    }
    setEditor(null);
    if (storageKey !== null) {
      storePendingWrite(storageKey, null);
    }
    if (receipt.action === "create") {
      setCursors([]);
    }
    void utils.computers.integrations.list.invalidate({ agentId });
    return true;
  }

  async function submit(remove: boolean): Promise<void> {
    if (
      !editor ||
      list.type !== "loaded" ||
      write !== "idle" ||
      storageKey === null ||
      busy.current
    ) {
      return;
    }
    const id = editor.saved?.id;
    const draft = connection && {
      displayName: editor.draft.displayName.trim(),
      connection,
    };
    const change = remove
      ? id &&
        editor.confirming && { type: "delete" as const, integrationId: id }
      : canSave &&
        draft &&
        (id
          ? { type: "update" as const, integrationId: id, draft }
          : { type: "create" as const, draft });
    if (!change) {
      return;
    }
    const requestId = crypto.randomUUID();
    const expected: PendingMcpWrite =
      change.type === "create"
        ? { requestId, action: "create" }
        : { requestId, action: change.type, id: change.integrationId };
    if (
      readPendingWrite(storageKey) !== null ||
      !storePendingWrite(storageKey, expected)
    ) {
      setEditor({ ...editor, rejected: true });
      return;
    }
    const current = session.current;
    busy.current = true;
    revision.current += 1;
    setProgress("saving");
    try {
      const receipt = await client.change.mutate({
        agentId,
        requestId,
        change,
      });
      if (current !== session.current || acceptReceipt(expected, receipt)) {
        return;
      }
    } catch (error) {
      if (current !== session.current) {
        return;
      }
      if (
        error instanceof TRPCClientError &&
        knownRejection((error as TRPCClientError<AppRouter>).data?.reason)
      ) {
        setEditor({ ...editor, rejected: true, test: { type: "idle" } });
        storePendingWrite(storageKey, null);
        return;
      }
    } finally {
      busy.current = false;
      if (current === session.current) {
        setProgress(null);
      }
    }
    setEditor({
      ...editor,
      draft: { ...editor.draft, bearer: "" },
      test: { type: "idle" },
    });
  }

  async function check(): Promise<void> {
    if (write !== "uncertain" || !pending || user === null || busy.current) {
      return;
    }
    busy.current = true;
    const current = session.current;
    setProgress("checking");
    try {
      const receipt = await client.result.mutate({
        agentId,
        requestId: pending.requestId,
      });
      if (current !== session.current || acceptReceipt(pending, receipt)) {
        return;
      }
    } catch {
      // A failed or empty read cannot settle an outstanding write.
    } finally {
      busy.current = false;
      if (current === session.current) {
        setProgress(null);
      }
    }
  }
  return {
    copy,
    list,
    mobile,
    editor: list.type === "hidden" ? null : editor,
    write,
    canTest,
    canSave,
    keepsToken: connection?.auth.type === "keep",
    onRetry: () =>
      void utils.computers.integrations.list.invalidate({ agentId }),
    onNext: () => {
      const next = query.data?.nextAfter;
      if (next && !cursors.includes(next)) {
        setCursors([...cursors, next]);
      }
    },
    onPrevious: () => setCursors(cursors.slice(0, -1)),
    onOpen: (saved) => {
      if (busy.current || write !== "idle" || list.type !== "loaded") {
        return;
      }
      revision.current += 1;
      setEditor({
        saved,
        draft: draftFor(saved),
        confirming: false,
        rejected: false,
        test: { type: "idle" },
      });
    },
    onClose: () => {
      if (busy.current || write === "saving") {
        return;
      }
      revision.current += 1;
      setEditor(null);
    },
    onDraft: changeDraft,
    onTest: () => void testConnection(),
    onSave: () => void submit(false),
    onConfirmDelete: (confirming) => {
      if (!busy.current && write === "idle") {
        setEditor((current) => current && { ...current, confirming });
      }
    },
    onDelete: () => void submit(true),
    onCheck: () => void check(),
  };
}
