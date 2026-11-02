"use client";

import { Box, NavLink } from "@mantine/core";
import { createContext } from "react";
import { AgentMascotMotion } from "./AgentMascotMotion";
import classes from "./AppNavigation.module.css";
import type { ReactElement, ReactNode } from "react";

export interface NavigationItem {
  key: string;
  label: string;
  active?: boolean;
  href?: string;
  onSelect?: () => void;
}
export interface AppNavigation {
  label: string;
  items: readonly NavigationItem[];
  header: ReactNode;
  trigger: ReactNode;
  homeHref: string | null;
  environment?: string;
}
/** The product supplies current membership, destinations and transient controls. */
export const AppNavigationContext = createContext<AppNavigation | null>(null);

export function NavigationPanel({
  label,
  items,
  header,
  basePath,
  mascotReplayLabel,
  onSelect,
  mobileDrawer = false,
}: {
  label: string;
  items: readonly NavigationItem[];
  header?: ReactNode;
  basePath: string;
  mascotReplayLabel: string;
  onSelect?: () => void;
  mobileDrawer?: boolean;
}): ReactElement {
  return (
    <Box className={classes.panel} data-drawer={mobileDrawer || void 0}>
      {header ? <Box className={classes.header}>{header}</Box> : null}
      <Box component="nav" aria-label={label} className={classes.links}>
        {items.map((item) => {
          const common = {
            label: item.label,
            active: item.active,
            "aria-current": item.active ? ("page" as const) : void 0,
            onClick: (): void => {
              item.onSelect?.();
              onSelect?.();
            },
          };
          return item.href ? (
            <NavLink
              key={item.key}
              component="a"
              href={item.href}
              {...common}
            />
          ) : (
            <NavLink
              key={item.key}
              component="button"
              type="button"
              {...common}
            />
          );
        })}
      </Box>
      <Box className={classes.companion}>
        <AgentMascotMotion
          basePath={basePath}
          size={72}
          playing
          replayLabel={mascotReplayLabel}
        />
      </Box>
    </Box>
  );
}
