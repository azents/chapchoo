"use client";
import { AgentTheme } from "@chapchoo/ui/AgentTheme";
import {
  ApplicationShell,
  WorkspaceHeading,
} from "@chapchoo/ui/ApplicationShell";
import { CharacterArtwork } from "@chapchoo/ui/CharacterArtwork";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { NavigationIcon } from "@chapchoo/ui/NavigationIcon";
import { PageFailure } from "@chapchoo/ui/PageFailure";
import { computerOperation } from "@chapchoo/web-core/computer-messages";
import {
  agentChatPath,
  agentsPath,
  newAgentPath,
} from "@chapchoo/web-core/workspace";
import {
  Alert,
  Anchor,
  Box,
  Button,
  Group,
  Stack,
  Text,
  Title,
} from "@mantine/core";
import { profileStyle } from "../profilePalette";
import { AgentProfile } from "./AgentProfile";
import { ComputerControls } from "./ComputerControls";
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
        <AgentTheme
          palette={data.type === "detail" ? data.agent.paletteKey : null}
        >
          {data.type === "detail" ? (
            <Anchor href={listHref} mb="lg" display="inline-block">
              {c.back}
            </Anchor>
          ) : (
            <WorkspaceHeading
              title={c.agents}
              actions={
                data.page.items.length ? (
                  <Button
                    component="a"
                    href={newAgentPath(props.basePath, data.teamHandle)}
                  >
                    <NavigationIcon name="add" size="sm" />{" "}
                    {props.profileCopy.newAgent}
                  </Button>
                ) : null
              }
            />
          )}
        </AgentTheme>
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
      <AgentTheme
        palette={data.type === "detail" ? data.agent.paletteKey : null}
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
            {data.page.items.length ? (
              <Text size="xs" c="dimmed">
                • {props.profileCopy.companions}
              </Text>
            ) : null}
            {data.page.items.length === 0 ? (
              <Stack align="center" gap="lg" className={classes.emptyState}>
                <CharacterArtwork
                  basePath={props.basePath}
                  assetKey="01-retriever"
                  className={classes.emptyPortrait}
                />
                <Title order={2} size="h3" ta="center">
                  {c.noAgents}
                </Title>
                <Text size="sm" c="dimmed" ta="center">
                  {props.profileCopy.emptyHint}
                </Text>
                <Button
                  component="a"
                  href={newAgentPath(props.basePath, data.teamHandle)}
                  mt="sm"
                >
                  <NavigationIcon name="add" size="sm" />{" "}
                  {props.profileCopy.newAgent}
                </Button>
              </Stack>
            ) : (
              <Box className={classes.cards}>
                {data.page.items.map((item, index) => (
                  <Box
                    key={item.id}
                    component="a"
                    href={agentChatPath(
                      props.basePath,
                      data.teamHandle,
                      item.handle,
                    )}
                    aria-label={item.name}
                    className={classes.card}
                    style={profileStyle(item.paletteKey)}
                  >
                    <CharacterArtwork
                      basePath={props.basePath}
                      assetKey={item.profileImage.assetKey}
                      delayMs={(index % 6) * 180}
                      className={classes.cardPortrait}
                    />
                    <Box className={classes.cardIdentity}>
                      <Text className={classes.cardName}>{item.name}</Text>
                      <Text className={classes.previewHandle}>
                        @{item.handle}
                      </Text>
                      <Box className={classes.cardStatus}>
                        <span
                          className={classes.statusDot}
                          aria-hidden="true"
                        />
                        {c[item.computer.connector]}
                      </Box>
                      <Text size="xs" className={classes.cardFacts}>
                        <span
                          aria-label={`${c.power}: ${c[item.computer.power]}`}
                        >
                          {c[item.computer.power]}
                        </span>
                        {" · "}
                        <span
                          aria-label={`${c.phase}: ${computerOperation(item.computer, c).label}`}
                        >
                          {computerOperation(item.computer, c).label}
                        </span>
                      </Text>
                      <Box className={classes.open}>
                        {props.profileCopy.open}
                        <span className={classes.openArrow} aria-hidden="true">
                          <NavigationIcon name="forward" size="sm" />
                        </span>
                      </Box>
                    </Box>
                  </Box>
                ))}
              </Box>
            )}
            {data.after || data.page.nextAfter ? (
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
            ) : null}
            {data.page.items.length ? (
              <Box className={classes.listFooter}>
                <Anchor
                  size="sm"
                  c="dimmed"
                  href={newAgentPath(props.basePath, data.teamHandle)}
                >
                  <NavigationIcon name="add" size="sm" />{" "}
                  {props.profileCopy.addCompanion}
                </Anchor>
              </Box>
            ) : null}
          </>
        ) : (
          <>
            <AgentProfile {...props} agent={data.agent} />
            {props.authenticationControl}
            {data.agent.computer.startDelayed ? (
              <Alert color="yellow" role="status">
                {c.delayed}
              </Alert>
            ) : null}
            <ComputerControls {...props} agent={data.agent} />
            <Stack
              component="section"
              aria-label={c.delete}
              align="flex-end"
              mt="xl"
            >
              {data.agent.computer.actions
                .filter((item) => item.action === "delete")
                .map((item) => (
                  <Group key={item.action}>
                    <Button
                      color="red"
                      variant="subtle"
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
      </AgentTheme>
    </ApplicationShell>
  );
}
