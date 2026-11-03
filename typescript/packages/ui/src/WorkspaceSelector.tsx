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
} from "@mantine/core";
import { NavigationIcon } from "./NavigationIcon";
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
        <NavigationIcon name={choice.kind} />
        <Text size="sm" className={classes.label}>
          {choice.label}
        </Text>
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
        <Button
          variant="subtle"
          justify="flex-start"
          className={classes.back}
          leftSection={<NavigationIcon name="back" />}
          onClick={props.onClose}
        >
          {props.copy.back}
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
            leftSection={<NavigationIcon name="add" />}
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
        {props.current ? <NavigationIcon name={props.current.kind} /> : null}
        <Text fw={600} size="sm" className={classes.label}>
          {props.current?.label ?? props.copy.choose}
        </Text>
        <NavigationIcon name="down" size="sm" className={classes.trailing} />
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
