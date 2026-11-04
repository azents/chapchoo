import { Image, Paper, rem } from "@mantine/core";
import { profilePalette } from "../profilePalette";
import type { AgentPaletteKey } from "@chapchoo/web-core/agent-profile";
import type { ReactElement } from "react";

export function AgentPortrait({
  basePath,
  assetKey,
  paletteKey,
  compact = false,
}: {
  basePath: string;
  assetKey: string;
  paletteKey: AgentPaletteKey;
  compact?: boolean;
}): ReactElement {
  return (
    <Paper
      p={compact ? "xs" : "md"}
      radius="lg"
      bg={profilePalette[paletteKey].surface}
      w="fit-content"
    >
      <Image
        w={rem(compact ? 40 : 96)}
        h={rem(compact ? 40 : 96)}
        fit="contain"
        alt=""
        aria-hidden="true"
        src={`${basePath}/design-assets/${assetKey}-216.webp`}
      />
    </Paper>
  );
}
