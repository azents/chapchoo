"use client";

import { Alert, Button, Group, Modal, Stack, Text } from "@mantine/core";
import type { ReactNode } from "react";

export type ConfirmationState =
  { type: "READY" } | { type: "PENDING" } | { type: "ERROR"; message: string };

export interface ConfirmationDialogProps {
  opened: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  state?: ConfirmationState;
  destructive?: boolean;
}

/** Controlled confirmation with app-owned mutation state and explicit consent. */
export function ConfirmationDialog({
  opened,
  title,
  message,
  confirmLabel,
  cancelLabel,
  onConfirm,
  onCancel,
  state = { type: "READY" },
  destructive = false,
}: ConfirmationDialogProps): ReactNode {
  const pending = state.type === "PENDING";
  return (
    <Modal
      opened={opened}
      onClose={onCancel}
      title={title}
      centered
      closeOnClickOutside={!pending}
      closeOnEscape={!pending}
      withCloseButton={false}
      returnFocus
    >
      <Stack gap="lg">
        <Text size="sm" style={{ overflowWrap: "anywhere" }}>
          {message}
        </Text>
        {state.type === "ERROR" ? (
          <Alert color="red" role="alert">
            {state.message}
          </Alert>
        ) : null}
        <Group justify="flex-end" gap="sm">
          <Button variant="default" onClick={onCancel} disabled={pending}>
            {cancelLabel}
          </Button>
          <Button
            color={destructive ? "red" : undefined}
            onClick={onConfirm}
            loading={pending}
          >
            {confirmLabel}
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
