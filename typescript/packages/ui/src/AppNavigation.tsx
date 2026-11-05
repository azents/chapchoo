"use client";
import { createContext } from "react";
import type { NavigationIconProps } from "./NavigationIcon";
import type { ReactNode } from "react";

export interface NavigationItem {
  key: string;
  label: string;
  active?: boolean;
  icon?: NavigationIconProps["name"];
  href?: string;
  onSelect?: () => void;
}
export interface AppNavigation {
  label: string;
  items: readonly NavigationItem[];
  header: ReactNode;
  selector?: ReactNode;
  overlayActive?: boolean;
  trigger: ReactNode;
  homeHref: string | null;
  homeTarget?: "_blank";
  environment?: string;
  content?: ReactNode;
}
/** The product supplies current membership, destinations and transient controls. */
export const AppNavigationContext = createContext<AppNavigation | null>(null);
