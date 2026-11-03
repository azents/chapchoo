"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import {
  Alert,
  Anchor,
  Button,
  Group,
  rem,
  SimpleGrid,
  Stack,
  Table,
  Text,
} from "@mantine/core";
import type { AgentsProps } from "../types";
import type { ReactElement } from "react";

export function Agents(props: AgentsProps): ReactElement {
  const { copy: c, data } = props;
  const listHref = `${props.basePath}/agents?team=${encodeURIComponent(data.teamId)}`;
  if (props.access !== "authorized") {
    return (
      <PageFailure
        title={c.agents}
        message={c.accessUnavailable}
        appearance={props.appearance}
        retryLabel={props.access === "signedOut" ? c.signIn : c.retry}
        onRetry={props.onRefresh}
      />
    );
  }
  return (
    <ApplicationShell
      basePath={props.basePath}
      appearance={props.appearance}
      contextLabel={c.agents}
      title={data.type === "detail" ? data.agent.name : c.agents}
      identityLabel={data.identity}
      mascotReplayLabel={c.replay}
      actions={
        <Button variant="subtle" onClick={props.onLogout}>
          {c.logout}
        </Button>
      }
      navigation={[
        {
          key: "agents",
          label: c.agents,
          active: true,
          href: listHref,
        },
        { key: "accounts", label: c.accounts, href: `${props.basePath}/` },
      ]}
    >
      <Group justify={data.type === "list" ? "flex-end" : "space-between"}>
        {data.type === "detail" ? (
          <Anchor href={listHref}>{c.back}</Anchor>
        ) : null}
        <Group gap="sm">
          {data.type === "list" ? (
            <Button
              component="a"
              href={`${props.basePath}/agents/new?team=${encodeURIComponent(data.teamId)}`}
            >
              {c.create}
            </Button>
          ) : null}
          {data.type === "detail" || props.stale ? (
            <Button
              variant="subtle"
              onClick={props.onRefresh}
              disabled={props.pending}
            >
              {data.type === "list" ? c.retryRead : c.retry}
            </Button>
          ) : null}
        </Group>
      </Group>
      {props.stale ? (
        <Alert color="yellow" role="status">
          {c.stale}
        </Alert>
      ) : null}
      {props.error && !props.confirmation ? (
        <Alert color="red" role="alert">
          {props.error}
        </Alert>
      ) : null}
      {data.type === "list" ? (
        <>
          {data.page.items.length === 0 ? (
            <Text c="dimmed">{c.noAgents}</Text>
          ) : (
            <Table.ScrollContainer minWidth={rem(320)}>
              <Table>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>{c.name}</Table.Th>
                    <Table.Th>{c.computerStatus}</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {data.page.items.map((item) => (
                    <Table.Tr key={item.id}>
                      <Table.Td>
                        <Anchor
                          href={`${props.basePath}/agents/${encodeURIComponent(item.id)}?team=${encodeURIComponent(data.teamId)}`}
                        >
                          {item.name}
                        </Anchor>
                      </Table.Td>
                      <Table.Td>
                        <Text size="sm">{c[item.computer.power]}</Text>
                        <Text size="xs" c="dimmed">
                          {c.phase}: {c[item.computer.phase]}
                        </Text>
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
          )}
          <Group>
            {data.after ? <Anchor href={listHref}>{c.first}</Anchor> : null}
            {data.page.nextAfter ? (
              <Anchor
                href={`${listHref}&after=${encodeURIComponent(data.page.nextAfter)}`}
              >
                {c.next}
              </Anchor>
            ) : null}
          </Group>
        </>
      ) : (
        <>
          {data.agent.computer.startDelayed ? (
            <Alert color="yellow" role="status">
              {c.delayed}
            </Alert>
          ) : null}
          <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="lg">
            {[
              { label: c.intent, value: c[data.agent.computer.intent] },
              { label: c.power, value: c[data.agent.computer.power] },
              { label: c.phase, value: c[data.agent.computer.phase] },
              { label: c.management, value: c[data.agent.computer.management] },
              { label: c.connector, value: c[data.agent.computer.connector] },
              { label: c.saved, value: c[data.agent.computer.saved] },
            ].map((item) => (
              <Stack key={item.label} gap="xs">
                <Text size="sm" c="dimmed">
                  {item.label}
                </Text>
                <Text fw={600}>{item.value}</Text>
              </Stack>
            ))}
          </SimpleGrid>
          <Text size="sm" c="dimmed">
            {data.agent.computer.observedAt === null
              ? c.neverObserved
              : `${c.observed}: ${props.formatTime(data.agent.computer.observedAt)}`}
          </Text>
          {data.agent.computer.power === "unknown" &&
          data.agent.computer.lastKnownPower !== "unknown" ? (
            <Text size="sm">
              {c.lastKnown}: {c[data.agent.computer.lastKnownPower]}
            </Text>
          ) : null}
          <Stack gap="md">
            {data.agent.computer.preparationRetryAvailable ? (
              <Group>
                <Button
                  variant="light"
                  disabled={props.pending || props.stale}
                  onClick={() => props.onAction("retryPreparation")}
                >
                  {c.retryPreparation}
                </Button>
              </Group>
            ) : null}
            {data.agent.computer.actions.map((item) => (
              <Group key={item.action}>
                <Button
                  variant={item.action === "delete" ? "outline" : "light"}
                  color={item.action === "delete" ? "red" : void 0}
                  onClick={() => props.onAction(item.action)}
                  disabled={props.pending || props.stale || !item.available}
                >
                  {c[item.action]}
                </Button>
                {!item.available && item.reason ? (
                  <Text size="sm" c="dimmed">
                    {c[item.reason]}
                  </Text>
                ) : null}
              </Group>
            ))}
          </Stack>
          {data.agent.computer.forceRemoveAvailable ? (
            <Group>
              <Button
                color="red"
                variant="subtle"
                disabled={props.pending || props.stale}
                onClick={() => props.onAction("force")}
              >
                {c.force}
              </Button>
            </Group>
          ) : null}
        </>
      )}
      <ConfirmationDialog
        opened={props.confirmation !== null}
        title={
          props.confirmation === "force"
            ? c.force
            : props.confirmation
              ? c[props.confirmation]
              : c.confirm
        }
        message={
          props.confirmation === "force"
            ? c.forceWarning
            : props.confirmation === "delete"
              ? c.deleteWarning
              : props.confirmation === "retryPreparation"
                ? c.retryPreparationWarning
                : c.stopWarning
        }
        confirmLabel={c.confirm}
        cancelLabel={c.cancel}
        onConfirm={props.onConfirm}
        onCancel={props.onCancel}
        destructive={
          props.confirmation === "delete" || props.confirmation === "force"
        }
        state={
          props.pending
            ? { type: "PENDING" }
            : props.error
              ? { type: "ERROR", message: props.error }
              : { type: "READY" }
        }
      />
    </ApplicationShell>
  );
}
