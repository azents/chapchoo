"use client";

import {
  invitationAction,
  invitationStatus,
  membershipRole,
  teamKind,
  teamRole,
} from "@chapchoo/web-core/browser";
import { useQueryClient } from "@tanstack/react-query";
import { useMessages } from "next-intl";
import { useEffect, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { accountError } from "../errors";
import type {
  MemberView,
  Submission,
  TeamAction,
  WorkspaceProps,
} from "../types";
import type { Team } from "@chapchoo/web-core/browser";

function role(value: Team["callerRole"]): MemberView["role"] {
  return value === teamRole.owner
    ? "OWNER"
    : value === teamRole.manager
      ? "MANAGER"
      : "MEMBER";
}

export function useWorkspaceContainer({
  teamId,
  basePath,
}: {
  teamId: string;
  basePath: string;
}): WorkspaceProps {
  const copy = useMessages().accounts;
  const client = useQueryClient();
  const utils = trpc.useUtils();
  const [name, setName] = useState("");
  const [handle, setHandle] = useState("");
  const [inviteEmail, setInviteEmail] = useState("");
  const [confirmation, setConfirmation] = useState<TeamAction | null>(null);
  const [submission, setSubmission] = useState<Submission>({ type: "READY" });
  const busy = useRef(false);
  const identity = trpc.auth.identity.useQuery(void 0, { retry: false });
  const enabled = Boolean(identity.data);
  const contexts = trpc.teams.list.useQuery(void 0, { enabled, retry: false });
  const received = trpc.invitations.received.useQuery(void 0, {
    enabled,
    retry: false,
  });
  const team = trpc.teams.get.useQuery(
    { teamId },
    { enabled: enabled && Boolean(teamId), retry: false },
  );
  const collaborative =
    enabled && Boolean(teamId) && team.data?.kind === teamKind.collaborative;
  const members = trpc.teams.members.useQuery(
    { teamId },
    { enabled: collaborative, retry: false },
  );
  const canManage = collaborative && team.data?.callerRole !== teamRole.member;
  const invitations = trpc.invitations.team.useQuery(
    { teamId },
    { enabled: canManage, retry: false },
  );
  async function invalidate(): Promise<void> {
    await Promise.all([
      utils.auth.identity.invalidate(),
      utils.teams.invalidate(),
      utils.invitations.invalidate(),
    ]);
  }
  const create = trpc.teams.create.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const update = trpc.teams.update.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const invite = trpc.invitations.invite.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const process = trpc.invitations.process.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const setRole = trpc.teams.setRole.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const remove = trpc.teams.removeMember.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const transfer = trpc.teams.transferOwnership.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const leave = trpc.teams.leave.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const deleteTeam = trpc.teams.delete.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  const logout = trpc.auth.logout.useMutation({
    retry: false,
    onError: (error) => fail(error.data?.reason ?? null),
  });
  function fail(reason: string | null): void {
    setSubmission({ type: "ERROR", message: accountError(reason, copy) });
    if (reason === "DOMAIN_REASON_INVALID_SESSION") {
      client.clear();
      window.location.replace(`${basePath}/login`);
    } else {
      void invalidate();
    }
  }
  const queryError =
    identity.error ??
    contexts.error ??
    received.error ??
    team.error ??
    members.error ??
    invitations.error;
  const invalidSession =
    queryError?.data?.reason === "DOMAIN_REASON_INVALID_SESSION";
  useEffect(() => {
    if (invalidSession) {
      client.clear();
      window.location.replace(`${basePath}/login`);
    }
  }, [invalidSession, client, basePath]);
  let state: WorkspaceProps["state"];
  if (queryError) {
    state = {
      type: "ERROR",
      message: accountError(queryError.data?.reason ?? null, copy),
    };
  } else if (
    !identity.data ||
    !contexts.data ||
    !received.data ||
    (teamId && !team.data) ||
    (collaborative && !members.data) ||
    (canManage && !invitations.data)
  ) {
    state = { type: "LOADING" };
  } else {
    state = {
      type: "LOADED",
      data: {
        identity: identity.data.email,
        contexts: contexts.data.items.map((item) => ({
          id: item.id,
          name: item.name,
          personal: item.kind === teamKind.personal,
          role: role(item.callerRole),
        })),
        received: received.data.items
          .filter((item) => item.status === invitationStatus.pending)
          .map((item) => ({
            id: item.id,
            teamId: item.teamId,
            email: item.recipientEmail,
            label: item.teamName,
          })),
        team:
          teamId && team.data
            ? {
                id: team.data.id,
                name: team.data.name,
                handle: team.data.handle,
                personal: team.data.kind === teamKind.personal,
                role: role(team.data.callerRole),
                members: (members.data?.items ?? []).map((item) => ({
                  id: item.userId,
                  email: item.email,
                  role: role(item.role),
                })),
                invitations: (invitations.data?.items ?? [])
                  .filter((item) => item.status === invitationStatus.pending)
                  .map((item) => ({
                    id: item.id,
                    teamId: item.teamId,
                    email: item.recipientEmail,
                    label: item.teamName,
                  })),
              }
            : null,
      },
    };
  }
  async function execute(operation: () => Promise<void>): Promise<void> {
    if (busy.current) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "PENDING" });
    try {
      await operation();
      setConfirmation(null);
      await invalidate();
      setSubmission({ type: "DONE", message: copy.saved });
    } catch {
      // Typed mutation callbacks handle failures without replaying the operation.
    } finally {
      busy.current = false;
    }
  }
  async function confirm(): Promise<void> {
    if (!confirmation) {
      return;
    }
    const action = confirmation;
    await execute(async () => {
      switch (action.type) {
        case "REMOVE":
          await remove.mutateAsync({ teamId, userId: action.userId });
          break;
        case "ROLE":
          await setRole.mutateAsync({
            teamId,
            userId: action.userId,
            role:
              action.role === "MANAGER"
                ? membershipRole.manager
                : membershipRole.member,
          });
          break;
        case "TRANSFER":
          await transfer.mutateAsync({ teamId, userId: action.userId });
          break;
        case "LEAVE":
          await leave.mutateAsync({ teamId });
          window.location.replace(`${basePath}/`);
          break;
        case "DELETE":
          await deleteTeam.mutateAsync({ teamId });
          window.location.replace(`${basePath}/`);
          break;
      }
    });
  }
  async function signOut(): Promise<void> {
    if (busy.current) {
      return;
    }
    busy.current = true;
    setSubmission({ type: "PENDING" });
    try {
      await logout.mutateAsync();
      client.clear();
      window.location.replace(`${basePath}/login`);
    } catch {
      // The mutation callback preserves the current session on uncertain failure.
    } finally {
      busy.current = false;
    }
  }
  return {
    basePath,
    copy,
    state,
    submission,
    name,
    handle,
    inviteEmail,
    confirmation,
    onName: setName,
    onHandle: setHandle,
    onInviteEmail: setInviteEmail,
    onCreate: () => {
      void execute(async () => {
        const result = await create.mutateAsync({ name, handle });
        setName("");
        setHandle("");
        window.location.assign(
          `${basePath}/teams/${encodeURIComponent(result.id)}`,
        );
      });
    },
    onRename: () => {
      void execute(async () => {
        await update.mutateAsync({
          teamId,
          ...(name ? { name } : {}),
          ...(handle ? { handle } : {}),
        });
        setName("");
        setHandle("");
      });
    },
    onInvite: () => {
      void execute(async () => {
        await invite.mutateAsync({ teamId, email: inviteEmail });
        setInviteEmail("");
      });
    },
    onInvitation: (id, accept) => {
      void execute(async () => {
        await process.mutateAsync({
          invitationId: id,
          action: accept ? invitationAction.accept : invitationAction.decline,
        });
      });
    },
    onAction: (action) => {
      if (!busy.current) {
        setSubmission({ type: "READY" });
        setConfirmation(action);
      }
    },
    onConfirm: () => {
      void confirm();
    },
    onCancel: () => {
      if (!busy.current) {
        setConfirmation(null);
        setSubmission({ type: "READY" });
      }
    },
    onRetry: () => {
      void invalidate();
    },
    onLogout: () => {
      void signOut();
    },
  };
}
