"use client";

import {
  ActionIcon,
  Avatar,
  Button,
  Divider,
  Drawer,
  Group,
  Popover,
  Stack,
  Text,
  rem,
} from "@mantine/core";
import {
  DisplayFields,
  DisplayIcon,
  DisplaySaveFeedback,
} from "./DisplaySettings";
import classes from "./AccountSettings.module.css";
import { InteractiveSurface } from "./InteractiveSurface";
import { NavigationIcon } from "./NavigationIcon";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { DisplayPanel } from "./DisplaySettings";
import type { ComponentProps, ReactElement } from "react";

export type AccountSettingsProps = Omit<
  ComponentProps<typeof DisplayPanel>,
  "footer"
> & {
  mobile: boolean;
  appearance: ColorModeMenuProps;
  identityLabel?: string;
  profileHref: string;
  profileLabel: string;
  loginHref: string;
  loginLabel: string;
  logoutLabel: string;
  onOpen: () => void;
  onLogout: () => void;
};
function UserIcon(): ReactElement {
  return (
    <svg
      width={rem(22)}
      height={rem(22)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.65"
      aria-hidden="true"
    >
      <circle cx="12" cy="8" r="3.5" />
      <path d="M4.5 21v-2a7.5 7.5 0 0 1 15 0v2" />
    </svg>
  );
}
function SettingsContent(props: AccountSettingsProps): ReactElement {
  return (
    <Stack gap="lg">
      {props.identityLabel ? (
        <>
          <InteractiveSurface
            href={props.profileHref}
            aria-label={props.profileLabel}
            onClick={props.onClose}
            className={classes.profile}
          >
            <Group wrap="nowrap" align="center">
              <Avatar radius="xl">
                <UserIcon />
              </Avatar>
              <Stack gap="xs" className={classes.identity}>
                <Text size="sm" fw={600}>
                  {props.identityLabel}
                </Text>
                <Text size="sm" c="var(--mantine-color-anchor)">
                  {props.profileLabel}
                </Text>
              </Stack>
              <NavigationIcon name="forward" size="sm" />
            </Group>
          </InteractiveSurface>
          <Divider />
        </>
      ) : null}
      <Stack gap="sm">
        <Text size="sm" fw={500}>
          {props.appearance.label}
        </Text>
        <Group gap="xs" grow wrap="nowrap">
          {(["system", "light", "dark"] as const).map((mode) => (
            <Button
              key={mode}
              variant={props.appearance.value === mode ? "light" : "subtle"}
              aria-pressed={props.appearance.value === mode}
              onClick={() => props.appearance.onChange(mode)}
              size="sm"
            >
              {props.appearance.labels[mode]}
            </Button>
          ))}
        </Group>
      </Stack>
      <DisplayFields {...props} withinPortal={false} />
      <DisplaySaveFeedback
        copy={props.copy}
        error={props.error}
        storageError={props.storageError}
        onRetry={props.onRetry}
      />
      {props.changed && !props.error && !props.disabled ? (
        <Text size="xs" c="dimmed" role="status">
          {props.copy.saved}
        </Text>
      ) : null}
      <Divider />
      {props.identityLabel ? (
        <Button variant="subtle" color="red" onClick={props.onLogout}>
          {props.logoutLabel}
        </Button>
      ) : (
        <Button component="a" href={props.loginHref}>
          {props.loginLabel}
        </Button>
      )}
    </Stack>
  );
}
/** Shared account entry. Apps retain auth, preference and persistence authority. */
export function AccountSettings(props: AccountSettingsProps): ReactElement {
  const trigger = (
    <ActionIcon
      size={rem(44)}
      variant="subtle"
      aria-label={props.copy.title}
      title={props.copy.title}
      onClick={props.opened ? props.onClose : props.onOpen}
      onKeyDown={(event) => {
        if (event.key === "Escape" && props.opened) {
          props.onClose();
        }
      }}
      aria-expanded={props.opened}
    >
      {props.identityLabel ? <UserIcon /> : <DisplayIcon kind="settings" />}
    </ActionIcon>
  );
  return (
    <>
      {props.mobile ? (
        trigger
      ) : (
        <Popover
          opened={props.opened}
          onChange={(opened) => (opened ? props.onOpen() : props.onClose())}
          position="bottom-end"
          width={rem(360)}
          shadow="md"
          trapFocus
          returnFocus
          closeOnClickOutside
          closeOnEscape
        >
          <Popover.Target>{trigger}</Popover.Target>
          <Popover.Dropdown className={classes.panel} p="xl">
            <SettingsContent {...props} />
          </Popover.Dropdown>
        </Popover>
      )}
      <Drawer
        opened={props.opened && props.mobile}
        onClose={props.onClose}
        position="right"
        size="sm"
        title={props.copy.title}
        closeButtonProps={{ "aria-label": props.copy.close }}
        padding="xl"
      >
        <SettingsContent {...props} />
      </Drawer>
    </>
  );
}
