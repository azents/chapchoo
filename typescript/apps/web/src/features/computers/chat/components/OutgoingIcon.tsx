import type { ReactElement } from "react";

/** Compact status/action icons shared by outgoing-message review states. */
export function OutgoingIcon({
  kind,
}: {
  kind: "error" | "retry" | "delete";
}): ReactElement {
  const paths = {
    error: "M12 8v4m0 4h.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
    retry:
      // Tabler Icons IconRefresh geometry (MIT), also used by Azents.
      "M20 11a8.1 8.1 0 0 0 -15.5 -2m-.5 -4v4h4 M4 13a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4",
    delete: "M4 7h16M10 11v6m4-6v6M6 7l1 13h10l1-13M9 7V4h6v3",
  };
  return (
    <svg
      width="1.125rem"
      height="1.125rem"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={paths[kind]} />
    </svg>
  );
}
