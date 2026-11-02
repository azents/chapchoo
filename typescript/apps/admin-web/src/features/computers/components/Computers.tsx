"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import {
  Alert,
  Anchor,
  Badge,
  Button,
  Group,
  Paper,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import type { ComputersProps } from "../types";
import type { ReactElement } from "react";
export function Computers(props: ComputersProps): ReactElement {
  const { copy: c, data, basePath } = props;
  if (props.access !== "authorized") {
    return (
      <PageFailure
        title={c.inventory}
        message={c.accessUnavailable}
        appearance={props.appearance}
        retryLabel={props.access === "signedOut" ? c.signIn : c.retry}
        onRetry={props.onRefresh}
      />
    );
  }
  const path =
    data.type === "providers"
      ? `${basePath}/computer-providers`
      : `${basePath}/computer-providers/${encodeURIComponent(data.providerId)}`;
  return (
    <ApplicationShell
      basePath={basePath}
      appearance={props.appearance}
      title={data.type === "providers" ? c.providers : c.inventory}
      contextLabel={c.readOnly}
      mascotReplayLabel={c.replay}
      identityLabel={data.identity}
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
          active: true,
        },
        { key: "accounts", label: c.accounts, href: `${basePath}/` },
      ]}
    >
      <Group justify="space-between">
        <Text size="sm" c="dimmed">
          {c.readOnly}
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
      {data.type === "providers" ? (
        <>
          {data.page.items.length === 0 ? (
            <Text c="dimmed">{c.noProviders}</Text>
          ) : (
            data.page.items.map((item) => (
              <Paper key={item.id} withBorder p="md">
                <Group justify="space-between">
                  <Anchor
                    href={`${basePath}/computer-providers/${encodeURIComponent(item.id)}`}
                  >
                    {item.name}
                  </Anchor>
                  <Group gap="sm">
                    <Text>{c[item.management]}</Text>
                    <Badge color={item.canCreate ? "teal" : "gray"}>
                      {item.canCreate ? c.canCreate : c.cannotCreate}
                    </Badge>
                  </Group>
                </Group>
              </Paper>
            ))
          )}
        </>
      ) : (
        <>
          {data.page.items.length === 0 ? (
            <Text c="dimmed">{c.noComputers}</Text>
          ) : (
            data.page.items.map((item) => (
              <Paper key={item.id} withBorder p="lg">
                <Stack gap="md">
                  <Text fw={600} style={{ overflowWrap: "anywhere" }}>
                    {item.id}
                  </Text>
                  <Anchor
                    href={`${basePath}/computers/${encodeURIComponent(item.id)}`}
                  >
                    {c.resources}
                  </Anchor>
                  <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="md">
                    {[
                      { label: c.intent, value: c[item.intent] },
                      { label: c.power, value: c[item.power] },
                      { label: c.phase, value: c[item.phase] },
                      { label: c.management, value: c[item.management] },
                      { label: c.connector, value: c[item.connector] },
                      { label: c.saved, value: c[item.saved] },
                    ].map((fact) => (
                      <Stack key={fact.label} gap="xs">
                        <Text size="sm" c="dimmed">
                          {fact.label}
                        </Text>
                        <Text>{fact.value}</Text>
                      </Stack>
                    ))}
                  </SimpleGrid>
                  {item.power === "unknown" &&
                  item.lastKnownPower !== "unknown" ? (
                    <Text size="sm">
                      {c.lastKnown}: {c[item.lastKnownPower]}
                    </Text>
                  ) : null}
                  <Text size="sm" c="dimmed">
                    {item.observedAt === null
                      ? c.neverObserved
                      : `${c.observed}: ${props.formatTime(item.observedAt)}`}
                  </Text>
                  <Group>
                    {item.residual ? (
                      <Badge color="yellow">{c.residual}</Badge>
                    ) : null}
                    {item.cleanupCompletedAt !== null ? (
                      <Text size="sm">
                        {c.cleanupCompleted}:{" "}
                        {props.formatTime(item.cleanupCompletedAt)}
                      </Text>
                    ) : null}
                  </Group>
                </Stack>
              </Paper>
            ))
          )}
        </>
      )}
      <Group>
        {data.after ? <Anchor href={path}>{c.first}</Anchor> : null}
        {data.page.nextAfter ? (
          <Anchor
            href={`${path}?after=${encodeURIComponent(data.page.nextAfter)}`}
          >
            {c.next}
          </Anchor>
        ) : null}
      </Group>
    </ApplicationShell>
  );
}
