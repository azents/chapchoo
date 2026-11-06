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
          <path d="m12 2 7.8 3.8 1.9 8.4-5.4 6.7H7.7l-5.4-6.7 1.9-8.4Z" />
          <circle cx="12" cy="12" r="5" />
          <circle cx="12" cy="12" r="1.5" />
          {Array.from({ length: 7 }, (_, index) => (
            <path
              key={index}
              d="M12 5v5.5"
              transform={`rotate(${(index * 360) / 7} 12 12)`}
            />
          ))}
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
