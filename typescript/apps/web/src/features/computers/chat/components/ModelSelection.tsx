"use client";

import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { SingleChoiceRows } from "@chapchoo/ui/SingleChoiceRows";
import { Alert, Button, Group, Stack, Text } from "@mantine/core";
import { TRPCClientError } from "@trpc/client";
import { useLocale } from "next-intl";
import { useRef, useState } from "react";
import { trpc } from "@/shared/rpc/client";
import { modelSelectionCopy } from "../modelSelectionCopy";
import type { ReactElement, Ref } from "react";

type Saving =
  | { state: "idle" | "rejected" }
  | { state: "saving" | "uncertain"; requestId: string };

type ModelTarget = {
  conversationId: string;
  launchOperationId: string;
  native: { snapshotId: string; nativeSessionId: string };
};

type Edit = {
  target: ModelTarget;
  expectedRequestId: string | null;
};

function matchesEdit(
  edit: Edit,
  target: ModelTarget,
  prior: string | null,
): boolean {
  return (
    target.conversationId === edit.target.conversationId &&
    target.launchOperationId === edit.target.launchOperationId &&
    target.native.snapshotId === edit.target.native.snapshotId &&
    target.native.nativeSessionId === edit.target.native.nativeSessionId &&
    prior === edit.expectedRequestId
  );
}

function rejected(error: unknown): boolean {
  if (!(error instanceof TRPCClientError)) {
    return false;
  }
  const data: unknown = error.data;
  return Boolean(
    data &&
    typeof data === "object" &&
    "code" in data &&
    typeof data.code === "string" &&
    [
      "BAD_REQUEST",
      "CONFLICT",
      "NOT_FOUND",
      "FORBIDDEN",
      "UNAUTHORIZED",
      "PRECONDITION_FAILED",
    ].includes(data.code),
  );
}

export function useModelSelection(
  agentId: string,
  conversationId: string | null,
  unavailable: boolean,
) {
  const [edit, setEdit] = useState<Edit | null>(null);
  const [draft, setDraft] = useState<string | null>(null);
  const [saving, setSaving] = useState<Saving>({ state: "idle" });
  const copy = modelSelectionCopy(useLocale());
  const actionInFlight = useRef(false);
  const utils = trpc.useUtils();
  const query = trpc.conversations.model.useQuery(
    { agentId },
    {
      enabled: !unavailable && Boolean(conversationId),
      retry: false,
      refetchInterval: 5_000,
    },
  );
  const mutation = trpc.conversations.saveModel.useMutation({ retry: false });
  const scoped =
    query.data?.target?.conversationId === conversationId ? query.data : null;
  const scopedTarget = scoped?.target;
  const target = scopedTarget?.native
    ? { ...scopedTarget, native: scopedTarget.native }
    : null;
  const stillCurrent =
    edit !== null &&
    target !== null &&
    matchesEdit(edit, target, scoped?.savedRequestId ?? null);
  const canEdit = target !== null && !query.isError && !unavailable;
  const choices = stillCurrent ? scoped?.options?.choices : void 0;
  const current = scoped?.current;
  const recovery =
    query.isError || unavailable ? null : scoped?.recovery?.model;
  const busy = saving.state === "saving";
  const locked = busy || saving.state === "uncertain";

  async function save(): Promise<void> {
    if (
      actionInFlight.current ||
      locked ||
      unavailable ||
      query.isError ||
      !edit ||
      !stillCurrent ||
      !draft ||
      !choices?.some((choice) => choice.id === draft) ||
      draft === current?.id
    ) {
      return;
    }
    const requestId = crypto.randomUUID();
    actionInFlight.current = true;
    setSaving({ state: "saving", requestId });
    try {
      const receipt = await mutation.mutateAsync({
        agentId,
        requestId,
        target: edit.target,
        modelId: draft,
        expectedRequestId: edit.expectedRequestId ?? void 0,
      });
      if (receipt.requestId !== requestId || !receipt.model) {
        setSaving({ state: "uncertain", requestId });
        return;
      }
      // The receipt is historical if the scope or accepted save has changed.
      utils.conversations.model.setData({ agentId }, (previous) => {
        const cached = previous?.target;
        return previous &&
          cached?.native &&
          matchesEdit(
            edit,
            { ...cached, native: cached.native },
            previous.savedRequestId ?? null,
          )
          ? {
              ...previous,
              current: receipt.model,
              savedRequestId: receipt.requestId,
              recovery: void 0,
            }
          : previous;
      });
      setEdit(null);
      setSaving({ state: "idle" });
      await utils.conversations.model.invalidate({ agentId });
    } catch (error) {
      setSaving(
        rejected(error)
          ? { state: "rejected" }
          : { state: "uncertain", requestId },
      );
    } finally {
      actionInFlight.current = false;
    }
  }

  async function checkSave(): Promise<void> {
    if (saving.state !== "uncertain" || actionInFlight.current) {
      return;
    }
    const { requestId } = saving;
    actionInFlight.current = true;
    setSaving({ state: "saving", requestId });
    try {
      const result = await utils.conversations.modelSaveResult.fetch({
        agentId,
        requestId,
      });
      if (result.receipt?.requestId === requestId && result.receipt.model) {
        setEdit(null);
        setSaving({ state: "idle" });
        await utils.conversations.model.invalidate({ agentId });
        return;
      }
    } catch {
      // A failed or missing receipt cannot establish a failed write.
    } finally {
      actionInFlight.current = false;
    }
    setSaving({ state: "uncertain", requestId });
  }

  function leave(): void {
    if (!locked && !actionInFlight.current) {
      setEdit(null);
      setDraft(null);
      setSaving({ state: "idle" });
    }
  }

  function enter(useCurrent: boolean): void {
    if (locked || !canEdit) {
      return;
    }
    setDraft(useCurrent ? (current?.id ?? null) : null);
    setEdit({
      target,
      expectedRequestId: scoped?.savedRequestId ?? null,
    });
  }

  return {
    copy,
    editing: edit !== null,
    canEdit,
    draft,
    choices,
    current,
    recovery,
    busy,
    locked,
    saving,
    leave,
    error: query.isError || (edit !== null && !stillCurrent),
    loading: query.isPending && query.isFetching,
    canSave:
      !unavailable &&
      !locked &&
      stillCurrent &&
      !query.isError &&
      !!draft &&
      draft !== current?.id &&
      choices?.some((choice) => choice.id === draft) === true,
    edit: () => enter(true),
    recover: () => enter(false),
    choose: setDraft,
    save: () => {
      void save();
    },
    check: () => {
      void checkSave();
    },
    refresh: () => {
      void query.refetch();
    },
  };
}

