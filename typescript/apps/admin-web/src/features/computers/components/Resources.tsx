"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import {
  Alert,
  Anchor,
  Button,
  Group,
  Paper,
  Stack,
  Text,
} from "@mantine/core";
import type { ResourcesProps } from "../resourceTypes";
import type { ReactElement } from "react";
export function Resources(props: ResourcesProps): ReactElement {
  const { copy: c, data, basePath, initial } = props;
  if (props.access !== "authorized") {
    return (
      <PageFailure
        title={c.resources}
        message={c.accessUnavailable}
        appearance={props.appearance}
        retryLabel={props.access === "signedOut" ? c.signIn : c.retry}
        onRetry={props.onRefresh}
      />
    );
  }
  const path = `${basePath}/computers/${encodeURIComponent(initial.selection.computerId)}`;
  const next = data.nextAfter
    ? new URLSearchParams({ ...data.nextAfter }).toString()
    : null;
  return (
    <ApplicationShell
      appearance={props.appearance}
      basePath={basePath}
      title={c.resources}
      contextLabel={c.readOnly}
      identityLabel={initial.identity}
      mascotReplayLabel={c.replay}
      actions={
        <Button variant="subtle" onClick={props.onLogout}>
          {c.logout}
        </Button>
      }
      navigation={[
        {
          key: "providers",
          label: c.providers,
          href: `${basePath}/computer-providers`,
        },
        { key: "accounts", label: c.accounts, href: `${basePath}/` },
      ]}
    >
      <Group justify="space-between">
        <Text style={{ overflowWrap: "anywhere" }}>
          {initial.selection.computerId}
        </Text>
        <Button variant="subtle" onClick={props.onRefresh}>
          {c.retry}
        </Button>
      </Group>
      {props.stale ? (
        <Alert color="yellow" role="status">
          {c.stale}
        </Alert>
      ) : null}
      {data.items.length === 0 ? <Text c="dimmed">{c.noResources}</Text> : null}
      {data.items.map((item) => (
        <Paper
          key={`${item.identity.kind}/${item.identity.namespace}/${item.identity.name}`}
          withBorder
          p="md"
        >
          <Stack gap="xs" style={{ overflowWrap: "anywhere" }}>
            <Text fw={600}>
              {item.identity.kind === "POD"
                ? c.pod
                : item.identity.kind === "PVC"
                  ? c.pvc
                  : c.bootstrapSecret}
            </Text>
            <Text>
              {c.namespace}: {item.identity.namespace}
            </Text>
            <Text>
              {c.resourceName}: {item.identity.name}
            </Text>
            <Text>
              {c.presence}: {c[item.presence]}
            </Text>
            <Text>
              {c.uid}: {item.uid ?? c.unknown}
            </Text>
            {item.executionId ? (
              <Text>
                {c.execution}: {item.executionId}
              </Text>
            ) : null}
            {item.observedAt !== null ? (
              <Text size="sm" c="dimmed">
                {props.formatTime(item.observedAt)}
              </Text>
            ) : null}
          </Stack>
        </Paper>
      ))}
      <Group>
        {initial.selection.after ? (
          <Anchor href={path}>{c.first}</Anchor>
        ) : null}
        {next ? <Anchor href={`${path}?${next}`}>{c.next}</Anchor> : null}
      </Group>
    </ApplicationShell>
  );
}
