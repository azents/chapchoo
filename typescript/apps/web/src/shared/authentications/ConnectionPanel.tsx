import { InteractiveSurface } from "@chapchoo/ui/InteractiveSurface";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import {
  Alert,
  Anchor,
  Badge,
  Box,
  Button,
  Code,
  Divider,
  Group,
  Loader,
  Paper,
  PasswordInput,
  SegmentedControl,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { useFormatter } from "next-intl";
import { AuthenticationDialog } from "./AuthenticationDialog";
import { CodexMark } from "./CodexMark";
import classes from "./ConnectionPanel.module.css";
import { scopeKey } from "./types";
import type { ConnectionPanelProps } from "./types";
import type { ReactElement } from "react";

export function ConnectionPanel(props: ConnectionPanelProps): ReactElement {
  const { copy: c, dialog: d } = props;
  const format = useFormatter();
  const created = (instant: number): string =>
    format.dateTime(instant, { dateStyle: "medium", timeStyle: "short" });
  const activeDevice =
    d.type === "device" &&
    !["cancelled", "expired", "failed", "superseded"].includes(
      d.progress.state.type,
    );
  const cannotClose = props.busy || d.type === "uncertain" || activeDevice;
  const target = "target" in d ? d.target : null;
  const heading =
    d.type === "manage"
      ? target?.name || (target ? created(target.createdAt) : c.manage)
      : d.type === "rename"
        ? c.rename
        : d.type === "delete"
          ? c.remove
          : d.type === "key"
            ? target
              ? c.replace
              : c.apiKey
            : d.type === "uncertain"
              ? c.recover
              : c.connect;
  const state = d.type === "device" ? d.progress.state : null;
  return (
    <Stack gap="xl">
      {props.scopes.length > 1 ||
      !props.canManage ||
      props.showConnectAction !== false ? (
        <Group justify="space-between" wrap="wrap">
          {props.scopes.length > 1 ? (
            <SegmentedControl
              value={scopeKey(props.scope)}
              disabled={d.type !== "closed"}
              onChange={props.onScope}
              data={props.scopes.map((value) => ({
                value: scopeKey(value),
                label: value.type === "personal" ? c.personal : c.team,
              }))}
            />
          ) : (
            <span />
          )}
          {props.canManage && props.showConnectAction !== false ? (
            <Button
              onClick={() => props.onDialog({ type: "connect" })}
              disabled={props.unavailable || props.loading}
            >
              {c.connect}
            </Button>
          ) : !props.canManage ? (
            <Text size="sm" c="dimmed">
              {c.ownerOnly}
            </Text>
          ) : null}
        </Group>
      ) : null}
      {props.unavailable ? (
        <Alert color="yellow">
          <Stack gap="sm">
            <Text>{c.unavailable}</Text>
            <Button variant="subtle" onClick={props.onReload}>
              {c.retry}
            </Button>
          </Stack>
        </Alert>
      ) : props.loading ? (
        <Loader size="sm" />
      ) : props.items.length === 0 ? (
        <Stack className={classes.empty} align="center" gap="md">
          <CodexMark basePath={props.basePath} />
          <Title order={2}>{c.emptyTitle}</Title>
          <Text c="dimmed" ta="center">
            {c.emptyHint}
          </Text>
          {props.canManage ? (
            <Button onClick={() => props.onDialog({ type: "connect" })}>
              {c.connect}
            </Button>
          ) : null}
        </Stack>
      ) : (
        <SimpleGrid
          cols={{ base: 1, sm: props.selectable ? 1 : 2 }}
          spacing="xl"
        >
          {props.items.map((item) => (
            <Paper
              key={item.id}
              withBorder
              radius="lg"
              className={classes.card}
              data-selected={props.selectedId === item.id || void 0}
            >
              <Group justify="space-between" wrap="nowrap" align="start">
                <Group gap="sm" wrap="nowrap">
                  <CodexMark basePath={props.basePath} />
                  <Text size="sm" c="dimmed">
                    {c.codex}
                    {item.kind === "openaiApiKey" ? " · API" : ""}
                  </Text>
                </Group>
                {props.canManage && !props.selectable ? (
                  <Button
                    variant="subtle"
                    size="compact-sm"
                    onClick={() => props.onManage(item)}
                  >
                    {c.manage}
                  </Button>
                ) : null}
              </Group>
              <Title order={3} size="lg" mt="lg" className={classes.name}>
                {item.name || created(item.createdAt)}
              </Title>
              <Text size="xs" c="dimmed" mt="xs">
                {c.created}:{" "}
                <time dateTime={new Date(item.createdAt).toISOString()}>
                  {created(item.createdAt)}
                </time>
              </Text>
              <Text c="dimmed" size="sm" mt="xs" className={classes.name}>
                {item.email ??
                  (item.kind === "openaiApiKey" ? c.apiKey : c.chatgpt)}
              </Text>
              <Badge
                color={item.kind === "openaiApiKey" ? "yellow" : "green"}
                variant="light"
                mt="md"
              >
                {item.kind === "openaiApiKey" ? c.unverified : c.registered}
              </Badge>
              {props.selectable ? (
                <Box mt="lg">
                  <Button
                    variant={
                      props.selectedId === item.id ? "filled" : "default"
                    }
                    onClick={() => props.onSelect(item)}
                    disabled={props.selectedId === item.id}
                    leftSection={
                      props.selectedId === item.id ? (
                        <NavigationIcon name="check" size="sm" />
                      ) : (
                        void 0
                      )
                    }
                  >
                    {props.selectedId === item.id ? c.selected : c.choose}
                  </Button>
                </Box>
              ) : null}
            </Paper>
          ))}
        </SimpleGrid>
      )}
      {props.after || props.nextAfter ? (
        <Group justify="space-between">
          {props.after ? (
            <Button variant="subtle" onClick={() => props.onPage(null)}>
              {c.first}
            </Button>
          ) : (
            <span />
          )}
          {props.nextAfter ? (
            <Button
              variant="subtle"
              onClick={() => props.onPage(props.nextAfter)}
            >
              {c.more}
            </Button>
          ) : null}
        </Group>
      ) : null}
      <AuthenticationDialog
        mobile={props.mobile}
        opened={d.type !== "closed"}
        onClose={props.onClose}
        title={heading}
        locked={cannotClose}
        closeLabel={c.close}
        size="md"
      >
        <Stack gap="lg">
          {d.type === "connect" ? (
            <Stack gap="md">
              <InteractiveSurface
                className={classes.option}
                onClick={() =>
                  props.onDialog({ type: "deviceName", target: null })
                }
              >
                <Text component="span" fw={600}>
                  {c.chatgpt}
                </Text>
                <NavigationIcon name="forward" size="sm" />
              </InteractiveSurface>
              <InteractiveSurface
                className={classes.option}
                onClick={() => props.onDialog({ type: "key", target: null })}
              >
                <Text component="span" fw={600}>
                  {c.apiKey}
                </Text>
                <NavigationIcon name="forward" size="sm" />
              </InteractiveSurface>
            </Stack>
          ) : null}
          {d.type === "manage" ? (
            <Stack gap="sm">
              <Text c="dimmed">
                {d.target.email ??
                  (d.target.kind === "openaiApiKey" ? c.apiKey : c.chatgpt)}
              </Text>
              <Text size="sm">{c.storedOnly}</Text>
              <Button
                variant="default"
                onClick={() =>
                  props.onDialog({ type: "rename", target: d.target })
                }
              >
                {c.rename}
              </Button>
              {d.target.kind === "openaiApiKey" ? (
                <Button
                  variant="default"
                  onClick={() =>
                    props.onDialog({ type: "key", target: d.target })
                  }
                >
                  {c.replace}
                </Button>
              ) : (
                <Button
                  variant="default"
                  onClick={() =>
                    props.onDialog({ type: "deviceName", target: d.target })
                  }
                >
                  {c.reauthenticate}
                </Button>
              )}
              <Button
                color="red"
                variant="subtle"
                onClick={() =>
                  props.onDialog({ type: "delete", target: d.target })
                }
              >
                {c.remove}
              </Button>
            </Stack>
          ) : null}
          {d.type === "key" ||
          d.type === "rename" ||
          d.type === "deviceName" ? (
            <Box
              component="form"
              onSubmit={(event) => {
                event.preventDefault();
                props.onSubmit();
              }}
            >
              <Stack gap="lg">
                {d.type !== "key" || !d.target ? (
                  <TextInput
                    label={c.name}
                    value={props.name}
                    onChange={(event) =>
                      props.onName(event.currentTarget.value)
                    }
                    maxLength={512}
                    disabled={props.busy}
                  />
                ) : null}
                {d.type === "key" ? (
                  <>
                    <PasswordInput
                      label={c.key}
                      visibilityToggleButtonProps={{
                        "aria-label": c.keyVisibility,
                      }}
                      value={props.apiKey}
                      onChange={(event) =>
                        props.onKey(event.currentTarget.value)
                      }
                      autoComplete="off"
                      required
                      disabled={props.busy}
                    />
                    <Anchor
                      href="https://platform.openai.com/api-keys"
                      target="_blank"
                      rel="noopener noreferrer"
                      size="sm"
                    >
                      {c.issueKey}
                    </Anchor>
                    <Text size="sm" c="dimmed">
                      {c.billing}
                    </Text>
                    {d.target ? (
                      <Alert color="yellow">{c.replaceHint}</Alert>
                    ) : null}
                  </>
                ) : null}
                <Group justify="flex-end">
                  <Button
                    variant="default"
                    onClick={props.onClose}
                    disabled={props.busy}
                  >
                    {c.cancel}
                  </Button>
                  <Button
                    type="submit"
                    loading={props.busy}
                    disabled={d.type === "key" && !props.apiKey}
                  >
                    {d.type === "deviceName" ? c.connect : c.save}
                  </Button>
                </Group>
              </Stack>
            </Box>
          ) : null}
          {d.type === "delete" ? (
            <>
              <Text>{c.removeHint}</Text>
              <Group justify="flex-end">
                <Button
                  variant="default"
                  disabled={props.busy}
                  onClick={props.onClose}
                >
                  {c.cancel}
                </Button>
                <Button
                  color="red"
                  loading={props.busy}
                  onClick={props.onSubmit}
                >
                  {c.remove}
                </Button>
              </Group>
            </>
          ) : null}
          {d.type === "device" && state ? (
            <>
              {state.type === "pending" ? (
                <>
                  <Text>{c.deviceHint}</Text>
                  <Code block className={classes.code}>
                    {state.userCode}
                  </Code>
                  <Text size="sm" c="dimmed">
                    {c.remaining}{" "}
                    {Math.floor((props.remainingSeconds ?? 0) / 60)}:
                    {String((props.remainingSeconds ?? 0) % 60).padStart(
                      2,
                      "0",
                    )}
                  </Text>
                  <Button
                    component="a"
                    href={state.verificationUri}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {c.openDevice}
                  </Button>
                  <Button
                    variant="default"
                    loading={props.busy}
                    onClick={props.onDeviceCheck}
                  >
                    {c.check}
                  </Button>
                </>
              ) : state.type === "starting" ||
                state.type === "checking" ||
                state.type === "exchanging" ? (
                <Group>
                  <Loader size="sm" />
                  <Text role="status">{c.checking}</Text>
                </Group>
              ) : state.type === "expired" ||
                state.type === "cancelled" ||
                state.type === "failed" ||
                state.type === "superseded" ? (
                <Text>{c[state.type]}</Text>
              ) : (
                <Text>{c.uncertain}</Text>
              )}
              {activeDevice ? (
                <>
                  <Button
                    variant="subtle"
                    disabled={props.busy}
                    onClick={props.onDeviceStatus}
                  >
                    {c.retry}
                  </Button>
                  <Divider />
                  <Button
                    variant="default"
                    loading={props.busy}
                    onClick={props.onDeviceCancel}
                  >
                    {c.cancel}
                  </Button>
                </>
              ) : (
                <Group justify="flex-end">
                  <Button variant="default" onClick={props.onClose}>
                    {c.close}
                  </Button>
                  {state.type === "expired" || state.type === "failed" ? (
                    <Button onClick={props.onDeviceRestart}>{c.connect}</Button>
                  ) : null}
                </Group>
              )}
              {d.progress.providerFailure ? (
                <Alert color="yellow">
                  {d.progress.providerFailure === "uncertain"
                    ? c.uncertain
                    : c.unavailable}
                </Alert>
              ) : null}
            </>
          ) : null}
          {d.type === "uncertain" ? (
            <>
              <Text>{c.uncertain}</Text>
              <Button loading={props.busy} onClick={props.onRecover}>
                {c.recover}
              </Button>
            </>
          ) : null}
          {d.type === "recordUnavailable" ? (
            <>
              <Text>{c.recordUnavailable}</Text>
              <Button onClick={props.onClose}>{c.close}</Button>
            </>
          ) : null}
          {props.error ? (
            <Alert color="yellow" role="alert">
              {props.error}
            </Alert>
          ) : null}
        </Stack>
      </AuthenticationDialog>
    </Stack>
  );
}
