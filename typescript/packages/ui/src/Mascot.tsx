import { Image } from "@mantine/core";
import { defaultMascot, mascotAssets } from "./designTokens";
import type { MascotId } from "./designTokens";
import type { ReactElement } from "react";

/** Decorative identity art; URLs retain the caller's runtime mount. */
export function Mascot({
  basePath = "",
  size = 64,
  character = defaultMascot,
}: {
  basePath?: string;
  size?: number;
  character?: MascotId;
}): ReactElement {
  return (
    <Image
      src={`${basePath}/design-assets/${mascotAssets[character].asset}-216.webp`}
      alt=""
      aria-hidden="true"
      w={size}
      h={size}
      radius="lg"
      fit="contain"
      style={{ flexShrink: 0 }}
    />
  );
}
