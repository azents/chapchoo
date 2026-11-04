"use client";
import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import {
  agentPath,
  agentsPath,
  newAgentPath,
} from "@chapchoo/web-core/workspace";
import {
  Accordion,
  Alert,
  Anchor,
  Button,
  Divider,
  Group,
  Paper,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import { profilePalette } from "../profilePalette";
import { AgentPortrait } from "./AgentPortrait";
import { AgentProfile } from "./AgentProfile";
import type { AgentsProps } from "../types";
import type { ReactElement } from "react";

export function Agents(props: AgentsProps): ReactElement {
  const { copy: c, data } = props;
  const listHref = agentsPath(props.basePath, data.teamHandle);
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
              href={newAgentPath(props.basePath, data.teamHandle)}
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
            <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="lg">
              {data.page.items.map((item) => (
                <Paper
                  key={item.id}
                  component="a"
                  href={agentPath(props.basePath, data.teamHandle, item.handle)}
                  aria-label={item.name}
                  p="lg"
                  radius="lg"
                  withBorder
                  bg={profilePalette[item.paletteKey].surface}
                  c={profilePalette[item.paletteKey].foreground}
                  style={{ textDecoration: "none", minWidth: 0 }}
                >
                  <Stack gap="lg">
                    <Group wrap="nowrap" align="center">
                      <AgentPortrait
                        basePath={props.basePath}
                        assetKey={item.profileImage.assetKey}
                        paletteKey={item.paletteKey}
                      />
                      <Stack gap="xs" style={{ minWidth: 0 }}>
                        <Text
                          fw={600}
                          size="xl"
                          style={{ overflowWrap: "anywhere" }}
                        >
                          {item.name}
                        </Text>
                        <Text
                          size="sm"
                          c={profilePalette[item.paletteKey].emphasis}
                        >
                          @{item.handle}
                        </Text>
                      </Stack>
                    </Group>
                    <Group justify="space-between" gap="sm">
                      <Text size="sm">
                        {c.power}: {c[item.computer.power]}
                      </Text>
                      <Text size="xs">
                        {c.phase}: {c[item.computer.phase]}
                      </Text>
                    </Group>
                  </Stack>
                </Paper>
              ))}
            </SimpleGrid>
          )}
          <Group>
            {data.after ? <Anchor href={listHref}>{c.first}</Anchor> : null}
            {data.page.nextAfter ? (
              <Anchor
                href={`${listHref}?after=${encodeURIComponent(data.page.nextAfter)}`}
              >
                {c.next}
              </Anchor>
            ) : null}
          </Group>
        </>
      ) : (
        <>
          <AgentProfile {...props} agent={data.agent} />
          {data.agent.computer.startDelayed ? (
            <Alert color="yellow" role="status">
              {c.delayed}
            </Alert>
          ) : null}
          <Stack gap="xs">
            <Text size="sm" c="dimmed">
              {c.power}
            </Text>
            <Text fw={600}>{c[data.agent.computer.power]}</Text>
          </Stack>
          <Accordion>
            <Accordion.Item value="diagnostics">
              <Accordion.Control>{c.details}</Accordion.Control>
              <Accordion.Panel>
                <Stack gap="lg">
                  <SimpleGrid cols={{ base: 1, sm: 2 }} spacing="lg">
                    {[
                      { label: c.intent, value: c[data.agent.computer.intent] },
                      { label: c.phase, value: c[data.agent.computer.phase] },
                      {
                        label: c.management,
                        value: c[data.agent.computer.management],
                      },
                      {
                        label: c.connector,
                        value: c[data.agent.computer.connector],
                      },
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
                </Stack>
              </Accordion.Panel>
            </Accordion.Item>
          </Accordion>
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
            {data.agent.computer.actions
              .filter((item) => item.action !== "delete")
              .map((item) => (
                <Group key={item.action}>
                  <Button
                    variant="light"
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
          <Divider mt="xl" />
          <Stack component="section" aria-label={c.delete}>
            {data.agent.computer.actions
              .filter((item) => item.action === "delete")
              .map((item) => (
                <Group key={item.action}>
                  <Button
                    color="red"
                    variant="outline"
                    onClick={() => props.onAction(item.action)}
                    disabled={props.pending || props.stale || !item.available}
                  >
                    {c.delete}
                  </Button>
                  {!item.available && item.reason ? (
                    <Text size="sm" c="dimmed">
                      {c[item.reason]}
                    </Text>
                  ) : null}
                </Group>
              ))}
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
          </Stack>
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
