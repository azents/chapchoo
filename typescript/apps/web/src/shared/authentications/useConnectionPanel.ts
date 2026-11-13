"use client";
import { useAuth } from "@chapchoo/web-core/auth-context";
import { authenticationCopy } from "@chapchoo/web-core/authentication-messages";
import { initialQuery } from "@chapchoo/web-core/page-state";
import { useMediaQuery } from "@mantine/hooks";
import { useLocale, useNow } from "next-intl";
import { useEffect, useEffectEvent, useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import {
  authenticationTargetChanged,
  definitiveAuthenticationRejection,
  withdrawAuthenticationSession,
} from "./failure";
import { catalogueOwnerMatches, scopeKey } from "./types";
import type {
  ConnectionDialog,
  ConnectionPanelInput,
  ConnectionPanelProps,
} from "./types";
import type {
  AuthenticationAccepted,
  AuthenticationView,
  DeviceProgress,
} from "@chapchoo/web-core/external-authentication";

export function useConnectionPanel(
  input: ConnectionPanelInput,
): ConnectionPanelProps {
  const auth = useAuth();
  const mobile = useMediaQuery("(max-width: 48em)", false);
  const copy = authenticationCopy(useLocale());
  const utils = trpc.useUtils();
  const client = utils.client;
  const first = input.snapshots[0];
  if (!first) {
    throw new Error("An authorized catalogue snapshot is required");
  }
  const [scopeId, setScopeId] = useState(scopeKey(first.scope));
  const snapshot =
    input.snapshots.find((value) => scopeKey(value.scope) === scopeId) ?? first;
  const scope = snapshot.scope;
  const [localAfter, setAfter] = useState<string | null>(null);
  const after = input.onPage ? (input.after ?? null) : localAfter;
  const [dialog, setDialog] = useState<ConnectionDialog>(() =>
    input.managed
      ? { type: "manage", target: input.managed }
      : { type: "closed" },
  );
  const now = useNow(dialog.type === "device" ? { updateInterval: 1000 } : {});
  const [name, setName] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [clockOffset, setClockOffset] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const pending = useRef(false);
  const deviceTarget = useRef<AuthenticationView | null>(null);
  const fence = useRef(0);
  const userId =
    auth.state.type === "AUTHENTICATED" ? auth.state.user.id : null;
  const authorized = catalogueOwnerMatches(snapshot, userId);
  useEffect(
    () => () => {
      fence.current += 1;
    },
    [userId, scopeId],
  );
  const query = trpc.authentications.list.useQuery(
    { scope, ...(after ? { after } : {}) },
    {
      ...(authorized &&
      snapshot.page !== null &&
      after === (input.after ?? null)
        ? initialQuery(snapshot.page, snapshot.renderedAt)
        : {}),
      enabled: authorized && input.enabled !== false,
      retry: false,
    },
  );
  function transition(value: ConnectionDialog): void {
    if (
      pending.current ||
      !authorized ||
      dialog.type === "uncertain" ||
      (dialog.type === "device" &&
        !["cancelled", "expired", "failed", "superseded"].includes(
          dialog.progress.state.type,
        ))
    ) {
      return;
    }
    setApiKey("");
    setError(null);
    setName(
      "target" in value && value.target
        ? value.target.name
        : dialog.type === "connect" && value.type === "key"
          ? name
          : "",
    );
    setDialog(value);
  }
  async function complete(
    value: AuthenticationAccepted,
    revision: number,
  ): Promise<void> {
    if (revision !== fence.current) {
      return;
    }
    setApiKey("");
    setError(null);
    if (value.type !== "record" || !input.onSelect) {
      setDialog({ type: "closed" });
      input.onCloseManage?.();
    }
    try {
      await Promise.all([
        utils.authentications.list.invalidate(),
        utils.computers.invalidate(),
      ]);
    } catch (failure) {
      if (revision === fence.current) {
        withdrawAuthenticationSession(failure, auth.clearIdentity);
      }
    }
    if (revision !== fence.current) {
      return;
    }
    if (value.type === "record" && input.onSelect) {
      try {
        const saved = await client.authentications.get.query({
          scope,
          authenticationId: value.authenticationId,
        });
        if (revision !== fence.current) {
          return;
        }
        setDialog({ type: "closed" });
        input.onSelect(saved);
      } catch (failure) {
        if (revision !== fence.current) {
          return;
        }
        withdrawAuthenticationSession(failure, auth.clearIdentity);
        setDialog({
          type: "recordUnavailable",
          authenticationId: value.authenticationId,
        });
        return;
      }
    }
  }
  async function acceptProgress(
    value: DeviceProgress,
    revision: number,
  ): Promise<void> {
    if (revision !== fence.current) {
      return;
    }
    if (value.state.type === "registered") {
      await complete(
        { type: "record", authenticationId: value.state.authenticationId },
        revision,
      );
    } else {
      setClockOffset(value.observedAt - Date.now());
      setDialog({ type: "device", progress: value });
    }
  }
  async function run(work: (revision: number) => Promise<void>): Promise<void> {
    if (pending.current || !authorized) {
      return;
    }
    pending.current = true;
    setBusy(true);
    setError(null);
    const revision = fence.current;
    try {
      await work(revision);
    } catch (failure) {
      withdrawAuthenticationSession(failure, auth.clearIdentity);
      if (revision === fence.current) {
        setError(copy.unavailable);
      }
    } finally {
      pending.current = false;
      if (revision === fence.current) {
        setBusy(false);
      }
    }
  }
  async function submit(current = dialog): Promise<void> {
    if (
      !["key", "rename", "delete", "deviceName"].includes(current.type) ||
      !snapshot.canManage
    ) {
      return;
    }
    const requestId = crypto.randomUUID();
    await run(async (revision) => {
      let accepted = false;
      try {
        if (current.type === "key") {
          const value = current.target
            ? await client.authentications.replaceKey.mutate({
                scope,
                requestId,
                authenticationId: current.target.id,
                expectedRequestId: current.target.currentRequestId,
                key: apiKey,
              })
            : await client.authentications.registerKey.mutate({
                scope,
                requestId,
                name: name.trim(),
                key: apiKey,
              });
          accepted = true;
          await complete(value, revision);
        } else if (current.type === "rename") {
          const value = await client.authentications.rename.mutate({
            scope,
            requestId,
            authenticationId: current.target.id,
            expectedRequestId: current.target.currentRequestId,
            name: name.trim(),
          });
          accepted = true;
          await complete(value, revision);
        } else if (current.type === "delete") {
          const value = await client.authentications.delete.mutate({
            scope,
            requestId,
            authenticationId: current.target.id,
            expectedRequestId: current.target.currentRequestId,
          });
          if (revision !== fence.current) {
            return;
          }
          if (value.type === "inUse") {
            setError(copy.inUse);
          } else {
            accepted = true;
            await complete(value.result, revision);
          }
        } else if (current.type === "deviceName") {
          deviceTarget.current = current.target;
          const value = await client.authentications.startDevice.mutate({
            scope,
            requestId,
            name: current.target?.name ?? name.trim(),
            target: current.target
              ? {
                  authenticationId: current.target.id,
                  expectedRequestId: current.target.currentRequestId,
                }
              : void 0,
          });
          accepted = true;
          await acceptProgress(value, revision);
        }
      } catch (failure) {
        withdrawAuthenticationSession(failure, auth.clearIdentity);
        if (revision !== fence.current) {
          return;
        }
        if (!accepted && definitiveAuthenticationRejection(failure)) {
          if (
            "target" in current &&
            current.target &&
            authenticationTargetChanged(failure)
          ) {
            setApiKey("");
            setName("");
            deviceTarget.current = null;
            try {
              await Promise.all([
                utils.authentications.list.invalidate({ scope }),
                utils.computers.invalidate(),
              ]);
            } finally {
              if (revision === fence.current) {
                setDialog({ type: "closed" });
                input.onCloseManage?.();
              }
            }
            return;
          }
          setError(copy.unavailable);
          return;
        }
        setApiKey("");
        setDialog({
          type: "uncertain",
          requestId,
          operation:
            current.type === "delete"
              ? "deleted"
              : current.type === "deviceName"
                ? "device"
                : "record",
          attemptId: null,
        });
        setError(copy.uncertain);
      }
    });
  }
  async function deviceRead(advance: boolean): Promise<void> {
    if (dialog.type !== "device") {
      return;
    }
    const current = dialog.progress;
    await run(async (revision) => {
      try {
        const value = advance
          ? await client.authentications.pollDevice.mutate({
              attemptId: current.attemptId,
            })
          : await client.authentications.deviceStatus.query({
              attemptId: current.attemptId,
            });
        await acceptProgress(value, revision);
      } catch (failure) {
        withdrawAuthenticationSession(failure, auth.clearIdentity);
        if (revision === fence.current) {
          // An uncertain advance is not repeated. Status is the next operation.
          setDialog({
            type: "device",
            progress: { ...current, state: { type: "uncertain" } },
          });
          setError(copy.uncertain);
        }
      }
    });
  }
  const checkDevice = useEffectEvent((advance: boolean) => {
    void deviceRead(advance);
  });
  useEffect(() => {
    if (
      !authorized ||
      input.enabled === false ||
      busy ||
      dialog.type !== "device"
    ) {
      return;
    }
    const progress = dialog.progress;
    if (
      !["pending", "starting", "checking", "exchanging"].includes(
        progress.state.type,
      )
    ) {
      return;
    }
    const advance = progress.state.type === "pending";
    const serverNow = Date.now() + clockOffset;
    const next =
      progress.state.type === "pending"
        ? progress.state.nextCheckAt
        : serverNow + 1000;
    const deadline = Math.min(next, progress.expiresAt);
    const timer = window.setTimeout(
      () => checkDevice(advance),
      Math.max(250, deadline - serverNow),
    );
    return () => window.clearTimeout(timer);
  }, [authorized, input.enabled, busy, dialog, clockOffset]);
  async function cancelDevice(): Promise<void> {
    if (dialog.type !== "device") {
      return;
    }
    const attemptId = dialog.progress.attemptId;
    const requestId = crypto.randomUUID();
    await run(async (revision) => {
      try {
        await acceptProgress(
          await client.authentications.cancelDevice.mutate({
            attemptId,
            requestId,
          }),
          revision,
        );
      } catch (failure) {
        withdrawAuthenticationSession(failure, auth.clearIdentity);
        if (revision === fence.current) {
          setDialog({
            type: "uncertain",
            requestId,
            operation: "device",
            attemptId,
          });
        }
      }
    });
  }
  async function recover(): Promise<void> {
    if (dialog.type !== "uncertain") {
      return;
    }
    const current = dialog;
    await run(async (revision) => {
      const value = await client.authentications.requestResult.query({
        requestId: current.requestId,
      });
      if (revision !== fence.current) {
        return;
      }
      if (value.type === "notObserved") {
        if (current.attemptId !== null) {
          await acceptProgress(
            await client.authentications.deviceStatus.query({
              attemptId: current.attemptId,
            }),
            revision,
          );
          return;
        }
        setError(copy.notObserved);
        return;
      }
      if (value.result.type !== current.operation) {
        setError(copy.unavailable);
        return;
      }
      if (value.result.type === "device") {
        await acceptProgress(
          await client.authentications.deviceStatus.query({
            attemptId: value.result.attemptId,
          }),
          revision,
        );
      } else {
        await complete(value.result, revision);
      }
    });
  }
  return {
    display:
      auth.state.type === "AUTHENTICATED"
        ? (auth.state.user.display ?? null)
        : null,
    mobile,
    copy,
    scope,
    scopes: input.snapshots.map((value) => value.scope),
    selectedId: authorized ? (input.selectedId ?? null) : null,
    selectable: Boolean(input.onSelect),
    canManage: authorized && snapshot.canManage,
    items: authorized ? (query.data?.items ?? snapshot.page?.items ?? []) : [],
    nextAfter: query.data?.nextAfter ?? null,
    after,
    unavailable: query.isError || !authorized,
    loading: query.isPending,
    dialog: authorized ? dialog : { type: "closed" },
    name: authorized ? name : "",
    apiKey: authorized ? apiKey : "",
    busy,
    error,
    remainingSeconds:
      dialog.type === "device"
        ? Math.max(
            0,
            Math.ceil(
              (dialog.progress.expiresAt - (now.getTime() + clockOffset)) /
                1000,
            ),
          )
        : null,
    onName: (value) => {
      if (!pending.current) {
        setName(value);
      }
    },
    onKey: (value) => {
      if (!pending.current) {
        setApiKey(value);
      }
    },
    onScope: (value) => {
      if (!pending.current && dialog.type === "closed") {
        fence.current += 1;
        setScopeId(value);
        setAfter(null);
      }
    },
    onPage: (value) => {
      if (!pending.current) {
        if (input.onPage) {
          input.onPage(value);
        } else {
          setAfter(value);
        }
      }
    },
    onReload: () => void utils.authentications.list.invalidate({ scope }),
    onSelect: (value) => {
      if (
        authorized &&
        !pending.current &&
        !query.isError &&
        dialog.type === "closed"
      ) {
        input.onSelect?.(value);
      }
    },
    onManage: (value) => {
      if (authorized) {
        if (input.onManage) {
          input.onManage(value);
        } else {
          transition({ type: "manage", target: value });
        }
      }
    },
    onDialog: transition,
    onClose: () => {
      if (
        pending.current ||
        dialog.type === "uncertain" ||
        (dialog.type === "device" &&
          !["cancelled", "expired", "failed", "superseded"].includes(
            dialog.progress.state.type,
          ))
      ) {
        return;
      }
      transition({ type: "closed" });
      input.onCloseManage?.();
    },
    onSubmit: () => void submit(),
    onDeviceStart: (target) => void submit({ type: "deviceName", target }),
    onDeviceCheck: () => void deviceRead(true),
    onDeviceStatus: () => void deviceRead(false),
    onDeviceCancel: () => void cancelDevice(),
    onRecover: () => void recover(),
    onDeviceRestart: () =>
      void submit({ type: "deviceName", target: deviceTarget.current }),
  };
}
