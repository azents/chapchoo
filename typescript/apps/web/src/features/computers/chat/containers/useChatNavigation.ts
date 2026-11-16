"use client";
import { useMediaQuery } from "@mantine/hooks";
import { useEffect, useState } from "react";

export interface ChatNavigation {
  navOpened: boolean;
  desktopNavCollapsed: boolean;
  onOpenNav: () => void;
  onCloseNav: () => void;
  onToggleDesktopNav: () => void;
}
export function useChatNavigation(): ChatNavigation {
  const [navOpened, setNavOpened] = useState(false);
  const [desktopNavCollapsed, setDesktopNavCollapsed] = useState(false);
  const desktop = useMediaQuery("(min-width: 48.001em)");
  useEffect(() => {
    if (desktop) {
      setNavOpened(false);
    }
  }, [desktop]);
  return {
    navOpened: !desktop && navOpened,
    desktopNavCollapsed,
    onOpenNav: () => setNavOpened(true),
    onCloseNav: () => setNavOpened(false),
    onToggleDesktopNav: () => setDesktopNavCollapsed((value) => !value),
  };
}
