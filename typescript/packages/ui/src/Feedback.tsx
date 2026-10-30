"use client";

import {
  Alert,
  Button,
  Group,
  Loader,
  Stack,
  Text,
  Title,
} from "@mantine/core";
import type { ReactNode } from "react";

export interface LoadingFeedbackProps {
  message: string;
}

export interface ErrorFeedbackProps {
  title: string;
  message: string;
  retryLabel?: string;
  onRetry?: () => void;
}

export interface EmptyFeedbackProps {
  title: string;
  message?: string;
  action?: ReactNode;
}

export type FeedbackState =
  | ({ type: "LOADING" } & LoadingFeedbackProps)
  | ({ type: "ERROR" } & ErrorFeedbackProps)
  | ({ type: "EMPTY" } & EmptyFeedbackProps)
  | { type: "LOADED"; children: ReactNode };

export interface FeedbackProps {
  state: FeedbackState;
}

export function LoadingFeedback({ message }: LoadingFeedbackProps): ReactNode {
  return (
    <Group gap="sm" role="status" aria-live="polite" py="lg">
      <Loader size="sm" aria-hidden="true" />
      <Text size="sm" c="dimmed">
        {message}
      </Text>
    </Group>
  );
}

export function ErrorFeedback({
  title,
  message,
  retryLabel,
  onRetry,
}: ErrorFeedbackProps): ReactNode {
  return (
    <Alert color="red" title={title} role="alert">
      <Stack gap="sm">
        <Text size="sm">{message}</Text>
        {onRetry && retryLabel ? (
          <Button variant="outline" size="xs" onClick={onRetry} w="fit-content">
            {retryLabel}
          </Button>
        ) : null}
      </Stack>
    </Alert>
  );
}

export function EmptyFeedback({
  title,
  message,
  action,
}: EmptyFeedbackProps): ReactNode {
  return (
    <Stack gap="sm" py="lg" align="flex-start">
      <Title order={2} size="h4">
        {title}
      </Title>
      {message ? (
        <Text size="sm" c="dimmed">
          {message}
        </Text>
      ) : null}
      {action}
    </Stack>
  );
}

export function Feedback({ state }: FeedbackProps): ReactNode {
  switch (state.type) {
    case "LOADING":
      return <LoadingFeedback {...state} />;
    case "ERROR":
      return <ErrorFeedback {...state} />;
    case "EMPTY":
      return <EmptyFeedback {...state} />;
    case "LOADED":
      return state.children;
  }
}
