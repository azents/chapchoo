"use client";
import { AppNavigationContext, NavigationPanel } from "./AppNavigation";
import {
  WorkspaceBanner,
  WorkspaceChoices,
  WorkspaceSelector,
} from "./WorkspaceSelector";
import {
  ActionIcon,
  Alert,
  Button,
  Drawer,
  Group,
  Modal,
  rem,
  Stack,
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
    cancel: string;
    create: string;
  };
  mobile: boolean;
  drawer: boolean;
  picker: boolean;
  creating: boolean;
  name: string;
  handle: string;
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
        header: selector,
        trigger,
        homeHref: props.homeHref,
        environment: props.environment,
      }}
    >
      {props.children}
      <Drawer
        opened={props.drawer && props.mobile}
        onClose={props.closeDrawer}
        position="left"
        size="sm"
        title={<WorkspaceBanner {...selectorProps} />}
        closeButtonProps={{ "aria-label": props.copy.close }}
        padding="lg"
        styles={{
          header: { alignItems: "flex-start" },
          title: { flex: 1, minWidth: 0 },
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
          header={props.picker ? <WorkspaceChoices {...selectorProps} /> : null}
          items={props.picker ? [] : props.items}
          basePath={props.basePath}
          mascotReplayLabel={props.mascotReplayLabel}
          onSelect={props.closeDrawer}
        />
      </Drawer>
      <Modal
        opened={props.creating}
        onClose={props.closeCreation}
        title={props.copy.createTeam}
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
          <Stack gap="lg">
            <TextInput
              label={props.copy.teamName}
              value={props.name}
              onChange={(event) => props.onName(event.currentTarget.value)}
              required
              maxLength={120}
              disabled={props.busy}
              data-autofocus
            />
            <TextInput
              label={props.copy.teamHandle}
              value={props.handle}
              onChange={(event) => props.onHandle(event.currentTarget.value)}
              required
              maxLength={80}
              disabled={props.busy}
            />
            {props.error ? (
              <Alert color="red" role="alert">
                {props.error}
              </Alert>
            ) : null}
            <Group justify="flex-end">
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
                disabled={!props.name.trim() || !props.handle.trim()}
              >
                {props.copy.create}
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>
    </AppNavigationContext.Provider>
  );
}
