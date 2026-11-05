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
  Box,
  Button,
  Divider,
  Group,
  Image,
  SimpleGrid,
  Stack,
  Text,
} from "@mantine/core";
import { profileStyle } from "../profilePalette";
import { AgentProfile } from "./AgentProfile";
import classes from "./ProfilePresentation.module.css";
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
      contentClassName={classes.content}
      header={
        data.type === "detail" ? (
          <Anchor href={listHref} mb="lg" display="inline-block">
            {c.back}
          </Anchor>
        ) : (
          void 0
        )
      }
      headingActions={
        data.type === "list" ? (
          <Button
            component="a"
            href={newAgentPath(props.basePath, data.teamHandle)}
          >
            <span aria-hidden="true">＋</span> {c.create}
          </Button>
        ) : (
          void 0
        )
      }
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
      {props.stale ? (
        <Group justify="flex-end">
          <Button
            variant="subtle"
            onClick={props.onRefresh}
            disabled={props.pending}
          >
            {data.type === "list" ? c.retryRead : c.retry}
          </Button>
        </Group>
      ) : null}
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
            <Box className={classes.cards}>
              {data.page.items.map((item) => (
                <Box
                  key={item.id}
                  component="a"
                  href={agentPath(props.basePath, data.teamHandle, item.handle)}
                  aria-label={item.name}
                  className={classes.card}
                  style={profileStyle(item.paletteKey)}
                >
                  <Image
                    src={`${props.basePath}/design-assets/${item.profileImage.assetKey}-216.webp`}
                    alt=""
                    className={classes.cardPortrait}
                  />
                  <Box className={classes.cardIdentity}>
                    <Text className={classes.cardName}>{item.name}</Text>
                    <Text className={classes.previewHandle}>
                      @{item.handle}
                    </Text>
                    <Box className={classes.cardStatus}>
                      <span className={classes.statusDot} aria-hidden="true" />
                      {c[item.computer.connector]}
                    </Box>
                    <Box className={classes.open}>
                      {props.profileCopy.open}
                      <span className={classes.openArrow} aria-hidden="true">
                        →
                      </span>
                    </Box>
                  </Box>
                </Box>
              ))}
            </Box>
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
          <Box className={classes.computerPanel}>
            <Group justify="space-between" mb="lg">
              <Text size="sm" c="dimmed">
                {c.computerStatus}
              </Text>
              <Button
                variant="subtle"
                size="compact-xs"
                disabled={props.pending}
                onClick={props.onRefresh}
              >
                {c.retry}
              </Button>
            </Group>
            <Stack gap="xs" mb="xl">
              <Text size="sm" c="dimmed">
                {c.power}
              </Text>
              <Text fw={600}>{c[data.agent.computer.power]}</Text>
            </Stack>
            <Group gap="sm" align="start">
              {data.agent.computer.preparationRetryAvailable ? (
                <Button
                  variant="light"
                  disabled={props.pending || props.stale}
                  onClick={() => props.onAction("retryPreparation")}
                >
                  {c.retryPreparation}
                </Button>
              ) : null}
              {data.agent.computer.actions
                .filter((item) => item.action !== "delete")
                .map((item) => (
                  <Stack key={item.action} gap="xs">
                    <Button
                      variant="light"
                      onClick={() => props.onAction(item.action)}
                      disabled={props.pending || props.stale || !item.available}
                    >
                      {c[item.action]}
                    </Button>
                    {!item.available && item.reason ? (
                      <Text size="xs" c="dimmed">
                        {c[item.reason]}
                      </Text>
                    ) : null}
                  </Stack>
                ))}
            </Group>
          </Box>
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
