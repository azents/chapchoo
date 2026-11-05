import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import {
  ActionIcon,
  Box,
  Button,
  Group,
  Modal,
  Stack,
  Text,
  Title,
  Tooltip,
} from "@mantine/core";
import classes from "./ProfilePresentation.module.css";
import type { AgentsProps } from "../types";
import type { AgentView } from "@chapchoo/web-core/computer";
import type { ReactElement } from "react";

type Props = Pick<
  AgentsProps,
  | "copy"
  | "detailCopy"
  | "stale"
  | "pending"
  | "detailsOpened"
  | "onOpenDetails"
  | "onCloseDetails"
  | "onAction"
  | "onRefresh"
  | "formatTime"
> & { agent: AgentView };
/** Project backend eligibility without creating optimistic power or connection state. */
export function ComputerControls(props: Props): ReactElement {
  const { computer } = props.agent;
  const { copy: c, detailCopy: d } = props;
  const primary = computer.power === "stopped" ? "start" : "stop";
  const admission = computer.actions.find((item) => item.action === primary);
  const restart = computer.actions.find((item) => item.action === "restart");
  const locked = props.pending || props.stale;
  const status = c[computer.power];
  const blockedReason = admission?.reason;
  return (
    <>
      <Group
        className={classes.controlRow}
        justify="space-between"
        wrap="nowrap"
        gap="sm"
      >
        <Stack gap="xs" style={{ minWidth: 0 }}>
          <Text size="sm" fw={600}>
            {d.computer} · {status}
          </Text>
          <Group gap="xs" c="dimmed">
            <span className={classes.statusDot} aria-hidden="true" />
            <Text size="xs">{c[computer.connector]}</Text>
            {computer.phase !== "completed" ? (
              <Text size="xs">· {c[computer.phase]}</Text>
            ) : null}
          </Group>
        </Stack>
        <Group gap="sm" wrap="nowrap">
          <Button
            variant="light"
            disabled={
              locked || computer.power === "unknown" || !admission?.available
            }
            onClick={() => props.onAction(primary)}
            leftSection={<NavigationIcon name={primary} size="sm" />}
          >
            {c[primary]}
          </Button>
          <Tooltip label={c.restart}>
            <span>
              <ActionIcon
                size="2.75rem"
                variant="subtle"
                aria-label={c.restart}
                disabled={
                  locked || computer.power === "unknown" || !restart?.available
                }
                onClick={() => props.onAction("restart")}
              >
                <NavigationIcon name="restart" />
              </ActionIcon>
            </span>
          </Tooltip>
          <Tooltip label={c.details}>
            <ActionIcon
              size="2.75rem"
              variant="subtle"
              aria-label={c.details}
              onClick={props.onOpenDetails}
            >
              <NavigationIcon name="info" />
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>
      {blockedReason &&
      blockedReason !== "requiresRunning" &&
      blockedReason !== "requiresStopped" ? (
        <Text size="sm" c="dimmed">
          {c[blockedReason]}
        </Text>
      ) : null}
      {computer.preparationRetryAvailable ? (
        <Button
          variant="light"
          disabled={locked}
          onClick={() => props.onAction("retryPreparation")}
        >
          {c.retryPreparation}
        </Button>
      ) : null}
      <Modal
        opened={props.detailsOpened}
        onClose={props.onCloseDetails}
        title={c.details}
        centered
        size="sm"
        overlayProps={{ backgroundOpacity: 0.35, blur: 4 }}
      >
        <Stack gap="xl" pt="sm">
          <Stack gap="xs">
            <Title order={3} size="h4">
              {computer.power === "running"
                ? d.on
                : computer.power === "stopped"
                  ? d.off
                  : d.unknown}
            </Title>
            <Text size="xs" c="dimmed">
              {computer.observedAt === null
                ? c.neverObserved
                : `${c.observed}: ${props.formatTime(computer.observedAt)}`}
            </Text>
            {computer.power === "unknown" &&
            computer.lastKnownPower !== "unknown" ? (
              <Text size="sm">
                {c.lastKnown}: {c[computer.lastKnownPower]}
              </Text>
            ) : null}
          </Stack>
          <Stack gap="lg">
            {[
              {
                label: d.connection,
                value: c[computer.connector],
                hint: d.connectionHint,
              },
              {
                label: d.management,
                value: c[computer.management],
                hint: d.managementHint,
              },
              {
                label: d.storage,
                value: c[computer.saved],
                hint: d.storageHint,
              },
            ].map((item) => (
              <Stack gap="xs" key={item.label}>
                <Group justify="space-between">
                  <Text size="sm" fw={500}>
                    {item.label}
                  </Text>
                  <Text size="sm">{item.value}</Text>
                </Group>
                <Text size="xs" c="dimmed">
                  {item.hint}
                </Text>
              </Stack>
            ))}
            <Group justify="space-between">
              <Text size="sm" fw={500}>
                {d.operation}
              </Text>
              <Text size="sm">{c[computer.phase]}</Text>
            </Group>
          </Stack>
          <Box component="details">
            <Box component="summary" className={classes.diagnosticSummary}>
              {d.diagnostics}
            </Box>
            <Text size="xs" mt="sm">
              {c.intent}: {c[computer.intent]}
            </Text>
          </Box>
          <Group justify="flex-end">
            <Button variant="default" onClick={props.onCloseDetails}>
              {d.close}
            </Button>
            <Button
              variant="light"
              onClick={props.onRefresh}
              disabled={props.pending}
            >
              {c.retry}
            </Button>
          </Group>
        </Stack>
      </Modal>
    </>
  );
}
