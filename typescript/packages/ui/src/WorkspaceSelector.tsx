"use client";

import {
  Alert,
  Box,
  Button,
  Divider,
  Group,
  Loader,
  Popover,
  Stack,
  Text,
  UnstyledButton,
  rem,
} from "@mantine/core";
import classes from "./WorkspaceSelector.module.css";
import type { ReactElement } from "react";

export interface WorkspaceChoice {
  key: string;
  kind: "personal" | "team" | "admin";
  label: string;
  href: string;
  selected: boolean;
}
export interface WorkspaceSelectorProps {
  current: WorkspaceChoice | null;
  choices: readonly WorkspaceChoice[];
  status: "ready" | "loading" | "error";
  mobile: boolean;
  opened: boolean;
  copy: {
    choose: string;
    teams: string;
    create: string;
    back: string;
    unavailable: string;
    retry: string;
  };
  onOpen: () => void;
  onClose: () => void;
  onCreate: () => void;
  onRetry: () => void;
  onSelect: (choice: WorkspaceChoice) => void;
}
export function WorkspaceIcon({
  kind,
}: {
  kind: WorkspaceChoice["kind"];
}): ReactElement {
  return (
    <svg
      width={rem(20)}
      height={rem(20)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {kind === "admin" ? (
        <>
          <path d="M12 3 4.5 6v5c0 5 3 8 7.5 10 4.5-2 7.5-5 7.5-10V6Z" />
          <path d="m8.5 12 2.5 2.5 4.5-5" />
        </>
      ) : kind === "personal" ? (
        <>
          <path d="m3 11 9-8 9 8M5 10v11h14V10M9 21v-7h6v7" />
        </>
      ) : (
        <>
          <circle cx="9" cy="8" r="3" />
          <path d="M2 21v-2a7 7 0 0 1 14 0v2M16 5a3 3 0 0 1 0 6M22 21v-2a7 7 0 0 0-4-6" />
        </>
      )}
    </svg>
  );
}
function Choice({
  choice,
  onSelect,
}: {
  choice: WorkspaceChoice;
  onSelect: WorkspaceSelectorProps["onSelect"];
}): ReactElement {
  return (
    <UnstyledButton
      component="a"
      href={choice.href}
      className={classes.choice}
      data-selected={choice.selected || void 0}
      aria-current={choice.selected ? "page" : void 0}
      onClick={() => onSelect(choice)}
    >
      <Group gap="sm" wrap="nowrap">
        <WorkspaceIcon kind={choice.kind} />
        <Text size="sm" className={classes.label}>
          {choice.label}
        </Text>
        {choice.selected ? (
          <Text
            component="span"
            aria-hidden="true"
            className={classes.trailing}
          >
            ✓
          </Text>
        ) : null}
      </Group>
    </UnstyledButton>
  );
}
export function WorkspaceChoices(props: WorkspaceSelectorProps): ReactElement {
  const personal = props.choices.filter((choice) => choice.kind === "personal");
  const teams = props.choices.filter((choice) => choice.kind === "team");
  const admin = props.choices.filter((choice) => choice.kind === "admin");
  return (
    <Stack gap="xs">
      {props.mobile ? (
        <Button variant="subtle" justify="flex-start" onClick={props.onClose}>
          ‹ {props.copy.back}
        </Button>
      ) : null}
      {personal.map((choice) => (
        <Choice key={choice.key} choice={choice} onSelect={props.onSelect} />
      ))}
      {props.status === "loading" ? (
        <Box p="sm">
          <Loader size="sm" />
        </Box>
      ) : props.status === "error" ? (
        <Alert color="yellow">
          <Stack gap="sm">
            <Text size="sm">{props.copy.unavailable}</Text>
            <Button variant="subtle" onClick={props.onRetry}>
              {props.copy.retry}
            </Button>
          </Stack>
        </Alert>
      ) : (
        <>
          <Text size="xs" c="dimmed" px="sm" mt="sm">
            {props.copy.teams}
          </Text>
          {teams.map((choice) => (
            <Choice
              key={choice.key}
              choice={choice}
              onSelect={props.onSelect}
            />
          ))}
          <Button
            variant="subtle"
            justify="flex-start"
            leftSection={<span aria-hidden="true">+</span>}
            onClick={props.onCreate}
          >
            {props.copy.create}
          </Button>
        </>
      )}
      {admin.length ? (
        <>
          <Divider my="xs" />
          {admin.map((choice) => (
            <Choice
              key={choice.key}
              choice={choice}
              onSelect={props.onSelect}
            />
          ))}
        </>
      ) : null}
    </Stack>
  );
}
function banner(props: WorkspaceSelectorProps): ReactElement {
  return (
    <UnstyledButton
      className={classes.trigger}
      onClick={props.opened ? props.onClose : props.onOpen}
      aria-label={props.copy.choose}
      aria-expanded={props.opened}
      onKeyDown={(event) => {
        if (event.key === "Escape" && props.opened) {
          props.onClose();
        }
      }}
    >
      <Group gap="sm" wrap="nowrap">
        {props.current ? <WorkspaceIcon kind={props.current.kind} /> : null}
        <Text fw={600} size="sm" className={classes.label}>
          {props.current?.label ?? props.copy.choose}
        </Text>
        <Text component="span" aria-hidden="true" className={classes.trailing}>
          ⌄
        </Text>
      </Group>
    </UnstyledButton>
  );
}
export function WorkspaceBanner(props: WorkspaceSelectorProps): ReactElement {
  return banner(props);
}
/** Presentation only: actual Team membership and destinations are app-owned. */
export function WorkspaceSelector(props: WorkspaceSelectorProps): ReactElement {
  const trigger = banner(props);
  return props.mobile ? (
    props.opened ? (
      <WorkspaceChoices {...props} />
    ) : (
      trigger
    )
  ) : (
    <Popover
      opened={props.opened}
      onChange={(opened) => (opened ? props.onOpen() : props.onClose())}
      position="bottom-start"
      width="target"
      trapFocus
      returnFocus
      shadow="sm"
    >
      <Popover.Target>{trigger}</Popover.Target>
      <Popover.Dropdown p="sm" className={classes.dropdown}>
        <WorkspaceChoices {...props} />
      </Popover.Dropdown>
    </Popover>
  );
}
