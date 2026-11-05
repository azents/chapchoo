"use client";
import { AppNavigationContext, NavigationPanel } from "./AppNavigation";
import { WorkspaceSelector } from "./WorkspaceSelector";
import { useRef } from "react";
import classes from "./WorkspaceChrome.module.css";
import {
  ActionIcon,
  Alert,
  Button,
  Drawer,
  Group,
  Modal,
  rem,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import type {
  WorkspaceSelectorProps,
  WorkspaceChoice,
} from "./WorkspaceSelector";
import type { NavigationItem } from "./AppNavigation";
import type { ReactElement, ReactNode } from "react";

export interface WorkspaceChromeProps {
  children: ReactNode;
  identity: boolean;
  basePath: string;
  homeHref: string | null;
  environment?: string;
  copy: {
    chooseContext: string;
    teams: string;
    createTeam: string;
    back: string;
    unavailable: string;
    retry: string;
    navigation: string;
    close: string;
    teamName: string;
    teamHandle: string;
    teamAddress: string;
    teamHandleHint: string;
    cancel: string;
    create: string;
  };
  mobile: boolean;
  drawer: boolean;
  picker: boolean;
  creating: boolean;
  name: string;
  handle: string;
  prospectiveAddress: string | null;
  handleStatus: {
    message: string | null;
    description: string;
    available: boolean;
  };
  error: string | null;
  choices: WorkspaceChoice[];
  current: WorkspaceChoice | null;
  items: NavigationItem[];
  status: "ready" | "loading" | "error";
  busy: boolean;
  mascotReplayLabel: string;
  openDrawer: () => void;
  closeDrawer: () => void;
  openPicker: () => void;
  closePicker: () => void;
  openCreation: () => void;
  closeCreation: () => void;
  onName: (value: string) => void;
  onHandle: (value: string) => void;
  onSubmit: () => void;
  retry: () => void;
  select: (choice: WorkspaceChoice) => void;
}

export function WorkspaceChrome(props: WorkspaceChromeProps): ReactElement {
  const workspaceTrigger = useRef<HTMLButtonElement>(null);
  if (!props.identity) {
    return (
      <AppNavigationContext.Provider
        value={{
          label: props.copy.chooseContext,
          items: [],
          header: null,
          trigger: null,
          homeHref: props.homeHref,
          environment: props.environment,
        }}
      >
        {props.children}
      </AppNavigationContext.Provider>
    );
  }
  const selectorProps: WorkspaceSelectorProps = {
    current: props.current,
    choices: props.choices,
    status: props.status,
    mobile: props.mobile,
    opened: props.picker,
    triggerRef: workspaceTrigger,
    copy: {
      choose: props.copy.chooseContext,
      teams: props.copy.teams,
      create: props.copy.createTeam,
      back: props.copy.back,
      unavailable: props.copy.unavailable,
      retry: props.copy.retry,
    },
    onOpen: props.openPicker,
    onClose: props.closePicker,
    onCreate: props.openCreation,
    onRetry: props.retry,
    onSelect: props.select,
  };
  const selector = <WorkspaceSelector {...selectorProps} />;
  const trigger = (
    <ActionIcon
      size={rem(44)}
      variant="subtle"
      aria-label={props.copy.navigation}
      onClick={props.openDrawer}
      aria-expanded={props.drawer}
    >
      <svg
        width={rem(22)}
        height={rem(22)}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.7"
        aria-hidden="true"
      >
        <path d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </ActionIcon>
  );
  return (
    <AppNavigationContext.Provider
      value={{
        label: props.current?.label ?? props.copy.chooseContext,
        items: props.items,
        header: props.mobile ? null : selector,
        selector,
        overlayActive: props.picker || props.creating,
        trigger,
        homeHref: props.homeHref,
        environment: props.environment,
      }}
    >
      {props.children}
      <Drawer
        opened={props.drawer && props.mobile}
        closeOnEscape={!props.picker && !props.creating}
        closeOnClickOutside={!props.creating}
        trapFocus={!props.creating}
        onClose={props.closeDrawer}
        position="left"
        size="20rem"
        title={props.mobile ? selector : null}
        closeButtonProps={{ "aria-label": props.copy.close, size: rem(44) }}
        classNames={{
          header: classes.drawerHeader,
          title: classes.drawerTitle,
          close: classes.drawerClose,
        }}
        padding="lg"
        styles={{
          body: {
            minHeight: "calc(100dvh - 5rem)",
            display: "flex",
            flexDirection: "column",
          },
        }}
      >
        <NavigationPanel
          mobileDrawer
          label={props.copy.navigation}
          items={props.items}
          basePath={props.basePath}
          mascotReplayLabel={props.mascotReplayLabel}
          onSelect={props.closeDrawer}
        />
      </Drawer>
      <Modal
        opened={props.creating}
        returnFocus={false}
        onExitTransitionEnd={() =>
          workspaceTrigger.current?.focus({ preventScroll: true })
        }
        onClose={props.closeCreation}
        title={props.copy.createTeam}
        size="30rem"
        padding={0}
        radius="xl"
        overlayProps={{ backgroundOpacity: 0.35, blur: 4 }}
        classNames={{
          header: classes.createHeader,
          title: classes.createTitle,
          body: classes.createBody,
        }}
        closeButtonProps={{ "aria-label": props.copy.close }}
        centered
        closeOnClickOutside={!props.busy}
        closeOnEscape={!props.busy}
      >
        <form
          onSubmit={(event) => {
            event.preventDefault();
            props.onSubmit();
          }}
        >
          <Stack gap="lg" className={classes.createFields}>
            <TextInput
              label={props.copy.teamName}
              value={props.name}
              onChange={(event) => props.onName(event.currentTarget.value)}
              required
              maxLength={120}
              withAsterisk={false}
              disabled={props.busy}
              data-autofocus
            />
            <TextInput
              label={
                <span className={classes.handleLabel}>
                  <span>{props.copy.teamHandle}</span>
                  <span aria-hidden="true">{props.handle.length}/16</span>
                </span>
              }
              labelProps={{ className: classes.fieldLabel }}
              value={props.handle}
              autoCapitalize="none"
              autoCorrect="off"
              autoComplete="off"
              spellCheck={false}
              inputMode="text"
              onChange={(event) => props.onHandle(event.currentTarget.value)}
              required
              maxLength={16}
              withAsterisk={false}
              disabled={props.busy}
            />
            <Text size="xs" c="dimmed">
              {props.copy.teamHandleHint}
            </Text>
            {props.handleStatus.message ? (
              <Text
                size="xs"
                c={
                  props.handleStatus.available
                    ? "var(--chapchoo-color-success)"
                    : "dimmed"
                }
                role="status"
              >
                {props.handleStatus.message}
              </Text>
            ) : null}
            <Text size="xs" c="dimmed">
              {props.handleStatus.description}
            </Text>
            {props.prospectiveAddress ? (
              <Stack gap="xs" className={classes.addressPreview}>
                <Text size="xs" c="dimmed">
                  {props.copy.teamAddress}
                </Text>
                <Text size="sm" style={{ overflowWrap: "anywhere" }}>
                  {props.prospectiveAddress}
                </Text>
              </Stack>
            ) : null}
            {props.error ? (
              <Alert color="red" role="alert">
                {props.error}
              </Alert>
            ) : null}
          </Stack>
          <Group justify="flex-end" className={classes.createFooter}>
            <Button
              variant="default"
              onClick={props.closeCreation}
              disabled={props.busy}
            >
              {props.copy.cancel}
            </Button>
            <Button
              type="submit"
              loading={props.busy}
              disabled={!props.name.trim() || !props.handleStatus.available}
            >
              {props.copy.createTeam}
            </Button>
          </Group>
        </form>
      </Modal>
    </AppNavigationContext.Provider>
  );
}
