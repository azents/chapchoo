"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import {
  definitiveAuthenticationRejection,
  withdrawAuthenticationSession,
} from "./failure";
import { catalogueOwnerMatches } from "./types";
import { useConnectionPanel } from "./useConnectionPanel";
import type { CatalogueSnapshot, ConnectionPanelProps } from "./types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { AuthenticationView } from "@chapchoo/web-core/external-authentication";
export interface AgentAuthenticationInput {
  basePath: string;
  agent: AgentView;
  snapshots: CatalogueSnapshot[];
  unavailable: boolean;
}
export type SelectionTask =
  | { type: "ready" }
  | { type: "pending" }
  | { type: "uncertain"; requestId: string; authenticationId: string }
  | { type: "error" };
export interface AgentAuthenticationProps extends AgentAuthenticationInput {
  authorized: boolean;
  panel: ConnectionPanelProps;
  opened: boolean;
  task: SelectionTask;
  onOpen: () => void;
  onClose: () => void;
  onRecover: () => void;
}
export function useAgentAuthentication(
  input: AgentAuthenticationInput,
): AgentAuthenticationProps {
  const auth = useAuth();
  const utils = trpc.useUtils();
  const [opened, setOpened] = useState(false);
  const [task, setTask] = useState<SelectionTask>({ type: "ready" });
  const pending = useRef(false);
  const fence = useRef(0);
  const user = auth.state.type === "AUTHENTICATED" ? auth.state.user.id : null;
  const authorized =
    input.snapshots.length > 0 &&
    input.snapshots.every((value) => catalogueOwnerMatches(value, user));
  useEffect(
    () => () => {
      fence.current += 1;
    },
    [input.agent.id, user],
  );
  async function refresh(revision: number): Promise<void> {
    await utils.computers.invalidate();
    if (revision === fence.current) {
      setTask({ type: "ready" });
      setOpened(false);
    }
  }
  async function select(value: AuthenticationView): Promise<void> {
    if (
      pending.current ||
      task.type === "uncertain" ||
      input.unavailable ||
      !authorized
    ) {
      return;
    }
    pending.current = true;
    setTask({ type: "pending" });
    const requestId = crypto.randomUUID();
    const revision = fence.current;
    try {
      const expected = input.agent.authentication;
      const result = await utils.client.authentications.selectAgent.mutate({
        requestId,
        agentId: input.agent.id,
        authenticationId: value.id,
        expected: expected
          ? {
              authenticationId: expected.authenticationId,
              requestId: expected.requestId,
            }
          : void 0,
      });
      if (revision !== fence.current) {
        return;
      }
      if (
        result.agentId !== input.agent.id ||
        result.authenticationId !== value.id
      ) {
        throw new Error("Selection result did not match the request");
      }
      await refresh(revision);
    } catch (error) {
      withdrawAuthenticationSession(error, auth.clearIdentity);
      if (revision === fence.current) {
        setTask(
          definitiveAuthenticationRejection(error)
            ? { type: "error" }
            : { type: "uncertain", requestId, authenticationId: value.id },
        );
        void utils.computers.invalidate();
      }
    } finally {
      pending.current = false;
    }
  }
  async function recover(): Promise<void> {
    if (task.type !== "uncertain" || pending.current || !authorized) {
      return;
    }
    const original = task;
    const revision = fence.current;
    pending.current = true;
    try {
      const history = await utils.client.authentications.requestResult.query({
        requestId: original.requestId,
      });
      if (revision !== fence.current) {
        return;
      }
      if (
        history.type === "accepted" &&
        history.operation === "select" &&
        history.result.type === "selection" &&
        history.result.agentId === input.agent.id &&
        history.result.authenticationId === original.authenticationId
      ) {
        await refresh(revision);
      }
    } finally {
      pending.current = false;
    }
  }
  const panel = useConnectionPanel({
    enabled: opened,
    snapshots: input.snapshots,
    selectedId: input.agent.authentication?.authenticationId,
    onSelect: (value) => void select(value),
  });
  return {
    ...input,
    authorized,
    panel,
    opened,
    task,
    onOpen: () => {
      if (!pending.current && !input.unavailable && authorized) {
        setOpened(true);
      }
    },
    onClose: () => {
      if (
        !pending.current &&
        task.type !== "uncertain" &&
        panel.dialog.type === "closed"
      ) {
        setOpened(false);
      }
    },
    onRecover: () => {
      void recover().catch((failure: unknown) => {
        withdrawAuthenticationSession(failure, auth.clearIdentity);
      });
    },
  };
}
