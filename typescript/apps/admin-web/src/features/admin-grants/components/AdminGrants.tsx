"use client";

import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { ColorModeSelector } from "@chapchoo/ui/ColorModeSelector";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { ContentSection } from "@chapchoo/ui/ContentSection";
import { EmailField } from "@chapchoo/ui/fields";
import {
  Alert,
  Anchor,
  Badge,
  Button,
  Group,
  rem,
  Stack,
  Table,
  Text,
  TextInput,
} from "@mantine/core";
import type en from "../../../../messages/en-US.json";
import type { ColorModeSelectorProps } from "@chapchoo/ui/ColorModeSelector";
import type { ConfirmationState } from "@chapchoo/ui/ConfirmationDialog";
import type { ReactElement } from "react";

export type GrantPageKind = "signup" | "reset";
export interface GrantRow {
  id: string;
  email: string;
  issuedAt: string;
  expiresAt: string;
  status: "active" | "used" | "revoked" | "expired";
}
export type GrantViewState =
  | { type: "LOADING" }
  | { type: "ERROR"; message: string }
  | {
      type: "LOADED";
      items: GrantRow[];
      range: string;
      hasPrevious: boolean;
      hasNext: boolean;
    };
export interface AdminGrantsProps {
  appearance?: ColorModeSelectorProps;
  kind: GrantPageKind;
  basePath: string;
  identityLabel: string | null;
  signInLabel: string;
  state: GrantViewState;
  email: string;
  pending: boolean;
  error: string | null;
  issued: { url: string; email: string } | null;
  copied: boolean;
  revokeId: string | null;
  revokeState: ConfirmationState;
  text: (key: keyof typeof en.AdminGrants) => string;
  onEmail: (value: string) => void;
  onIssue: () => void;
  onCopy: () => void;
  onDismiss: () => void;
  onRevoke: (id: string) => void;
  onCancelRevoke: () => void;
  onConfirmRevoke: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onRefresh: () => void;
  onLogout: () => void;
}

export function AdminGrants(props: AdminGrantsProps): ReactElement {
  const { text, state } = props;
  const disabled = props.pending || state.type !== "LOADED";
  return (
    <ApplicationShell
      basePath={props.basePath}
      contextLabel={text("context")}
      title={text(props.kind === "signup" ? "signupTitle" : "resetTitle")}
      {...(props.identityLabel ? { identityLabel: props.identityLabel } : {})}
      navigation={[
        {
          key: "signup",
          label: text("signupTitle"),
          href: `${props.basePath}/`,
          active: props.kind === "signup",
        },
        {
          key: "reset",
          label: text("resetTitle"),
          href: `${props.basePath}/password-resets`,
          active: props.kind === "reset",
        },
      ]}
      actions={
        <>
          {props.appearance ? (
            <ColorModeSelector {...props.appearance} />
          ) : null}
          <Button
            variant="subtle"
            onClick={props.onLogout}
            disabled={props.pending}
          >
            {text("logout")}
          </Button>
        </>
      }
    >
      <ContentSection
        title={text("issueHeading")}
        description={text(
          props.kind === "signup" ? "signupDescription" : "resetDescription",
        )}
      >
        <form
          onSubmit={(event) => {
            event.preventDefault();
            props.onIssue();
          }}
        >
          <Group align="flex-end" gap="md" wrap="wrap">
            <EmailField
              label={text("email")}
              value={props.email}
              onValueChange={props.onEmail}
              required
              disabled={disabled}
              style={{ flex: 1, minWidth: 0 }}
            />
            <Button type="submit" loading={props.pending} disabled={disabled}>
              {text("issue")}
            </Button>
          </Group>
        </form>
      </ContentSection>
      {props.error ? (
        <Alert color="red" role="alert">
          {props.error}
        </Alert>
      ) : null}
      {props.issued ? (
        <Alert title={text("issuedLink")}>
          <Stack gap="sm">
            <Text size="sm">{props.issued.email}</Text>
            <TextInput
              aria-label={text("issuedLink")}
              value={props.issued.url}
              readOnly
            />
            <Text size="sm">{text("linkWarning")}</Text>
            <Group>
              <Button size="xs" onClick={props.onCopy}>
                {text(props.copied ? "copied" : "copy")}
              </Button>
              <Button size="xs" variant="subtle" onClick={props.onDismiss}>
                {text("dismiss")}
              </Button>
            </Group>
          </Stack>
        </Alert>
      ) : null}
      {state.type === "LOADING" ? (
        <Text role="status">{text("loading")}</Text>
      ) : null}
      {state.type === "ERROR" ? (
        <Alert color="red" role="alert">
          <Stack gap="sm">
            <Text size="sm">{state.message}</Text>
            <Button variant="light" onClick={props.onRefresh}>
              {text("retry")}
            </Button>
            <Anchor href={`${props.basePath}/login`}>
              {props.signInLabel}
            </Anchor>
          </Stack>
        </Alert>
      ) : null}
      {state.type === "LOADED" ? (
        <Stack gap="md">
          {state.items.length === 0 ? (
            <Stack gap="xs">
              <Text fw={600}>{text("emptyTitle")}</Text>
              <Text c="dimmed" size="sm">
                {text("emptyDescription")}
              </Text>
            </Stack>
          ) : (
            <Table.ScrollContainer minWidth={rem(720)}>
              <Table highlightOnHover withTableBorder verticalSpacing="md">
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>{text("email")}</Table.Th>
                    <Table.Th>{text("issuedAt")}</Table.Th>
                    <Table.Th>{text("expiresAt")}</Table.Th>
                    <Table.Th>{text("status")}</Table.Th>
                    <Table.Th>{text("actions")}</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {state.items.map((item) => (
                    <Table.Tr key={item.id}>
                      <Table.Td style={{ overflowWrap: "anywhere" }}>
                        {item.email}
                      </Table.Td>
                      <Table.Td>{item.issuedAt}</Table.Td>
                      <Table.Td>{item.expiresAt}</Table.Td>
                      <Table.Td>
                        <Badge
                          variant="light"
                          color={item.status === "active" ? "green" : "gray"}
                        >
                          {text(item.status)}
                        </Badge>
                      </Table.Td>
                      <Table.Td>
                        {item.status === "active" ? (
                          <Button
                            size="xs"
                            variant="subtle"
                            color="red"
                            disabled={props.pending}
                            onClick={() => props.onRevoke(item.id)}
                          >
                            {text("revoke")}
                          </Button>
                        ) : null}
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
          )}
          <Group justify="space-between" wrap="wrap">
            <Text size="sm" c="dimmed">
              {state.range}
            </Text>
            <Group gap="xs">
              <Button
                variant="default"
                disabled={!state.hasPrevious || props.pending}
                onClick={props.onPrevious}
              >
                {text("previous")}
              </Button>
              <Button
                variant="default"
                disabled={!state.hasNext || props.pending}
                onClick={props.onNext}
              >
                {text("next")}
              </Button>
            </Group>
          </Group>
        </Stack>
      ) : null}
      <ConfirmationDialog
        opened={props.revokeId !== null}
        title={text("revokeTitle")}
        message={text("revokeDescription")}
        confirmLabel={text("revoke")}
        cancelLabel={text("cancel")}
        state={props.revokeState}
        onConfirm={props.onConfirmRevoke}
        onCancel={props.onCancelRevoke}
        destructive
      />
    </ApplicationShell>
  );
}
