"use client";

import { Badge, Box, Group, NavLink, Stack, Text, Title } from "@mantine/core";
import classes from "./ApplicationShell.module.css";
import type { ReactNode } from "react";

export interface NavigationItem {
  key: string;
  label: string;
  active?: boolean;
  href?: string;
  onSelect?: () => void;
}

export interface ApplicationShellProps {
  brand?: string;
  contextLabel: string;
  title: string;
  identityLabel?: string;
  navigation: readonly NavigationItem[];
  actions?: ReactNode;
  children: ReactNode;
}

/** Visible app context with app-owned navigation and no routing assumptions. */
export function ApplicationShell({
  brand = "Chapchoo",
  contextLabel,
  title,
  identityLabel,
  navigation,
  actions,
  children,
}: ApplicationShellProps): ReactNode {
  return (
    <Box className={classes.shell}>
      <Box component="header" className={classes.header}>
        <Group justify="space-between" gap="md" wrap="wrap">
          <Group gap="sm">
            <Text fw={700}>{brand}</Text>
            <Badge variant="light">{contextLabel}</Badge>
          </Group>
          <Group gap="sm" wrap="wrap">
            {identityLabel ? (
              <Text
                size="sm"
                c="dimmed"
                style={{ overflowWrap: "anywhere", minWidth: 0 }}
              >
                {identityLabel}
              </Text>
            ) : null}
            {actions}
          </Group>
        </Group>
      </Box>
      <Box className={classes.layout}>
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
        <Box component="main" className={classes.main}>
          <Stack gap="lg">
            <Title order={1} size="h2" style={{ overflowWrap: "anywhere" }}>
              {title}
            </Title>
            {children}
          </Stack>
        </Box>
      </Box>
    </Box>
  );
}
