"use client";

import { Badge, Box, Group, Stack, Text, Title } from "@mantine/core";
import { useContext } from "react";
import { HeaderControlsContext } from "./AppHeader";
import { AppNavigationContext } from "./AppNavigation";
import { ContextSidebar } from "./ContextSidebar";
import { AppFrame } from "./AppFrame";
import classes from "./ApplicationShell.module.css";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { ReactNode } from "react";
import type { NavigationItem } from "./AppNavigation";
export type { NavigationItem } from "./AppNavigation";
export interface ApplicationShellProps {
  mascotReplayLabel: string;
  appearance?: ColorModeMenuProps;
  brand?: string;
  contextLabel: string;
  title: string;
  header?: ReactNode;
  headingActions?: ReactNode;
  contentClassName?: string;
  identityLabel?: string;
  navigation: readonly NavigationItem[];
  actions?: ReactNode;
  children: ReactNode;
  basePath?: string;
}
export type ApplicationShellContentProps = Omit<
  ApplicationShellProps,
  "appearance" | "brand"
>;

/** Page orientation follows the same already-authorized workspace chrome. */
export function WorkspaceHeading({
  title,
  actions,
}: {
  title: string;
  actions?: ReactNode;
}): ReactNode {
  const workspace = useContext(AppNavigationContext);
  return (
    <Box className={classes.workspaceHeading}>
      {workspace ? (
        <Text size="sm" c="dimmed" mb="sm">
          {workspace.label}
        </Text>
      ) : null}
      <Group justify="space-between" align="center" gap="lg">
        <Title order={1} className={classes.workspaceTitle}>
          {title}
        </Title>
        {actions}
      </Group>
    </Box>
  );
}

/** Narrow account actions also remain available outside loaded business shells. */
export function ApplicationAccountActions({
  identityLabel,
  actions,
}: {
  identityLabel?: string;
  actions?: ReactNode;
}): ReactNode {
  const headerControls = useContext(HeaderControlsContext);
  if (headerControls) {
    return null;
  }
  return (
    <Box className={classes.contextHeader}>
      <Text size="sm" c="dimmed" className={classes.identity}>
        {identityLabel}
      </Text>
      <Group gap="sm" className={classes.actions}>
        {actions}
      </Group>
    </Box>
  );
}

/** Product navigation and orientation without routing or business assumptions. */
export function ApplicationShell({
  brand,
  appearance,
  ...content
}: ApplicationShellProps): ReactNode {
  return (
    <AppFrame
      brand={brand}
      appearance={appearance}
      identityLabel={content.identityLabel}
      actions={content.actions}
    >
      <ApplicationShellContent {...content} />
    </AppFrame>
  );
}

/** Body composition for consumers retaining a shared frame across feedback states. */
export function ApplicationShellContent({
  contextLabel,
  title,
  identityLabel,
  navigation,
  actions,
  children,
  header,
  headingActions,
  contentClassName,
}: ApplicationShellContentProps): ReactNode {
  const current = useContext(AppNavigationContext);
  return (
    <Box className={classes.shell} data-contextual={Boolean(current) || void 0}>
      <Box component="aside" className={classes.sidebar}>
        <ContextSidebar
          label={current?.label ?? contextLabel}
          header={
            current?.header ?? (
              <Badge variant="light" size="sm">
                {contextLabel}
              </Badge>
            )
          }
          items={current?.items ?? navigation}
          content={current?.content}
        />
      </Box>
      <div className={classes.workspace}>
        <ApplicationAccountActions
          identityLabel={identityLabel}
          actions={actions}
        />
        <Box component="main" className={classes.main}>
          <div
            className={[classes.content, contentClassName]
              .filter(Boolean)
              .join(" ")}
          >
            {header !== undefined ? (
              header
            ) : (
              <Group justify="space-between" align="center" mb="xl">
                <Title order={1} size="h2" className={classes.heading}>
                  {title}
                </Title>
                {headingActions}
              </Group>
            )}
            <Stack gap="xl">{children}</Stack>
          </div>
        </Box>
      </div>
    </Box>
  );
}