export type ModelSelectionControls = ReturnType<typeof useModelSelection>;

export function ModelSelectionEntry({
  controls,
  buttonRef,
}: {
  controls: ModelSelectionControls;
  buttonRef?: Ref<HTMLButtonElement>;
}): ReactElement {
  const { copy } = controls;
  return (
    <Button
      variant="default"
      fullWidth
      ref={buttonRef}
      onClick={controls.edit}
      disabled={!controls.canEdit}
      justify="space-between"
      rightSection={
        <Text size="sm" c="dimmed">
          {controls.current?.label ?? copy.unknown}
        </Text>
      }
    >
      {copy.model}
    </Button>
  );
}

export function ModelSelectionPage({
  controls,
  actionRef,
}: {
  controls: ModelSelectionControls;
  actionRef?: Ref<HTMLButtonElement>;
}): ReactElement {
  const { copy } = controls;
  return (
    <Stack gap="sm">
      <Group>
        <Button
          variant="subtle"
          leftSection={<NavigationIcon name="back" />}
          ref={controls.locked ? null : actionRef}
          disabled={controls.locked}
          onClick={controls.leave}
          data-autofocus={!controls.locked || void 0}
        >
          {copy.back}
        </Button>
      </Group>
      <Text size="sm" c="dimmed">
        {copy.apply}
      </Text>
      {controls.loading ? (
        <Text aria-busy="true" size="sm">
          …
        </Text>
      ) : controls.error || !controls.choices ? (
        <Group justify="space-between">
          <Text size="sm">{copy.unavailable}</Text>
          <Button size="xs" variant="subtle" onClick={controls.refresh}>
            {copy.refresh}
          </Button>
        </Group>
      ) : controls.choices.length === 0 ? (
        <Text size="sm">{copy.empty}</Text>
      ) : (
        <SingleChoiceRows
          label={copy.change}
          choices={controls.choices}
          value={controls.draft}
          disabled={controls.locked}
          onChange={controls.choose}
        />
      )}
      {controls.saving.state === "rejected" ? (
        <Alert color="red">{copy.rejected}</Alert>
      ) : null}
      {controls.saving.state === "uncertain" ? (
        <Alert color="yellow">
          <Stack gap="xs">
            {copy.uncertain}
            <Button
              ref={actionRef}
              variant="default"
              onClick={controls.check}
              data-autofocus
            >
              {copy.check}
            </Button>
          </Stack>
        </Alert>
      ) : null}
      <Group justify="flex-end">
        <Button
          variant="default"
          onClick={controls.leave}
          disabled={controls.locked}
        >
          {copy.cancel}
        </Button>
        <Button
          loading={controls.busy}
          onClick={controls.save}
          disabled={!controls.canSave}
        >
          {copy.save}
        </Button>
      </Group>
    </Stack>
  );
}
