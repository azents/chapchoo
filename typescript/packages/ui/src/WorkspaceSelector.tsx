"use client";

import {
  Alert,
  Box,
  Button,
  Divider,
  Group,
  Loader,
  Stack,
  Text,
  UnstyledButton,
} from "@mantine/core";
import { NavigationIcon } from "./NavigationIcon";
import { DropdownButton } from "./DropdownButton";
import classes from "./WorkspaceSelector.module.css";
import type { ReactElement, Ref } from "react";

export type WorkspaceChoice = {
  key: string;
  label: string;
  href: string;
  selected: boolean;
} & ({ kind: "team"; handle: string } | { kind: "personal" | "admin" });
export interface WorkspaceSelectorProps {
  current: WorkspaceChoice | null;
  choices: readonly WorkspaceChoice[];
  status: "ready" | "loading" | "error";
  mobile: boolean;
  opened: boolean;
  triggerRef?: Ref<HTMLButtonElement>;
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
        <Stack gap={0} className={classes.label}>
          <Text size="sm">{choice.label}</Text>
          {choice.kind === "team" ? (
            <Text size="xs" c="dimmed">
              @{choice.handle}
            </Text>
          ) : null}
        </Stack>
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
/** Presentation only: actual Team membership and destinations are app-owned. */
export function WorkspaceSelector(props: WorkspaceSelectorProps): ReactElement {
  return (
    <DropdownButton
      opened={props.opened}
      triggerRef={props.triggerRef}
      onChange={(opened) => (opened ? props.onOpen() : props.onClose())}
      width="min(20rem, calc(100vw - 3rem))"
      withinPortal={!props.mobile}
      accessibleLabel={props.copy.choose}
      className={classes.trigger}
      dropdownClassName={classes.dropdown}
      label={
        <Group gap="sm" wrap="nowrap">
          {props.current ? <NavigationIcon name={props.current.kind} /> : null}
          <Stack gap={0} style={{ minWidth: 0 }}>
            <Text fw={600} size="sm" className={classes.triggerLabel}>
              {props.current?.label ?? props.copy.choose}
            </Text>
            {props.current?.kind === "team" ? (
              <Text size="xs" c="dimmed">
                @{props.current.handle}
              </Text>
            ) : null}
          </Stack>
          <NavigationIcon name="down" size="sm" className={classes.trailing} />
        </Group>
      }
    >
      <WorkspaceChoices {...props} />
    </DropdownButton>
  );
}
