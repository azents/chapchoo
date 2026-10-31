"use client";

import { Badge, Box, Group, NavLink, Stack, Text, Title } from "@mantine/core";
import { Mascot } from "./Mascot";
import { AppFrame } from "./AppFrame";
import classes from "./ApplicationShell.module.css";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { ReactNode } from "react";

export interface NavigationItem {
  key: string;
  label: string;
  active?: boolean;
  href?: string;
  onSelect?: () => void;
}
export interface ApplicationShellProps {
  appearance?: ColorModeMenuProps;
  brand?: string;
  contextLabel: string;
  title: string;
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

/** Narrow account actions also remain available outside loaded business shells. */
export function ApplicationAccountActions({
  identityLabel,
  actions,
}: {
  identityLabel?: string;
  actions?: ReactNode;
}): ReactNode {
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
  basePath = "",
}: ApplicationShellContentProps): ReactNode {
  return (
    <Box className={classes.shell}>
      <Box component="aside" className={classes.sidebar}>
        <div className={classes.brand}>
          <Badge variant="light" size="sm">
            {contextLabel}
          </Badge>
        </div>
        <Box
          component="nav"
          aria-label={contextLabel}
          className={classes.navigation}
        >
          {navigation.map((item) =>
            item.href ? (
              <NavLink
                key={item.key}
                component="a"
                href={item.href}
                label={item.label}
                active={item.active}
                aria-current={item.active ? "page" : undefined}
                onClick={item.onSelect}
              />
            ) : (
              <NavLink
                key={item.key}
                component="button"
                type="button"
                label={item.label}
                active={item.active}
                aria-current={item.active ? "page" : undefined}
                onClick={item.onSelect}
              />
            ),
          )}
        </Box>
        <div className={classes.companion}>
          <Mascot basePath={basePath} size={72} />
        </div>
      </Box>
      <div className={classes.workspace}>
        <ApplicationAccountActions
          identityLabel={identityLabel}
          actions={actions}
        />
        <Box component="main" className={classes.main}>
          <div className={classes.content}>
            <Title order={1} size="h2" className={classes.title}>
              {title}
            </Title>
            <Stack gap="xl">{children}</Stack>
          </div>
        </Box>
      </div>
    </Box>
  );
}
