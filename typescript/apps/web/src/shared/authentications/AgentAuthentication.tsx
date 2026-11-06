import { ResponsiveDialog } from "@chapchoo/ui/ResponsiveDialog";
import { Alert, Box, Button, Group, Stack, Text } from "@mantine/core";
import { CodexMark } from "./CodexMark";
import { ConnectionPanel } from "./ConnectionPanel";
import classes from "./ConnectionPanel.module.css";
import type { AgentAuthenticationProps } from "./useAgentAuthentication";
import type { ReactElement } from "react";
export function AgentAuthentication(
  props: AgentAuthenticationProps,
): ReactElement {
  const c = props.panel.copy;
  if (!props.authorized) {
    return <Alert color="yellow">{c.unavailable}</Alert>;
  }
  const locked =
    props.task.type === "pending" ||
    props.task.type === "uncertain" ||
    props.panel.dialog.type !== "closed";
  return (
    <>
      <Stack gap="xs" mt="xl">
        <Text size="sm">{c.execution}</Text>
        <Group
          className={classes.savedRow}
          justify="space-between"
          wrap="nowrap"
        >
          <Group wrap="nowrap" gap="md">
            <CodexMark basePath={props.basePath} />
            <Stack gap={0} style={{ minWidth: 0 }}>
              <Text size="sm">{c.codex}</Text>
              <Text size="xs" c="dimmed" className={classes.name}>
                {props.agent.authentication
                  ? props.agent.authentication.name ||
                    (props.agent.authentication.kind === "chatgpt"
                      ? c.chatgpt
                      : c.apiKey)
                  : c.unselected}
              </Text>
            </Stack>
          </Group>
          <Button
            variant="default"
            disabled={props.unavailable || props.task.type === "pending"}
            onClick={props.onOpen}
          >
            {c.change}
          </Button>
        </Group>
      </Stack>
      <ResponsiveDialog
        mobile={props.panel.mobile}
        opened={props.opened}
        onClose={props.onClose}
        title={c.choose}
        size="lg"
        locked={locked}
        closeLabel={c.close}
      >
        <Stack gap="lg">
          {props.task.type === "uncertain" ? (
            <Alert color="yellow">
              <Stack gap="md">
                <Text>{c.uncertain}</Text>
                <Button onClick={props.onRecover}>{c.recover}</Button>
              </Stack>
            </Alert>
          ) : (
            <>
              {props.task.type === "error" ? (
                <Alert color="yellow">{c.unavailable}</Alert>
              ) : null}
              <Box
                component="fieldset"
                disabled={props.task.type === "pending"}
                className={classes.fieldset}
              >
                <ConnectionPanel {...props.panel} basePath={props.basePath} />
              </Box>
            </>
          )}
        </Stack>
      </ResponsiveDialog>
    </>
  );
}
