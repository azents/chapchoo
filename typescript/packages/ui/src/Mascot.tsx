import { Image } from "@mantine/core";
import type { ReactElement } from "react";

/** Decorative identity art; URLs retain the caller's runtime mount. */
export function Mascot({
  basePath = "",
  size = 64,
}: {
  basePath?: string;
  size?: number;
}): ReactElement {
  return (
    <Image
      src={`${basePath}/design-assets/02-corgi.png`}
      alt=""
      aria-hidden="true"
      w={size}
      h={size}
      radius="lg"
      fit="contain"
      style={{ mixBlendMode: "multiply", flexShrink: 0 }}
    />
  );
}
