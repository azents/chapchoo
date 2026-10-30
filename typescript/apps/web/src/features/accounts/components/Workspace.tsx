"use client";

import { ApplicationShell } from "@chapchoo/ui/ApplicationShell";
import { ConfirmationDialog } from "@chapchoo/ui/ConfirmationDialog";
import { ContentSection } from "@chapchoo/ui/ContentSection";
import {
  EmptyFeedback,
  ErrorFeedback,
  LoadingFeedback,
} from "@chapchoo/ui/Feedback";
import { EmailField } from "@chapchoo/ui/fields";
import {
  Alert,
  Anchor,
  Badge,
  Button,
  Group,
  Menu,
  Paper,
  rem,
  Stack,
  Table,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import classes from "./Workspace.module.css";
import type { Copy, WorkspaceProps } from "../types";
import type { ReactNode } from "react";

function roleLabel(role: string, c: Copy): string {
  return role === "OWNER" ? c.owner : role === "MANAGER" ? c.manager : c.member;
}

export function Workspace(props: WorkspaceProps): ReactNode {
  const { copy: c, state, submission } = props;
  if (state.type === "LOADING") {
    return (
      <Stack component="main" p="lg">
        <LoadingFeedback message={c.loading} />
      </Stack>
    );
  }
  if (state.type === "ERROR") {
    return (
      <Stack component="main" p="lg">
        <ErrorFeedback
          title={c.errorTitle}
          message={state.message}
          retryLabel={c.retry}
          onRetry={props.onRetry}
        />
        <Group>
          <Anchor href={`${props.basePath}/`}>{c.contexts}</Anchor>
          <Anchor href={`${props.basePath}/login`}>{c.login}</Anchor>
        </Group>
      </Stack>
    );
  }
  const { identity, contexts, received, team } = state.data;
  const pending = submission.type === "PENDING";
  const canManage = team !== null && !team.personal && team.role !== "MEMBER";
  const action = props.confirmation;
  const actionLabel =
    action?.type === "ROLE"
      ? action.role === "MANAGER"
        ? c.makeManager
        : c.makeMember
      : action?.type === "TRANSFER"
        ? c.transfer
        : action?.type === "REMOVE"
          ? c.remove
          : action?.type === "LEAVE"
            ? c.leaveTeam
            : c.deleteTeam;
  return (
    <ApplicationShell
      basePath={props.basePath}
      contextLabel={team?.personal ? c.personal : c.team}
      title={team?.name ?? c.contexts}
      identityLabel={identity}
      navigation={[
        {
          key: "home",
          label: c.contexts,
          href: `${props.basePath}/`,
          active: team === null,
        },
        ...contexts.map((item) => ({
          key: item.id,
          label: item.personal ? c.personal : item.name,
          href: `${props.basePath}/teams/${encodeURIComponent(item.id)}`,
          active: item.id === team?.id,
        })),
      ]}
      actions={
        <Button
          variant="subtle"
          size="xs"
          onClick={props.onLogout}
          disabled={pending}
        >
          {c.logout}
        </Button>
      }
    >
      {submission.type === "ERROR" && props.confirmation === null ? (
        <Alert color="red" role="alert">
          {submission.message}
        </Alert>
      ) : null}
      {submission.type === "DONE" ? (
        <Alert color="green" role="status">
          {submission.message}
        </Alert>
      ) : null}
      {team === null ? (
        <>
          <Text c="dimmed">{c.contextDescription}</Text>
          <Stack gap="sm">
            {contexts.map((item) => (
              <Paper key={item.id} className={classes.contextRow}>
                <Group justify="space-between" wrap="wrap">
                  <Stack gap="xs">
                    <Anchor
                      href={`${props.basePath}/teams/${encodeURIComponent(item.id)}`}
                      fw={600}
                      style={{ overflowWrap: "anywhere" }}
                    >
                      {item.personal ? c.personal : item.name}
                    </Anchor>
                    <Text size="sm" c="dimmed">
                      {item.personal
                        ? c.personalDescription
                        : roleLabel(item.role, c)}
                    </Text>
                  </Stack>
                  <Badge variant="light">
                    {item.personal ? c.personal : c.team}
                  </Badge>
                </Group>
              </Paper>
            ))}
          </Stack>
          <ContentSection title={c.receivedInvitations}>
            {received.length === 0 ? (
              <EmptyFeedback
                title={c.noInvitations}
                message={c.invitationDescription}
              />
            ) : (
              received.map((item) => (
                <Paper
                  key={item.id}
                  withBorder
                  p="md"
                  className={classes.invitation}
                >
                  <Group justify="space-between" wrap="wrap">
                    <Text fw={500} style={{ overflowWrap: "anywhere" }}>
                      {item.label}
                    </Text>
                    <Group gap="xs">
                      <Button
                        size="xs"
                        onClick={() => props.onInvitation(item.id, true)}
                        disabled={pending}
                      >
                        {c.accept}
                      </Button>
                      <Button
                        size="xs"
                        variant="default"
                        onClick={() => props.onInvitation(item.id, false)}
                        disabled={pending}
                      >
                        {c.decline}
                      </Button>
                    </Group>
                  </Group>
                </Paper>
              ))
            )}
          </ContentSection>
          <ContentSection title={c.createTeam}>
            <form
              onSubmit={(event) => {
                event.preventDefault();
                props.onCreate();
              }}
            >
              <Stack gap="sm">
                <TextInput
                  label={c.teamName}
                  value={props.name}
                  onChange={(event) => props.onName(event.currentTarget.value)}
                  required
                  disabled={pending}
                />
                <TextInput
                  label={c.handle}
                  description={c.handleDescription}
                  value={props.handle}
                  onChange={(event) =>
                    props.onHandle(event.currentTarget.value)
                  }
                  required
                  disabled={pending}
                />
                <Button type="submit" w="fit-content" loading={pending}>
                  {c.createTeam}
                </Button>
              </Stack>
            </form>
          </ContentSection>
        </>
      ) : team.personal ? (
        <EmptyFeedback title={c.personal} message={c.personalDescription} />
      ) : (
        <>
          <Group gap="sm">
            <Badge>{roleLabel(team.role, c)}</Badge>
            <Text size="sm" c="dimmed" style={{ overflowWrap: "anywhere" }}>
              @{team.handle} · {c.teamDescription}
            </Text>
          </Group>
          <ContentSection title={c.members}>
            <Table.ScrollContainer minWidth={rem(560)}>
              <Table verticalSpacing="md" className={classes.memberTable}>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>{c.email}</Table.Th>
                    <Table.Th>{c.role}</Table.Th>
                    <Table.Th>{c.actions}</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {team.members.map((member) => (
                    <Table.Tr key={member.id}>
                      <Table.Td style={{ overflowWrap: "anywhere" }}>
                        {member.email}
                      </Table.Td>
                      <Table.Td>{roleLabel(member.role, c)}</Table.Td>
                      <Table.Td>
                        {canManage &&
                        member.role !== "OWNER" &&
                        member.email !== identity &&
                        (team.role === "OWNER" || member.role === "MEMBER") ? (
                          <Menu position="bottom-end">
                            <Menu.Target>
                              <Button
                                size="xs"
                                variant="default"
                                disabled={pending}
                              >
                                {c.actions}
                              </Button>
                            </Menu.Target>
                            <Menu.Dropdown>
                              {team.role === "OWNER" ? (
                                <>
                                  <Menu.Item
                                    onClick={() =>
                                      props.onAction({
                                        type: "ROLE",
                                        userId: member.id,
                                        role:
                                          member.role === "MANAGER"
                                            ? "MEMBER"
                                            : "MANAGER",
                                        label: member.email,
                                      })
                                    }
                                  >
                                    {member.role === "MANAGER"
                                      ? c.makeMember
                                      : c.makeManager}
                                  </Menu.Item>
                                  <Menu.Item
                                    onClick={() =>
                                      props.onAction({
                                        type: "TRANSFER",
                                        userId: member.id,
                                        label: member.email,
                                      })
                                    }
                                  >
                                    {c.transfer}
                                  </Menu.Item>
                                </>
                              ) : null}
                              <Menu.Item
                                color="red"
                                onClick={() =>
                                  props.onAction({
                                    type: "REMOVE",
                                    userId: member.id,
                                    label: member.email,
                                  })
                                }
                              >
                                {c.remove}
                              </Menu.Item>
                            </Menu.Dropdown>
                          </Menu>
                        ) : (
                          <Text size="sm" c="dimmed">
                            —
                          </Text>
                        )}
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
          </ContentSection>
          {canManage ? (
            <>
              <ContentSection title={c.settings}>
                <form
                  onSubmit={(event) => {
                    event.preventDefault();
                    props.onRename();
                  }}
                >
                  <Stack gap="sm">
                    <TextInput
                      label={c.teamName}
                      placeholder={team.name}
                      value={props.name}
                      onChange={(event) =>
                        props.onName(event.currentTarget.value)
                      }
                      maxLength={120}
                      disabled={pending}
                    />
                    <TextInput
                      label={c.handle}
                      placeholder={team.handle}
                      description={c.handleDescription}
                      value={props.handle}
                      onChange={(event) =>
                        props.onHandle(event.currentTarget.value)
                      }
                      maxLength={80}
                      disabled={pending}
                    />
                    <Button
                      w="fit-content"
                      type="submit"
                      loading={pending}
                      disabled={!props.name && !props.handle}
                    >
                      {c.save}
                    </Button>
                  </Stack>
                </form>
              </ContentSection>
              <ContentSection
                title={c.invite}
                description={c.inviteDescription}
              >
                <form
                  onSubmit={(event) => {
                    event.preventDefault();
                    props.onInvite();
                  }}
                >
                  <Stack gap="sm">
                    <EmailField
                      label={c.email}
                      value={props.inviteEmail}
                      onValueChange={props.onInviteEmail}
                      required
                      disabled={pending}
                    />
                    <Button w="fit-content" type="submit" loading={pending}>
                      {c.invite}
                    </Button>
                  </Stack>
                </form>
                <Title order={3} size="h5">
                  {c.pendingInvitations}
                </Title>
                {team.invitations.length === 0 ? (
                  <Text size="sm" c="dimmed">
                    {c.noInvitations}
                  </Text>
                ) : (
                  team.invitations.map((item) => (
                    <Text
                      size="sm"
                      key={item.id}
                      style={{ overflowWrap: "anywhere" }}
                    >
                      {item.email}
                    </Text>
                  ))
                )}
              </ContentSection>
            </>
          ) : null}
          <ContentSection title={c.membership}>
            {team.role === "OWNER" ? (
              <>
                <Text size="sm" c="dimmed">
                  {c.ownerLeaveDescription}
                </Text>
                <Button
                  color="red"
                  variant="outline"
                  w="fit-content"
                  disabled={pending}
                  onClick={() =>
                    props.onAction({ type: "DELETE", label: team.name })
                  }
                >
                  {c.deleteTeam}
                </Button>
              </>
            ) : (
              <Button
                color="red"
                variant="outline"
                w="fit-content"
                disabled={pending}
                onClick={() =>
                  props.onAction({ type: "LEAVE", label: team.name })
                }
              >
                {c.leaveTeam}
              </Button>
            )}
          </ContentSection>
        </>
      )}
      <ConfirmationDialog
        opened={props.confirmation !== null}
        title={c.confirmTitle}
        message={`${actionLabel} · ${props.confirmation?.label ?? ""} — ${props.confirmation?.type === "TRANSFER" ? c.transferWarning : props.confirmation?.type === "DELETE" ? c.deleteWarning : c.confirmDescription}`}
        confirmLabel={c.confirm}
        cancelLabel={c.cancel}
        onConfirm={props.onConfirm}
        onCancel={props.onCancel}
        destructive={
          props.confirmation?.type !== "ROLE" &&
          props.confirmation?.type !== "TRANSFER"
        }
        state={
          pending
            ? { type: "PENDING" }
            : submission.type === "ERROR"
              ? { type: "ERROR", message: submission.message }
              : { type: "READY" }
        }
      />
    </ApplicationShell>
  );
}
