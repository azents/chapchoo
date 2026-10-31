"use client";

import { Group, Text } from "@mantine/core";
import { createContext, useContext } from "react";
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
  return (
    <header className={classes.header}>
      <Text component="div" fw={750} size="lg" className={classes.brand}>
        {brand}
      </Text>
      <Group gap="md" wrap="nowrap" className={classes.tools}>
        <Group gap="xl" wrap="nowrap" className={classes.account}>
          {identityLabel ? (
            <Text size="sm" c="dimmed" className={classes.identity}>
              {identityLabel}
            </Text>
          ) : null}
          {actions}
        </Group>
        {controls}
        {appearance ? <ColorModeMenu {...appearance} /> : null}
      </Group>
    </header>
  );
}
