"use client";

import { Button, Container, Stack, Title } from "@mantine/core";
import { AppFrame } from "./AppFrame";
import { ApplicationAccountActions } from "./ApplicationShell";
import { ErrorFeedback } from "./Feedback";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { ReactNode } from "react";

export interface PageFailureProps {
  title: string;
  message: string;
  retryLabel: string;
  onRetry: () => void;
  appearance?: ColorModeMenuProps;
  identityLabel?: string;
  logout?: { label: string; onLogout: () => void };
}
export function PageFailure(props: PageFailureProps): ReactNode {
  const actions = props.logout ? (
    <Button variant="subtle" size="xs" onClick={props.logout.onLogout}>
      {props.logout.label}
    </Button>
  ) : undefined;
  return (
    <AppFrame
      appearance={props.appearance}
      identityLabel={props.identityLabel}
      actions={actions}
    >
      {props.identityLabel || actions ? (
        <ApplicationAccountActions
          identityLabel={props.identityLabel}
          actions={actions}
        />
      ) : null}
      <Container component="main" size="sm" py="xl">
        <Stack>
          <Title order={1}>{props.title}</Title>
          <ErrorFeedback
            message={props.message}
            title={props.title}
            retryLabel={props.retryLabel}
            onRetry={props.onRetry}
          />
        </Stack>
      </Container>
    </AppFrame>
  );
}
