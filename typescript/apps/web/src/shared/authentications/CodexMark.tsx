import { Image } from "@mantine/core";
import type { ReactElement } from "react";

/** Fixed vendor artwork is distributed with its license, not theme colors. */
export function CodexMark({
  basePath = "",
}: {
  basePath?: string;
}): ReactElement {
  return (
    <Image
      src={`${basePath}/design-assets/icons/codex/logo.svg`}
      alt=""
      aria-hidden="true"
      w={36}
      h={36}
      fit="contain"
      style={{ flexShrink: 0 }}
    />
  );
}
