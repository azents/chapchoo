import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { computerOperation } from "@chapchoo/web-core/computer-messages";
import {
  ActionIcon,
  Alert,
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
> & { agent: AgentView; onDetailsExit?: () => void };
/** Project backend eligibility without creating optimistic power or connection state. */
export function ComputerControls(props: Props): ReactElement {
  const { computer } = props.agent;
  const { copy: c, detailCopy: d } = props;
  const stopping =
    computer.operation === "stop" &&
    (computer.phase === "retiring" || computer.phase === "uncertain");
  const primary = computer.power === "stopped" || stopping ? "start" : "stop";
  const admission = computer.actions.find((item) => item.action === primary);
  const restart = computer.actions.find((item) => item.action === "restart");
  const locked = props.pending || props.stale;
  const status = c[computer.power];
  const operation = computerOperation(computer, c);
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
              <Text size="xs">· {operation.label}</Text>
            ) : null}
          </Group>
        </Stack>
        <Group gap="sm" wrap="nowrap">
          <Button
            variant="light"
            disabled={
              locked ||
              (computer.power === "unknown" && !stopping) ||
              !admission?.available
            }
            onClick={() => props.onAction(primary)}
            leftSection={<NavigationIcon name={primary} size="sm" />}
          >
            {stopping && computer.intent === "running"
              ? c.startAfterStop
              : c[primary]}
          </Button>
          <Tooltip
            label={
              !restart?.available && restart?.reason
                ? `${c.restart}: ${c[restart.reason]}`
                : c.restart
            }
            events={{ hover: true, focus: true, touch: true }}
          >
            <span
              role="group"
              aria-label={c.restart}
              tabIndex={!restart?.available && restart?.reason ? 0 : void 0}
              className={classes.controlHelp}
            >
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
      {operation.hint ? (
        <Text size="sm" c="dimmed">
          {operation.hint}
        </Text>
      ) : null}
      {blockedReason &&
      !(blockedReason === "conflicting" && (operation.hint || stopping)) &&
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
        returnFocus={!props.onDetailsExit}
        onExitTransitionEnd={props.onDetailsExit}
        onClose={props.onCloseDetails}
        title={c.details}
        centered
        size="sm"
        overlayProps={{ backgroundOpacity: 0.35, blur: 4 }}
      >
        <Stack gap="xl" pt="sm">
          {props.stale ? (
            <Alert color="yellow" role="status">
              {c.stale}
            </Alert>
          ) : null}
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
              <Text size="sm">{operation.label}</Text>
            </Group>
            {operation.hint ? (
              <Text size="sm" c="dimmed">
                {operation.hint}
              </Text>
            ) : null}
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
