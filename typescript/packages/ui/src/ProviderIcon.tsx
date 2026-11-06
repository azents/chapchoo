import { rem } from "@mantine/core";
import type { ReactElement } from "react";

export function ProviderIcon({
  kind,
  size = 24,
}: {
  kind: "kubernetes" | "docker";
  size?: number;
}): ReactElement {
  return (
    <svg
      width={rem(size)}
      height={rem(size)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {kind === "kubernetes" ? (
        <>
          <path d="m12 2 9 5v10l-9 5-9-5V7Z" />
          <path d="M12 6v12M7 9l10 6M7 15l10-6" />
        </>
      ) : (
        <>
          <path d="M3 13h14c2 0 3-1 3-3 2 1 2 4 0 5-2 4-6 5-10 5s-7-3-7-7Z" />
          <path d="M5 9h3v4H5zm4 0h3v4H9zm4 0h3v4h-3zM9 5h3v4H9zm4 0h3v4h-3z" />
        </>
      )}
    </svg>
  );
}
