"use client";

import { Badge, Group, Text } from "@mantine/core";
import { createContext, useContext } from "react";
import { AppNavigationContext } from "./AppNavigation";
import { ColorModeMenu } from "./ColorModeMenu";
import classes from "./AppHeader.module.css";
import "./appChrome.css";
import type { ColorModeMenuProps } from "./ColorModeMenu";
import type { ReactElement, ReactNode } from "react";

export const HeaderControlsContext = createContext<ReactNode>(null);

export interface AppHeaderProps {
  brand?: string;
  appearance?: ColorModeMenuProps;
  identityLabel?: string;
  actions?: ReactNode;
}

/** Shared application chrome, independent of route, auth and storage ownership. */
export function AppHeader({
  brand = "chapchoo",
  appearance,
  identityLabel,
  actions,
}: AppHeaderProps): ReactElement {
  const controls = useContext(HeaderControlsContext);
  const navigation = useContext(AppNavigationContext);
  return (
    <header className={classes.header}>
      <Group gap="sm" wrap="nowrap">
        {navigation?.trigger ? (
          <span className={classes.mobileNavigation}>{navigation.trigger}</span>
        ) : null}
        {navigation?.homeHref ? (
          <Text
            component="a"
            href={navigation.homeHref}
            fw={750}
            size="lg"
            className={classes.brand}
          >
            {brand}
          </Text>
        ) : (
          <Text component="div" fw={750} size="lg" className={classes.brand}>
            {brand}
          </Text>
        )}
        {navigation?.environment ? (
          <Badge variant="outline" color="gray" tt="none" size="sm">
            {navigation.environment}
          </Badge>
        ) : null}
      </Group>
      <Group gap="md" wrap="nowrap" className={classes.tools}>
        {controls ?? (
          <>
            <Group gap="xl" wrap="nowrap" className={classes.account}>
              {identityLabel ? (
                <Text size="sm" c="dimmed" className={classes.identity}>
                  {identityLabel}
                </Text>
              ) : null}
              {actions}
            </Group>
            {appearance ? <ColorModeMenu {...appearance} /> : null}
          </>
        )}
      </Group>
    </header>
  );
}
